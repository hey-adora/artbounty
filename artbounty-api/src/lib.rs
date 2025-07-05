use std::sync::LazyLock;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

// static DB: LazyLock<DbKv> = LazyLock::new(Db::new_kv);

pub mod api {
    use thiserror::Error;

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

    pub mod profile {
        use leptos::{prelude::*, server};
        use server_fn::codec::Rkyv;
        use thiserror::Error;
        use tracing::trace;

        pub struct ApiProfile {
            pub username: String,
        }

        #[server(
            prefix = "/api",
            endpoint = "profile",
            input = Rkyv,
            output = Rkyv, 
        )]
        // #[middleware(crate::middleware::auth::AuthLayer)]
        pub async fn profile() -> Result<(), ServerFnError<ProfileErr>>{
            use axum::http::{Request};
            use leptos_axum::extract;
            use http::HeaderMap;

            let header: HeaderMap = extract().await.unwrap();
            // let header: HeaderMap = extract().await.map_err(|_| ProfileErr::ServerErr)?;
            trace!("headermap {header:#?}");

            Ok(())
        }

        async fn profile_inner() {

        }
        
        #[derive(
            Debug,
            Error,
            Clone,
            strum::Display,
            strum::EnumString,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub enum ProfileErr {
            ServerErr,
        }

        #[cfg(test)]
        pub mod test_profile {
            use artbounty_db::db::DB;
            use http::{request::Parts, Extensions, HeaderMap, Method, Request, Uri, Version};
            use leptos::prelude::provide_context;
            use test_log::test;

            use crate::api::profile::profile;

            #[test(tokio::test)]
            async fn test_profile() {
                // DB.connect().await;
                // DB.migrate().await.unwrap();
                // let builder = Request::builder();
                // let r = builder.method(Method::POST).uri("http://localhost:3000/api/login").version(Version::HTTP_11).extension(Extensions::new()).header("Bearer", "foo").body(()).unwrap();
                // let (parts, ()) = r.into_parts();
                // provide_context::<Parts>(parts);

                // provide_context(Parts {
                //     version: Version::HTTP_11,
                //     extensions: Extensions::new(),
                //     headers: {
                //         let map = HeaderMap::new();
                //         // map.insert(key, val)
                //         map
                //     },
                //     method: Method::POST,
                //     uri: Uri::from_static("http://localhost:3000/api/login"),
                //     ..Default::default()
                // });

                // let result = profile().await.unwrap();
            }
        }
    }

    pub mod login {
        use http::HeaderValue;
        use jiff::Timestamp;
        // use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
        use leptos::{prelude::*, server};
        use server_fn::codec::{Json, Rkyv, RkyvEncoding};
        use tracing::trace;
        use std::{string::ToString, time::Duration};
        use thiserror::Error;

        // use crate::auth::verify_password;
        #[server(
            prefix = "/api",
            endpoint = "login",
            input = Rkyv,
            output = Rkyv, 
        )]
        // #[middleware(crate::middleware::auth::AuthLayer)]
        pub async fn login(
            email: String,
            password: String,
        ) -> Result<String, ServerFnError<LoginErr>> {
            use artbounty_db::db::{AddUserErr, DB};
            use leptos_axum::ResponseOptions;
            use crate::auth::{Claims, encode_token, get_nanos, verify_password};

            let response = expect_context::<ResponseOptions>();
            // response.set_status(Sta);
            // response.;
            trace!("1");

            let password_hash = DB.get_user_password_hash(email).await.map_err(|_| LoginErr::Incorrect)?;
            trace!("1.5");
            let password_correct = verify_password(password, password_hash);
            if !password_correct {
                return Err(ServerFnError::from(LoginErr::Incorrect));
            }
            
            trace!("2");

            let time = get_nanos();
            let token = encode_token("secret", Claims::new("hey", time)).map_err(|_| LoginErr::ServerErr)?;
            trace!("2.5");
            let r = DB.add_session(token.clone()).await;
            trace!("r {r:#?}");

            r.map_err(|_| LoginErr::ServerErr)?;
            let cookie = format!("Bearer={token}; Secure; HttpOnly");

            trace!("3");
            response.append_header(
                http::header::SET_COOKIE,
                HeaderValue::from_str(&cookie).unwrap(),
            );


            // response.append_header(
            //     http::header::SET_COOKIE,
            //     HeaderValue::from_str("authorization=yowza; Secure; HttpOnly").unwrap(),
            // );

            // use leptos_axum::{extract, extract_with_state};
            // use tokio::time::sleep;

            // sleep(Duration::from_secs(3)).await;
            // let res = DB.add_user(username, email, password).await.map_err(|err| match err {
            //     AddUserErr::Email(_) => CreateErr::Email,
            //     _ => CreateErr::ServerErr
            // }).map_err(MidErr::ReqErr)?;
            // let (db):(State<DbKv>) = extract_with_state().await?;
            Ok("login".to_string())
        }
        #[derive(
            Debug,
            Error,
            Clone,
            // Default,
            strum::Display,
            strum::EnumString,
            //strum::Display,
            //strum::EnumString,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub enum LoginErr {
            // #[default]
            // #[error("internal server error")]
            ServerErr,
            Incorrect,
            // #[error("invalid email")]
            // Email,
        }


    }
    pub mod register {
        // use artbounty_db::db::DbKv;
        use leptos::{prelude::*, server};
        use server_fn::codec::{Json, Rkyv, RkyvEncoding};
        use std::{string::ToString, time::Duration};
        use thiserror::Error;


        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub struct RegisterResult {
            pub email: String,
        }

        // #[derive(
        //     Debug,
        //     Error,
        //     Clone,
        //     serde::Serialize,
        //     serde::Deserialize,
        //     rkyv::Archive,
        //     rkyv::Serialize,
        //     rkyv::Deserialize,
        // )]
        // pub struct User {
        //     pub 
        // }

        // use tower::timeout::TimeoutLayer;

        // use crate::api::MidErr;

        // use crate::middleware::MidErr;
        // static a: std::sync::Arc<tower_governor::governor::GovernorConfig> = std::sync::Arc::new(tower_governor::governor::GovernorConfig::default());
        // use strum::{Display, EnumString};

        #[server(
            prefix = "/api",
            endpoint = "register",
            input = Rkyv,
            output = Rkyv, 
        )]
        // #[middleware(crate::middleware::auth::AuthLayer)]
        // #[middleware(tower_http::timeout::TimeoutLayer::new(std::time::Duration::from_secs(2)))]
        // #[middleware((TimeoutLayer::new(Duration::from_secs(5))))]
        // #[middleware((TimeoutLayer::new(Duration::from_secs(5)), crate::middleware::log::LogLayer))]
        // #[middleware(crate::middleware::log::LogLayer)]
        // #[middleware(tower_governor::GovernorLayer{config: std::sync::Arc::new(tower_governor::governor::GovernorConfig::default())})]
        pub async fn register(
            username: String,
            email: String,
            password: String,
        ) -> Result<RegisterResult, ServerFnError<RegisterErr>> {
            use artbounty_db::db::{AddUserErr, DB};
            use leptos_axum::{extract, extract_with_state};
            use tokio::time::sleep;
            use artbounty_shared::auth::{proccess_email, proccess_username, proccess_password};
            use crate::auth::hash_password;


            // sleep(Duration::from_secs(3)).await;

            let username = proccess_username(username).map_err(|err| RegisterErr::UsernameInvalid(err))?;
            let email = proccess_email(email).map_err(|err| RegisterErr::EmailInvalid(err))?;
            let password = proccess_password(password, None).and_then(|pss| hash_password(pss).map_err(|_| "hasher error".to_string())).map_err(|err| RegisterErr::PasswordInvalid(err))?;


            let res = DB
                .add_user(username, email, password)
                .await
                .map_err(|err| match err {
                    // AddUserErr::EmailInvalid(_) => CreateErr::EmailInvalid,
                    AddUserErr::EmailIsTaken(_) => RegisterErr::EmailTaken,
                    AddUserErr::UsernameIsTaken(_) => RegisterErr::UsernameTaken,
                    // AddUserErr::UsernameInvalid(_) => CreateErr::UsernameInvalid,
                    _ => RegisterErr::ServerErr,
                })?;
            

            // let (db):(State<DbKv>) = extract_with_state().await?;
            let result = RegisterResult {
                email: res.email.to_string(),
            };
            Ok(result)
            // Ok(())
        }

        // #[cfg(feature = "ssr")]
        // pub async fn register_inner() {

        // }

        #[derive(
            Debug,
            Error,
            Clone,
            // Default,
            strum::Display,
            strum::EnumString,
            //strum::Display,
            //strum::EnumString,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
        )]
        pub enum RegisterErr {
            // #[default]
            // #[error("internal server error")]
            ServerErr,

            // #[error("invalid email")]
            EmailInvalid(String),
            EmailTaken,
            UsernameTaken,
            UsernameInvalid(String),
            PasswordInvalid(String),
        }

        // pub fn err_to_string(err: RegisterErr) {
        //     match err
        // }

        #[cfg(test)]
        mod test_register {
            use test_log::test;
            use tracing::trace;
            use artbounty_db::db::{AddUserErr, DB};
            use crate::api::register::register;

            #[test(tokio::test)]
            async fn test_api_register() {
                // DB.connect().await;
                // DB.migrate().await.unwrap();
                // let r = register("hey".to_string(), "hey@hey.com".to_string(), "hey".to_string()).await.unwrap();
                // trace!("API RESULT: {r:#?}");
            }
        }
    }
}

#[cfg(feature = "ssr")]
pub mod auth {
    use std::time::{SystemTime, UNIX_EPOCH};

    use argon2::{password_hash::{self, rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordVerifier};
    use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
    use argon2::PasswordHasher;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct Claims {
        username: String,
        created_at: u128,
        exp: u64,
    }

    impl Claims {
        pub fn new<S: Into<String>>(username: S, time: u128) -> Self {
            let username: String = username.into();
            Claims { username, created_at: time, exp: 0 }
        }
    }

    pub fn get_nanos() -> u128 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    }

    pub fn verify_password<T: AsRef<[u8]>, S2: AsRef<str>>(password: T, hash: S2) -> bool { 
        let password = password.as_ref();
        let hash = hash.as_ref();
        PasswordHash::new(hash).and_then(|hash|Argon2::default().verify_password(password, &hash) ).is_ok()
        
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

    pub fn encode_token<Key: AsRef<[u8]>>(key: Key, claims: Claims) -> Result<String, jsonwebtoken::errors::Error> {
        let header = Header::new(Algorithm::HS512);
        let key = EncodingKey::from_secret(key.as_ref());

        let token = encode(&header, &claims, &key);

        token
    }

    pub fn decode_token<Key: AsRef<[u8]>, S: AsRef<str>>(key: Key, token: S)-> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
        let token = token.as_ref();
        let key = DecodingKey::from_secret(key.as_ref());
        let mut validation = Validation::new(Algorithm::HS512);
        validation.validate_exp = false;
        let claims = decode::<Claims>(token, &key, &validation);

        claims

    }

    #[cfg(test)]
    mod login_auth {
        use std::time::{SystemTime, UNIX_EPOCH};

        use jiff::Timestamp;
        use test_log::test;
        use tracing::trace;
        use super::{decode_token, encode_token, Claims};

        #[test]
        fn test_login() {
            let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            trace!("time {time}");
            // let time = Timestamp::now();
            let claims = Claims::new("hey", time);
            let token = encode_token("secret", claims).unwrap();
            trace!("\ntoken: {token}");
            let decoded_token = decode_token("secret", &token).unwrap();
            trace!("\ndecoded: {decoded_token:?}");
            // let token2 = encode_token("secret", time).unwrap();
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

        use axum::{
            body::Body,
            http::{Request, Response, StatusCode},
            middleware::Next,
            response::IntoResponse,
        };
        // use biscotti::{Processor, ProcessorConfig, RequestCookies};
        use pin_project_lite::pin_project;
        use server_fn::ServerFnError;
        use thiserror::Error;
        use tower::{BoxError, Layer, Service};
        use tracing::trace;

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
                trace!("layer neyer");
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
            // S::Error: std::fmt::Debug+ Default
        {
            type Response = S::Response;
            type Error = S::Error;
            // type Future = ResponseF<T::Future>;
            type Future = AuthServiceFuture<S::Future>;

            fn poll_ready(
                &mut self,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), Self::Error>> {
                // Self::Error::
                trace!("i am here");
                self.inner.poll_ready(cx)
                // Poll::Ready(Err(Box::new(KaboomErr::Boom)))
            }

            fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
                // req..
                // let err = Err(ServerFnError::MiddlewareError("unauthorized".to_string()));

                let logged_in = req
                    .headers()
                    .get(http::header::COOKIE)
                    .and_then(|h| h.to_str().ok())
                    .map(verify_cookie).unwrap_or_default();
                    // .and_then(|header| {
                    //     let processor: Processor = ProcessorConfig::default().into();
                    //      RequestCookies::parse_header(header, &processor).ok()
                    // })
                    // .and_then(|cookies| cookies.get("Bearer"))
                    // .map(|cookie| cookie.value() == "yowza").unwrap_or_default();
                // let cookies = 
                    // .and_then(|h| h.to_str().ok())
                    // .and_then(|cookies| {
                    //     cookies.split(";").find(|cookie| {
                    //         cookie
                    //             .split("=")
                    //             .next()
                    //             .map(|value| value.trim() == "hello")
                    //             .unwrap_or_default()
                    //     })
                    // })
                    // .map(|cookie| cookie.trim());
                trace!("HEADER BEADER: {logged_in:#?}");
                trace!("where the hell am i: {req:#?}");
                if logged_in {
                    AuthServiceFuture::Future {
                        future: self.inner.call(req),
                    }
                } else {
                    AuthServiceFuture::Unauthorized
                }
                // .and_then(|value| value.to_str().ok()?.parse::<usize>().ok());

            }
        }

        pub fn verify_cookie(header: &str) -> bool {
        use biscotti::{Processor, ProcessorConfig, RequestCookies};
            let processor: Processor = ProcessorConfig::default().into();
            RequestCookies::parse_header(header, &processor).ok()
                .and_then(|cookies| cookies.get("Bearer"))
                .map(|cookie| cookie.value() == "yowza").unwrap_or_default()
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
                // cx.

                match self.project() {
                    ResFutProj::Unauthorized => {
                        let err = Err(ServerFnError::MiddlewareError("unauthorized".to_string()));
                        Poll::Ready(err)
                    }
                    ResFutProj::Future { future } => future.poll(cx),
                }
                // this.inner.
                // trace!("before output");
                //         let err = Err(ServerFnError::MiddlewareError("unauthorized".to_string()));
                //         Poll::Ready(err)
                // match this.inner.poll(cx) {
                //     Poll::Pending => Poll::Pending,
                //     Poll::Ready(output) => {
                //         // output.
                //         trace!("OUTPUT: {output:#?}");
                //         // let mut res: Response<ServerFnError<Err>> = Response::new( ServerFnError::MiddlewareError("aaaaaaaaaaaaaaa".to_string()) );
                //         // let mut res = Response::new(Body::default());
                //         // *res.status_mut() = StatusCode::UNAUTHORIZED;
                //         // res.body_mut().push_str("hello world");

                //         trace!("runing middleware 3");
                //         // Poll::Ready(output.map_err(Into::into))
                //         // Poll::Ready(Err(res))
                //         let err = Err(ServerFnError::MiddlewareError("unauthorized".to_string()));
                //         Poll::Ready(err)
                //     }
                // }
            }
        }

        #[cfg(test)]
        mod auth_tests {
            use crate::middleware::auth::verify_cookie;

            #[test]
            fn test_verify_cookie() {
                assert!(verify_cookie("Bearer=yowza; authorization=yowza"));
                assert!(verify_cookie("Bearer2=yowza; Bearer=yowza"));
                assert!(!verify_cookie("Bearer2=yowza; Bearer3=yowza"));
            }

        }

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
