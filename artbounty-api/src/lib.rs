#[cfg(feature = "ssr")]
pub mod utils {
    use axum::extract::Multipart;
    use bytecheck::CheckBytes;
    use rkyv::{
        Deserialize,
        api::high::{HighDeserializer, HighValidator},
    };
    use thiserror::Error;

    pub async fn decode<T, A>(mut multipart: Multipart) -> Result<T, DecodeErr>
    where
        A: rkyv::Portable
            + for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<T, HighDeserializer<rkyv::rancor::Error>>,
    {
        let mut bytes = Vec::<u8>::new();
        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|_| DecodeErr::NextFieldFailed)?
        {
            if field.name().map(|name| name == "data").unwrap_or_default() {
                bytes = field
                    .bytes()
                    .await
                    .map_err(|_| DecodeErr::FieldToBytesFailed)?
                    .to_vec();
            }
        }

        if bytes.is_empty() {
            return Err(DecodeErr::MissingDataField);
        }

        let archived =
            rkyv::access::<A, rkyv::rancor::Error>(&bytes).map_err(|_| DecodeErr::RkyvAccessErr)?;
        let args = rkyv::deserialize::<T, rkyv::rancor::Error>(archived)
            .map_err(|_| DecodeErr::RkyvErr)?;
        Ok(args)
    }

    #[derive(
        Error,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        rkyv::Archive,
        rkyv::Serialize,
        rkyv::Deserialize,
    )]
    pub enum DecodeErr {
        #[error("failed to convert data field to bytes")]
        FieldToBytesFailed,

        #[error("failed to parse multipart")]
        NextFieldFailed,

        #[error("data field is missing in multipart")]
        MissingDataField,

        #[error("rkyv failed to access")]
        RkyvAccessErr,

        #[error("rkyv failed to encode")]
        RkyvErr,
    }
}

pub mod api {

    // #[derive(
    //     Debug,
    //     Error,
    //     Clone,
    //     Default,
    //     //strum::Display,
    //     strum::EnumString,
    //     //strum::Display,
    //     //strum::EnumString,
    //     serde::Serialize,
    //     serde::Deserialize,
    //     rkyv::Archive,
    //     rkyv::Serialize,
    //     rkyv::Deserialize,
    // )]
    // pub enum MidErr<
    // // Err: std::default::Default + std::error::Error + std::fmt::Debug + std::fmt::Display + rkyv::Archive + rkyv::Serialize + rkyv::Deserialize + serde::Deserialize<_> + serde::Serialize
    // Err : std::default::Default
    // >   {

    //     #[default]
    //     #[error("internal server err")]
    //     ServerErr,

    //     #[error("account does not meet required permissions")]
    //     Unauthorized,

    //     // #[strum(disabled)]
    //     #[error("response err {0}")]
    //     ReqErr( Err)
    // }
    // // impl <T: Default> From<&str> for MidErr<T> {
    // //     fn from(value: &str) -> Self {
    // //         match value {
    // //             "Unauthorized" => Self::Unauthorized,
    // //             _ => Self::ServerErr,
    // //         }
    // //     }
    // // }

    // pub struct User {

    // }

    // pub mod profile {
    //     use leptos::{prelude::*, server};
    //     use server_fn::codec::Rkyv;
    //     use thiserror::Error;
    //     use tracing::trace;

    //     pub struct ApiProfile {
    //         pub username: String,
    //     }

    //     #[derive(
    //         Debug,
    //         Clone,
    //         serde::Serialize,
    //         serde::Deserialize,
    //         rkyv::Archive,
    //         rkyv::Serialize,
    //         rkyv::Deserialize,
    //     )]
    //     pub struct Wtf {

    //     }

    //     // async fn auth(mut req: axum::extract::Request, next: axum::middleware::Next) -> Result<axum::response::Response, axum::http::StatusCode> {

    //     //     Err(axum::http::StatusCode::FORBIDDEN)
    //     // }
    //     // async fn auth(mut req: axum::extract::Request, next: axum::middleware::Next) -> Result<axum::response::Response, ServerFnError<ProfileErr>> {

    //     //     Err(axum::http::StatusCode::FORBIDDEN)
    //     // }

    //     #[server(
    //         prefix = "/api",
    //         endpoint = "profile",
    //         input = Rkyv,
    //         output = Rkyv,
    //     )]
    //     // #[middleware(axum::middleware::from_fn(auth))]
    //     #[middleware(crate::middleware::auth::AuthLayer)]
    //     pub async fn profile() -> Result<(), ServerFnError<ProfileErr>> {
    //         use axum::http::Request;
    //         use http::HeaderMap;
    //         use leptos_axum::extract;
    //         use http::Extensions;
    //         // let o = axum::middleware::from_fn(auth);

    //         // let header: HeaderMap = extract().await.unwrap();
    //         let header: Extensions = extract().await.map_err(|_| ProfileErr::ServerErr)?;
    //         let r = header.get::<String>();
    //         trace!("r5 {r:#?}");
    //         // let a = header.get::<jsonwebtoken::TokenData<crate::auth::Claims>>();
    //         // trace!("a {a:#?}");

    //         Ok(())
    //     }

    //     async fn profile_inner() {}

    //     #[derive(
    //         Debug,
    //         Error,
    //         Clone,
    //         strum::Display,
    //         strum::EnumString,
    //         serde::Serialize,
    //         serde::Deserialize,
    //         rkyv::Archive,
    //         rkyv::Serialize,
    //         rkyv::Deserialize,
    //     )]
    //     pub enum ProfileErr {
    //         ServerErr,
    //     }

    //     #[cfg(test)]
    //     pub mod test_profile {
    //         use artbounty_db::db::DB;
    //         use http::{Extensions, HeaderMap, Method, Request, Uri, Version, request::Parts};
    //         use leptos::prelude::provide_context;
    //         use test_log::test;

    //         use crate::api::profile::profile;

    //         #[test(tokio::test)]
    //         async fn test_profile() {
    //             // DB.connect().await;
    //             // DB.migrate().await.unwrap();
    //             // let builder = Request::builder();
    //             // let r = builder.method(Method::POST).uri("http://localhost:3000/api/login").version(Version::HTTP_11).extension(Extensions::new()).header("Bearer", "foo").body(()).unwrap();
    //             // let (parts, ()) = r.into_parts();
    //             // provide_context::<Parts>(parts);

    //             // provide_context(Parts {
    //             //     version: Version::HTTP_11,
    //             //     extensions: Extensions::new(),
    //             //     headers: {
    //             //         let map = HeaderMap::new();
    //             //         // map.insert(key, val)
    //             //         map
    //             //     },
    //             //     method: Method::POST,
    //             //     uri: Uri::from_static("http://localhost:3000/api/login"),
    //             //     ..Default::default()
    //             // });

    //             // let result = profile().await.unwrap();
    //         }
    //     }
    // }

    pub mod login {

        use thiserror::Error;
        use tracing::trace;

        #[derive(
            Debug,
            Clone,
            // serde::Serialize,
            // serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct Args {
            pub email: String,
            pub password: String,
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
        pub enum Err {
            #[error("internal server error")]
            ServerErr,

            #[error("perfect")]
            Incorrect,
        }

        pub async fn post(args: Args) -> Result<(), Err> {
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&args)
                .unwrap()
                .to_vec();
            let part = reqwest::multipart::Part::bytes(bytes);
            let form = reqwest::multipart::Form::new().part("data", part);
            let res: Result<reqwest::Response, reqwest::Error> = reqwest::Client::new()
                .post("http://localhost:3000/api2/login")
                .multipart(form)
                .send()
                .await;
            trace!("{:#?}", res);
            Ok(())
        }

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for Err {
            fn into_response(self) -> axum::response::Response {
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&Err::<(), Err>(self))
                    .unwrap()
                    .to_vec();
                // let body = axum::body::Body::from(bytes);
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, bytes).into_response()
            }
        }

        #[cfg(feature = "ssr")]
        pub async fn server(
            multipart: axum::extract::Multipart,
        ) -> impl axum::response::IntoResponse {
            use crate::{api, utils};
            //
            // use artbounty_db::db::DB;
            // use axum::http::StatusCode;
            // use tracing::trace;
            // let mut bytes = Vec::<u8>::new();
            // while let Some(field) = multipart
            //     .next_field()
            //     .await
            //     .map_err(|_| api::login::Err::ServerErr)?
            // {
            //     if field.name().map(|name| name == "data").unwrap_or_default() {
            //         bytes = field
            //             .bytes()
            //             .await
            //             .map_err(|_| api::login::Err::ServerErr)?
            //             .to_vec();
            //     }
            // }

            // if bytes.is_empty() {
            //     return Err(api::login::Err::ServerErr);
            // }

            let args = utils::decode::<api::login::Args, api::login::ArchivedArgs>(multipart).await;
            // let archived =
            //     rkyv::access::<api::login::ArchivedArgs, rkyv::rancor::Error>(&*bytes).unwrap();
            // // let archived = rkyv::access::<Example, rkyv::rancor::Error>(&*bytes).unwrap();
            // let args =
            //     rkyv::deserialize::<api::login::Args, rkyv::rancor::Error>(archived).unwrap();

            trace!("args: {args:#?}");

            // let response = expect_context::<ResponseOptions>();
            // response.set_status(Sta);
            // response.;
            // trace!("1");

            // let user = DB
            //     .get_user_by_email(email)
            //     .await
            //     .map_err(|_| LoginErr::Incorrect)?;
            // let password_hash = user.password;
            // let username = user.username;

            // trace!("1.5");
            // let password_correct = verify_password(password, password_hash);
            // if !password_correct {
            //     return Err(ServerFnError::from(LoginErr::Incorrect));
            // }

            // trace!("2");

            // let time = get_nanos();
            // let (token, cookie) =
            //     create_cookie("secret", username.clone(), time).map_err(|_| LoginErr::ServerErr)?;
            // // let token = encode_token("secret", Claims::new(username, time)).map_err(|_| LoginErr::ServerErr)?;
            // // trace!("2.5");
            // // let cookie = format!("Bearer={token}; Secure; HttpOnly");
            // let r = DB.add_session(token.clone(), username).await;
            // trace!("r {r:#?}");
            // r.map_err(|_| LoginErr::ServerErr)?;

            // trace!("3");
            // response.append_header(
            //     http::header::SET_COOKIE,
            //     HeaderValue::from_str(&cookie).unwrap(),
            // );
            // Ok(())
            http::StatusCode::FORBIDDEN
        }

        // use crate::auth::verify_password;
        // #[server(
        //     prefix = "/api",
        //     endpoint = "login",
        //     input = Rkyv,
        //     output = Rkyv,
        // )]
        // // #[middleware(crate::middleware::auth::AuthLayer)]
        // pub async fn login(
        //     email: String,
        //     password: String,
        // ) -> Result<String, ServerFnError<LoginErr>> {
        //     use crate::auth::{Claims, create_cookie, encode_token, get_nanos, verify_password};
        //     use artbounty_db::db::{AddUserErr, DB};
        //     use leptos_axum::ResponseOptions;

        //     let response = expect_context::<ResponseOptions>();
        //     // response.set_status(Sta);
        //     // response.;
        //     trace!("1");

        //     let user = DB
        //         .get_user_by_email(email)
        //         .await
        //         .map_err(|_| LoginErr::Incorrect)?;
        //     let password_hash = user.password;
        //     let username = user.username;

        //     trace!("1.5");
        //     let password_correct = verify_password(password, password_hash);
        //     if !password_correct {
        //         return Err(ServerFnError::from(LoginErr::Incorrect));
        //     }

        //     trace!("2");

        //     let time = get_nanos();
        //     let (token, cookie) =
        //         create_cookie("secret", username.clone(), time).map_err(|_| LoginErr::ServerErr)?;
        //     // let token = encode_token("secret", Claims::new(username, time)).map_err(|_| LoginErr::ServerErr)?;
        //     // trace!("2.5");
        //     // let cookie = format!("Bearer={token}; Secure; HttpOnly");
        //     let r = DB.add_session(token.clone(), username).await;
        //     trace!("r {r:#?}");
        //     r.map_err(|_| LoginErr::ServerErr)?;

        //     trace!("3");
        //     response.append_header(
        //         http::header::SET_COOKIE,
        //         HeaderValue::from_str(&cookie).unwrap(),
        //     );

        //     // response.append_header(
        //     //     http::header::SET_COOKIE,
        //     //     HeaderValue::from_str("authorization=yowza; Secure; HttpOnly").unwrap(),
        //     // );

        //     // use leptos_axum::{extract, extract_with_state};
        //     // use tokio::time::sleep;

        //     // sleep(Duration::from_secs(3)).await;
        //     // let res = DB.add_user(username, email, password).await.map_err(|err| match err {
        //     //     AddUserErr::Email(_) => CreateErr::Email,
        //     //     _ => CreateErr::ServerErr
        //     // }).map_err(MidErr::ReqErr)?;
        //     // let (db):(State<DbKv>) = extract_with_state().await?;
        //     Ok("login".to_string())
        // }
        // #[derive(
        //     Debug,
        //     Error,
        //     Clone,
        //     // Default,
        //     strum::Display,
        //     strum::EnumString,
        //     //strum::Display,
        //     //strum::EnumString,
        //     serde::Serialize,
        //     serde::Deserialize,
        //     rkyv::Archive,
        //     rkyv::Serialize,
        //     rkyv::Deserialize,
        // )]
        // pub enum LoginErr {
        //     // #[default]
        //     // #[error("internal server error")]
        //     ServerErr,
        //     Incorrect,
        //     // #[error("invalid email")]
        //     // Email,
        // }
    }
    pub mod register {
        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct Res {
            pub email: String,
        }
    }
    //     // use artbounty_db::db::DbKv;
    //     use leptos::{prelude::*, server};
    //     use server_fn::codec::{Json, Rkyv, RkyvEncoding};
    //     use std::{string::ToString, time::Duration};
    //     use thiserror::Error;

    //     // #[derive(
    //     //     Debug,
    //     //     Error,
    //     //     Clone,
    //     //     serde::Serialize,
    //     //     serde::Deserialize,
    //     //     rkyv::Archive,
    //     //     rkyv::Serialize,
    //     //     rkyv::Deserialize,
    //     // )]
    //     // pub struct User {
    //     //     pub
    //     // }

    //     // use tower::timeout::TimeoutLayer;

    //     // use crate::api::MidErr;

    //     // use crate::middleware::MidErr;
    //     // static a: std::sync::Arc<tower_governor::governor::GovernorConfig> = std::sync::Arc::new(tower_governor::governor::GovernorConfig::default());
    //     // use strum::{Display, EnumString};

    //     #[server(
    //         prefix = "/api",
    //         endpoint = "register",
    //         input = Rkyv,
    //         output = Rkyv,
    //     )]
    //     // #[middleware(crate::middleware::auth::AuthLayer)]
    //     // #[middleware(tower_http::timeout::TimeoutLayer::new(std::time::Duration::from_secs(2)))]
    //     // #[middleware((TimeoutLayer::new(Duration::from_secs(5))))]
    //     // #[middleware((TimeoutLayer::new(Duration::from_secs(5)), crate::middleware::log::LogLayer))]
    //     // #[middleware(crate::middleware::log::LogLayer)]
    //     // #[middleware(tower_governor::GovernorLayer{config: std::sync::Arc::new(tower_governor::governor::GovernorConfig::default())})]
    //     pub async fn register(
    //         username: String,
    //         email: String,
    //         password: String,
    //     ) -> Result<RegisterResult, ServerFnError<RegisterErr>> {
    //         use crate::auth::hash_password;
    //         use artbounty_db::db::{AddUserErr, DB};
    //         use artbounty_shared::auth::{proccess_email, proccess_password, proccess_username};
    //         use leptos_axum::{extract, extract_with_state};
    //         use tokio::time::sleep;

    //         // sleep(Duration::from_secs(3)).await;

    //         let username =
    //             proccess_username(username).map_err(|err| RegisterErr::UsernameInvalid(err))?;
    //         let email = proccess_email(email).map_err(|err| RegisterErr::EmailInvalid(err))?;
    //         let password = proccess_password(password, None)
    //             .and_then(|pss| hash_password(pss).map_err(|_| "hasher error".to_string()))
    //             .map_err(|err| RegisterErr::PasswordInvalid(err))?;

    //         let res = DB
    //             .add_user(username, email, password)
    //             .await
    //             .map_err(|err| match err {
    //                 // AddUserErr::EmailInvalid(_) => CreateErr::EmailInvalid,
    //                 AddUserErr::EmailIsTaken(_) => RegisterErr::EmailTaken,
    //                 AddUserErr::UsernameIsTaken(_) => RegisterErr::UsernameTaken,
    //                 // AddUserErr::UsernameInvalid(_) => CreateErr::UsernameInvalid,
    //                 _ => RegisterErr::ServerErr,
    //             })?;

    //         // let (db):(State<DbKv>) = extract_with_state().await?;
    //         let result = RegisterResult {
    //             email: res.email.to_string(),
    //         };
    //         Ok(result)
    //         // Ok(())
    //     }

    //     // #[cfg(feature = "ssr")]
    //     // pub async fn register_inner() {

    //     // }

    //     #[derive(
    //         Debug,
    //         Error,
    //         Clone,
    //         // Default,
    //         strum::Display,
    //         strum::EnumString,
    //         //strum::Display,
    //         //strum::EnumString,
    //         serde::Serialize,
    //         serde::Deserialize,
    //         rkyv::Archive,
    //         rkyv::Serialize,
    //         rkyv::Deserialize,
    //     )]
    //     pub enum RegisterErr {
    //         // #[default]
    //         // #[error("internal server error")]
    //         ServerErr,

    //         // #[error("invalid email")]
    //         EmailInvalid(String),
    //         EmailTaken,
    //         UsernameTaken,
    //         UsernameInvalid(String),
    //         PasswordInvalid(String),
    //     }

    //     // pub fn err_to_string(err: RegisterErr) {
    //     //     match err
    //     // }

    //     #[cfg(test)]
    //     mod test_register {
    //         use crate::api::register::register;
    //         use artbounty_db::db::{AddUserErr, DB};
    //         use test_log::test;
    //         use tracing::trace;

    //         #[test(tokio::test)]
    //         async fn test_api_register() {
    //             // DB.connect().await;
    //             // DB.migrate().await.unwrap();
    //             // let r = register("hey".to_string(), "hey@hey.com".to_string(), "hey".to_string()).await.unwrap();
    //             // trace!("API RESULT: {r:#?}");
    //         }
    //     }
    // }
}

#[cfg(feature = "ssr")]
pub mod auth {
    use std::time::{SystemTime, UNIX_EPOCH};

    use argon2::PasswordHasher;
    use argon2::{
        Argon2, PasswordHash, PasswordVerifier,
        password_hash::{self, SaltString, rand_core::OsRng},
    };
    use artbounty_db::db::DB;
    use http::HeaderMap;
    use jsonwebtoken::{
        Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
    };
    use leptos_axum::extract;
    use server_fn::ServerFnError;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct Claims {
        username: String,
        created_at: u128,
        exp: u64,
    }

    impl Claims {
        pub fn new<S: Into<String>>(username: S, time: u128) -> Self {
            let username: String = username.into();
            Claims {
                username,
                created_at: time,
                exp: 0,
            }
        }
    }

    pub fn get_nanos() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    pub fn verify_password<T: AsRef<[u8]>, S2: AsRef<str>>(password: T, hash: S2) -> bool {
        let password = password.as_ref();
        let hash = hash.as_ref();
        PasswordHash::new(hash)
            .and_then(|hash| Argon2::default().verify_password(password, &hash))
            .is_ok()
    }

    pub fn hash_password<S: Into<String>>(password: S) -> Result<String, password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password = password.into();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        Ok(password_hash)
    }

    // fn foo<S: ToOwned<Owned = String>>(bar: S) -> String {
    //     bar.to_owned()
    // }

    pub fn encode_token<Key: AsRef<[u8]>>(
        key: Key,
        claims: Claims,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let header = Header::new(Algorithm::HS512);
        let key = EncodingKey::from_secret(key.as_ref());

        encode(&header, &claims, &key)
    }

    pub fn decode_token<Key: AsRef<[u8]>, S: AsRef<str>>(
        key: Key,
        token: S,
    ) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
        let token = token.as_ref();
        let key = DecodingKey::from_secret(key.as_ref());
        let mut validation = Validation::new(Algorithm::HS512);
        validation.validate_exp = false;

        decode::<Claims>(token, &key, &validation)
    }

    pub fn create_cookie<Key: AsRef<[u8]>, S: Into<String>>(
        key: Key,
        username: S,
        time: u128,
    ) -> Result<(String, String), jsonwebtoken::errors::Error> {
        let key = key.as_ref();
        let token = encode_token(key, Claims::new(username, time))?;
        let cookie = format!("Bearer={token}; Secure; HttpOnly");
        Ok((token, cookie))
    }

    pub fn verify_cookie<Key: AsRef<[u8]>, Cookie: AsRef<str>>(
        key: Key,
        cookie: Cookie,
    ) -> Result<(String, TokenData<Claims>), VerifyCookieErr> {
        use biscotti::{Processor, ProcessorConfig, RequestCookies};
        let processor: Processor = ProcessorConfig::default().into();
        let key = key.as_ref();
        let cookie = cookie.as_ref();
        RequestCookies::parse_header(cookie, &processor)
            .ok()
            .and_then(|cookies| cookies.get("Bearer"))
            .ok_or(VerifyCookieErr::CookieNotFound)
            .and_then(|cookie| {
                let token = cookie.value();
                decode_token(key, token)
                    .map(|data| (token.to_string(), data))
                    .map_err(VerifyCookieErr::from)
            })
    }

    pub async fn auth() -> Result<TokenData<Claims>, ServerFnError> {
        let header: HeaderMap = extract().await.unwrap();
        let Some((token, data)) = header
            .get(http::header::COOKIE)
            .and_then(|h| h.to_str().ok())
            .and_then(|cookie| verify_cookie("secret", cookie).ok())
        else {
            // trace!("AUTH BLOCK: {}", req.uri().to_string());
            return Err(ServerFnError::ServerError("unauthorized".to_string()));
        };
        let session = DB.get_session(token).await;

        Ok(data)
    }

    #[derive(
        Debug,
        Clone,
        thiserror::Error,
        // serde::Serialize,
        // serde::Deserialize,
        // rkyv::Archive,
        // rkyv::Serialize,
        // rkyv::Deserialize,
    )]
    pub enum VerifyCookieErr {
        #[error("jwt error {0}")]
        JWT(#[from] jsonwebtoken::errors::Error),

        #[error("Bearer cookie not found")]
        CookieNotFound,
    }

    #[cfg(test)]
    mod login_auth {
        use std::time::{SystemTime, UNIX_EPOCH};

        use crate::auth::{create_cookie, verify_cookie};

        use test_log::test;
        use tracing::trace;

        #[test]
        fn test_login() {
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let (_token, cookie) = create_cookie("secret", "hey", time).unwrap();
            let cookie = cookie.split(";").next().unwrap();
            trace!("cookie {cookie:#?}");
            let claims = verify_cookie("secret", cookie).unwrap();
            trace!("claims {claims:#?}");

            // trace!("time {time}");
            // // let time = Timestamp::now();
            // let claims = Claims::new("hey", time);
            // let token = encode_token("secret", claims).unwrap();
            // trace!("\ntoken: {token}");
            // let decoded_token = decode_token("secret", &token).unwrap();
            // trace!("\ndecoded: {decoded_token:?}");
            // // let token2 = encode_token("secret", time).unwrap();
        }
    }
}

#[cfg(feature = "ssr")]
pub mod middleware {

    pub mod auth {
        use std::{
            pin::Pin,
            task::{Context, Poll},
        };

        use axum::http::{Request, Response};

        // use biscotti::{Processor, ProcessorConfig, RequestCookies};
        use pin_project_lite::pin_project;
        use server_fn::ServerFnError;
        use thiserror::Error;

        use tower::{Layer, Service};

        use tracing::trace;

        use crate::auth::verify_cookie;

        // use crate::api::MidErr;

        #[derive(Error, Debug)]
        pub enum KaboomErr {
            #[error("boom")]
            Boom,
        }

        #[derive(Debug, Clone)]
        pub struct AuthLayer;

        impl<S> Layer<S> for AuthLayer {
            type Service = AuthService<S>;

            fn layer(&self, inner: S) -> Self::Service {
                AuthService { inner }
            }
        }

        #[derive(Debug, Clone)]
        pub struct AuthService<T> {
            inner: T,
        }

        impl<S, ReqBody, ResBody, Err> Service<Request<ReqBody>> for AuthService<S>
        where
            S: Service<Request<ReqBody>, Response = Response<ResBody>, Error = ServerFnError<Err>>,
            ResBody: Default + std::fmt::Debug,
            Err: std::fmt::Debug,
            ReqBody: std::fmt::Debug,
        {
            type Response = S::Response;
            type Error = S::Error;
            type Future = AuthServiceFuture<S::Future>;

            fn poll_ready(
                &mut self,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), Self::Error>> {
                self.inner.poll_ready(cx)
            }

            fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
                let Some((_token, _data)) = req
                    .headers()
                    .get(http::header::COOKIE)
                    .and_then(|h| h.to_str().ok())
                    .and_then(|cookie| verify_cookie("secret", cookie).ok())
                else {
                    trace!("AUTH BLOCK: {}", req.uri().to_string());
                    return AuthServiceFuture::Unauthorized;
                };

                // let session = DB.get_session(token).into_future().; // CANT

                {
                    trace!("AUTH PASS: {}", req.uri().to_string());
                    req.extensions_mut()
                        .insert(String::from("wooooooooooooooooow"));
                    // req.extensions_mut().insert(data);
                }
                {
                    let r = req.extensions().get::<String>();
                    trace!("r1 {r:#?}");
                }

                let f = self.inner.call(req);
                AuthServiceFuture::Future { future: f }
                // if session.is_ok() {
                //     AuthServiceFuture::Future {
                //         future: self.inner.call(req),
                //     }
                // } else {
                //     AuthServiceFuture::Unauthorized
                // }
            }
        }

        pin_project! {
            #[project = ResFutProj]
            pub enum AuthServiceFuture<F> {
                Unauthorized,
                Future {
                    #[pin]
                    future: F,
                }
            }
        }

        impl<F, Body, Err> Future for AuthServiceFuture<F>
        where
            F: Future<Output = Result<Response<Body>, ServerFnError<Err>>>,
            Body: Default + std::fmt::Debug,
            Err: std::fmt::Debug,
        {
            type Output = Result<Response<Body>, ServerFnError<Err>>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                // let a = async move {
                //     trace!("the F is this dude");
                // }.into_future();
                // a.poll();

                // let session = DB.get_session("token").into_future(); // CANT
                // pin!(session);
                // let mut session = session;
                // let r = (session.as_mut()).poll(cx);
                // let session2 = DB.get_session("token").into_future(); // CANT
                // let mut a = Box::pin(session2);
                // let b = (a.as_mut()).poll(cx);

                // session.poll();
                // let f = Box::pin(async move { trace!("im dying"); });
                // let f2 = pin!(async move { trace!("im dying"); });
                // Future::

                match self.project() {
                    ResFutProj::Unauthorized => {
                        let err = Err(ServerFnError::MiddlewareError("unauthorized".to_string()));
                        Poll::Ready(err)
                    }
                    ResFutProj::Future { future } => future.poll(cx),
                }
            }
        }

        // #[cfg(test)]
        // mod auth_tests {
        //     use crate::middleware::auth::verify_cookie;

        // }

        // pub async fn verify(request: Request, next: Next) -> Result<impl IntoResponse, Response> {
        //     trace!("im a middleware");
        //     // let request = buffer_request_body(request).await?;

        //     Ok(next.run(request).await)
        // }
    }
    pub mod log {
        use std::{
            pin::Pin,
            task::{Context, Poll},
        };

        use axum::{
            body::Body,
            extract::Request,
            middleware::Next,
            response::{IntoResponse, Response},
        };
        use pin_project_lite::pin_project;
        use tower::{Layer, Service};
        use tracing::trace;

        #[derive(Debug, Clone)]
        pub struct LogLayer;

        impl<S> Layer<S> for LogLayer {
            type Service = LogService<S>;

            fn layer(&self, inner: S) -> Self::Service {
                LogService { inner }
            }
        }

        #[derive(Debug, Clone)]
        pub struct LogService<T> {
            inner: T,
        }

        impl<T> Service<Request<Body>> for LogService<T>
        where
            T: Service<Request>,
        {
            type Response = T::Response;
            type Error = T::Error;
            type Future = LogServiceFuture<T::Future>;

            fn poll_ready(
                &mut self,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), Self::Error>> {
                self.inner.poll_ready(cx)
            }

            fn call(&mut self, req: Request<Body>) -> Self::Future {
                // req.headers().
                trace!("log where the hell am i");
                LogServiceFuture {
                    inner: self.inner.call(req),
                }
            }
        }

        pin_project! {
            pub struct LogServiceFuture<T> {
                #[pin]
                inner: T,
            }
        }

        impl<T> Future for LogServiceFuture<T>
        where
            T: Future,
        {
            type Output = T::Output;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let this = self.project();

                match this.inner.poll(cx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(output) => {
                        trace!("log runing middleware 3");
                        Poll::Ready(output)
                    }
                }
            }
        }

        pub async fn verify(request: Request, next: Next) -> Result<impl IntoResponse, Response> {
            trace!("im a middleware");
            // let request = buffer_request_body(request).await?;

            Ok(next.run(request).await)
        }
    }
}
