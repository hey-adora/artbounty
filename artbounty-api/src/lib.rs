pub mod router {
    pub const API_PATH: &'static str = "/api";

    #[cfg(feature = "ssr")]
    pub fn new() -> axum::Router<crate::app_state::AppState> {
        use axum::{Router, routing::post};
        let routes = Router::new()
            .route(
                crate::auth::api::login::PATH,
                post(crate::auth::api::login::server),
            )
            .route(
                crate::auth::api::register::PATH,
                post(crate::auth::api::register::server),
            )
            .route(
                crate::auth::api::invite_decode::PATH,
                post(crate::auth::api::invite_decode::server),
            )
            .route(
                crate::auth::api::invite::PATH,
                post(crate::auth::api::invite::server),
            )
            .route(
                crate::auth::api::profile::PATH,
                post(crate::auth::api::profile::server),
            )
            .route(
                crate::auth::api::user::PATH,
                post(crate::auth::api::user::server),
            )
            .route(
                crate::auth::api::logout::PATH,
                post(crate::auth::api::logout::server),
            )
            .route(
                crate::post::api::create::PATH,
                post(crate::post::api::create::server),
            );
        Router::new().nest(API_PATH, routes)
    }
}
pub mod utils {
    use bytecheck::CheckBytes;
    use http::{HeaderMap, StatusCode};
    use leptos::prelude::location;
    use reqwest::RequestBuilder;
    use rkyv::{
        Archive, Deserialize,
        api::high::{HighDeserializer, HighSerializer, HighValidator},
        primitive::ArchivedI32,
        rancor::Strategy,
        result::ArchivedResult,
        ser::{allocator::ArenaHandle, sharing::Share},
        util::AlignedVec,
    };
    use thiserror::Error;
    use tracing::{debug, error, trace};

    use crate::router::API_PATH;

    // #[cfg(feature = "ssr")]
    // pub async fn recv<ClientInput, ServerOutput, ServerErr, Fut>(
    //     mut multipart: axum::extract::Multipart,
    //     server_fn: impl FnOnce(ClientInput) -> Fut,
    // ) -> impl axum::response::IntoResponse
    // where
    //     ClientInput: Archive,
    //     ClientInput::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
    //         + Deserialize<ClientInput, HighDeserializer<rkyv::rancor::Error>>,
    //     ServerOutput: for<'a> rkyv::Serialize<
    //             Strategy<
    //                 rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
    //                 bytecheck::rancor::Error,
    //             >,
    //         > + axum::response::IntoResponse,
    //     ServerErr: for<'a> rkyv::Serialize<
    //             Strategy<
    //                 rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
    //                 bytecheck::rancor::Error,
    //             >,
    //         > + Archive
    //         + std::error::Error
    //         + axum::response::IntoResponse
    //         + 'static,
    //     ServerErr::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
    //         + Deserialize<ServerErr, HighDeserializer<rkyv::rancor::Error>>,
    //     Fut: Future<Output = Result<ServerOutput, ServerErr>>,
    //     // FutOutput: axum::response::IntoResponse,
    // {
    //     use axum::response::IntoResponse;
    //
    //     let run = async || -> Result<ServerOutput, ResErr<ServerErr>> {
    //         trace!("1");
    //         let mut bytes = bytes::Bytes::new();
    //         while let Some(field) = multipart
    //             .next_field()
    //             .await
    //             .map_err(|_| ResErr::ServerDecodeErr(ServerDecodeErr::NextFieldFailed))?
    //         {
    //             trace!("2");
    //             if field.name().map(|name| name == "data").unwrap_or_default() {
    //                 trace!("3");
    //                 bytes = field.bytes().await.map_err(|_| {
    //                     ResErr::ServerDecodeErr(ServerDecodeErr::FieldToBytesFailed)
    //                 })?;
    //             }
    //         }
    //
    //         trace!("4");
    //         let archived = rkyv::access::<ClientInput::Archived, rkyv::rancor::Error>(&bytes)
    //             .map_err(|_| ResErr::ServerDecodeErr(ServerDecodeErr::RkyvAccessErr))?;
    //         trace!("5");
    //         let client_input = rkyv::deserialize::<ClientInput, rkyv::rancor::Error>(archived)
    //             .map_err(|_| ResErr::ServerDecodeErr(ServerDecodeErr::RkyvErr))?;
    //         trace!("6");
    //         let result = server_fn(client_input)
    //             .await
    //             .map_err(|err| ResErr::ServerErr(err));
    //         trace!("7");
    //
    //         result
    //     };
    //
    //     let response = run().await;
    //
    //     let result = match response {
    //         Ok(server_output) => server_output.into_response(),
    //         Err(ResErr::ServerDecodeErr(err)) => {
    //             let body = encode(&Result::<ServerOutput, ResErr<ServerErr>>::Err(
    //                 ResErr::ServerDecodeErr(err),
    //             ))
    //             .expect("serializing ServerDecodeErr should just work");
    //             trace!("sending body: {body:?}");
    //             (axum::http::StatusCode::BAD_REQUEST, body).into_response()
    //         }
    //         Err(ResErr::ServerErr(err)) => err.into_response(),
    //         Err(ResErr::ClientErr(_)) => {
    //             unreachable!("client error shouldnt be send by the server")
    //         }
    //     };
    //
    //     // make recv_inner return tuple of status and rkyv bytes maybe
    //     // trace!("sending response: {:#?}", result.body().);
    //
    //     result
    // }

    #[cfg(feature = "ssr")]
    pub async fn decode_multipart<ClientInput, ServerErr>(
        mut multipart: axum::extract::Multipart,
    ) -> Result<ClientInput, ResErr<ServerErr>>
    where
        ServerErr: std::error::Error + 'static,
        ClientInput: Archive,
        ClientInput::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ClientInput, HighDeserializer<rkyv::rancor::Error>>,
    {
        let mut bytes = bytes::Bytes::new();
        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|_| ResErr::ServerDecodeErr(ServerDecodeErr::NextFieldFailed))?
        {
            trace!("2");
            if field.name().map(|name| name == "data").unwrap_or_default() {
                trace!("3");
                bytes = field
                    .bytes()
                    .await
                    .map_err(|_| ResErr::ServerDecodeErr(ServerDecodeErr::FieldToBytesFailed))?;
            }
        }

        trace!("4");
        let archived = rkyv::access::<ClientInput::Archived, rkyv::rancor::Error>(&bytes)
            .map_err(|_| ResErr::ServerDecodeErr(ServerDecodeErr::RkyvAccessErr))?;
        trace!("5");
        let client_input = rkyv::deserialize::<ClientInput, rkyv::rancor::Error>(archived)
            .map_err(|_| ResErr::ServerDecodeErr(ServerDecodeErr::RkyvErr))?;
        trace!("6");

        Ok(client_input)
    }

    #[cfg(feature = "ssr")]
    pub fn encode_server_output<ServerOutput, ServerErr>(
        response: Result<ServerOutput, ResErr<ServerErr>>,
    ) -> axum::response::Response
    where
        ServerOutput: for<'a> rkyv::Serialize<
                Strategy<
                    rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                    bytecheck::rancor::Error,
                >,
            > + std::fmt::Debug
            + axum::response::IntoResponse,
        ServerErr: for<'a> rkyv::Serialize<
                Strategy<
                    rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                    bytecheck::rancor::Error,
                >,
            > + Archive
            + std::error::Error
            + axum::response::IntoResponse
            + std::fmt::Debug
            + 'static,
        ServerErr::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ServerErr, HighDeserializer<rkyv::rancor::Error>>,
    {
        use axum::response::IntoResponse;

        trace!("ENCODING SERVER INPUT: {:?}", response);

        let result = match response {
            Ok(server_output) => server_output.into_response(),
            // Err(ResErr::ServerDecodeErr(err)) => {
            //     let body: Result<ServerOutput, ResErr<ServerErr>> =
            //         Err(ResErr::ServerDecodeErr(err));
            //     trace!("encoding server output: {body:#?}");
            //     let body = encode_result::<ServerOutput, ServerErr>(&body);
            //     trace!("sending body: {body:?}");
            //     (axum::http::StatusCode::BAD_REQUEST, body).into_response()
            // }
            // Err(ResErr::ServerEndpointNotFoundErr(err)) => {
            //     let body: Result<ServerOutput, ResErr<ServerErr>> =
            //         Err(ResErr::ServerEndpointNotFoundErr(err));
            //     trace!("encoding server output: {body:#?}");
            //     let body = encode_result::<ServerOutput, ServerErr>(&body);
            //     trace!("sending body: {body:?}");
            //     (axum::http::StatusCode::NOT_FOUND, body).into_response()
            // }
            // Err(ResErr::Unauthorized(ResErrUnauthorized::BadToken)) => {
            //     let body: Result<ServerOutput, ResErr<ServerErr>> =
            //         Err(ResErr::ServerEndpointNotFoundErr(err));
            //     trace!("encoding server output: {body:#?}");
            //     let body = encode_result::<ServerOutput, ServerErr>(&body);
            //     trace!("sending body: {body:?}");
            //     (axum::http::StatusCode::NOT_FOUND, body).into_response()
            // }
            Err(ResErr::ServerErr(err)) => err.into_response(),
            Err(ResErr::ClientErr(_)) => {
                unreachable!("client error shouldnt be send by the server")
            }
            Err(ResErr::Unauthorized(ResErrUnauthorized::BadToken)) => {
                use http::header::{AUTHORIZATION, SET_COOKIE};

                let body: Result<ServerOutput, ResErr<ServerErr>> =
                    Err(ResErr::Unauthorized(ResErrUnauthorized::BadToken));
                trace!("encoding server output: {body:#?}");
                let body = encode_result::<ServerOutput, ServerErr>(&body);
                trace!("sending body: {body:?}");
                // let jar = jar.add(Cookie::new(
                //     AUTHORIZATION.as_str(),
                //     "Bearer=DELETED; Secure; HttpOnly; expires=Thu, 01 Jan 1970 00:00:00 GMT",
                // ));
                let headers = axum::response::AppendHeaders([(
                    SET_COOKIE,
                    "authorization=Bearer%3DDELETED%3B%20Secure%3B%20HttpOnly%3B%20expires%3DThu%2C%2001%20Jan%201970%2000%3A00%3A00%20GMT",
                )]);
                (axum::http::StatusCode::UNAUTHORIZED, headers, body).into_response()
            }
            Err(err) => {
                let body: Result<ServerOutput, ResErr<ServerErr>> = Err(err);
                trace!("encoding server output: {body:#?}");
                let body = encode_result::<ServerOutput, ServerErr>(&body);
                trace!("sending body: {body:?}");
                (axum::http::StatusCode::BAD_REQUEST, body).into_response()
            }
        };

        result
    }

    pub fn encode_result<ServerOutput, ServerErr>(
        result: &Result<ServerOutput, ResErr<ServerErr>>,
    ) -> Vec<u8>
    where
        ServerOutput: for<'a> rkyv::Serialize<
                Strategy<
                    rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                    bytecheck::rancor::Error,
                >,
            >,
        ServerErr: for<'a> rkyv::Serialize<
                Strategy<
                    rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                    bytecheck::rancor::Error,
                >,
            > + Archive
            + std::error::Error
            + 'static,
    {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(result)
            .unwrap()
            .to_vec();

        bytes
    }

    pub async fn send<ServerOutput, ServerErr>(
        path: impl AsRef<str>,
        input: &impl for<'a> rkyv::Serialize<
            Strategy<
                rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                bytecheck::rancor::Error,
            >,
        >,
    ) -> Result<ServerOutput, ResErr<ServerErr>>
    where
        ServerOutput: Archive + std::fmt::Debug,
        ServerOutput::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ServerOutput, HighDeserializer<rkyv::rancor::Error>>,
        ServerErr: Archive + std::error::Error + std::fmt::Debug + 'static,
        ServerErr::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ServerErr, HighDeserializer<rkyv::rancor::Error>>,
    {
        let origin = location().origin().unwrap();
        let path = path.as_ref();
        let builder = reqwest::Client::new().post(format!("{origin}{API_PATH}{path}"));
        send_from_builder::<ServerOutput, ServerErr>(builder, input)
            .await
            .1
    }

    pub async fn send_from_builder<ServerOutput, ServerErr>(
        req_builder: RequestBuilder,
        // host: impl AsRef<str>,
        // path: impl AsRef<str>,
        input: &impl for<'a> rkyv::Serialize<
            Strategy<
                rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                bytecheck::rancor::Error,
            >,
        >,
    ) -> (HeaderMap, Result<ServerOutput, ResErr<ServerErr>>)
    where
        ServerOutput: Archive + std::fmt::Debug,
        ServerOutput::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ServerOutput, HighDeserializer<rkyv::rancor::Error>>,
        ServerErr: Archive + std::error::Error + std::fmt::Debug + 'static,
        ServerErr::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ServerErr, HighDeserializer<rkyv::rancor::Error>>,
    {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(input)
            .unwrap()
            .to_vec();

        // const HEADERS_EMPTY: HeaderMap = HeaderMap::new();
        // let url = format!("{}{}", host.as_ref(), path.as_ref());
        let part = reqwest::multipart::Part::bytes(bytes);
        let form = reqwest::multipart::Form::new().part("data", part);
        let res = match req_builder
            .multipart(form)
            .send()
            .await
            .inspect_err(|err| error!("client err: {err}"))
            .map_err(|_| ResErr::ClientErr(ClientErr::FailedToSend))
        {
            Ok(res) => res,
            Err(err) => {
                return (HeaderMap::new(), Err(err));
            }
        };

        let headers = res.headers().clone();
        let status = res.status();
        let url = res.url().clone();
        let bytes = res.bytes().await;

        if status == StatusCode::NOT_FOUND
            && bytes.as_ref().map(|v| v.len() == 0).unwrap_or_default()
        {
            debug!("CLIENT RECV:\nstatus: {status}\nclient received headers: {headers:#?}");
            return (
                headers,
                Err(ResErr::ServerEndpointNotFoundErr(url.to_string())),
            );
        }

        let r = match
            bytes
            .inspect(|bytes| debug!("CLIENT RECV:\nstatus: {status}\nclient received: {bytes:?}\nclient received headers: {headers:#?}"))
            .inspect_err(|err| error!("client byte stream err: {err}"))
            .map_err(|_| ResErr::ClientErr(ClientErr::ByteStreamFail))
            .map(|res| res.to_vec())
            .and_then(|body| {
                let archive = rkyv::access::<
                    ArchivedResult<ServerOutput::Archived, ArchivedResErr<ServerErr>>,
                    rkyv::rancor::Error,
                >(&body)
                .map_err(|_| ResErr::ClientErr(ClientErr::from(ClientDecodeErr::RkyvAccessErr)))?;
                rkyv::deserialize::<Result<ServerOutput, ResErr<ServerErr>>, rkyv::rancor::Error>(
                    archive,
                )
                .map_err(|_| ResErr::ClientErr(ClientErr::from(ClientDecodeErr::RkyvErr)))
            }) {
            Ok(res) => res,
            Err(err) => {
                return (headers, Err(err));
            }
        };

        trace!("recv body: {r:#?}");
        // let archived = match rkyv::access::<
        //     ArchivedResult<ServerOutput::Archived, ArchivedResErr<ServerErr>>,
        //     rkyv::rancor::Error,
        // >(&body)
        // .map_err(|_| ResErr::ClientErr(ClientErr::from(ClientDecodeErr::RkyvAccessErr))) {
        //     Ok(archive) => archive,
        //     Err(err) => {
        //         return (headers, Err(err));
        //     }
        // };
        // let r = match rkyv::deserialize::<Result<ServerOutput, ResErr<ServerErr>>, rkyv::rancor::Error>(
        //     archived,
        // )
        // .map_err(|_| ResErr::ClientErr(ClientErr::from(ClientDecodeErr::RkyvErr))) {
        //
        // };
        // .map_err(|err| ResErr::from(err));

        (headers, r)
    }

    // pub fn encode(
    //     e: &impl for<'a> rkyv::Serialize<
    //         Strategy<
    //             rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
    //             bytecheck::rancor::Error,
    //         >,
    //     >,
    // ) -> Result<Vec<u8>, rkyv::rancor::Error> {
    //     rkyv::to_bytes::<rkyv::rancor::Error>(e).map(|v| v.to_vec())
    // }

    // pub fn encode_server_output<ServerOutput, ServerErr>(output: ServerOutput) -> Vec<u8>
    // where
    //     ServerOutput: for<'a> rkyv::Serialize<
    //             Strategy<
    //                 rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
    //                 bytecheck::rancor::Error,
    //             >,
    //         > + axum::response::IntoResponse,
    //     ServerErr: for<'a> rkyv::Serialize<
    //             Strategy<
    //                 rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
    //                 bytecheck::rancor::Error,
    //             >,
    //         > + Archive
    //         + std::error::Error
    //         + axum::response::IntoResponse
    //         + 'static,
    // {
    //     let bytes =
    //         rkyv::to_bytes::<rkyv::rancor::Error>(&Ok::<ServerOutput, ResErr<ServerErr>>(output))
    //             .unwrap()
    //             .to_vec();
    //
    //     bytes
    // }

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
    pub enum ResErr<E: std::error::Error + 'static> {
        #[error("client error: {0}")]
        ClientErr(ClientErr),

        #[error("server error: {0}")]
        ServerDecodeErr(ServerDecodeErr),

        #[error("server error: endpoint \"{0}\" not found")]
        ServerEndpointNotFoundErr(String),

        #[error("unauthorized")]
        Unauthorized(ResErrUnauthorized),

        #[error("server error: {0}")]
        ServerErr(#[from] E),
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
    pub enum ResErrUnauthorized {
        #[error("unauthorized")]
        Unauthorized,

        #[error("no auth cookie")]
        NoCookie,

        #[error("jwt error")]
        BadToken,

        #[error("something is terribly wrong")]
        DbErr,
    }

    // #[cfg(feature = "ssr")]
    // impl<ServerErr> axum::response::IntoResponse for ResErr<ServerErr>
    // where
    //     ServerErr: axum::response::IntoResponse + std::error::Error + 'static,
    // {
    //     fn into_response(self) -> axum::response::Response {
    //         match self {
    //             ResErr::ServerDecodeErr(err) => {
    //                 let body = encode(&err).expect("serializing ServerDecodeErr should just work");
    //                 (axum::http::StatusCode::BAD_REQUEST, body).into_response()
    //             }
    //             ResErr::ServerErr(server_err) => server_err.into_response(),
    //             _ => unreachable!(),
    //         }
    //     }
    // }

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
    pub enum ClientErr {
        #[error("failed to send")]
        FailedToSend,

        #[error("invalid response")]
        ByteStreamFail,

        #[error("failed to decode response")]
        DecodeErr(#[from] ClientDecodeErr),
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
    pub enum ServerDecodeErr {
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
    pub enum ClientDecodeErr {
        #[error("rkyv failed to access")]
        RkyvAccessErr,

        #[error("rkyv failed to encode")]
        RkyvErr,
    }
}

#[cfg(feature = "ssr")]
pub mod app_state {
    use std::{sync::Arc, time::Duration};

    use artbounty_db::db::{self, DbEngine};
    use tokio::sync::Mutex;

    use crate::{
        auth::get_timestamp,
        clock::Clock,
        settings::{self, Settings},
    };

    #[derive(Clone)]
    pub struct AppState {
        pub db: DbEngine,
        pub settings: Settings,
        pub clock: Clock,
    }

    impl AppState {
        pub async fn new() -> Self {
            let settings = settings::Settings::new_from_file();
            let db = db::new_local(&settings.db.path).await;
            let f = move || async move { get_timestamp() };
            let clock = Clock::new(f);

            Self {
                db,
                settings,
                clock,
            }
        }

        pub async fn new_testng(time: Arc<Mutex<Duration>>) -> Self {
            let db = db::new_mem().await;
            let settings = settings::Settings::new_testing();
            let f = move || {
                let time = time.clone();
                async move {
                    let t = *(time.lock().await);
                    t
                }
            };
            let clock = Clock::new(f);
            // let clock = Clock::new(async move { *(time.lock().await) });

            Self {
                db,
                settings,
                clock,
            }
        }
    }
}

#[cfg(feature = "ssr")]
pub mod settings {
    use config::{Config, File};

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct Settings {
        pub site: Site,
        pub auth: Auth,
        pub db: Db,
    }

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct Auth {
        pub secret: String,
    }

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct Db {
        pub path: String,
    }

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct Site {
        pub address: String,
        pub files_path: String,
    }

    impl Settings {
        pub fn new_from_file() -> Self {
            Config::builder()
                .add_source(File::with_name("artbounty"))
                .build()
                .unwrap()
                .try_deserialize()
                .unwrap()
        }

        pub fn new_testing() -> Self {
            Self {
                site: Site {
                    address: "http://localhost:3000".to_string(),
                    files_path: "../target/tmp/files".to_string(),
                },
                auth: Auth {
                    secret: "secret".to_string(),
                },
                db: Db {
                    path: "memory".to_string(),
                },
            }
        }
    }
}

#[cfg(feature = "ssr")]
pub mod clock {
    use futures::future::BoxFuture;
    use std::{pin::Pin, sync::Arc, time::Duration};
    use tokio::sync::Mutex;

    #[derive(Clone)]
    pub struct Clock {
        ticker: Arc<
            dyn Fn() -> Pin<Box<dyn Future<Output = Duration> + Sync + Send + 'static>>
                + Sync
                + Send
                + 'static,
        >, // ticker: BoxFuture<'static, Duration>
           // ticker: Arc<dyn Fn() -> BoxFuture<'static, Duration>>, // ticker: BoxFuture<'static, Duration>
    }

    impl Clock {
        pub fn new<
            F: Fn() -> Fut + Send + Sync + Clone + 'static,
            Fut: Future<Output = Duration> + Send + Sync + 'static,
        >(
            ticker: F,
        ) -> Self {
            // let f  = std::pin::Pin::new(Box::new(ticker)as Box<dyn Future<Output = Duration>>);
            let fut = Arc::new(move || {
                let ticker = (ticker.clone())();
                let f: Pin<Box<dyn Future<Output = Duration> + Sync + Send + 'static>> =
                    Box::pin(ticker);
                // let f: BoxFuture<'static, Duration> = Box::pin(ticker);
                f
            });

            Self { ticker: fut }
        }

        pub async fn now(&self) -> Duration {
            let mut fut = (self.ticker)();
            let fut = fut.as_mut();
            let duration = fut.await;
            duration
        }
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

    // pub mod register {
    //     #[derive(
    //         Debug,
    //         Clone,
    //         serde::Serialize,
    //         serde::Deserialize,
    //         rkyv::Archive,
    //         rkyv::Serialize,
    //         rkyv::Deserialize,
    //     )]
    //     pub struct Res {
    //         pub email: String,
    //     }
    // }
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

pub mod post {
    pub mod api {
        pub mod create {
            #[cfg(feature = "ssr")]
            use std::sync::Arc;
            use std::time::Duration;

            use crate::utils::{ResErr, ServerDecodeErr, encode_result, send, send_from_builder};
            #[cfg(feature = "ssr")]
            use axum::Extension;
            use thiserror::Error;
            use tracing::{error, trace};

            pub const PATH: &'static str = "/post";

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
                pub title: String,
                pub description: String,
                pub files: Vec<Vec<u8>>,
            }

            // #[derive(
            //     Debug,
            //     Clone,
            //     serde::Serialize,
            //     serde::Deserialize,
            //     rkyv::Archive,
            //     rkyv::Serialize,
            //     rkyv::Deserialize,
            // )]
            // pub struct InputFile {
            //     // pub extension: String,
            //     pub data: Vec<u8>,
            // }

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
                pub imgs: Vec<ServerOutputImg>,
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
            pub struct ServerOutputImg {
                pub hash: String,
                pub extension: String,
            }

            #[cfg(feature = "ssr")]
            impl axum::response::IntoResponse for ServerOutput {
                fn into_response(self) -> axum::response::Response {
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

                #[error("unauthorized")]
                Unauthorized,

                #[error("img errors")]
                ImgErrors(Vec<ServerErrImg>),

                #[error("failed to create output dir for imgs")]
                ImgFailedToCreateOutputDir(String),

                #[error("failed to save images metadata")]
                ImgFailedToSaveImgMeta,

                #[error("failed to save images to disk {0}")]
                ImgFailedToSaveImgToDisk(String),
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
            pub enum ServerErrImg {
                #[error("unsupported format {0}")]
                UnsupportedFormat(String),

                #[error("failed to decode {0}")]
                FailedToDecode(String),

                #[error("failed to create webp encoder {0}")]
                FailedToCreateWebpEncoder(String),

                #[error("failed to encode webp {0}")]
                FailedToEncodeWebp(String),

                #[error("failed to read metadata {0}")]
                FailedToReadMetadata(String),
            }

            #[cfg(feature = "ssr")]
            impl axum::response::IntoResponse for ServerErr {
                fn into_response(self) -> axum::response::Response {
                    let status = match &self {
                        // ServerErr::NoCookie => axum::http::StatusCode::OK,
                        _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    };
                    let bytes =
                        encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                    (status, bytes).into_response()
                }
            }

            pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
                send::<ServerOutput, ServerErr>(PATH, &input).await
            }

            #[cfg(feature = "ssr")]
            pub async fn server(
                axum::extract::State(app_state): axum::extract::State<crate::app_state::AppState>,
                jar: axum_extra::extract::cookie::CookieJar,
                // username: Extension<String>,
                multipart: axum::extract::Multipart,
            ) -> impl axum::response::IntoResponse {
                // let username = &*username;
                trace!("executing post api");
                use std::{io::Cursor, path::Path, str::FromStr};

                use crate::auth::check_auth;
                use axum_extra::extract::cookie::Cookie;
                use gxhash::{gxhash64, gxhash128};
                use http::header::AUTHORIZATION;
                use image::{ImageFormat, ImageReader};
                use little_exif::{filetype::FileExtension, metadata::Metadata};
                use tokio::fs;

                use crate::{
                    auth::{
                        AuthToken, cut_cookie_value_decoded, decode_token, encode_token, get_nanos,
                    },
                    utils::{decode_multipart, encode_server_output},
                };

                let result = (async || -> Result<ServerOutput, ResErr<ServerErr>> {
                    use artbounty_db::db::post::PostFile;

                    let auth_token = check_auth(&app_state, &jar).await?;

                    let input = decode_multipart::<Input, ServerErr>(multipart).await?;
                    // trace!("{input:?}");
                    let files = input
                        .files
                        .into_iter()
                        .map(|v| {
                            // let exif_remover_format = FileExtension::from_str(&v.extension)
                            //     .map_err(|_| ServerErrImg::UnsupportedFormat("uwknown".to_string()))?;
                            // let encoder_format: ImageFormat = ImageFormat::from_extension(&v.extension)
                            //     .ok_or(ServerErrImg::UnsupportedFormat("uwknown".to_string()))?;
                            // let metadata = Metadata
                            let img_data_for_thumbnail = v.clone();
                            let img_data_for_org = v;
                            ImageReader::new(Cursor::new(img_data_for_thumbnail))
                                .with_guessed_format()
                                .inspect_err(|err| error!("error guesing the format {err}"))
                                .map_err(|err| ServerErrImg::UnsupportedFormat(err.to_string()))
                                .and_then(|v| {
                                    let img_format = v.format().ok_or(
                                        ServerErrImg::UnsupportedFormat("uwknown".to_string()),
                                    )?;
                                    v.decode()
                                        .inspect_err(|err| error!("error decoding img {err}"))
                                        .map_err(|err| {
                                            ServerErrImg::FailedToDecode(err.to_string())
                                        })
                                        .map(|img| (img_format, img))
                                })
                                .and_then(|(img_format, img)| {
                                    let width = img.width();
                                    let height = img.height();
                                    webp::Encoder::from_image(&img)
                                        .inspect_err(|err| {
                                            error!("failed to create webp encoder {err}")
                                        })
                                        .map_err(|err| {
                                            ServerErrImg::FailedToDecode(err.to_string())
                                        })
                                        .and_then(|encoder| {
                                            encoder
                                                .encode_simple(false, 90.0)
                                                .inspect_err(|err| {
                                                    error!("failed to create webp encoder {err:?}")
                                                })
                                                .map_err(|err| {
                                                    ServerErrImg::FailedToEncodeWebp(format!(
                                                        "{err:?}"
                                                    ))
                                                })
                                        })
                                        .map(|img| (img_format, (width, height), img))
                                })
                                .and_then(|(img_format, (width, height), img_data_thumbnail)| {
                                    let img_format = img_format.extensions_str()[0];
                                    let mut img_data_org = img_data_for_org;
                                    FileExtension::from_str(img_format)
                                        .map_err(|_| {
                                            ServerErrImg::UnsupportedFormat(img_format.to_string())
                                        })
                                        .and_then(|img_format| {
                                            little_exif::metadata::Metadata::clear_metadata(
                                                &mut img_data_org,
                                                img_format,
                                            )
                                            .inspect_err(|err| {
                                                error!("failed to read metadata {err:?}")
                                            })
                                            .map_err(
                                                |err| {
                                                    ServerErrImg::FailedToReadMetadata(
                                                        err.to_string(),
                                                    )
                                                },
                                            )
                                        })
                                        .map(|_| {
                                            use artbounty_db::db::post::PostFile;

                                            (
                                                PostFile {
                                                    extension: img_format.to_string(),
                                                    hash: format!(
                                                        "{:X}",
                                                        gxhash128(&img_data_org, 0)
                                                    ),
                                                    width,
                                                    height,
                                                },
                                                img_data_org,
                                                img_data_thumbnail.to_vec(),
                                            )
                                        })
                                })
                        })
                        .fold(
                            (
                                Vec::<(PostFile, Vec<u8>, Vec<u8>)>::new(),
                                Vec::<ServerErrImg>::new(),
                            ),
                            |(mut oks, mut errs), file| {
                                match file {
                                    Ok(v) => {
                                        oks.push(v);
                                    }
                                    Err(v) => {
                                        errs.push(v);
                                    }
                                }

                                (oks, errs)
                            },
                        );
                    if !files.1.is_empty() {
                        return Err(ResErr::from(ServerErr::ImgErrors(files.1)));
                    }

                    let files = files.0;
                    let root_path = Path::new(&app_state.settings.site.files_path);
                    let mut output_imgs = Vec::<ServerOutputImg>::new();
                    for file in &files {
                        let file_path =
                            root_path.join(format!("{}.{}", &file.0.hash, &file.0.extension));
                        if file_path.exists() {
                            trace!(
                                "file already exists {}",
                                file_path.to_str().unwrap_or("err")
                            );
                            continue;
                        }

                        // root_path.join(format!("{:X}.{}", &file.0.hash, &file.0.extension));
                        trace!("saving {}", file_path.to_str().unwrap_or("err"));
                        // let result = fs::write(&path, &file.1).await;
                        (match fs::write(&file_path, &file.1).await {
                            Ok(v) => Ok(v),
                            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                                fs::create_dir_all(&root_path)
                                    .await
                                    .inspect(|_| trace!("created img output dir {:?}", &file_path))
                                    .inspect_err(|err| {
                                        error!("error creating img output dir {err}")
                                    })
                                    .map_err(|err| {
                                        ServerErr::ImgFailedToCreateOutputDir(err.to_string())
                                    })?;
                                fs::write(&file_path, &file.1).await
                            }
                            Err(err) => {
                                //
                                Err(err)
                            }
                        })
                        .inspect_err(|err| error!("failed to save img to disk {err:?}"))
                        .map_err(|err| ServerErr::ImgFailedToSaveImgToDisk(err.to_string()))?;
                        output_imgs.push(ServerOutputImg {
                            hash: file.0.hash.clone(),
                            extension: file.0.extension.clone(),
                        });
                        // if let Err(ref err) = result
                        //     && err.kind() == std::io::ErrorKind::NotFound
                        // {
                        //     fs::create_dir_all(path)
                        //         .await
                        //         .inspect_err(|err| error!("error creating img output dir {err}"))
                        //         .map_err(|err| {
                        //             ServerErr::ImgFailedToCreateOutputDir(err.to_string())
                        //         })?;
                        // }
                        //
                        // result
                        //     .inspect_err(|err| error!("failed to save img to disk {err:?}"))
                        //     .map_err(|err| ServerErr::ImgFailedToSaveImgToDisk(err.to_string()))?;
                        // app_state.settings.site.files_path
                    }

                    let time = app_state.clock.now().await;
                    let post_files = files.into_iter().map(|v| v.0).collect::<Vec<PostFile>>();
                    app_state
                        .db
                        .add_post(
                            time,
                            &auth_token.username,
                            &input.title,
                            &input.description,
                            post_files,
                        )
                        .await
                        .inspect_err(|err| error!("failed to save images {err:?}"))
                        .map_err(|_| ServerErr::ImgFailedToSaveImgMeta)?;

                    Result::<ServerOutput, ResErr<ServerErr>>::Ok(ServerOutput {
                        imgs: output_imgs,
                    })
                })()
                .await;

                // let exif_remover_format = ;

                // let metadata = ;
                // .map(|img| {
                //     let width = img.width();
                //     let height = img.height();
                //     let color = img.color();
                //     let rgb = img.to_rgb8();
                //
                //     let encoder = webp::Encoder::from_image(&img);
                //
                //     // let webp_encoder = image::codecs::webp::WebPEncoder::encode(
                //     //     &rgb, color, width, height,
                //     // );
                //     // let webp_encoder = webp::Encoder::new(
                //     //     &self.bytes,
                //     //     self.color,
                //     //     self.width,
                //     //     self.height,
                //     // );
                //     // webp_encoder.encode_simple(false, 10f32).or_else(|e| {
                //     //     Err(ImgDataEncodeWebpError::Encode(format!("{:?}", e)))
                //     // })?;
                //     // let bytes: Vec<u8> = r.to_vec();
                // });

                // let mut img: image::DynamicImage =
                //     image::io::Reader::new(Cursor::new(org_bytes))
                //         .with_guessed_format()?
                //         .decode()?;
                // let width = img.width();
                // let height = img.height();

                // Result::<AddPostDto, ServerErrImg>::Ok(AddPostDto {
                //     extension: ".png".to_string(),
                //     hash: "123".to_string(),
                //     width: 1,
                //     height: 1,
                // })
                // for file in input.files {
                //
                // }

                // app_state
                //     .db
                //     .delete_session(token)
                //     .await
                //     .map_err(|err| ResErr::ServerErr(ServerErr::ServerErr))?;
                // let token = match decode_token::<AuthToken>(
                //     &app_state.settings.auth.secret,
                //     &token,
                //     false,
                // ) {
                //     Ok(v) => v,
                //     Err(err) => {
                //         error!("invalid token was stored {err}");
                //         return Err(ResErr::ServerErr(ServerErr::JWT));
                //     }
                // };

                // let jar = jar.add(Cookie::new(
                //     AUTHORIZATION.as_str(),
                //     "Bearer=DELETED; Secure; HttpOnly; expires=Thu, 01 Jan 1970 00:00:00 GMT",
                // ));
                // (jar, encode_server_output(result))
                encode_server_output(result)
            }

            #[cfg(test)]
            pub async fn test_send<
                Token: Into<String>,
                Files: AsRef<[File]>,
                File: AsRef<std::path::Path>,
            >(
                server: &axum_test::TestServer,
                title: impl Into<String>,
                description: impl Into<String>,
                file_paths: Files,
                token: Token,
            ) -> (http::HeaderMap, Result<ServerOutput, ResErr<ServerErr>>) {
                use std::{ffi::OsStr, path::Path};

                use tokio::fs;

                use crate::router::API_PATH;
                let mut files = Vec::new();
                let paths = file_paths.as_ref();
                let title = title.into();
                let description = description.into();
                for path in paths {
                    let path = path.as_ref();
                    trace!("reading path: {path:?}");
                    // let name = path.extension().unwrap().to_str().unwrap().to_string();
                    let data = fs::read(path).await.unwrap();
                    files.push(data);
                    // files.push(InputFile {
                    //     extension: name,
                    //     data,
                    // });
                }

                let input = Input {
                    title,
                    description,
                    files,
                };
                let path = format!("{}{}", API_PATH, PATH);
                let token: String = token.into();
                let builder = server.reqwest_post(&path).header(
                    http::header::COOKIE,
                    format!("authorization=Bearer%3D{}%3B%20Secure%3B%20HttpOnly", token),
                );
                let res = send_from_builder::<ServerOutput, ServerErr>(builder, &input).await;
                trace!("RESPONSE: {res:#?}");
                res
            }

            #[cfg(test)]
            mod api {
                use std::path::Path;
                use std::sync::Arc;
                use std::time::Duration;
                use tokio::fs;

                use crate::app_state::AppState;
                use crate::auth::api::invite::InviteToken;
                use crate::auth::{
                    create_cookie, cut_cookie_full_with_expiration_encoded, decode_token,
                    encode_token, get_nanos, get_timestamp, test_extract_cookie,
                    test_extract_cookie_and_decode,
                };
                use crate::clock::Clock;
                use crate::utils::send_from_builder;
                use crate::{router, settings};

                use artbounty_db::db;
                use axum::{Router, middleware};
                // use axum::routing::post;
                use axum_test::TestServer;
                use gxhash::gxhash128;
                use http::header::SET_COOKIE;
                use test_log::test;
                use tokio::sync::Mutex;
                use tokio::time::sleep;
                use tracing::trace;

                #[test(tokio::test)]
                async fn post() {
                    let current_time = get_timestamp();
                    let time = Arc::new(Mutex::new(current_time));
                    let app_state = AppState::new_testng(time).await;
                    let my_app = router::new()
                        // .route_layer(middleware::from_fn_with_state(
                        //     app_state.clone(),
                        // ))
                        .with_state(app_state.clone());

                    let server = TestServer::builder()
                        .http_transport()
                        .build(my_app)
                        .unwrap();

                    {
                        let time = app_state.clock.now().await;
                        let exp = time + Duration::from_secs(60 * 30);

                        crate::auth::api::invite::test_send(&server, "hey1@hey.com")
                            .await
                            .1
                            .unwrap();
                        let invite = app_state
                            .db
                            .get_invite("hey1@hey.com", current_time)
                            .await
                            .unwrap();
                        let (cookies, res) = crate::auth::api::register::test_send(
                            &server,
                            "hey",
                            &invite.token_raw,
                            "wowowowow123@",
                        )
                        .await;
                        let token = test_extract_cookie(&cookies).unwrap();
                        let dir = std::env::current_dir()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string();
                        trace!("current working dir: {dir}");
                        // crate::auth::api::post::test_send(&server, [ format!("{}../assets/favicon.ico", dir) ], token)
                        let res = crate::post::api::create::test_send(
                            &server,
                            "hello",
                            "stuff",
                            ["/mnt/hdd2/pictures/me/EX6P5GmWsAE3uij.jpg"],
                            token,
                        )
                        .await
                        .1
                        .unwrap();
                        trace!("post api server ouput: {res:#?}");

                        for img in res.imgs {
                            let file_path = format!(
                                "{}/{}.{}",
                                &app_state.settings.site.files_path, img.hash, img.extension
                            );
                            let file_path = Path::new(&file_path);
                            let data = fs::read(file_path).await.unwrap();
                            let new_hash = format!("{:X}", gxhash128(&data, 0));
                            assert_eq!(img.hash, new_hash);
                            fs::remove_file(&file_path).await.unwrap();
                        }
                        // let res =
                        //     crate::auth::api::logout::test_send(&server, &invite.token_raw).await;
                        // let cookie = cut_cookie_full_with_expiration_encoded(
                        //     res.0.get(SET_COOKIE).unwrap().to_str().unwrap(),
                        // );
                        // assert_eq!(cookie, "DELETED");
                    }
                    // res.1.unwrap();
                }
            }
        }
    }
}

pub mod auth {
    use tracing::error;

    #[cfg(feature = "ssr")]
    use crate::utils::ResErr;

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
    #[cfg(feature = "ssr")]
    pub fn get_timestamp() -> std::time::Duration {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
    }

    #[cfg(feature = "ssr")]
    pub fn get_nanos() -> u128 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    #[cfg(feature = "ssr")]
    pub fn hash_password<S: Into<String>>(
        password: S,
    ) -> Result<String, argon2::password_hash::Error> {
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

        use crate::auth::encode_token;

        let key = key.as_ref();
        let token = encode_token(key, AuthToken::new(username, time.as_nanos()))
            .inspect_err(|err| error!("jwt exploded {err}"))?;
        trace!("token created: {token:?}");
        // .map_err(|_| OutputErr::CreateCookieErr)?;
        let cookie = format!("Bearer={token}; Secure; HttpOnly");
        trace!("cookie created: {cookie:?}");
        Ok((token, cookie))
    }

    pub fn cut_cookie_value_decoded(v: &str) -> &str {
        let start = "Bearer=";
        let end = "; Secure; HttpOnly";
        &v[start.len()..v.len() - end.len()]
    }

    pub fn cut_cookie_full_encoded(v: &str) -> &str {
        let start = "authorization=Bearer%3D";
        let end = "%3B%20Secure%3B%20HttpOnly";
        &v[start.len()..v.len() - end.len()]
    }

    pub fn cut_cookie_full_with_expiration_encoded(v: &str) -> &str {
        let start = "authorization=Bearer%3D";
        let end = "%3B%20Secure%3B%20HttpOnly%3B%20expires%3DThu%2C%2001%20Jan%201970%2000%3A00%3A00%20GMT";
        &v[start.len()..v.len() - end.len()]
    }

    #[cfg(feature = "ssr")]
    pub async fn check_auth<ServerErr>(
        app_state: &crate::app_state::AppState,
        jar: &axum_extra::extract::cookie::CookieJar,
    ) -> Result<AuthToken, ResErr<ServerErr>>
    where
        ServerErr: std::error::Error + 'static,
        //     ServerOutput: for<'a> rkyv::Serialize<
        //             rkyv::rancor::Strategy<
        //                 rkyv::ser::Serializer<rkyv::util::AlignedVec, rkyv::ser::allocator::ArenaHandle<'a>, rkyv::ser::Sharing::Share>,
        //                 bytecheck::rancor::Error,
        //             >,
        //         > + std::fmt::Debug
        //         + axum::response::IntoResponse,
        //     ServerErr: for<'a> rkyv::Serialize<
        //             rkyv::rancor::Strategy<
        //                 rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
        //                 bytecheck::rancor::Error,
        //             >,
        //         > + Archive
        //         + std::error::Error
        //         + axum::response::IntoResponse
        //         + std::fmt::Debug
        //         + 'static,
    {
        use axum_extra::extract::cookie::Cookie;
        use http::header::AUTHORIZATION;

        use crate::utils::{ResErr, ResErrUnauthorized};

        let token = jar
            .get(AUTHORIZATION.as_str())
            .ok_or(ResErr::Unauthorized(ResErrUnauthorized::NoCookie))
            .map(|v| cut_cookie_value_decoded(v.value()).to_string())?;

        let _session = app_state
            .db
            .get_session(&token)
            .await
            .map_err(|err| ResErr::<ServerErr>::Unauthorized(ResErrUnauthorized::NoCookie))?;

        let token = match decode_token::<AuthToken>(&app_state.settings.auth.secret, &token, false)
        {
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
        // let result = (async || -> Result<ServerOutput, ResErr<ServerErr>> {
        //     let token =
        //         match decode_token::<AuthToken>(&app_state.settings.auth.secret, &token, false) {
        //             Ok(v) => v,
        //             Err(err) => {
        //                 error!("invalid token was stored {err}");
        //                 app_state
        //                     .db
        //                     .delete_session(token)
        //                     .await
        //                     .map_err(|err| ResErr::ServerErr(ServerErr::ServerErr))?;
        //                 return Err(ResErr::ServerErr(ServerErr::JWT));
        //             }
        //         };
        //
        //     Ok(ServerOutput {
        //         username: token.claims.username,
        //     })
        // })()
        // .await;
        //
        // let jar = match &result {
        //     Err(ResErr::ServerErr(ServerErr::JWT)) => ,
        //     _ => jar,
        // };

        // let jar = jar.add(Cookie::new(
        //         AUTHORIZATION.as_str(),
        //         "Bearer=DELETED; Secure; HttpOnly; expires=Thu, 01 Jan 1970 00:00:00 GMT",
        //     ));

        Ok(token.claims)
    }

    #[cfg(test)]
    pub fn test_extract_cookie(headers: &http::HeaderMap) -> Option<String> {
        use crate::auth::decode_token;

        headers
            .get(http::header::SET_COOKIE)
            .map(|v| cut_cookie_full_encoded(v.to_str().unwrap()).to_string())
    }

    #[cfg(test)]
    pub fn test_extract_cookie_and_decode<Secret: Into<String>>(
        secret: Secret,
        headers: &http::HeaderMap,
    ) -> Option<(String, jsonwebtoken::TokenData<AuthToken>)> {
        use crate::auth::decode_token;

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
        use crate::auth::cut_cookie_value_decoded;

        #[test]
        fn cut_cookie() {
            cut_cookie_value_decoded("");
        }
    }

    pub mod api {
        pub mod logout {
            use std::time::Duration;

            use crate::utils::{ResErr, ServerDecodeErr, encode_result, send, send_from_builder};
            use thiserror::Error;
            use tracing::{error, trace};

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
                    let status = match &self {
                        // ServerErr::NoCookie => axum::http::StatusCode::OK,
                        _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    };
                    let bytes =
                        encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                    (status, bytes).into_response()
                }
            }

            pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
                send::<ServerOutput, ServerErr>(PATH, &input).await
            }

            #[cfg(feature = "ssr")]
            pub async fn server(
                axum::extract::State(app_state): axum::extract::State<crate::app_state::AppState>,
                jar: axum_extra::extract::cookie::CookieJar,
            ) -> impl axum::response::IntoResponse {
                trace!("executing profile api");
                use axum_extra::extract::cookie::Cookie;
                use http::header::AUTHORIZATION;

                use crate::{
                    auth::{
                        AuthToken, cut_cookie_value_decoded, decode_token, encode_token, get_nanos,
                    },
                    utils::encode_server_output,
                };

                let token = match jar
                    .get(AUTHORIZATION.as_str())
                    .ok_or(ResErr::ServerErr(ServerErr::NoCookie))
                    .map(|v| cut_cookie_value_decoded(v.value()).to_string())
                {
                    Ok(v) => v,
                    Err(err) => {
                        return (
                            jar,
                            encode_server_output(Result::<ServerOutput, ResErr<ServerErr>>::Err(
                                err,
                            )),
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
                    // let token = match decode_token::<AuthToken>(
                    //     &app_state.settings.auth.secret,
                    //     &token,
                    //     false,
                    // ) {
                    //     Ok(v) => v,
                    //     Err(err) => {
                    //         error!("invalid token was stored {err}");
                    //         return Err(ResErr::ServerErr(ServerErr::JWT));
                    //     }
                    // };

                    Ok(ServerOutput {})
                })()
                .await;

                let jar = jar.add(Cookie::new(
                    AUTHORIZATION.as_str(),
                    "Bearer=DELETED; Secure; HttpOnly; expires=Thu, 01 Jan 1970 00:00:00 GMT",
                ));
                (jar, encode_server_output(result))
            }

            #[cfg(test)]
            pub async fn test_send<Token: Into<String>>(
                server: &axum_test::TestServer,
                token: Token,
            ) -> (http::HeaderMap, Result<ServerOutput, ResErr<ServerErr>>) {
                use crate::router::API_PATH;

                let input = Input {
                    // token: token.into(),
                };
                let path = format!("{}{}", API_PATH, PATH);
                let token: String = token.into();
                let builder = server.reqwest_post(&path).header(
                    http::header::COOKIE,
                    format!("authorization=Bearer%3D{}%3B%20Secure%3B%20HttpOnly", token),
                );
                let res = send_from_builder::<ServerOutput, ServerErr>(builder, &input).await;
                trace!("RESPONSE: {res:#?}");
                res
            }

            #[cfg(test)]
            mod api {
                use std::sync::Arc;
                use std::time::Duration;

                use crate::app_state::AppState;
                use crate::auth::api::invite::InviteToken;
                use crate::auth::{
                    create_cookie, cut_cookie_full_with_expiration_encoded, decode_token,
                    encode_token, get_nanos, get_timestamp, test_extract_cookie,
                    test_extract_cookie_and_decode,
                };
                use crate::clock::Clock;
                use crate::utils::send_from_builder;
                use crate::{router, settings};

                use artbounty_db::db;
                use axum::Router;
                use axum::routing::post;
                use axum_test::TestServer;
                use http::header::SET_COOKIE;
                use test_log::test;
                use tokio::sync::Mutex;
                use tokio::time::sleep;
                use tracing::trace;

                #[test(tokio::test)]
                async fn logout() {
                    let current_time = get_timestamp();
                    let time = Arc::new(Mutex::new(current_time));
                    let app_state = AppState::new_testng(time).await;
                    let my_app = router::new().with_state(app_state.clone());

                    let server = TestServer::builder()
                        .http_transport()
                        .build(my_app)
                        .unwrap();

                    {
                        let time = app_state.clock.now().await;
                        let exp = time + Duration::from_secs(60 * 30);

                        crate::auth::api::invite::test_send(&server, "hey1@hey.com")
                            .await
                            .1
                            .unwrap();
                        let invite = app_state
                            .db
                            .get_invite("hey1@hey.com", current_time)
                            .await
                            .unwrap();

                        let (cookies, res) = crate::auth::api::register::test_send(
                            &server,
                            "hey",
                            &invite.token_raw,
                            "wowowowow123@",
                        )
                        .await;
                        let res =
                            crate::auth::api::logout::test_send(&server, &invite.token_raw).await;
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
            use std::time::Duration;

            use crate::utils::{ResErr, ServerDecodeErr, encode_result, send, send_from_builder};
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
                    let status = match &self {
                        ServerErr::NotFound => axum::http::StatusCode::NOT_FOUND,
                        _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    };
                    let bytes =
                        encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                    (status, bytes).into_response()
                }
            }

            pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
                send::<ServerOutput, ServerErr>(PATH, &input).await
            }

            #[cfg(feature = "ssr")]
            pub async fn server(
                axum::extract::State(app_state): axum::extract::State<crate::app_state::AppState>,
                jar: axum_extra::extract::cookie::CookieJar,
                multipart: axum::extract::Multipart,
            ) -> impl axum::response::IntoResponse {
                trace!("executing profile api");
                use artbounty_db::db::user::get_user_by_username;
                use axum_extra::extract::cookie::Cookie;
                use http::header::AUTHORIZATION;

                use crate::{
                    auth::{
                        AuthToken, cut_cookie_value_decoded, decode_token, encode_token, get_nanos,
                    },
                    utils::{decode_multipart, encode_server_output},
                };
                let result = (async || -> Result<ServerOutput, ResErr<ServerErr>> {
                    let input = decode_multipart::<Input, ServerErr>(multipart).await?;
                    trace!("input!!!!!! {input:#?}");
                    let user = app_state
                        .db
                        .get_user_by_username(input.username)
                        .await
                        .map_err(|err| match err {
                            get_user_by_username::GetUserByUsernameErr::UserNotFound => {
                                ServerErr::NotFound
                            }
                            _ => ServerErr::ServerErr,
                        })?;

                    Ok(ServerOutput {
                        username: user.username,
                    })
                })()
                .await;

                encode_server_output(result)
            }

            #[cfg(test)]
            pub async fn test_send(
                server: &axum_test::TestServer,
                username: impl Into<String>,
            ) -> (http::HeaderMap, Result<ServerOutput, ResErr<ServerErr>>) {
                use crate::router::API_PATH;

                let input = Input {
                    username: username.into(),
                };
                let path = format!("{}{}", API_PATH, PATH);
                let builder = server.reqwest_post(&path);
                let res = send_from_builder::<ServerOutput, ServerErr>(builder, &input).await;
                trace!("RESPONSE: {res:#?}");
                res
            }

            #[cfg(test)]
            mod api {
                use std::sync::Arc;
                use std::time::Duration;

                use crate::app_state::AppState;
                use crate::auth::api::invite::InviteToken;
                use crate::auth::{
                    create_cookie, cut_cookie_full_with_expiration_encoded, decode_token,
                    encode_token, get_nanos, get_timestamp, test_extract_cookie,
                    test_extract_cookie_and_decode,
                };
                use crate::clock::Clock;
                use crate::utils::{ResErr, send_from_builder};
                use crate::{router, settings};

                use super::ServerErr;
                use artbounty_db::db;
                use axum::Router;
                use axum::routing::post;
                use axum_test::TestServer;
                use http::header::SET_COOKIE;
                use test_log::test;
                use tokio::sync::Mutex;
                use tokio::time::sleep;
                use tracing::trace;

                #[test(tokio::test)]
                async fn user() {
                    let current_time = get_timestamp();
                    let time = Arc::new(Mutex::new(current_time));
                    let app_state = AppState::new_testng(time).await;
                    let my_app = router::new().with_state(app_state.clone());

                    let server = TestServer::builder()
                        .http_transport()
                        .build(my_app)
                        .unwrap();

                    {
                        let time = app_state.clock.now().await;
                        let exp = time + Duration::from_secs(60 * 30);
                        let user = crate::auth::api::user::test_send(&server, "hey").await;
                        assert!(matches!(
                            user.1,
                            Err(ResErr::ServerErr(ServerErr::NotFound))
                        ));
                        let _ = app_state.db.add_user("hey", "hey@hey.com", "123").await;
                        let user = crate::auth::api::user::test_send(&server, "hey")
                            .await
                            .1
                            .unwrap();
                    }
                }
            }
        }
        pub mod profile {
            use std::time::Duration;

            use crate::utils::{ResErr, ServerDecodeErr, encode_result, send, send_from_builder};
            use thiserror::Error;
            use tracing::{error, trace};

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
                //
                // #[error("jwt error")]
                // Unauthorized,
                //
                // #[error("jwt error")]
                // NoCookie,
                //
                // #[error("jwt error")]
                // JWT,
                // #[error("jwt expired error")]
                // JWTExpired,
            }

            #[cfg(feature = "ssr")]
            impl axum::response::IntoResponse for ServerErr {
                fn into_response(self) -> axum::response::Response {
                    let status = match &self {
                        // ServerErr::NoCookie => axum::http::StatusCode::OK,
                        _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    };
                    let bytes =
                        encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                    (status, bytes).into_response()
                }
            }

            pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
                send::<ServerOutput, ServerErr>(PATH, &input).await
            }

            #[cfg(feature = "ssr")]
            pub async fn server(
                axum::extract::State(app_state): axum::extract::State<crate::app_state::AppState>,
                jar: axum_extra::extract::cookie::CookieJar,
            ) -> impl axum::response::IntoResponse {
                trace!("executing profile api");
                use axum_extra::extract::cookie::Cookie;
                use http::header::AUTHORIZATION;

                use crate::{
                    auth::{
                        AuthToken, cut_cookie_value_decoded, decode_token, encode_token, get_nanos,
                    },
                    utils::encode_server_output,
                };

                // let token = match jar
                //     .get(AUTHORIZATION.as_str())
                //     .ok_or(ResErr::ServerErr(ServerErr::NoCookie))
                //     .map(|v| cut_cookie_value_decoded(v.value()).to_string())
                // {
                //     Ok(v) => v,
                //     Err(err) => {
                //         return (
                //             jar,
                //             encode_server_output(Result::<ServerOutput, ResErr<ServerErr>>::Err(
                //                 err,
                //             )),
                //         );
                //     }
                // };

                let result = (async || -> Result<ServerOutput, ResErr<ServerErr>> {
                    use crate::auth::check_auth;

                    let auth_token = check_auth(&app_state, &jar).await?;

                    // let _session = app_state
                    //     .db
                    //     .get_session(&token)
                    //     .await
                    //     .map_err(|err| ResErr::ServerErr(ServerErr::Unauthorized))?;
                    // let token = match decode_token::<AuthToken>(
                    //     &app_state.settings.auth.secret,
                    //     &token,
                    //     false,
                    // ) {
                    //     Ok(v) => v,
                    //     Err(err) => {
                    //         error!("invalid token was stored {err}");
                    //         app_state
                    //             .db
                    //             .delete_session(token)
                    //             .await
                    //             .map_err(|err| ResErr::ServerErr(ServerErr::ServerErr))?;
                    //         return Err(ResErr::ServerErr(ServerErr::JWT));
                    //     }
                    // };

                    Ok(ServerOutput {
                        username: auth_token.username,
                    })
                })()
                .await;

                // let jar = match &result {
                //     Err(ResErr::ServerErr(ServerErr::JWT)) => jar.add(Cookie::new(
                //         AUTHORIZATION.as_str(),
                //         "Bearer=DELETED; Secure; HttpOnly; expires=Thu, 01 Jan 1970 00:00:00 GMT",
                //     )),
                //     _ => jar,
                // };
                // (jar, encode_server_output(result))
                encode_server_output(result)

                // let token = match  {
                //         Ok(v) => v,
                //         Err(err) => {
                //             return (
                //                 jar,
                //                 encode_server_output(
                //                     Result::<ServerOutput, ResErr<ServerErr>>::Err(ResErr::ServerErr(err)),
                //                 ),
                //             );
                //         },
                //     };
                //
                //
                //
                //     .and_then(|v| {
                //         decode_token::<AuthToken>(&app_state.settings.auth.secret, v, false).map_err(
                //             |err| match err.kind() {
                //                 jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                //                     ServerErr::JWTExpired
                //                 }
                //                 _ => ServerErr::JWT,
                //             },
                //         )
                //     }) {
                //     Ok(v) => v,
                //     Err(err) => {
                //             return (
                //                 jar,
                //                 encode_server_output(
                //                     Result::<ServerOutput, ResErr<ServerErr>>::Err(ResErr::ServerErr(err)),
                //                 ),
                //             );
                //     }
                //
                // };
                //
                //     .map({
                //         let jar = jar.clone();
                //         |v| {
                //             (
                //                 jar,
                //                 encode_server_output(
                //                     Result::<ServerOutput, ResErr<ServerErr>>::Ok(ServerOutput {
                //                         username: v.claims.username,
                //                     }),
                //                 ),
                //             )
                //         }
                //     })
                //     .map_err(|err| ResErr::ServerErr(err))
                //     .unwrap_or_else(|err| {
                //         (
                //             // jar.add(Cookie::new(AUTHORIZATION.as_str(), "Bearer=DELETED; Secure; HttpOnly")),
                //             jar,
                //             encode_server_output(Result::<ServerOutput, ResErr<ServerErr>>::Err(
                //                 err,
                //             )),
                //         )
                //     })
            }

            #[cfg(test)]
            pub async fn test_send<Token: Into<String>>(
                server: &axum_test::TestServer,
                token: Token,
            ) -> (http::HeaderMap, Result<ServerOutput, ResErr<ServerErr>>) {
                use crate::router::API_PATH;

                let input = Input {
                    // token: token.into(),
                };
                let path = format!("{}{}", API_PATH, PATH);
                let token: String = token.into();
                let builder = server.reqwest_post(&path).header(
                    http::header::COOKIE,
                    format!("authorization=Bearer%3D{}%3B%20Secure%3B%20HttpOnly", token),
                );
                let res = send_from_builder::<ServerOutput, ServerErr>(builder, &input).await;
                trace!("RESPONSE: {res:#?}");
                res
            }

            #[cfg(test)]
            mod api {
                use std::sync::Arc;
                use std::time::Duration;

                use crate::app_state::AppState;
                use crate::auth::api::invite::InviteToken;
                use crate::auth::{
                    create_cookie, cut_cookie_full_with_expiration_encoded, decode_token,
                    encode_token, get_nanos, get_timestamp, test_extract_cookie,
                    test_extract_cookie_and_decode,
                };
                use crate::clock::Clock;
                use crate::utils::{ResErrUnauthorized, send_from_builder};
                use crate::{router, settings};

                use artbounty_db::db;
                use axum::Router;
                use axum::routing::post;
                use axum_test::TestServer;
                use http::header::SET_COOKIE;
                use test_log::test;
                use tokio::sync::Mutex;
                use tokio::time::sleep;
                use tracing::trace;

                #[test(tokio::test)]
                async fn profile() {
                    let current_time = get_timestamp();
                    let time = Arc::new(Mutex::new(current_time));
                    let app_state = AppState::new_testng(time).await;
                    let my_app = router::new().with_state(app_state.clone());

                    let server = TestServer::builder()
                        .http_transport()
                        .build(my_app)
                        .unwrap();

                    {
                        let time = app_state.clock.now().await;
                        let exp = time + Duration::from_secs(60 * 30);
                        let invite = InviteToken::new("hey@hey.com", time, exp);
                        let (token, _cookie) =
                            create_cookie(&app_state.settings.auth.secret, "hey", time).unwrap();
                        // let invite_token =
                        //     encode_token(&app_state.settings.auth.secret, invite).unwrap();
                        let res = crate::auth::api::profile::test_send(&server, token).await;
                        trace!("RESPONSE: {res:#?}");
                        assert!(matches!(
                            res.1,
                            Err(crate::utils::ResErr::Unauthorized(
                                ResErrUnauthorized::NoCookie
                            ))
                        ));

                        crate::auth::api::invite::test_send(&server, "hey1@hey.com")
                            .await
                            .1
                            .unwrap();
                        let invite = app_state
                            .db
                            .get_invite("hey1@hey.com", current_time)
                            .await
                            .unwrap();

                        let (cookies, res) = crate::auth::api::register::test_send(
                            &server,
                            "hey",
                            invite.token_raw,
                            "wowowowow123@",
                        )
                        .await;
                        let (token_raw, token) = test_extract_cookie_and_decode(
                            &app_state.settings.auth.secret,
                            &cookies,
                        )
                        .unwrap();
                        assert_eq!(token.claims.username, "hey");

                        let res = crate::auth::api::profile::test_send(&server, token_raw).await;
                        trace!("RESPONSE: {res:#?}");
                        assert!(matches!(res.1, Ok(_)));

                        let session = app_state.db.add_session("uwu", "hey").await.unwrap();

                        let res = crate::auth::api::profile::test_send(&server, "uwu").await;
                        trace!("RESPONSE: {res:#?}");
                        let cookie = cut_cookie_full_with_expiration_encoded(
                            res.0.get(SET_COOKIE).unwrap().to_str().unwrap(),
                        );
                        // let cookie = test_extract_cookie(&res.0).unwrap();
                        assert_eq!(cookie, "DELETED");
                        let session = app_state.db.get_session("uwu").await;
                        assert!(session.is_err());
                        // let input = crate::auth::api::invite::Input {
                        //     email: "hey@hey.com".to_string(),
                        // };
                        // let builder = server.reqwest_post(crate::auth::api::invite::PATH);
                        // let res = send_from_builder::<
                        //     crate::auth::api::invite::ServerOutput,
                        //     crate::auth::api::invite::ServerErr,
                        // >(builder, &input)
                        // .await;
                        // res.1.unwrap();
                    }
                    // res.1.unwrap();
                }
            }
        }
        pub mod invite_decode {
            use std::time::Duration;

            use crate::utils::{ResErr, ServerDecodeErr, encode_result, send, send_from_builder};
            use thiserror::Error;
            use tracing::{error, trace};

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
                    let status = match &self {
                        _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    };
                    let bytes =
                        encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                    (status, bytes).into_response()
                }
            }

            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            pub struct InviteToken {
                pub email: String,
                pub created_at: u128,
                pub exp: u64,
            }

            impl InviteToken {
                pub fn new<S: Into<String>>(
                    email: S,
                    created_at: std::time::Duration,
                    exp: std::time::Duration,
                ) -> Self {
                    Self {
                        email: email.into(),
                        created_at: created_at.as_nanos(),
                        exp: exp.as_secs(),
                    }
                }
            }

            pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
                send::<ServerOutput, ServerErr>(PATH, &input).await
            }

            #[cfg(feature = "ssr")]
            pub async fn server(
                axum::extract::State(app_state): axum::extract::State<crate::app_state::AppState>,
                // axum::extract::State(settings): axum::extract::State<crate::settings::Settings>,
                // axum::extract::State(clock): axum::extract::State<crate::clock::Clock>,
                multipart: axum::extract::Multipart,
            ) -> impl axum::response::IntoResponse {
                trace!("executing invite api");
                use artbounty_shared::auth::{
                    proccess_email, proccess_password, proccess_username,
                };

                use crate::{
                    api,
                    auth::{
                        AuthToken, decode_token, encode_token, get_nanos, get_timestamp,
                        hash_password,
                    },
                    utils::{self, decode_multipart, encode_server_output},
                };
                // tokio::time::sleep(Duration::from_secs(2)).await;
                let wrap = async || {
                    let input = decode_multipart::<Input, ServerErr>(multipart).await?;
                    trace!("input!!!!!! {input:#?}");
                    let time = app_state.clock.now().await;
                    let exp = time + Duration::from_secs(60 * 30);

                    let token = decode_token::<InviteToken>(
                        &app_state.settings.auth.secret,
                        input.token,
                        true,
                    )
                    .map_err(|err| match err.kind() {
                        jsonwebtoken::errors::ErrorKind::ExpiredSignature => ServerErr::JWTExpired,
                        _ => ServerErr::JWT,
                    })?;

                    // let token = match token {
                    //     Ok(token) => token,
                    //     Err() =>
                    // };

                    // match result {
                    //     Ok(_) => {}
                    //     Err(artbounty_db::db::invite::add_invite::AddInviteErr::DB(_)) => {
                    //         return Result::<ServerOutput, ResErr<ServerErr>>::Err(
                    //             ResErr::ServerErr(ServerErr::ServerErr),
                    //         );
                    //     }
                    //     Err(_) => {}
                    // }

                    Result::<ServerOutput, ResErr<ServerErr>>::Ok(ServerOutput {
                        email: token.claims.email,
                    })
                };
                trace!("1");
                let res = wrap().await;
                let res = encode_server_output(res);
                res
            }

            #[cfg(test)]
            pub async fn test_send<Token: Into<String>>(
                server: &axum_test::TestServer,
                token: Token,
            ) -> (http::HeaderMap, Result<ServerOutput, ResErr<ServerErr>>) {
                use crate::router::API_PATH;

                let input = Input {
                    token: token.into(),
                };
                let path = format!("{}{}", API_PATH, PATH);
                let builder = server.reqwest_post(&path);
                let res = send_from_builder::<ServerOutput, ServerErr>(builder, &input).await;
                trace!("RESPONSE: {res:#?}");
                res
            }

            #[cfg(test)]
            mod api {
                use std::sync::Arc;
                use std::time::Duration;

                use crate::app_state::AppState;
                use crate::auth::api::invite::InviteToken;
                use crate::auth::{decode_token, encode_token, get_nanos, get_timestamp};
                use crate::clock::Clock;
                use crate::utils::send_from_builder;
                use crate::{router, settings};

                use artbounty_db::db;
                use axum::Router;
                use axum::routing::post;
                use axum_test::TestServer;
                use test_log::test;
                use tokio::sync::Mutex;
                use tokio::time::sleep;
                use tracing::trace;

                #[test(tokio::test)]
                async fn token() {
                    // let time = get_nanos();
                    let time = get_timestamp();
                    let exp = time + Duration::from_secs(2);
                    let invite_token = InviteToken::new("hey@hey.com", time, exp);
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
                    let my_app = router::new().with_state(app_state.clone());

                    let server = TestServer::builder()
                        .http_transport()
                        .build(my_app)
                        .unwrap();

                    {
                        let time = app_state.clock.now().await;
                        let exp = time + Duration::from_secs(60 * 30);
                        let invite = InviteToken::new("hey@hey.com", time, exp);
                        let invite_token =
                            encode_token(&app_state.settings.auth.secret, invite).unwrap();
                        let res =
                            crate::auth::api::invite_decode::test_send(&server, invite_token).await;
                        // let input = crate::auth::api::invite::Input {
                        //     email: "hey@hey.com".to_string(),
                        // };
                        // let builder = server.reqwest_post(crate::auth::api::invite::PATH);
                        // let res = send_from_builder::<
                        //     crate::auth::api::invite::ServerOutput,
                        //     crate::auth::api::invite::ServerErr,
                        // >(builder, &input)
                        // .await;
                        trace!("RESPONSE: {res:#?}");
                        res.1.unwrap();
                    }
                    // res.1.unwrap();
                }
            }
        }
        pub mod invite {
            use std::time::Duration;

            use crate::utils::{ResErr, ServerDecodeErr, encode_result, send, send_from_builder};
            use thiserror::Error;
            use tracing::{error, trace};

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
                    let status = match &self {
                        _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    };
                    let bytes =
                        encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                    (status, bytes).into_response()
                }
            }

            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            pub struct InviteToken {
                pub email: String,
                pub created_at: u128,
                pub exp: u64,
            }

            impl InviteToken {
                pub fn new<S: Into<String>>(
                    email: S,
                    created_at: std::time::Duration,
                    exp: std::time::Duration,
                ) -> Self {
                    Self {
                        email: email.into(),
                        created_at: created_at.as_nanos(),
                        exp: exp.as_secs(),
                    }
                }
            }

            pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
                send::<ServerOutput, ServerErr>(PATH, &input).await
            }

            #[cfg(feature = "ssr")]
            pub async fn server(
                axum::extract::State(app_state): axum::extract::State<crate::app_state::AppState>,
                // axum::extract::State(settings): axum::extract::State<crate::settings::Settings>,
                // axum::extract::State(clock): axum::extract::State<crate::clock::Clock>,
                multipart: axum::extract::Multipart,
            ) -> impl axum::response::IntoResponse {
                trace!("executing invite api");
                use artbounty_shared::{
                    auth::{proccess_email, proccess_password, proccess_username},
                    fe_router::registration,
                };
                use tracing::debug;

                use crate::{
                    api,
                    auth::{encode_token, get_nanos, get_timestamp, hash_password},
                    utils::{self, decode_multipart, encode_server_output},
                };
                // tokio::time::sleep(Duration::from_secs(2)).await;
                let wrap = async || {
                    let input = decode_multipart::<Input, ServerErr>(multipart).await?;
                    trace!("input!!!!!! {input:#?}");
                    let time = app_state.clock.now().await;
                    let exp = time + Duration::from_secs(60 * 30);
                    let invite = InviteToken::new(input.email.clone(), time, exp);
                    let invite_token = encode_token(&app_state.settings.auth.secret, invite)
                        .map_err(|_| ServerErr::JWT)?;

                    trace!("invite token created: {invite_token}");

                    let invite = app_state
                        .db
                        .add_invite(time.clone(), invite_token, input.email, exp)
                        .await;
                    trace!("result {invite:?}");

                    match invite {
                        Ok(invite) => {
                            let link = format!(
                                "{}{}",
                                &app_state.settings.site.address,
                                registration::link_reg(&invite.token_raw),
                            );
                            trace!("{link}");
                        }
                        Err(err) => {
                            debug!("invite failed {err}");
                        }
                    }

                    // match result {
                    //     Ok(_) => {}
                    //     Err(artbounty_db::db::invite::add_invite::AddInviteErr::DB(_)) => {
                    //         return Result::<ServerOutput, ResErr<ServerErr>>::Err(
                    //             ResErr::ServerErr(ServerErr::ServerErr),
                    //         );
                    //     }
                    //     Err(_) => {}
                    // }

                    Result::<ServerOutput, ResErr<ServerErr>>::Ok(ServerOutput {})
                };
                trace!("1");
                let res = wrap().await;
                let res = encode_server_output(res);
                res
            }

            #[cfg(test)]
            pub async fn test_send<Email: Into<String>>(
                server: &axum_test::TestServer,
                email: Email,
            ) -> (http::HeaderMap, Result<ServerOutput, ResErr<ServerErr>>) {
                use crate::router::API_PATH;

                let input = Input {
                    email: email.into(),
                };
                let path = format!("{}{}", API_PATH, PATH);
                let builder = server.reqwest_post(&path);
                let res = send_from_builder::<ServerOutput, ServerErr>(builder, &input).await;
                trace!("RESPONSE: {res:#?}");
                res
            }

            #[cfg(test)]
            mod api {
                use std::sync::Arc;
                use std::time::Duration;

                use crate::app_state::AppState;
                use crate::auth::api::invite::{InviteToken, test_send};
                use crate::auth::{decode_token, encode_token, get_nanos, get_timestamp};
                use crate::clock::Clock;
                use crate::utils::send_from_builder;
                use crate::{router, settings};

                use artbounty_db::db;
                use artbounty_db::db::invite::get_invite::GetInviteErr;
                use axum::Router;
                use axum::routing::post;
                use axum_test::TestServer;
                use test_log::test;
                use tokio::sync::Mutex;
                use tokio::time::sleep;
                use tracing::trace;

                #[test(tokio::test)]
                async fn token() {
                    // let time = get_nanos();
                    let time = get_timestamp();
                    let exp = time + Duration::from_secs(2);
                    let invite_token = InviteToken::new("hey@hey.com", time, exp);
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
                    let my_app = router::new().with_state(app_state.clone());

                    let server = TestServer::builder()
                        .http_transport()
                        .build(my_app)
                        .unwrap();

                    {
                        let res =
                            crate::auth::api::invite::test_send(&server, "hey1@hey.com").await;
                        assert!(matches!(
                            res.1,
                            Ok(crate::auth::api::invite::ServerOutput {})
                        ));
                        let invite = app_state
                            .db
                            .get_invite("hey1@hey.com", current_time)
                            .await
                            .unwrap();
                        let res = crate::auth::api::register::test_send(
                            &server,
                            "hey",
                            invite.token_raw,
                            "hey1@hey.com",
                        )
                        .await;
                        let res =
                            crate::auth::api::invite::test_send(&server, "hey1@hey.com").await;
                        // trace!("{}");
                        assert!(matches!(
                            res.1,
                            Ok(crate::auth::api::invite::ServerOutput {})
                        ));
                        let invite = app_state.db.get_invite("hey1@hey.com", current_time).await;
                        assert!(matches!(invite, Err(GetInviteErr::NotFound)));
                        let invite = app_state.db.get_invite("hey2@hey.com", current_time).await;
                        assert!(matches!(invite, Err(GetInviteErr::NotFound)));
                        let res =
                            crate::auth::api::invite::test_send(&server, "hey2@hey.com").await;
                        assert!(matches!(
                            res.1,
                            Ok(crate::auth::api::invite::ServerOutput {})
                        ));
                        let invite = app_state.db.get_invite("hey2@hey.com", current_time).await;
                        assert!(matches!(invite, Ok(_)));
                        // let input = crate::auth::api::invite::Input {
                        //     email: "hey@hey.com".to_string(),
                        // };
                        // let builder = server.reqwest_post(crate::auth::api::invite::PATH);
                        // let res = send_from_builder::<
                        //     crate::auth::api::invite::ServerOutput,
                        //     crate::auth::api::invite::ServerErr,
                        // >(builder, &input)
                        // .await;
                        // trace!("RESPONSE: {res:#?}");
                        // res.1.unwrap();
                    }
                    // res.1.unwrap();
                }
            }
        }
        pub mod register {
            use crate::utils::{ResErr, ServerDecodeErr, encode_result, send, send_from_builder};
            use thiserror::Error;
            use tracing::{error, trace};

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

                // #[error("failed to decode input")]
                // DecodeErr(#[from] ServerDecodeErr),
                #[error("jwt error")]
                JWT,

                #[error("jwt expired error")]
                JWTExpired,

                #[error("create cookie err")]
                CreateCookieErr,

                #[error("internal server error")]
                ServerErr,
            }

            #[cfg(feature = "ssr")]
            impl axum::response::IntoResponse for ServerErr {
                fn into_response(self) -> axum::response::Response {
                    let status = match self {
                        ServerErr::EmailInvalid(_)
                        | ServerErr::UsernameInvalid(_)
                        | ServerErr::PasswordInvalid(_) => axum::http::StatusCode::BAD_REQUEST,
                        ServerErr::EmailTaken | ServerErr::UsernameTaken => {
                            axum::http::StatusCode::OK
                        }
                        _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    };
                    let bytes =
                        encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                    (status, bytes).into_response()
                }
            }

            pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
                send::<ServerOutput, ServerErr>(PATH, &input).await
            }

            #[cfg(feature = "ssr")]
            pub async fn server(
                axum::extract::State(app_state): axum::extract::State<crate::app_state::AppState>,
                jar: axum_extra::extract::cookie::CookieJar,
                multipart: axum::extract::Multipart,
            ) -> impl axum::response::IntoResponse {
                use artbounty_db::db::user::add_user::AddUserErr;
                use artbounty_shared::auth::{
                    proccess_email, proccess_password, proccess_username,
                };
                use axum_extra::extract::{CookieJar, cookie::Cookie};

                use crate::{
                    api,
                    auth::{
                        api::invite::InviteToken, create_cookie, decode_token, get_nanos,
                        hash_password,
                    },
                    utils::{self, decode_multipart, encode_server_output},
                };
                let wrap = async || {
                    let input = decode_multipart::<Input, ServerErr>(multipart).await?;
                    trace!("input!!!!!! {input:#?}");
                    let token_raw = input.email_token;
                    let time = app_state.clock.now().await;
                    let email_token = decode_token::<InviteToken>(
                        &app_state.settings.auth.secret,
                        &token_raw,
                        true,
                    )
                    .map_err(|err| match err.kind() {
                        jsonwebtoken::errors::ErrorKind::ExpiredSignature => ServerErr::JWTExpired,
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
                        .use_invite(token_raw, time)
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
                        let res = encode_server_output(res);
                        (cookies, res)
                    }
                    Err(err) => {
                        let res = Result::<ServerOutput, ResErr<ServerErr>>::Err(err);
                        let res = encode_server_output(res);
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
                use crate::router::API_PATH;

                let input = crate::auth::api::register::Input {
                    username: username.into(),
                    email_token: email_token.into(),
                    password: password.into(),
                };
                let path = format!("{}{}", API_PATH, PATH);
                let builder = server.reqwest_post(&path);
                let res = send_from_builder::<
                    crate::auth::api::register::ServerOutput,
                    crate::auth::api::register::ServerErr,
                >(builder, &input)
                .await;
                trace!("RESPONSE: {res:#?}");
                res
            }

            #[cfg(test)]
            mod api {
                use std::sync::Arc;
                use std::time::Duration;

                use crate::app_state::AppState;
                use crate::auth::api::invite::InviteToken;
                use crate::auth::{encode_token, get_timestamp, test_extract_cookie_and_decode};
                use crate::router;
                use crate::utils::{ResErr, send_from_builder};

                use artbounty_db::db;
                use axum::Router;
                use axum::routing::post;
                use axum_test::TestServer;
                use test_log::test;
                use tokio::sync::Mutex;
                use tracing::trace;

                #[test(tokio::test)]
                async fn register() {
                    let current_time = get_timestamp();
                    let time = Arc::new(Mutex::new(current_time));
                    let app_state = AppState::new_testng(time).await;
                    let secret = app_state.settings.auth.secret.clone();
                    let db = app_state.db.clone();
                    let my_app = router::new().with_state(app_state.clone());
                    let server = TestServer::builder()
                        .http_transport()
                        .build(my_app)
                        .unwrap();

                    {
                        // let email_token = {
                        //     let exp = current_time + Duration::from_secs(60 * 30);
                        //     let invite = InviteToken::new("hey@hey.com", current_time, exp);
                        //     encode_token(&secret, invite).unwrap()
                        // };
                        let res =
                            crate::auth::api::invite::test_send(&server, "hey1@hey.com").await;
                        res.1.unwrap();

                        let invite = db.get_invite("hey1@hey.com", current_time).await.unwrap();

                        let res = crate::auth::api::register::test_send(
                            &server,
                            "hey",
                            "broken",
                            "hey1@hey.com",
                        )
                        .await;
                        assert!(matches!(
                            res.1,
                            Err(ResErr::ServerErr(
                                crate::auth::api::register::ServerErr::JWT
                            ))
                        ));

                        let token =
                            test_extract_cookie_and_decode(&app_state.settings.auth.secret, &res.0);
                        assert!(token.is_none());

                        let res = crate::auth::api::register::test_send(
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

                        // let input = crate::auth::api::register::Input {
                        //     username: "hey".to_string(),
                        //     email_token,
                        //     password: "hey1@hey.com".to_string(),
                        // };
                        // let builder = server.reqwest_post(crate::auth::api::register::PATH);
                        // let res = send_from_builder::<
                        //     crate::auth::api::register::ServerOutput,
                        //     crate::auth::api::register::ServerErr,
                        // >(builder, &input)
                        // .await;
                        // trace!("RESPONSE: {res:#?}");
                    }
                }
            }
        }
        pub mod login {
            use crate::auth::AuthToken;
            use crate::utils::{ResErr, ServerDecodeErr, encode_result, send, send_from_builder};
            use thiserror::Error;
            use tracing::{error, trace};

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
                    // let bytes = match encode(&self) {
                    //     Ok(e) => e,
                    //     Err(err) => encode(&err).unwrap()
                    // };
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

                // #[error("failed to decode input")]
                // DecodeErr(#[from] ServerDecodeErr),
                #[error("incorrect email or password")]
                Incorrect,

                #[error("internal server error")]
                ServerErr,
            }

            #[cfg(feature = "ssr")]
            impl axum::response::IntoResponse for ServerErr {
                fn into_response(self) -> axum::response::Response {
                    let status = match self {
                        // ServerErr::DecodeErr(_) => axum::http::StatusCode::BAD_REQUEST,
                        ServerErr::Incorrect => axum::http::StatusCode::OK,
                        ServerErr::ServerErr | ServerErr::CreateCookieErr => {
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR
                        }
                    };
                    let bytes =
                        encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                    trace!("sending body: {bytes:?}");
                    (status, bytes).into_response()
                }
            }

            pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
                send::<ServerOutput, ServerErr>(PATH, &input).await
            }

            #[cfg(feature = "ssr")]
            pub async fn server(
                axum::extract::State(app_state): axum::extract::State<crate::app_state::AppState>,
                jar: axum_extra::extract::cookie::CookieJar,
                multipart: axum::extract::Multipart,
                // axum::extract::State(db): axum::extract::State<Wtf>,
            ) -> impl axum::response::IntoResponse {
                use std::time::Duration;

                use axum_extra::extract::cookie::Cookie;
                use tokio::time::sleep;

                // todo!();
                use crate::{
                    api,
                    auth::{create_cookie, get_nanos},
                    utils::{self, decode_multipart, encode_server_output},
                };
                // let db = artbounty_db::db::new_mem().await;
                trace!("yo wtf??");
                let result = (async || {
                    let input = decode_multipart::<Input, ServerErr>(multipart).await?;
                    trace!("input!!!!!! {input:#?}");
                    let user = app_state
                        .db
                        .get_user_by_email(input.email)
                        .await
                        .map_err(|_| ServerErr::Incorrect)?;
                    verify_password(input.password, user.password)
                        .map_err(|_| ServerErr::Incorrect)?;
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
                (jar, encode_server_output(output))
                // let res = match wrap().await {
                //     Ok(cookie) => {
                //         let server_output =
                //             Result::<ServerOutput, ResErr<ServerErr>>::Ok(ServerOutput {});
                //         let server_output = encode_server_output(server_output);
                //         let cookies =
                //             jar.add(Cookie::new(http::header::AUTHORIZATION.as_str(), cookie));
                //         (cookies, server_output)
                //     }
                //     Err(err) => {
                //         let server_output = Result::<ServerOutput, ResErr<ServerErr>>::Err(err);
                //         let server_output = encode_server_output(server_output);
                //         (jar, server_output)
                //     }
                // };
                // res
                // let r = recv(multipart, async |input: Input| {
                //     trace!("looking");
                //     // sleep(Duration::from_secs(2)).await;
                //
                //     trace!("input: {input:#?}");
                //
                //     // return Result::<ServerOutput, ServerErr>::Ok(ServerOutput {});
                //     // let username = validate_password(db, input.email, input.password).await?;
                //     // trace!("username: {username:#?}");
                //     let username = validate_password(db, input.email, input.password).await?;
                //     let time = get_nanos();
                //     let (_token, cookie) = create_cookie("secret", username.clone(), time)
                //         .map_err(|_| ServerErr::CreateCookieErr)?;
                //
                //     let a = jar.add(Cookie::new(http::header::AUTHORIZATION.as_str(), cookie));
                //
                //     // let Input { email, password } =
                //     //     utils::decode_multipart::<Input, ArchivedInput>(multipart).await?;
                //     //
                //
                //     // let archived =
                //     //     rkyv::access::<api::login::ArchivedArgs, rkyv::rancor::Error>(&*bytes).unwrap();
                //     // // let archived = rkyv::access::<Example, rkyv::rancor::Error>(&*bytes).unwrap();
                //     // let args =
                //     //     rkyv::deserialize::<api::login::Args, rkyv::rancor::Error>(archived).unwrap();
                //
                //     // trace!("args: {args:#?}");
                //
                //     // let response = expect_context::<ResponseOptions>();
                //     // response.set_status(Sta);
                //     // response.;
                //     // trace!("1");
                //
                //     // trace!("2");
                //
                //     // let time = get_nanos();
                //     // let (token, cookie) =
                //     //     create_cookie("secret", username.clone(), time).map_err(|_| LoginErr::ServerErr)?;
                //     // // let token = encode_token("secret", Claims::new(username, time)).map_err(|_| LoginErr::ServerErr)?;
                //     // // trace!("2.5");
                //     // // let cookie = format!("Bearer={token}; Secure; HttpOnly");
                //     // let r = DB.add_session(token.clone(), username).await;
                //     // trace!("r {r:#?}");
                //     // r.map_err(|_| LoginErr::ServerErr)?;
                //
                //     // trace!("3");
                //     // response.append_header(
                //     //     http::header::SET_COOKIE,
                //     //     HeaderValue::from_str(&cookie).unwrap(),
                //     // );
                //
                //     // Result::<Result<ServerOutput, ServerErr>>::Ok(ServerOutput {  })
                //     Result::<ServerOutput, ServerErr>::Ok(ServerOutput {})
                // })
                // .await;
                //
                // r
            }

            // #[cfg(feature = "ssr")]
            // async fn validate_password<
            //     C: artbounty_db::db::Connection,
            //     S: Into<String>,
            //     P: AsRef<[u8]>,
            // >(
            //     db: artbounty_db::db::Db<C>,
            //     email: S,
            //     password: P,
            // ) -> Result<String, ServerErr> {
            //     let user = db
            //         .get_user_by_email(email)
            //         .await
            //         .map_err(|_| ServerErr::Incorrect)?;
            //     let password_hash = user.password;
            //     let username = user.username;
            //
            //     trace!("1.5");
            //     let password_correct = verify_password(password, password_hash);
            //     if !password_correct {
            //         return Err(ServerErr::Incorrect);
            //     }
            //     Ok(username)
            // }

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
                use crate::router::API_PATH;

                let input = Input {
                    email: email.into(),
                    password: password.into(),
                };
                let path = format!("{}{}", API_PATH, PATH);
                let builder = server.reqwest_post(&path);
                let res = send_from_builder::<ServerOutput, ServerErr>(builder, &input).await;
                trace!("RESPONSE: {res:#?}");
                res
            }

            #[cfg(test)]
            mod api {
                use std::sync::Arc;
                use std::time::Duration;

                use crate::app_state::AppState;
                use crate::auth::api::invite::InviteToken;
                use crate::auth::{
                    AuthToken, decode_token, encode_token, get_timestamp,
                    test_extract_cookie_and_decode,
                };
                use crate::router;
                use crate::utils::{ResErr, send_from_builder};

                use artbounty_db::db;
                use axum::routing::post;
                use axum::{
                    Router,
                    body::Body,
                    extract::{FromRequest, Multipart, State},
                };
                use axum_test::TestServer;
                use http::request;
                use test_log::test;
                use tokio::sync::Mutex;
                use tracing::trace;

                #[test(tokio::test)]
                async fn login() {
                    let current_time = get_timestamp();
                    let time = Arc::new(Mutex::new(current_time));
                    let app_state = AppState::new_testng(time.clone()).await;
                    let db = app_state.db.clone();
                    // let secret = app_state.settings.auth.secret.clone();
                    let my_app = router::new().with_state(app_state.clone());
                    let server = TestServer::builder()
                        .http_transport()
                        .build(my_app)
                        .unwrap();

                    let res = crate::auth::api::invite::test_send(&server, "hey@hey.com").await;
                    res.1.unwrap();
                    let invite = db.get_invite("hey@hey.com", current_time).await.unwrap();

                    let res = crate::auth::api::register::test_send(
                        &server,
                        "hey",
                        invite.token_raw,
                        "hey1@hey.com",
                    )
                    .await;
                    assert!(matches!(
                        res.1,
                        Ok(crate::auth::api::register::ServerOutput { username })
                    ));
                    {
                        *time.lock().await += Duration::from_secs(1);
                    }
                    let res =
                        crate::auth::api::login::test_send(&server, "hey@hey.com", "hey1@hey.com")
                            .await;
                    let (token_raw, token) =
                        test_extract_cookie_and_decode(&app_state.settings.auth.secret, &res.0)
                            .unwrap();
                    assert_eq!(token.claims.username, "hey");
                    let session = app_state.db.get_session(&token_raw).await.unwrap();

                    let res = crate::auth::api::invite::test_send(&server, "hey2@hey.com").await;
                    res.1.unwrap();
                    let invite = db.get_invite("hey2@hey.com", current_time).await.unwrap();
                    let res = crate::auth::api::register::test_send(
                        &server,
                        "hey",
                        invite.token_raw,
                        "hey1@hey.com",
                    )
                    .await;
                    assert!(matches!(
                        res.1,
                        Err(ResErr::ServerErr(
                            crate::auth::api::register::ServerErr::UsernameTaken
                        ))
                    ));
                    //
                    // let register = async |username: &str, email: &str, password: &str| {
                    //     let email_token = {
                    //         let exp = current_time + Duration::from_secs(60 * 30);
                    //         let invite = InviteToken::new(email, current_time, exp);
                    //         encode_token(&secret, invite).unwrap()
                    //     };
                    //     let input = crate::auth::api::register::Input {
                    //         username: username.to_string(),
                    //         email_token,
                    //         password: password.to_string(),
                    //     };
                    //     let builder = server.reqwest_post(crate::auth::api::register::PATH);
                    //     let res = send_from_builder::<
                    //         crate::auth::api::register::ServerOutput,
                    //         crate::auth::api::register::ServerErr,
                    //     >(builder, &input)
                    //     .await;
                    //     trace!("RESPONSE: {res:#?}");
                    //     res.1
                    // };
                    //
                    // let login = async |email: &str, password: &str| {
                    //     let input = crate::auth::api::login::Input {
                    //         email: email.to_string(),
                    //         password: password.to_string(),
                    //     };
                    //     let builder = server.reqwest_post(crate::auth::api::login::PATH);
                    //     let res = send_from_builder::<
                    //         crate::auth::api::login::ServerOutput,
                    //         crate::auth::api::login::ServerErr,
                    //     >(builder, &input)
                    //     .await;
                    //     trace!("RESPONSE: {res:#?}");
                    //     res.0.get(http::header::SET_COOKIE).map(|v| {
                    //         let v = v.to_str().unwrap();
                    //         let start = "authorization=Bearer%3D";
                    //         let end = "%3B%20Secure%3B%20HttpOnly";
                    //         v[start.len()..v.len() - end.len()].to_string()
                    //     })
                    // };
                    // let reg = register("hey", "hey@hey.com", "hey1@hey.com").await;
                    // assert!(matches!(
                    //     reg,
                    //     Ok(crate::auth::api::register::ServerOutput {})
                    // ));
                    //
                    // let token = login("hey@hey.com", "hey1@hey.com").await.unwrap();
                    // trace!("encoded {token:#?}");
                    // let token = decode_token::<AuthToken>("secret", token, false).unwrap();
                    // trace!("decoded {token:#?}");
                    //

                    // let reg = register("hey", "hey2@hey.com", "hey1@hey.com").await;

                    // let db = artbounty_db::db::new_mem().await;
                    // let state = axum::extract::State(db);
                    // let body = Body::empty();
                    // let req = request::Builder::new().body(body).mu.unwrap();
                    // let multipart = Multipart::from_request(req, &State(())).await.unwrap();
                    // let result = server(state, multipart).await;
                }
            }
        }
    }

    // pub mod middleware {
    //     use axum::{
    //         RequestExt,
    //         extract::{FromRequest, Request},
    //         middleware::{self, Next},
    //         response::{IntoResponse, Response},
    //     };
    //     use http::header::AUTHORIZATION;
    //     use tower::Layer;
    //     use tracing::trace;
    //
    //     #[cfg(feature = "ssr")]
    //     use crate::auth::AuthToken;
    //     use crate::auth::{cut_cookie_value_decoded, decode_token};
    //
    //     #[derive(
    //         Debug,
    //         Clone,
    //         thiserror::Error,
    //         serde::Serialize,
    //         serde::Deserialize,
    //         rkyv::Archive,
    //         rkyv::Serialize,
    //         rkyv::Deserialize,
    //     )]
    //     pub enum VerifyCookieErr {
    //         #[error("jwt error")]
    //         JWT,
    //
    //         #[error("Bearer cookie not found")]
    //         CookieNotFound,
    //     }
    //
    //     pub async fn index(
    //         axum::extract::State(app_state): axum::extract::State<crate::app_state::AppState>,
    //         mut req: Request,
    //         next: Next,
    //     ) -> Response {
    //         let extensions = req.extensions_mut();
    //         extensions.insert(String::from("hello"));
    //         let jar: axum_extra::extract::cookie::CookieJar = req.extract_parts().await.unwrap();
    //         // let a = axum_extra::extract::cookie::CookieJar::from_request(, state).await;
    //         let cooks = jar
    //             .get(AUTHORIZATION.as_str())
    //             .map(|cookie| cut_cookie_value_decoded(cookie.value()).to_string())
    //             .and_then(|token| {
    //                 decode_token::<AuthToken>(&app_state.settings.auth.secret, &token, false).ok()
    //             });
    //         // trace!("cooooks {cooks:?}");
    //         // let r = "oof".into_response();
    //         // encode_server_output(Result::<ServerOutput, ResErr<ServerErr>>::Err(err));
    //
    //         // let token = match jar
    //         //     .get(AUTHORIZATION.as_str())
    //         //     .ok_or(ResErr::ServerErr(ServerErr::NoCookie))
    //         //     .map(|v| cut_cookie_value_decoded(v.value()).to_string())
    //         // {
    //         //     Ok(v) => v,
    //         //     Err(err) => {
    //         //         return (
    //         //             jar,
    //         //             encode_server_output(Result::<ServerOutput, ResErr<ServerErr>>::Err(err)),
    //         //         );
    //         //     }
    //         // };
    //         //
    //         // let result = (async || -> Result<ServerOutput, ResErr<ServerErr>> {
    //         //     let _session = app_state
    //         //         .db
    //         //         .get_session(&token)
    //         //         .await
    //         //         .map_err(|err| ResErr::ServerErr(ServerErr::Unauthorized))?;
    //         //     let token =
    //         //         match decode_token::<AuthToken>(&app_state.settings.auth.secret, &token, false)
    //         //         {
    //         //             Ok(v) => v,
    //         //             Err(err) => {
    //         //                 error!("invalid token was stored {err}");
    //         //                 app_state
    //         //                     .db
    //         //                     .delete_session(token)
    //         //                     .await
    //         //                     .map_err(|err| ResErr::ServerErr(ServerErr::ServerErr))?;
    //         //                 return Err(ResErr::ServerErr(ServerErr::JWT));
    //         //             }
    //         //         };
    //         //
    //         //     Ok(ServerOutput {
    //         //         username: token.claims.username,
    //         //     })
    //         // })()
    //         // .await;
    //         //
    //         // let jar = match &result {
    //         //     Err(ResErr::ServerErr(ServerErr::JWT)) => jar.add(Cookie::new(
    //         //         AUTHORIZATION.as_str(),
    //         //         "Bearer=DELETED; Secure; HttpOnly; expires=Thu, 01 Jan 1970 00:00:00 GMT",
    //         //     )),
    //         //     _ => jar,
    //         // };
    //
    //         let r = next.run(req).await;
    //         r
    //     }
    //
    //     // #[cfg(feature = "ssr")]
    //     // pub fn verify_cookie<Key: AsRef<[u8]>, Cookie: AsRef<str>>(
    //     //     secret: Key,
    //     //     cookie: Cookie,
    //     // ) -> Result<(String, jsonwebtoken::TokenData<AuthToken>), VerifyCookieErr> {
    //     //     use biscotti::{Processor, ProcessorConfig, RequestCookies};
    //     //     let processor: Processor = ProcessorConfig::default().into();
    //     //     let secret = secret.as_ref();
    //     //     let cookie = cookie.as_ref();
    //     //     RequestCookies::parse_header(cookie, &processor)
    //     //         .ok()
    //     //         .and_then(|cookies| cookies.get("Bearer"))
    //     //         .ok_or(VerifyCookieErr::CookieNotFound)
    //     //         .and_then(|cookie| {
    //     //             let token = cookie.value();
    //     //             decode_token(secret, token)
    //     //                 .map(|data| (token.to_string(), data))
    //     //                 // .inspect_err(|err| error!("jwt exploded {err}"))
    //     //                 .map_err(|_| VerifyCookieErr::JWT)
    //     //         })
    //     // }
    //
    //     #[cfg(feature = "ssr")]
    //     pub fn get_auth_cookie<Cookie: AsRef<str>>(cookie: Cookie) -> Option<String> {
    //         use biscotti::{Processor, ProcessorConfig, RequestCookies};
    //         let processor: Processor = ProcessorConfig::default().into();
    //         let cookie = cookie.as_ref();
    //         RequestCookies::parse_header(cookie, &processor)
    //             .ok()
    //             .and_then(|cookies| cookies.get(http::header::AUTHORIZATION.as_str()))
    //             .map(|cookie| cookie.value().to_string())
    //     }
    // }

    // pub async fn auth() -> Result<TokenData<Claims>, ServerFnError> {
    //     let header: HeaderMap = extract().await.unwrap();
    //     let Some((token, data)) = header
    //         .get(http::header::COOKIE)
    //         .and_then(|h| h.to_str().ok())
    //         .and_then(|cookie| verify_cookie("secret", cookie).ok())
    //     else {
    //         // trace!("AUTH BLOCK: {}", req.uri().to_string());
    //         return Err(ServerFnError::ServerError("unauthorized".to_string()));
    //     };
    //     let session = DB.get_session(token).await;

    //     Ok(data)
    // }

    // #[cfg(test)]
    // mod login_auth {
    //     use std::time::{SystemTime, UNIX_EPOCH};

    //     use crate::auth::{create_cookie, verify_cookie};

    //     use test_log::test;
    //     use tracing::trace;

    //     #[test]
    //     fn test_login() {
    //         let time = SystemTime::now()
    //             .duration_since(UNIX_EPOCH)
    //             .unwrap()
    //             .as_nanos();
    //         let (_token, cookie) = create_cookie("secret", "hey", time).unwrap();
    //         let cookie = cookie.split(";").next().unwrap();
    //         trace!("cookie {cookie:#?}");
    //         let claims = verify_cookie("secret", cookie).unwrap();
    //         trace!("claims {claims:#?}");

    //         // trace!("time {time}");
    //         // // let time = Timestamp::now();
    //         // let claims = Claims::new("hey", time);
    //         // let token = encode_token("secret", claims).unwrap();
    //         // trace!("\ntoken: {token}");
    //         // let decoded_token = decode_token("secret", &token).unwrap();
    //         // trace!("\ndecoded: {decoded_token:?}");
    //         // // let token2 = encode_token("secret", time).unwrap();
    //     }
    // }
}

// #[cfg(feature = "ssr")]
// pub mod middleware {

//     pub mod auth {
//         use std::{
//             pin::Pin,
//             task::{Context, Poll},
//         };

//         use axum::http::{Request, Response};

//         // use biscotti::{Processor, ProcessorConfig, RequestCookies};
//         use pin_project_lite::pin_project;
//         use server_fn::ServerFnError;
//         use thiserror::Error;

//         use tower::{Layer, Service};

//         use tracing::trace;

//         use crate::auth::verify_cookie;

//         // use crate::api::MidErr;

//         #[derive(Error, Debug)]
//         pub enum KaboomErr {
//             #[error("boom")]
//             Boom,
//         }

//         #[derive(Debug, Clone)]
//         pub struct AuthLayer;

//         impl<S> Layer<S> for AuthLayer {
//             type Service = AuthService<S>;

//             fn layer(&self, inner: S) -> Self::Service {
//                 AuthService { inner }
//             }
//         }

//         #[derive(Debug, Clone)]
//         pub struct AuthService<T> {
//             inner: T,
//         }

//         impl<S, ReqBody, ResBody, Err> Service<Request<ReqBody>> for AuthService<S>
//         where
//             S: Service<Request<ReqBody>, Response = Response<ResBody>, Error = ServerFnError<Err>>,
//             ResBody: Default + std::fmt::Debug,
//             Err: std::fmt::Debug,
//             ReqBody: std::fmt::Debug,
//         {
//             type Response = S::Response;
//             type Error = S::Error;
//             type Future = AuthServiceFuture<S::Future>;

//             fn poll_ready(
//                 &mut self,
//                 cx: &mut std::task::Context<'_>,
//             ) -> std::task::Poll<Result<(), Self::Error>> {
//                 self.inner.poll_ready(cx)
//             }

//             fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
//                 let Some((_token, _data)) = req
//                     .headers()
//                     .get(http::header::COOKIE)
//                     .and_then(|h| h.to_str().ok())
//                     .and_then(|cookie| verify_cookie("secret", cookie).ok())
//                 else {
//                     trace!("AUTH BLOCK: {}", req.uri().to_string());
//                     return AuthServiceFuture::Unauthorized;
//                 };

//                 // let session = DB.get_session(token).into_future().; // CANT

//                 {
//                     trace!("AUTH PASS: {}", req.uri().to_string());
//                     req.extensions_mut()
//                         .insert(String::from("wooooooooooooooooow"));
//                     // req.extensions_mut().insert(data);
//                 }
//                 {
//                     let r = req.extensions().get::<String>();
//                     trace!("r1 {r:#?}");
//                 }

//                 let f = self.inner.call(req);
//                 AuthServiceFuture::Future { future: f }
//                 // if session.is_ok() {
//                 //     AuthServiceFuture::Future {
//                 //         future: self.inner.call(req),
//                 //     }
//                 // } else {
//                 //     AuthServiceFuture::Unauthorized
//                 // }
//             }
//         }

//         pin_project! {
//             #[project = ResFutProj]
//             pub enum AuthServiceFuture<F> {
//                 Unauthorized,
//                 Future {
//                     #[pin]
//                     future: F,
//                 }
//             }
//         }

//         impl<F, Body, Err> Future for AuthServiceFuture<F>
//         where
//             F: Future<Output = Result<Response<Body>, ServerFnError<Err>>>,
//             Body: Default + std::fmt::Debug,
//             Err: std::fmt::Debug,
//         {
//             type Output = Result<Response<Body>, ServerFnError<Err>>;

//             fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//                 // let a = async move {
//                 //     trace!("the F is this dude");
//                 // }.into_future();
//                 // a.poll();

//                 // let session = DB.get_session("token").into_future(); // CANT
//                 // pin!(session);
//                 // let mut session = session;
//                 // let r = (session.as_mut()).poll(cx);
//                 // let session2 = DB.get_session("token").into_future(); // CANT
//                 // let mut a = Box::pin(session2);
//                 // let b = (a.as_mut()).poll(cx);

//                 // session.poll();
//                 // let f = Box::pin(async move { trace!("im dying"); });
//                 // let f2 = pin!(async move { trace!("im dying"); });
//                 // Future::

//                 match self.project() {
//                     ResFutProj::Unauthorized => {
//                         let err = Err(ServerFnError::MiddlewareError("unauthorized".to_string()));
//                         Poll::Ready(err)
//                     }
//                     ResFutProj::Future { future } => future.poll(cx),
//                 }
//             }
//         }

//         // #[cfg(test)]
//         // mod auth_tests {
//         //     use crate::middleware::auth::verify_cookie;

//         // }

//         // pub async fn verify(request: Request, next: Next) -> Result<impl IntoResponse, Response> {
//         //     trace!("im a middleware");
//         //     // let request = buffer_request_body(request).await?;

//         //     Ok(next.run(request).await)
//         // }
//     }
//     pub mod log {
//         use std::{
//             pin::Pin,
//             task::{Context, Poll},
//         };

//         use axum::{
//             body::Body,
//             extract::Request,
//             middleware::Next,
//             response::{IntoResponse, Response},
//         };
//         use pin_project_lite::pin_project;
//         use tower::{Layer, Service};
//         use tracing::trace;

//         #[derive(Debug, Clone)]
//         pub struct LogLayer;

//         impl<S> Layer<S> for LogLayer {
//             type Service = LogService<S>;

//             fn layer(&self, inner: S) -> Self::Service {
//                 LogService { inner }
//             }
//         }

//         #[derive(Debug, Clone)]
//         pub struct LogService<T> {
//             inner: T,
//         }

//         impl<T> Service<Request<Body>> for LogService<T>
//         where
//             T: Service<Request>,
//         {
//             type Response = T::Response;
//             type Error = T::Error;
//             type Future = LogServiceFuture<T::Future>;

//             fn poll_ready(
//                 &mut self,
//                 cx: &mut std::task::Context<'_>,
//             ) -> std::task::Poll<Result<(), Self::Error>> {
//                 self.inner.poll_ready(cx)
//             }

//             fn call(&mut self, req: Request<Body>) -> Self::Future {
//                 // req.headers().
//                 trace!("log where the hell am i");
//                 LogServiceFuture {
//                     inner: self.inner.call(req),
//                 }
//             }
//         }

//         pin_project! {
//             pub struct LogServiceFuture<T> {
//                 #[pin]
//                 inner: T,
//             }
//         }

//         impl<T> Future for LogServiceFuture<T>
//         where
//             T: Future,
//         {
//             type Output = T::Output;

//             fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//                 let this = self.project();

//                 match this.inner.poll(cx) {
//                     Poll::Pending => Poll::Pending,
//                     Poll::Ready(output) => {
//                         trace!("log runing middleware 3");
//                         Poll::Ready(output)
//                     }
//                 }
//             }
//         }

//         pub async fn verify(request: Request, next: Next) -> Result<impl IntoResponse, Response> {
//             trace!("im a middleware");
//             // let request = buffer_request_body(request).await?;

//             Ok(next.run(request).await)
//         }
//     }
// }
