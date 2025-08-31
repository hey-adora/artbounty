use futures::TryFutureExt;
use leptos::prelude::*;
use reqwest::RequestBuilder;
use rkyv::result::ArchivedResult;
use thiserror::Error;
use tracing::{debug, error, trace};
use wasm_bindgen_futures::spawn_local;

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
pub enum ServerReq {
    GetPostAfter { time: u128, limit: u32 },
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
pub enum ServerRes {
    Posts(Vec<Post>),
}

#[derive(
    Debug,
    Clone,
    Error,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    PartialEq,
)]
pub enum ServerErr {
    #[error("failed to deserialize req {0}")]
    ClientDesErr(String),

    #[error("failed to send req {0}")]
    ClientSendErr(String),

    #[error("failed to get multipart from request {0}")]
    ServerDesGettingMultipartErr(String),

    #[error("failed to get next field from multipart")]
    ServerDesNextFieldErr,

    #[error("failed to run field to bytes")]
    ServerDesFieldToBytesErr,

    #[error("failed to run rkyv access")]
    ServerDesRkyvAccessErr,

    #[error("failed to run rkyv deserialization")]
    ServerDesRkyvErr,

    #[error("wrong variant")]
    ServerWrongInput,

    #[error("database err")]
    ServerDbErr,
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
pub struct Post {
    pub hash: String,
    pub extension: String,
    pub width: u32,
    pub height: u32,
    pub created_at: u128,
}
// pub async fn send_web(req_builder: RequestBuilder, req: ServerReq) -> Result<ServerRes, ServerErr> {
//
// }

// pub trait ReqBuilderProvider {
//     fn provide_builder(&self) -> RequestBuilder;
// }

// pub struct Api<'a> {
//     pub provider: Box<&'a dyn ReqBuilderProvider>,
//     // pub clock: Box<&'a dyn ReqBuilderProvider>,
// }
pub trait Api {
    fn provide_builder(&self, path: impl AsRef<str>) -> RequestBuilder;
    fn provide_signal_result(&self) -> RwSignal<Option<Result<ServerRes, ServerErr>>>;
    fn provide_signal_busy(&self) -> RwSignal<bool>;
    // async fn send(&self, req_builder: RequestBuilder, req: ServerReq) -> Result<ServerRes, ServerErr>;

    fn get_posts_after(&self, time: u128, limit: u32) -> ApiReq {
        let builder = self.provide_builder(crate::path::PATH_API_POST_GET_AFTER);
        let server_req = ServerReq::GetPostAfter { time, limit };
        let result_signal = self.provide_signal_result();
        let busy_signal = self.provide_signal_busy();
        ApiReq {
            builder,
            server_req,
            result: result_signal,
            busy: busy_signal,
        }
    }
}

pub struct ApiReq {
    pub builder: RequestBuilder,
    pub server_req: ServerReq,
    pub result: RwSignal<Option<Result<ServerRes, ServerErr>>>,
    pub busy: RwSignal<bool>,
}

impl ApiReq {
    pub fn send_web<F, Fut>(self, fut: F)
    where
        F: Fn(Result<ServerRes, ServerErr>) -> Fut + 'static,
        Fut: Future<Output = ()>,
    {
        let req = self.server_req;
        let builder = self.builder;
        let signal_busy = self.busy;
        let signal_result = self.result;
        signal_busy.set(true);
        spawn_local(async move {
            let result = send(builder, req).await;
            fut(result.clone()).await;
            signal_result.set(Some(result));
            signal_busy.set(false);
        });
    }

    // pub fn value_tracked(&self) -> Option<Result<ServerRes, ServerErr>> {
    //     self.result.get()
    // }
    //
    // pub fn is_complete_tracked(&self) -> bool {
    //     // self.inner.with(|v| v.value.as_ref().map(|v| v.is_ok()).unwrap_or_default() )
    //     self.busy.with(|v| v.value.as_ref().is_some())
    // }
    //
    // pub fn is_pending_tracked(&self) -> bool {
    //     // self.inner.with(|v| v.value.as_ref().map(|v| v.is_ok()).unwrap_or_default() )
    //     self.inner.with(|v| v.pending)
    // }
    // pub fn is_pending_untracked(&self) -> bool {
    //     // self.inner.with(|v| v.value.as_ref().map(|v| v.is_ok()).unwrap_or_default() )
    //     self.inner.with_untracked(|v| v.pending)
    // }
    //
    // pub fn is_succ_tracked(&self) -> bool {
    //     self.inner
    //         .with(|v| v.value.as_ref().map(|v| v.is_ok()).unwrap_or_default())
    // }
    //
    // pub fn is_err_tracked(&self) -> bool {
    //     self.inner
    //         .with(|v| v.value.as_ref().map(|v| v.is_err()).unwrap_or_default())
    // }
}

#[derive(Clone, Copy, Default)]
pub struct ApiWeb {
    pub busy: RwSignal<bool>,
    pub result: RwSignal<Option<Result<ServerRes, ServerErr>>>,
}

impl ApiWeb {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Api for ApiWeb {
    fn provide_builder(&self, path: impl AsRef<str>) -> RequestBuilder {
        let origin = location().origin().unwrap();
        let path = path.as_ref();
        let url = format!("{origin}{}{path}", crate::path::PATH_API);
        reqwest::Client::new().post(url)
    }

    fn provide_signal_result(&self) -> RwSignal<Option<Result<ServerRes, ServerErr>>> {
        self.result
    }

    fn provide_signal_busy(&self) -> RwSignal<bool> {
        self.busy
    }

    // async fn send(
    //     &self,
    //     req_builder: RequestBuilder,
    //     req: ServerReq,
    // ) -> Result<ServerRes, ServerErr> {
    //     self.busy.set(true);
    //     let result = send(req_builder, req).await;
    //     self.busy.set(false);
    //     result
    // }
}

// impl Api<'_> {
//     pub async fn get_posts_after(&self) -> Result<ServerRes, ServerErr> {
//         let req_builder = self.provider.provide_builder();
//         send(req_builder, ServerReq::GetPostAfter { time: 0, limit: 25 }).await
//     }
// }

#[cfg(feature = "ssr")]
pub async fn recv(mut multipart: axum::extract::Multipart) -> Result<ServerReq, ServerErr> {
    let mut bytes = bytes::Bytes::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ServerErr::ServerDesNextFieldErr)?
    {
        if field.name().map(|name| name == "data").unwrap_or_default() {
            bytes = field
                .bytes()
                .await
                .map_err(|_| ServerErr::ServerDesFieldToBytesErr)?;
        }
    }
    debug!("SERVER RECV:\n{bytes:X}");

    let archived = rkyv::access::<ArchivedServerReq, rkyv::rancor::Error>(&bytes)
        .map_err(|_| ServerErr::ServerDesRkyvAccessErr)?;
    trace!("5");
    let client_input = rkyv::deserialize::<ServerReq, rkyv::rancor::Error>(archived)
        .map_err(|_| ServerErr::ServerDesRkyvErr)?;

    Ok(client_input)
}

#[cfg(feature = "ssr")]
impl axum::response::IntoResponse for ServerErr {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            ServerErr::ServerDbErr => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ServerErr::ServerDesRkyvErr
            | ServerErr::ServerWrongInput
            | ServerErr::ServerDesGettingMultipartErr(_)
            | ServerErr::ServerDesNextFieldErr
            | ServerErr::ServerDesRkyvAccessErr
            | ServerErr::ServerDesFieldToBytesErr => axum::http::StatusCode::BAD_REQUEST,
            ServerErr::ClientDesErr(_) | ServerErr::ClientSendErr(_) => unreachable!(),
        };

        let result: Result<ServerRes, ServerErr> = Err(self);
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&result).unwrap();
        let bytes = bytes.to_vec();
        let bytes: bytes::Bytes = bytes.into();
        (status, bytes).into_response()
    }
}

#[cfg(feature = "ssr")]
impl axum::response::IntoResponse for ServerRes {
    fn into_response(self) -> axum::response::Response {
        let result: Result<ServerRes, ServerErr> = Ok(self);
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&result).unwrap();
        let bytes = bytes.to_vec();
        let bytes: bytes::Bytes = bytes.into();
        debug!("SERVER SEND:\n{bytes:X}");
        bytes.into_response()
    }
}

pub async fn send(req_builder: RequestBuilder, req: ServerReq) -> Result<ServerRes, ServerErr> {
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&req).unwrap();
    debug!(
        "CLIENT SEND:\n {:X}\n{req:#?}",
        bytes::Bytes::copy_from_slice(bytes.as_ref())
    );
    let part = reqwest::multipart::Part::bytes(bytes.to_vec());
    let form = reqwest::multipart::Form::new().part("data", part);
    let req = req_builder
        .multipart(form)
        .send()
        .await
        .map_err(|err| ServerErr::ClientSendErr(err.to_string()))?;
    let status = req.status();
    let headers = req.headers().clone();

    let bytes = req
        .bytes()
        .await
        .map_err(|err| ServerErr::ClientDesErr(err.to_string()))
        .inspect_err(|err| {
            error!("client byte stream status {status}\nheaders: {headers:#?}\nerr: {err}")
        })?;

    let body = rkyv::access::<
        ArchivedResult<ArchivedServerRes, ArchivedServerErr>,
        rkyv::rancor::Error,
    >(bytes.as_ref())
    .and_then(|archive| {
        rkyv::deserialize::<Result<ServerRes, ServerErr>, rkyv::rancor::Error>(archive)
    })
    .map_err(|err| ServerErr::ClientDesErr(err.to_string()))
    .flatten();
    // .inspect_err(|err| error!("client byte stream err: {err}"));

    debug!(
        "CLIENT RECV:\nstatus: {status}\nclient received headers: {headers:#?}\nclient received: {bytes:X}"
    );

    body

    // let res = match req_builder
    //     .multipart(form)
    //     .send()
    //     .await
    //     .inspect_err(|err| error!("client err: {err}"))
    //     .map_err(|_| ResErr::ClientErr(ClientErr::FailedToSend))
    // {
    //     Ok(res) => res,
    //     Err(err) => {
    //         return (HeaderMap::new(), Err(err));
    //     }
    // };
}

#[cfg(feature = "ssr")]
pub async fn get_posts_after(
    axum::extract::State(app_state): axum::extract::State<crate::controller::app_state::AppState>,
    jar: axum_extra::extract::cookie::CookieJar,
    // username: Extension<String>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    let ServerReq::GetPostAfter { time, limit } = req else {
        return Err(ServerErr::ServerWrongInput);
    };
    let posts = app_state
        .db
        .get_post_after(time, limit)
        .await
        .map_err(|_| ServerErr::ServerDbErr)?
        .into_iter()
        .map(|post| {
            use chrono::{DateTime, Utc};
            use std::time::Duration;

            // let a = Duration::from_nanos(0);
            // let b = a.as_nanos();
            // let created_at: DateTime<Utc> = post.created_at.to_string();

            post.file
                .first()
                .cloned()
                .map(|post_file| Post {
                    hash: post_file.hash,
                    extension: post_file.extension,
                    width: post_file.width,
                    height: post_file.height,
                    created_at: post.created_at,
                })
                .unwrap_or(Post {
                    hash: "404".to_string(),
                    extension: "webp".to_string(),
                    width: 300,
                    height: 200,
                    created_at: 0,
                })
        })
        .collect::<Vec<Post>>();

    Ok(ServerRes::Posts(posts))
}

#[cfg(feature = "ssr")]
impl<S> axum::extract::FromRequest<S> for ServerReq
where
    S: Send + Sync,
{
    type Rejection = ServerErr;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        use axum::extract::Multipart;

        // let (a, b) = req.into_parts();
        let multipart = Multipart::from_request(req, state)
            .await
            .map_err(|err| ServerErr::ServerDesGettingMultipartErr(err.to_string()))?;
        recv(multipart).await
    }
}
