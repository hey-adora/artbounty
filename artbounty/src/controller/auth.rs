use crate::controller::encode::ResErr;
use tracing::error;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthToken {
    pub username: String,
    pub created_at: u128,
    pub exp: u64,
}

impl AuthToken {
    pub fn new<S: Into<String>>(username: S, time: u128) -> Self {
        let username: String = username.into();
        AuthToken {
            username,
            created_at: time,
            exp: 0,
        }
    }
}

#[derive(
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    PartialEq,
)]
pub struct InviteToken {
    pub email: String,
    pub created_at: u128,
    pub exp: u64,
}

impl InviteToken {
    pub fn new<S: Into<String>>(
        email: S,
        created_at: u128,
        // exp: std::time::Duration,
    ) -> Self {
        Self {
            email: email.into(),
            created_at,
            exp: 0,
            // exp: exp.as_secs(),
        }
    }
}

#[cfg(feature = "ssr")]
pub fn hash_password<S: Into<String>>(password: S) -> Result<String, argon2::password_hash::Error> {
    use argon2::{
        Argon2, PasswordHasher,
        password_hash::{SaltString, rand_core::OsRng},
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password = password.into();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

#[cfg(feature = "ssr")]
pub fn encode_token<Key: AsRef<[u8]>, Claims: serde::Serialize>(
    key: Key,
    claims: Claims,
) -> Result<String, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

    let header = Header::new(Algorithm::HS512);
    let key = EncodingKey::from_secret(key.as_ref());

    encode(&header, &claims, &key)
}

#[cfg(feature = "ssr")]
pub fn decode_token<Claims: serde::de::DeserializeOwned>(
    secret: impl AsRef<[u8]>,
    token: impl AsRef<str>,
    validate_exp: bool,
) -> Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

    let token = token.as_ref();
    let secret = DecodingKey::from_secret(secret.as_ref());
    let mut validation = Validation::new(Algorithm::HS512);
    validation.validate_exp = validate_exp;
    validation.leeway = 0;

    decode::<Claims>(token, &secret, &validation)
}

#[cfg(feature = "ssr")]
pub fn create_cookie<Key: AsRef<[u8]>, S: Into<String>>(
    key: Key,
    username: S,
    time: std::time::Duration,
) -> Result<(String, String), jsonwebtoken::errors::Error> {
    use std::time::Duration;

    use tracing::trace;

    let key = key.as_ref();
    let token = encode_token(key, AuthToken::new(username, time.as_nanos()))
        .inspect_err(|err| error!("jwt exploded {err}"))?;
    trace!("token created: {token:?}");
    // .map_err(|_| OutputErr::CreateCookieErr)?;
    let cookie = format!("Bearer={token}; Secure; HttpOnly");
    trace!("cookie created: {cookie:?}");
    Ok((token, cookie))
}

pub fn cut_cookie<'a>(v: &'a str, start: &str, end: &str) -> &'a str {
    let start_len = start.len();
    let v_len = v.len();
    let end_len = end.len();
    if v_len <= end_len {
        return v;
    }
    let final_len = v_len - end_len;
    if final_len <= start_len {
        return v;
    }
    &v[start_len..final_len]
}

pub fn cut_cookie_value_decoded(v: &str) -> &str {
    let start = "Bearer=";
    let end = "; Secure; HttpOnly";
    cut_cookie(v, start, end)
}

pub fn cut_cookie_full_encoded(v: &str) -> &str {
    let start = "authorization=Bearer%3D";
    let end = "%3B%20Secure%3B%20HttpOnly";
    cut_cookie(v, start, end)
}

pub fn cut_cookie_full_with_expiration_encoded(v: &str) -> &str {
    let start = "authorization=Bearer%3D";
    let end =
        "%3B%20Secure%3B%20HttpOnly%3B%20expires%3DThu%2C%2001%20Jan%201970%2000%3A00%3A00%20GMT";
    cut_cookie(v, start, end)
}

#[cfg(feature = "ssr")]
pub async fn check_auth<ServerErr>(
    app_state: &crate::controller::app_state::AppState,
    jar: &axum_extra::extract::cookie::CookieJar,
) -> Result<AuthToken, ResErr<ServerErr>>
where
    ServerErr: std::error::Error + 'static,
{
    use http::header::AUTHORIZATION;

    use crate::controller::encode::{ResErr, ResErrUnauthorized};

    let token = jar
        .get(AUTHORIZATION.as_str())
        .ok_or(ResErr::Unauthorized(ResErrUnauthorized::NoCookie))
        .map(|v| cut_cookie_value_decoded(v.value()).to_string())?;

    let _session = app_state
        .db
        .get_session(&token)
        .await
        .map_err(|err| ResErr::<ServerErr>::Unauthorized(ResErrUnauthorized::Unauthorized))?;

    let token = match decode_token::<AuthToken>(&app_state.settings.auth.secret, &token, false) {
        Ok(v) => v,
        Err(err) => {
            error!("invalid token was stored {err}");
            app_state
                .db
                .delete_session(token)
                .await
                .map_err(|err| ResErr::Unauthorized(ResErrUnauthorized::DbErr))?;
            return Err(ResErr::Unauthorized(ResErrUnauthorized::BadToken));
        }
    };

    Ok(token.claims)
}

#[cfg(test)]
pub fn test_extract_cookie(headers: &http::HeaderMap) -> Option<String> {
    headers
        .get(http::header::SET_COOKIE)
        .map(|v| cut_cookie_full_encoded(v.to_str().unwrap()).to_string())
}

#[cfg(test)]
pub fn test_extract_cookie_and_decode<Secret: Into<String>>(
    secret: Secret,
    headers: &http::HeaderMap,
) -> Option<(String, jsonwebtoken::TokenData<AuthToken>)> {
    headers.get(http::header::SET_COOKIE).map(|v| {
        let cookie = cut_cookie_full_encoded(v.to_str().unwrap()).to_string();
        let secret = secret.into();
        (
            cookie.clone(),
            decode_token::<AuthToken>(secret, cookie, false).unwrap(),
        )
    })
}

#[cfg(test)]
mod util {
    use crate::controller::auth::{cut_cookie_full_encoded, cut_cookie_full_with_expiration_encoded, cut_cookie_value_decoded};

    #[test]
    fn cut_cookie() {
        cut_cookie_value_decoded("");
        cut_cookie_full_encoded("");
        cut_cookie_full_with_expiration_encoded("");
    }
}

pub mod route {
    pub mod logout {

        use thiserror::Error;
        use tracing::{error, trace};

        use crate::{
            controller::encode::{ResErr, send_web},
            path::{PATH_API_LOGIN, PATH_API_LOGOUT},
        };

        pub const PATH: &'static str = "/logout";

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct Input {}

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct ServerOutput {}

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerOutput {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::encode_result;

                let bytes = encode_result::<ServerOutput, ServerErr>(&Ok(self));
                (axum::http::StatusCode::OK, bytes).into_response()
            }
        }

        #[derive(
            Debug,
            Error,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub enum ServerErr {
            #[error("internal server error")]
            ServerErr,

            #[error("jwt error")]
            Unauthorized,

            #[error("jwt error")]
            NoCookie,

            #[error("jwt error")]
            JWT,
        }

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerErr {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::encode_result;

                let status = match &self {
                    _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                };
                let bytes = encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                (status, bytes).into_response()
            }
        }

        pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
            send_web::<ServerOutput, ServerErr>(PATH_API_LOGOUT, &input).await
        }

        #[cfg(feature = "ssr")]
        pub async fn server(
            axum::extract::State(app_state): axum::extract::State<
                crate::controller::app_state::AppState,
            >,
            jar: axum_extra::extract::cookie::CookieJar,
        ) -> impl axum::response::IntoResponse {
            trace!("executing profile api");
            use axum_extra::extract::cookie::Cookie;
            use http::header::AUTHORIZATION;

            use super::super::cut_cookie_value_decoded;
            use crate::controller::encode::encode_server_output_custom;

            let token = match jar
                .get(AUTHORIZATION.as_str())
                .ok_or(ResErr::ServerErr(ServerErr::NoCookie))
                .map(|v| cut_cookie_value_decoded(v.value()).to_string())
            {
                Ok(v) => v,
                Err(err) => {
                    return (
                        jar,
                        encode_server_output_custom(
                            Result::<ServerOutput, ResErr<ServerErr>>::Err(err),
                        ),
                    );
                }
            };

            let result = (async || -> Result<ServerOutput, ResErr<ServerErr>> {
                let _session = app_state
                    .db
                    .get_session(&token)
                    .await
                    .map_err(|err| ResErr::ServerErr(ServerErr::Unauthorized))?;

                app_state
                    .db
                    .delete_session(token)
                    .await
                    .map_err(|err| ResErr::ServerErr(ServerErr::ServerErr))?;

                Ok(ServerOutput {})
            })()
            .await;

            let jar = jar.add(Cookie::new(
                AUTHORIZATION.as_str(),
                "Bearer=DELETED; Secure; HttpOnly; expires=Thu, 01 Jan 1970 00:00:00 GMT",
            ));
            (jar, encode_server_output_custom(result))
        }

        #[cfg(test)]
        pub async fn test_send<Token: Into<String>>(
            server: &axum_test::TestServer,
            token: Token,
        ) -> (http::HeaderMap, Result<ServerOutput, ResErr<ServerErr>>) {
            use crate::{controller::encode::send_builder, path::PATH_API};

            let input = Input {
                    // token: token.into(),
                };
            let path = format!("{}{}", PATH_API, PATH_API_LOGOUT);
            let token: String = token.into();
            let builder = server.reqwest_post(&path).header(
                http::header::COOKIE,
                format!("authorization=Bearer%3D{}%3B%20Secure%3B%20HttpOnly", token),
            );
            let res = send_builder::<ServerOutput, ServerErr>(builder, &input).await;
            trace!("RESPONSE: {res:#?}");
            res
        }

        #[cfg(test)]
        mod api {
            use std::sync::Arc;
            use std::time::Duration;

            use axum_test::TestServer;
            use http::header::SET_COOKIE;
            use test_log::test;
            use tokio::sync::Mutex;

            use crate::controller;
            use crate::controller::app_state::AppState;
            use crate::controller::auth::cut_cookie_full_with_expiration_encoded;
            use crate::controller::clock::get_timestamp;
            use crate::server::create_api_router;

            #[test(tokio::test)]
            async fn logout() {
                let current_time = get_timestamp();
                let time = Arc::new(Mutex::new(current_time));
                let app_state = AppState::new_testng(time).await;
                let my_app = create_api_router(app_state.clone()).with_state(app_state.clone());

                let server = TestServer::builder()
                    .http_transport()
                    .build(my_app)
                    .unwrap();

                {
                    let time = app_state.clock.now().await;
                    let exp = time + Duration::from_secs(60 * 30);

                    controller::auth::route::invite::test_send(&server, "hey1@hey.com")
                        .await
                        .1
                        .unwrap();
                    let invite = app_state
                        .db
                        .get_invite("hey1@hey.com", current_time.as_nanos())
                        .await
                        .unwrap();

                    let (cookies, res) = controller::auth::route::register::test_send(
                        &server,
                        "hey",
                        &invite.token_raw,
                        "wowowowow123@",
                    )
                    .await;

                    let res =
                        controller::auth::route::logout::test_send(&server, &invite.token_raw)
                            .await;
                    let cookie = cut_cookie_full_with_expiration_encoded(
                        res.0.get(SET_COOKIE).unwrap().to_str().unwrap(),
                    );
                    assert_eq!(cookie, "DELETED");
                }
                // res.1.unwrap();
            }
        }
    }
    pub mod user {

        use crate::controller::encode::ResErr;
        use crate::controller::encode::send_web;
        use crate::path::PATH_API_USER;
        use thiserror::Error;
        use tracing::{error, trace};

        pub const PATH: &'static str = "/user";

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct Input {
            pub username: String,
        }

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct ServerOutput {
            pub username: String,
        }

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerOutput {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::encode_result;

                let bytes = encode_result::<ServerOutput, ServerErr>(&Ok(self));
                (axum::http::StatusCode::OK, bytes).into_response()
            }
        }

        #[derive(
            Debug,
            Error,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub enum ServerErr {
            #[error("internal server error")]
            ServerErr,

            #[error("user not found")]
            NotFound,
        }

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerErr {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::{ResErr, encode_result};

                let status = match &self {
                    ServerErr::NotFound => axum::http::StatusCode::NOT_FOUND,
                    _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                };
                let bytes = encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                (status, bytes).into_response()
            }
        }

        pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
            send_web::<ServerOutput, ServerErr>(PATH_API_USER, &input).await
        }

        #[cfg(feature = "ssr")]
        pub async fn server(
            axum::extract::State(app_state): axum::extract::State<
                crate::controller::app_state::AppState,
            >,
            jar: axum_extra::extract::cookie::CookieJar,
            multipart: axum::extract::Multipart,
        ) -> impl axum::response::IntoResponse {
            use crate::controller::encode::encode_server_output_custom;

            trace!("executing profile api");

            let result = (async || -> Result<ServerOutput, ResErr<ServerErr>> {
                use crate::{
                    controller::encode::decode_multipart,
                    db::user::get_user_by_username::GetUserByUsernameErr,
                };

                let input = decode_multipart::<Input, ServerErr>(multipart).await?;
                trace!("input!!!!!! {input:#?}");
                let user = app_state
                    .db
                    .get_user_by_username(input.username)
                    .await
                    .map_err(|err| match err {
                        GetUserByUsernameErr::UserNotFound => ServerErr::NotFound,
                        _ => ServerErr::ServerErr,
                    })?;

                Ok(ServerOutput {
                    username: user.username,
                })
            })()
            .await;

            encode_server_output_custom(result)
        }

        #[cfg(test)]
        pub async fn test_send(
            server: &axum_test::TestServer,
            username: impl Into<String>,
        ) -> (http::HeaderMap, Result<ServerOutput, ResErr<ServerErr>>) {
            use crate::{controller::encode::send_builder, path::PATH_API};

            let input = Input {
                username: username.into(),
            };
            let path = format!("{}{}", PATH_API, PATH);
            let builder = server.reqwest_post(&path);
            let res = send_builder::<ServerOutput, ServerErr>(builder, &input).await;
            trace!("RESPONSE: {res:#?}");
            res
        }

        #[cfg(test)]
        mod api {
            use std::sync::Arc;
            use std::time::Duration;

            use crate::controller;
            use crate::controller::app_state::AppState;
            use crate::controller::clock::get_timestamp;
            use crate::controller::encode::ResErr;
            use crate::server::create_api_router;

            use super::ServerErr;
            use axum_test::TestServer;
            use test_log::test;
            use tokio::sync::Mutex;

            #[test(tokio::test)]
            async fn user() {
                let current_time = get_timestamp();
                let time = Arc::new(Mutex::new(current_time));
                let app_state = AppState::new_testng(time).await;
                let my_app = create_api_router(app_state.clone()).with_state(app_state.clone());

                let server = TestServer::builder()
                    .http_transport()
                    .build(my_app)
                    .unwrap();

                {
                    let time = app_state.clock.now().await;
                    let exp = time + Duration::from_secs(60 * 30);
                    let user = controller::auth::route::user::test_send(&server, "hey").await;
                    assert!(matches!(
                        user.1,
                        Err(ResErr::ServerErr(ServerErr::NotFound))
                    ));
                    let _ = app_state.db.add_user("hey", "hey@hey.com", "123").await;
                    let user = controller::auth::route::user::test_send(&server, "hey")
                        .await
                        .1
                        .unwrap();
                }
            }
        }
    }
    pub mod profile {
        use thiserror::Error;
        use tracing::{error, trace};

        use crate::{
            controller::encode::{ResErr, send_web},
            path::PATH_API_PROFILE,
        };

        pub const PATH: &'static str = "/profile";

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct Input {}

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct ServerOutput {
            pub username: String,
        }

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerOutput {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::encode_result;

                let bytes = encode_result::<ServerOutput, ServerErr>(&Ok(self));
                (axum::http::StatusCode::OK, bytes).into_response()
            }
        }

        #[derive(
            Debug,
            Error,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub enum ServerErr {
            #[error("internal server error")]
            ServerErr,
        }

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerErr {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::encode_result;

                let status = match &self {
                    // ServerErr::NoCookie => axum::http::StatusCode::OK,
                    _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                };
                let bytes = encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                (status, bytes).into_response()
            }
        }

        pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
            send_web::<ServerOutput, ServerErr>(PATH_API_PROFILE, &input).await
        }

        #[cfg(feature = "ssr")]
        pub async fn server(
            axum::extract::State(app_state): axum::extract::State<
                crate::controller::app_state::AppState,
            >,
            jar: axum_extra::extract::cookie::CookieJar,
        ) -> impl axum::response::IntoResponse {
            use crate::controller::encode::encode_server_output_custom;

            trace!("executing profile api");

            let result = (async || -> Result<ServerOutput, ResErr<ServerErr>> {
                use crate::controller::auth::check_auth;

                let auth_token = check_auth(&app_state, &jar).await?;

                Ok(ServerOutput {
                    username: auth_token.username,
                })
            })()
            .await;

            encode_server_output_custom(result)
        }

        #[cfg(test)]
        pub async fn test_send<Token: Into<String>>(
            server: &axum_test::TestServer,
            token: Token,
        ) -> (http::HeaderMap, Result<ServerOutput, ResErr<ServerErr>>) {
            use crate::{controller::encode::send_builder, path::PATH_API};

            let input = Input {
                    // token: token.into(),
                };
            let path = format!("{}{}", PATH_API, PATH_API_PROFILE);
            let token: String = token.into();
            let builder = server.reqwest_post(&path).header(
                http::header::COOKIE,
                format!("authorization=Bearer%3D{}%3B%20Secure%3B%20HttpOnly", token),
            );
            let res = send_builder::<ServerOutput, ServerErr>(builder, &input).await;
            trace!("RESPONSE: {res:#?}");
            res
        }

        #[cfg(test)]
        mod api {
            use std::sync::Arc;
            use std::time::Duration;

            use axum::Router;
            use axum::routing::post;
            use axum_test::TestServer;
            use http::header::SET_COOKIE;
            use test_log::test;
            use tokio::sync::Mutex;
            use tokio::time::sleep;
            use tracing::trace;

            use crate::controller;
            use crate::controller::app_state::AppState;
            use crate::controller::auth::{
                InviteToken, create_cookie, cut_cookie_full_with_expiration_encoded,
                test_extract_cookie_and_decode,
            };
            use crate::controller::clock::get_timestamp;
            use crate::controller::encode::{ResErr, ResErrUnauthorized};
            use crate::server::create_api_router;

            #[test(tokio::test)]
            async fn profile() {
                let current_time = get_timestamp();
                let time = Arc::new(Mutex::new(current_time));
                let app_state = AppState::new_testng(time).await;
                let my_app = create_api_router(app_state.clone()).with_state(app_state.clone());

                let server = TestServer::builder()
                    .http_transport()
                    .build(my_app)
                    .unwrap();

                {
                    let time = app_state.clock.now().await;
                    let exp = time + Duration::from_secs(60 * 30);
                    let invite = InviteToken::new("hey@hey.com", time.as_nanos());
                    let (token, _cookie) =
                        create_cookie(&app_state.settings.auth.secret, "hey", time).unwrap();
                    // let invite_token =
                    //     encode_token(&app_state.settings.auth.secret, invite).unwrap();
                    let res = controller::auth::route::profile::test_send(&server, token).await;
                    trace!("RESPONSE: {res:#?}");
                    assert!(matches!(
                        res.1,
                        Err(ResErr::Unauthorized(ResErrUnauthorized::Unauthorized))
                    ));


                    controller::auth::route::invite::test_send(&server, "hey1@hey.com")
                        .await
                        .1
                        .unwrap();
                    let invite = app_state
                        .db
                        .get_invite("hey1@hey.com", current_time.as_nanos())
                        .await
                        .unwrap();

                    let (cookies, res) = controller::auth::route::register::test_send(
                        &server,
                        "hey",
                        invite.token_raw,
                        "wowowowow123@",
                    )
                    .await;
                    let (token_raw, token) =
                        test_extract_cookie_and_decode(&app_state.settings.auth.secret, &cookies)
                            .unwrap();
                    assert_eq!(token.claims.username, "hey");

                    let res = controller::auth::route::profile::test_send(&server, token_raw).await;
                    trace!("RESPONSE: {res:#?}");
                    assert!(matches!(res.1, Ok(_)));

                    let session = app_state.db.add_session("uwu", "hey").await.unwrap();

                    let res = controller::auth::route::profile::test_send(&server, "uwu").await;
                    trace!("RESPONSE: {res:#?}");
                    let cookie = cut_cookie_full_with_expiration_encoded(
                        res.0.get(SET_COOKIE).unwrap().to_str().unwrap(),
                    );
                    // let cookie = test_extract_cookie(&res.0).unwrap();
                    assert_eq!(cookie, "DELETED");
                    let session = app_state.db.get_session("uwu").await;
                    assert!(session.is_err());
                }
            }
        }
    }
    pub mod invite_decode {
        use std::time::Duration;

        use thiserror::Error;
        use tracing::{error, trace};

        use crate::{
            controller::encode::{ResErr, send_web},
            path::PATH_API_INVITE_DECODE,
        };

        pub const PATH: &'static str = "/invite_decode";

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct Input {
            pub token: String,
        }

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct ServerOutput {
            pub email: String,
        }

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerOutput {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::encode_result;

                let bytes = encode_result::<ServerOutput, ServerErr>(&Ok(self));
                (axum::http::StatusCode::OK, bytes).into_response()
            }
        }

        #[derive(
            Debug,
            Error,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub enum ServerErr {
            #[error("internal server error")]
            ServerErr,

            #[error("jwt error")]
            JWT,

            #[error("jwt expired error")]
            JWTExpired,
        }

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerErr {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::{ResErr, encode_result};

                let status = match &self {
                    _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                };
                let bytes = encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                (status, bytes).into_response()
            }
        }

        // #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        // pub struct InviteToken {
        //     pub email: String,
        //     pub created_at: u128,
        //     pub exp: u64,
        // }
        //
        // impl InviteToken {
        //     pub fn new<S: Into<String>>(
        //         email: S,
        //         created_at: std::time::Duration,
        //         exp: std::time::Duration,
        //     ) -> Self {
        //         Self {
        //             email: email.into(),
        //             created_at: created_at.as_nanos(),
        //             exp: exp.as_secs(),
        //         }
        //     }
        // }

        pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
            send_web::<ServerOutput, ServerErr>(PATH_API_INVITE_DECODE, &input).await
        }

        #[cfg(feature = "ssr")]
        pub async fn server(
            axum::extract::State(app_state): axum::extract::State<
                crate::controller::app_state::AppState,
            >,
            multipart: axum::extract::Multipart,
        ) -> impl axum::response::IntoResponse {
            use crate::controller::encode::encode_server_output_custom;

            trace!("executing invite api");

            let wrap = async || {
                use crate::controller::{
                    auth::{InviteToken, decode_token},
                    encode::decode_multipart,
                };

                let input = decode_multipart::<Input, ServerErr>(multipart).await?;
                trace!("input!!!!!! {input:#?}");
                let time = app_state.clock.now().await;
                let exp = time + Duration::from_secs(60 * 30);

                let token =
                    decode_token::<InviteToken>(&app_state.settings.auth.secret, input.token, true)
                        .map_err(|err| match err.kind() {
                            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                                ServerErr::JWTExpired
                            }
                            _ => ServerErr::JWT,
                        })?;

                Result::<ServerOutput, ResErr<ServerErr>>::Ok(ServerOutput {
                    email: token.claims.email,
                })
            };
            trace!("1");
            let res = wrap().await;
            let res = encode_server_output_custom(res);
            res
        }

        #[cfg(test)]
        pub async fn test_send<Token: Into<String>>(
            server: &axum_test::TestServer,
            token: Token,
        ) -> (http::HeaderMap, Result<ServerOutput, ResErr<ServerErr>>) {
            use crate::{controller::encode::send_builder, path::PATH_API};

            let input = Input {
                token: token.into(),
            };
            let path = format!("{}{}", PATH_API, PATH_API_INVITE_DECODE);
            let builder = server.reqwest_post(&path);
            let res = send_builder::<ServerOutput, ServerErr>(builder, &input).await;
            trace!("RESPONSE: {res:#?}");
            res
        }

        #[cfg(test)]
        mod api {
            use std::sync::Arc;
            use std::time::Duration;

            use axum_test::TestServer;
            use test_log::test;
            use tokio::sync::Mutex;
            use tokio::time::sleep;
            use tracing::trace;

            use crate::controller;
            use crate::controller::app_state::AppState;
            use crate::controller::auth::{InviteToken, decode_token, encode_token};
            use crate::controller::clock::get_timestamp;
            use crate::server::create_api_router;

            #[test(tokio::test)]
            async fn token() {
                // let time = get_nanos();
                let time = get_timestamp();
                let exp = time + Duration::from_secs(2);
                let invite_token = InviteToken::new("hey@hey.com", time.as_nanos());
                let invite_token = encode_token("secret", invite_token).unwrap();
                sleep(Duration::from_secs(1)).await;
                let invite_claims =
                    decode_token::<InviteToken>("secret", &invite_token, true).unwrap();
                trace!("invite claims: {invite_claims:#?}");
                sleep(Duration::from_secs(2)).await;
                let time2 = get_timestamp();
                trace!("\n1: {}\n2: {}", time2.as_nanos(), exp.as_nanos());
                let invite_claims = decode_token::<InviteToken>("secret", &invite_token, true);
                assert!(invite_claims.is_err());
            }

            #[test(tokio::test)]
            async fn invite_decode() {
                let current_time = get_timestamp();
                let time = Arc::new(Mutex::new(current_time));
                let app_state = AppState::new_testng(time).await;
                let my_app = create_api_router(app_state.clone()).with_state(app_state.clone());

                let server = TestServer::builder()
                    .http_transport()
                    .build(my_app)
                    .unwrap();

                {
                    let time = app_state.clock.now().await;
                    let exp = time + Duration::from_secs(60 * 30);
                    let invite = InviteToken::new("hey@hey.com", time.as_nanos());
                    let invite_token =
                        encode_token(&app_state.settings.auth.secret, invite).unwrap();
                    let res =
                        controller::auth::route::invite_decode::test_send(&server, invite_token)
                            .await;
                    trace!("RESPONSE: {res:#?}");
                    res.1.unwrap();
                }
                // res.1.unwrap();
            }
        }
    }
    pub mod invite {
        use std::time::Duration;

        use thiserror::Error;
        use tracing::{error, trace};

        use crate::{
            controller::encode::{ResErr, send_web},
            path::PATH_API_INVITE,
        };

        pub const PATH: &'static str = "/invite";

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct Input {
            pub email: String,
        }

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct ServerOutput {}

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerOutput {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::encode_result;

                let bytes = encode_result::<ServerOutput, ServerErr>(&Ok(self));
                (axum::http::StatusCode::OK, bytes).into_response()
            }
        }

        #[derive(
            Debug,
            Error,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub enum ServerErr {
            #[error("internal server error")]
            ServerErr,

            #[error("jwt error")]
            JWT,
        }

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerErr {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::{ResErr, encode_result};

                let status = match &self {
                    _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                };
                let bytes = encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                (status, bytes).into_response()
            }
        }

        pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
            send_web::<ServerOutput, ServerErr>(PATH_API_INVITE, &input).await
        }

        #[cfg(feature = "ssr")]
        pub async fn server(
            axum::extract::State(app_state): axum::extract::State<
                crate::controller::app_state::AppState,
            >,
            multipart: axum::extract::Multipart,
        ) -> impl axum::response::IntoResponse {
            trace!("executing invite api");
            use tracing::debug;

            use crate::controller::encode::encode_server_output_custom;

            // tokio::time::sleep(Duration::from_secs(2)).await;
            let wrap = async || {
                use crate::controller::{
                    auth::{InviteToken, encode_token},
                    encode::decode_multipart,
                };

                let input = decode_multipart::<Input, ServerErr>(multipart).await?;
                trace!("input!!!!!! {input:#?}");
                let time = app_state.clock.now().await;
                let exp = time + Duration::from_secs(60 * 30);
                let invite = InviteToken::new(input.email.clone(), time.as_nanos());
                let invite_token = encode_token(&app_state.settings.auth.secret, invite)
                    .map_err(|_| ServerErr::JWT)?;

                trace!("invite token created: {invite_token}");

                let invite = app_state
                    .db
                    .add_invite(time.clone().as_nanos(), invite_token, input.email, exp.as_nanos())
                    .await;
                trace!("result {invite:?}");

                match invite {
                    Ok(invite) => {
                        use crate::path;

                        let link = format!(
                            "{}{}",
                            &app_state.settings.site.address,
                            path::link_reg(&invite.token_raw),
                        );
                        trace!("{link}");
                    }
                    Err(err) => {
                        debug!("invite failed {err}");
                    }
                }

                Result::<ServerOutput, ResErr<ServerErr>>::Ok(ServerOutput {})
            };
            trace!("1");
            let res = wrap().await;
            let res = encode_server_output_custom(res);
            res
        }

        #[cfg(test)]
        pub async fn test_send<Email: Into<String>>(
            server: &axum_test::TestServer,
            email: Email,
        ) -> (http::HeaderMap, Result<ServerOutput, ResErr<ServerErr>>) {
            use crate::{controller::encode::send_builder, path::PATH_API};

            let input = Input {
                email: email.into(),
            };
            let path = format!("{}{}", PATH_API, PATH_API_INVITE);
            let builder = server.reqwest_post(&path);
            let res = send_builder::<ServerOutput, ServerErr>(builder, &input).await;
            trace!("RESPONSE: {res:#?}");
            res
        }

        #[cfg(test)]
        mod api {
            use std::sync::Arc;
            use std::time::Duration;

            use axum_test::TestServer;
            use test_log::test;
            use tokio::sync::Mutex;
            use tokio::time::sleep;
            use tracing::trace;

            use crate::controller::app_state::AppState;
            use crate::controller::auth::{InviteToken, decode_token, encode_token};
            use crate::controller::clock::get_timestamp;
            use crate::db::invite::get_invite::GetInviteErr;
            use crate::server::create_api_router;

            #[test(tokio::test)]
            async fn token() {
                let time = get_timestamp();
                let exp = time + Duration::from_secs(2);
                let invite_token = InviteToken::new("hey@hey.com", time.as_nanos());
                let invite_token = encode_token("secret", invite_token).unwrap();
                sleep(Duration::from_secs(1)).await;
                let invite_claims =
                    decode_token::<InviteToken>("secret", &invite_token, true).unwrap();
                trace!("invite claims: {invite_claims:#?}");
                sleep(Duration::from_secs(2)).await;
                let time2 = get_timestamp();
                trace!("\n1: {}\n2: {}", time2.as_nanos(), exp.as_nanos());
                let invite_claims = decode_token::<InviteToken>("secret", &invite_token, true);
                assert!(invite_claims.is_err());
            }

            #[test(tokio::test)]
            async fn invite() {
                let current_time = get_timestamp();
                let time = Arc::new(Mutex::new(current_time));
                let app_state = AppState::new_testng(time).await;
                let my_app = create_api_router(app_state.clone()).with_state(app_state.clone());

                let server = TestServer::builder()
                    .http_transport()
                    .build(my_app)
                    .unwrap();

                {
                    let res =
                        crate::controller::auth::route::invite::test_send(&server, "hey1@hey.com")
                            .await;
                    assert!(matches!(
                        res.1,
                        Ok(crate::controller::auth::route::invite::ServerOutput {})
                    ));
                    let invite = app_state
                        .db
                        .get_invite("hey1@hey.com", current_time.as_nanos())
                        .await
                        .unwrap();
                    let res = crate::controller::auth::route::register::test_send(
                        &server,
                        "hey",
                        invite.token_raw,
                        "hey1@hey.com",
                    )
                    .await;
                    let res =
                        crate::controller::auth::route::invite::test_send(&server, "hey1@hey.com")
                            .await;
                    assert!(matches!(
                        res.1,
                        Ok(crate::controller::auth::route::invite::ServerOutput {})
                    ));
                    let invite = app_state.db.get_invite("hey1@hey.com", current_time.as_nanos()).await;
                    assert!(matches!(invite, Err(GetInviteErr::NotFound)));
                    let invite = app_state.db.get_invite("hey2@hey.com", current_time.as_nanos()).await;
                    assert!(matches!(invite, Err(GetInviteErr::NotFound)));
                    let res =
                        crate::controller::auth::route::invite::test_send(&server, "hey2@hey.com")
                            .await;
                    assert!(matches!(
                        res.1,
                        Ok(crate::controller::auth::route::invite::ServerOutput {})
                    ));
                    let invite = app_state.db.get_invite("hey2@hey.com", current_time.as_nanos()).await;
                    assert!(matches!(invite, Ok(_)));
                }
            }
        }
    }
    pub mod register {
        use thiserror::Error;
        use tracing::{error, trace};

        use crate::{
            controller::encode::{ResErr, send_web},
            path::PATH_API_REGISTER,
        };

        pub const PATH: &'static str = "/register";

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct Input {
            pub username: String,
            pub email_token: String,
            pub password: String,
        }

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct ServerOutput {
            pub username: String,
        }

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerOutput {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::encode_result;

                let bytes = encode_result::<ServerOutput, ServerErr>(&Ok(self));
                (axum::http::StatusCode::OK, bytes).into_response()
            }
        }

        #[derive(
            Debug,
            Error,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub enum ServerErr {
            #[error("email is already in use")]
            EmailTaken,

            #[error("username is already in use")]
            UsernameTaken,

            #[error("{0}")]
            EmailInvalid(String),

            #[error("{0}")]
            UsernameInvalid(String),

            #[error("{0}")]
            PasswordInvalid(String),

            #[error("jwt error")]
            JWT,

            #[error("jwt expired error")]
            JWTExpired,

            #[error("create cookie err")]
            CreateCookieErr,

            #[error("invite token not found")]
            TokenNotFound,

            #[error("internal server error")]
            ServerErr,
        }

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerErr {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::encode_result;

                let status = match self {
                    ServerErr::EmailInvalid(_)
                    | ServerErr::UsernameInvalid(_)
                    | ServerErr::PasswordInvalid(_) => axum::http::StatusCode::BAD_REQUEST,
                    ServerErr::EmailTaken | ServerErr::UsernameTaken => axum::http::StatusCode::OK,
                    _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                };
                let bytes = encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                (status, bytes).into_response()
            }
        }

        pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
            send_web::<ServerOutput, ServerErr>(PATH_API_REGISTER, &input).await
        }

        #[cfg(feature = "ssr")]
        pub async fn server(
            axum::extract::State(app_state): axum::extract::State<
                crate::controller::app_state::AppState,
            >,
            jar: axum_extra::extract::cookie::CookieJar,
            multipart: axum::extract::Multipart,
        ) -> impl axum::response::IntoResponse {
            use crate::controller::encode::encode_server_output_custom;
            use axum_extra::extract::{CookieJar, cookie::Cookie};

            let wrap = async || {
                use crate::{
                    controller::{
                        auth::{InviteToken, create_cookie, decode_token, hash_password},
                        encode::decode_multipart,
                        valid::auth::{proccess_email, proccess_password, proccess_username},
                    },
                    db::user::add_user::AddUserErr,
                };

                let input = decode_multipart::<Input, ServerErr>(multipart).await?;
                trace!("input!!!!!! {input:#?}");
                let token_raw = input.email_token;
                let time = app_state.clock.now().await;
                let _invite = app_state
                    .db
                    .get_invite_by_token(&token_raw )
                    .await
                    .map_err(|err| {
                        error!("failed to run use_invite {err}");
                        ServerErr::TokenNotFound
                    })?;

                let email_token =
                    decode_token::<InviteToken>(&app_state.settings.auth.secret, &token_raw, false)
                        .map_err(|err| match err.kind() {
                            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                                ServerErr::JWTExpired
                            }
                            _ => ServerErr::JWT,
                        })?;

                let username = proccess_username(input.username)
                    .map_err(|err| ServerErr::UsernameInvalid(err))?;
                let email = proccess_email(&email_token.claims.email)
                    .map_err(|err| ServerErr::EmailInvalid(err))?;
                let password = proccess_password(input.password, None)
                    .and_then(|pss| hash_password(pss).map_err(|_| "hasher error".to_string()))
                    .map_err(|err| ServerErr::PasswordInvalid(err))?;

                let user = app_state
                    .db
                    .add_user(username, email, password)
                    .await
                    .map_err(|err| match err {
                        AddUserErr::EmailIsTaken(_) => ServerErr::EmailTaken,
                        AddUserErr::UsernameIsTaken(_) => ServerErr::UsernameTaken,
                        _ => ServerErr::ServerErr,
                    })?;

                let result = app_state
                    .db
                    .use_invite(token_raw, time.as_nanos())
                    .await
                    .map_err(|err| {
                        error!("failed to run use_invite {err}");
                        ServerErr::ServerErr
                    })?;

                let (token, cookie) =
                    create_cookie(&app_state.settings.auth.secret, &user.username, time)
                        .map_err(|_| ServerErr::CreateCookieErr)?;

                let _session = app_state
                    .db
                    .add_session(token, &user.username)
                    .await
                    .map_err(|err| ServerErr::ServerErr)?;

                Result::<(String, ServerOutput), ResErr<ServerErr>>::Ok((
                    cookie,
                    ServerOutput {
                        username: user.username,
                    },
                ))
            };
            let res = match wrap().await {
                Ok((cookie, output)) => {
                    let res = Result::<ServerOutput, ResErr<ServerErr>>::Ok(output);
                    let cookies =
                        jar.add(Cookie::new(http::header::AUTHORIZATION.as_str(), cookie));
                    let res = encode_server_output_custom(res);
                    (cookies, res)
                }
                Err(err) => {
                    let res = Result::<ServerOutput, ResErr<ServerErr>>::Err(err);
                    let res = encode_server_output_custom(res);
                    (jar, res)
                }
            };
            res
        }

        #[cfg(test)]
        pub async fn test_send<
            Username: Into<String>,
            EmailToken: Into<String>,
            Password: Into<String>,
        >(
            server: &axum_test::TestServer,
            username: Username,
            email_token: EmailToken,
            password: Password,
        ) -> (http::HeaderMap, Result<ServerOutput, ResErr<ServerErr>>) {
            use crate::{
                controller::{self, encode::send_builder},
                path::{PATH_API, PATH_REGISTER},
            };

            let input = controller::auth::route::register::Input {
                username: username.into(),
                email_token: email_token.into(),
                password: password.into(),
            };
            let path = format!("{}{}", PATH_API, PATH_REGISTER);
            let builder = server.reqwest_post(&path);
            let res = send_builder::<
                controller::auth::route::register::ServerOutput,
                controller::auth::route::register::ServerErr,
            >(builder, &input)
            .await;
            trace!("RESPONSE: {res:#?}");
            res
        }

        #[cfg(test)]
        mod api {
            use std::sync::Arc;
            use std::time::Duration;

            use axum::Router;
            use axum::routing::post;
            use axum_test::TestServer;
            use test_log::test;
            use tokio::sync::Mutex;
            use tracing::trace;

            use crate::controller::app_state::AppState;
            use crate::controller::auth::test_extract_cookie_and_decode;
            use crate::controller::clock::get_timestamp;
            use crate::controller::encode::ResErr;
            use crate::server::create_api_router;

            #[test(tokio::test)]
            async fn register() {
                let current_time = get_timestamp();
                let time = Arc::new(Mutex::new(current_time));
                let app_state = AppState::new_testng(time).await;
                let secret = app_state.settings.auth.secret.clone();
                let db = app_state.db.clone();
                let my_app = create_api_router(app_state.clone()).with_state(app_state.clone());
                let server = TestServer::builder()
                    .http_transport()
                    .build(my_app)
                    .unwrap();

                {
                    let res =
                        crate::controller::auth::route::invite::test_send(&server, "hey1@hey.com")
                            .await;
                    res.1.unwrap();

                    let invite = db.get_invite("hey1@hey.com", current_time.as_nanos()).await.unwrap();

                    let res = crate::controller::auth::route::register::test_send(
                        &server,
                        "hey",
                        "broken",
                        "hey1@hey.com",
                    )
                    .await;
                    assert!(matches!(
                        res.1,
                        Err(ResErr::ServerErr(
                            crate::controller::auth::route::register::ServerErr::TokenNotFound
                        ))
                    ));

                    let token =
                        test_extract_cookie_and_decode(&app_state.settings.auth.secret, &res.0);
                    assert!(token.is_none());


                    let res = crate::controller::auth::route::register::test_send(
                        &server,
                        "hey",
                        invite.token_raw,
                        "hey1@hey.com",
                    )
                    .await;

                    assert!(res.1.is_ok());

                    let (token_raw, token) =
                        test_extract_cookie_and_decode(&app_state.settings.auth.secret, &res.0)
                            .unwrap();
                    assert_eq!(token.claims.username, "hey");
                }
            }
        }
    }
    pub mod login {
        use thiserror::Error;
        use tracing::{error, trace};

        use crate::{
            controller::encode::{ResErr, send_web},
            path::PATH_API_LOGIN,
        };

        pub const PATH: &'static str = "/login";

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct Input {
            pub email: String,
            pub password: String,
        }

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct ServerOutput {
            pub username: String,
        }

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerOutput {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::encode_result;

                let bytes = encode_result::<ServerOutput, ServerErr>(&Ok(self));

                trace!("sending body: {bytes:?}");
                (axum::http::StatusCode::OK, bytes).into_response()
            }
        }

        #[derive(
            Debug,
            Error,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub enum ServerErr {
            #[error("create cookie err")]
            CreateCookieErr,

            #[error("incorrect email or password")]
            Incorrect,

            #[error("internal server error")]
            ServerErr,
        }

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerErr {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::encode_result;

                let status = match self {
                    // ServerErr::DecodeErr(_) => axum::http::StatusCode::BAD_REQUEST,
                    ServerErr::Incorrect => axum::http::StatusCode::OK,
                    ServerErr::ServerErr | ServerErr::CreateCookieErr => {
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR
                    }
                };
                let bytes = encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                trace!("sending body: {bytes:?}");
                (status, bytes).into_response()
            }
        }

        pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
            send_web::<ServerOutput, ServerErr>(PATH_API_LOGIN, &input).await
        }

        #[cfg(feature = "ssr")]
        pub async fn server(
            axum::extract::State(app_state): axum::extract::State<
                crate::controller::app_state::AppState,
            >,
            jar: axum_extra::extract::cookie::CookieJar,
            multipart: axum::extract::Multipart,
        ) -> impl axum::response::IntoResponse {
            use axum_extra::extract::cookie::Cookie;

            use crate::controller::encode::encode_server_output_custom;

            trace!("yo wtf??");
            let result = (async || {
                use crate::controller::{auth::create_cookie, encode::decode_multipart};

                let input = decode_multipart::<Input, ServerErr>(multipart).await?;
                trace!("input!!!!!! {input:#?}");
                let user = app_state
                    .db
                    .get_user_by_email(input.email)
                    .await
                    .map_err(|_| ServerErr::Incorrect)?;
                verify_password(input.password, user.password).map_err(|_| ServerErr::Incorrect)?;
                let time = app_state.clock.now().await;
                let (token, cookie) =
                    create_cookie(&app_state.settings.auth.secret, &user.username, time)
                        .map_err(|_| ServerErr::CreateCookieErr)?;
                let _session = app_state
                    .db
                    .add_session(token, &user.username)
                    .await
                    .map_err(|err| ServerErr::ServerErr)?;

                let output = ServerOutput {
                    username: user.username,
                };

                Result::<(String, ServerOutput), ResErr<ServerErr>>::Ok((cookie, output))
            })()
            .await;

            let jar = match result.as_ref() {
                Ok((cookie, _)) => jar.add(Cookie::new(
                    http::header::AUTHORIZATION.as_str(),
                    cookie.clone(),
                )),
                Err(_) => jar,
            };
            let output = result.map(|v| v.1);
            (jar, encode_server_output_custom(output))
        }

        #[cfg(feature = "ssr")]
        pub fn verify_password<T: AsRef<[u8]>, S2: AsRef<str>>(
            password: T,
            hash: S2,
        ) -> Result<(), argon2::password_hash::Error> {
            use argon2::{Argon2, PasswordHash, PasswordVerifier};

            let password = password.as_ref();
            let hash = hash.as_ref();
            PasswordHash::new(hash)
                .and_then(|hash| Argon2::default().verify_password(password, &hash))
        }

        #[cfg(test)]
        pub async fn test_send<Email: Into<String>, Password: Into<String>>(
            server: &axum_test::TestServer,
            email: Email,
            password: Password,
        ) -> (http::HeaderMap, Result<ServerOutput, ResErr<ServerErr>>) {
            use crate::{controller::encode::send_builder, path::PATH_API};

            let input = Input {
                email: email.into(),
                password: password.into(),
            };
            let path = format!("{}{}", PATH_API, PATH_API_LOGIN);
            let builder = server.reqwest_post(&path);
            let res = send_builder::<ServerOutput, ServerErr>(builder, &input).await;
            trace!("RESPONSE: {res:#?}");
            res
        }

        #[cfg(test)]
        mod api {
            use std::sync::Arc;
            use std::time::Duration;

            use axum::extract::{FromRequest, Multipart, State};
            use axum_test::TestServer;
            use test_log::test;
            use tokio::sync::Mutex;

            use crate::{
                controller::{
                    self, app_state::AppState, auth::test_extract_cookie_and_decode,
                    clock::get_timestamp, encode::ResErr,
                },
                server::create_api_router,
            };

            #[test(tokio::test)]
            async fn login() {
                let current_time = get_timestamp();
                let time = Arc::new(Mutex::new(current_time));
                let app_state = AppState::new_testng(time.clone()).await;
                let db = app_state.db.clone();
                let my_app = create_api_router(app_state.clone()).with_state(app_state.clone());
                let server = TestServer::builder()
                    .http_transport()
                    .build(my_app)
                    .unwrap();

                let res = controller::auth::route::invite::test_send(&server, "hey@hey.com").await;
                res.1.unwrap();
                let invite = db.get_invite("hey@hey.com", current_time.as_nanos()).await.unwrap();

                let res = crate::controller::auth::route::register::test_send(
                    &server,
                    "hey",
                    invite.token_raw,
                    "hey1@hey.com",
                )
                .await;
                assert!(matches!(
                    res.1,
                    Ok(crate::controller::auth::route::register::ServerOutput { username })
                ));
                {
                    *time.lock().await += Duration::from_secs(1);
                }
                let res = controller::auth::route::login::test_send(
                    &server,
                    "hey@hey.com",
                    "hey1@hey.com",
                )
                .await;
                let (token_raw, token) =
                    test_extract_cookie_and_decode(&app_state.settings.auth.secret, &res.0)
                        .unwrap();
                assert_eq!(token.claims.username, "hey");
                let session = app_state.db.get_session(&token_raw).await.unwrap();

                let res =
                    crate::controller::auth::route::invite::test_send(&server, "hey2@hey.com")
                        .await;
                res.1.unwrap();
                let invite = db.get_invite("hey2@hey.com", current_time.as_nanos()).await.unwrap();
                let res = crate::controller::auth::route::register::test_send(
                    &server,
                    "hey",
                    invite.token_raw,
                    "hey1@hey.com",
                )
                .await;
                assert!(matches!(
                    res.1,
                    Err(ResErr::ServerErr(
                        crate::controller::auth::route::register::ServerErr::UsernameTaken
                    ))
                ));
            }
        }
    }
}
