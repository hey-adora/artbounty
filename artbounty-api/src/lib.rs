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
    pub mod register {
        // use artbounty_db::db::DbKv;
        use leptos::{prelude::*, server};
        use server_fn::codec::{Json, Rkyv, RkyvEncoding};
        use thiserror::Error;
        use tower::timeout::TimeoutLayer;
        use std::{string::ToString, time::Duration};
        // static a: std::sync::Arc<tower_governor::governor::GovernorConfig> = std::sync::Arc::new(tower_governor::governor::GovernorConfig::default());
        // use strum::{Display, EnumString};

        #[server(
            prefix = "/api",
            endpoint = "register",
            input = Rkyv,
            output = Rkyv, 
        )]
        #[middleware(tower_http::timeout::TimeoutLayer::new(std::time::Duration::from_secs(1)))]
        // #[middleware((TimeoutLayer::new(Duration::from_secs(5))))]
        // #[middleware((TimeoutLayer::new(Duration::from_secs(5)), crate::middleware::log::LogLayer))]
        // #[middleware(crate::middleware::log::LogLayer)]
        // #[middleware(tower_governor::GovernorLayer{config: std::sync::Arc::new(tower_governor::governor::GovernorConfig::default())})]
        pub async fn create(username: String, email: String, password: String) -> Result<String, ServerFnError<CreateErr>> {
        use artbounty_db::db::{DB, AddUserErr};
        use leptos_axum::{extract, extract_with_state};

            
            let res = DB.add_user(username, email, password).await.map_err(|err| match err {
                AddUserErr::Email(_) => CreateErr::Email,
                _ => CreateErr::ServerErr
            })?;
            // let (db):(State<DbKv>) = extract_with_state().await?;
            Ok(res.id.to_string())
        }


        #[derive(
            Debug,
            Error,
            Clone,
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
        pub enum CreateErr {
            #[error("internal server error")]
            ServerErr,

            #[error("invalid email")]
            Email,
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
            body::Body, extract::Request,  middleware::Next, response::{IntoResponse}
        };
        use pin_project_lite::pin_project;
        use tower::{BoxError, Layer, Service};
        use tracing::trace;
        use thiserror::Error;

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

        impl<T> Service<Request<Body>> for AuthService<T>
        where
            T: Service<Request>,
            T::Error: Into<BoxError>
        {
            type Response = T::Response;
            type Error = BoxError;
            // type Future = ResponseF<T::Future>;
            type Future = AuthServiceFuture<T::Future>;

            fn poll_ready(
                &mut self,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), Self::Error>> {
                // Self::Error::
                
                //self.inner.poll_ready(cx)
                Poll::Ready(Err(Box::new(KaboomErr::Boom)))
            }

            fn call(&mut self, req: Request<Body>) -> Self::Future {
                // req..
                trace!("where the hell am i");
                AuthServiceFuture {
                    inner: self.inner.call(req),
                }
            }
        }

        pin_project! {
            pub struct AuthServiceFuture<T> {
                #[pin]
                inner: T,
            }
        }

        impl<F, Res, Err> Future for AuthServiceFuture<F>
        where
            F: Future<Output = Result<Res, Err>>,
            Err: Into<BoxError>
            
        {
            type Output = Result<Res, BoxError>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let this = self.project();
                
                match this.inner.poll(cx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(output) => {
                        trace!("runing middleware 3");
                        Poll::Ready(output.map_err(Into::into))
                    }
                }
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
