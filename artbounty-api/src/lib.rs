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
    pub mod login {
        use http::HeaderValue;
        use leptos::{prelude::*, server};
        use server_fn::codec::{Json, Rkyv, RkyvEncoding};
        use std::{string::ToString, time::Duration};
        use thiserror::Error;
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
            let response = expect_context::<ResponseOptions>();
            // response.set_status(Sta);
            // response.;
            response.append_header(
                http::header::SET_COOKIE,
                HeaderValue::from_str("Bearer=yowza; Secure; HttpOnly").unwrap(),
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
            Default,
            //strum::Display,
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
            #[default]
            #[error("internal server error")]
            ServerErr,
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
        #[middleware(crate::middleware::auth::AuthLayer)]
        // #[middleware(tower_http::timeout::TimeoutLayer::new(std::time::Duration::from_secs(2)))]
        // #[middleware((TimeoutLayer::new(Duration::from_secs(5))))]
        // #[middleware((TimeoutLayer::new(Duration::from_secs(5)), crate::middleware::log::LogLayer))]
        // #[middleware(crate::middleware::log::LogLayer)]
        // #[middleware(tower_governor::GovernorLayer{config: std::sync::Arc::new(tower_governor::governor::GovernorConfig::default())})]
        pub async fn register(
            username: String,
            email: String,
            password: String,
        ) -> Result<String, ServerFnError<CreateErr>> {
            use artbounty_db::db::{AddUserErr, DB};
            use leptos_axum::{extract, extract_with_state};
            use tokio::time::sleep;

            // sleep(Duration::from_secs(3)).await;
            let res = DB
                .add_user(username, email, password)
                .await
                .map_err(|err| match err {
                    // AddUserErr::EmailInvalid(_) => CreateErr::EmailInvalid,
                    AddUserErr::EmailIsTaken(_) => CreateErr::EmailTaken,
                    AddUserErr::UsernameIsTaken(_) => CreateErr::UsernameTaken,
                    // AddUserErr::UsernameInvalid(_) => CreateErr::UsernameInvalid,
                    _ => CreateErr::ServerErr,
                })?;
            // let (db):(State<DbKv>) = extract_with_state().await?;
            Ok(res.id.to_string())
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
        pub enum CreateErr {
            // #[default]
            // #[error("internal server error")]
            ServerErr,

            // #[error("invalid email")]
            EmailInvalid,
            EmailTaken,
            UsernameTaken,
            UsernameInvalid,
        }

        #[cfg(test)]
        mod test_register {
            use test_log::test;
            use tracing::trace;
            use artbounty_db::db::{AddUserErr, DB};
            use crate::api::register::register;

            #[test(tokio::test)]
            async fn test_api_register() {
                DB.connect().await;
                DB.migrate().await.unwrap();
                let r = register("hey".to_string(), "hey@hey.com".to_string(), "hey".to_string()).await;
                trace!("API RESULT: {r:#?}");
            }
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
