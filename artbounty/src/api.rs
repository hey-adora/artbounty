use http::header::AUTHORIZATION;
use leptos::prelude::*;
use reqwest::RequestBuilder;
use rkyv::result::ArchivedResult;
use thiserror::Error;
use tracing::{debug, error, trace};
use wasm_bindgen_futures::spawn_local;

#[cfg(feature = "ssr")]
pub mod app_state {
    use std::{sync::Arc, time::Duration};

    use tokio::sync::Mutex;

    use crate::{
        api::{
            clock::{Clock, get_timestamp},
            settings::Settings,
        },
        db::{self, DbEngine},
    };

    #[derive(Clone)]
    pub struct AppState {
        pub db: DbEngine,
        pub settings: Settings,
        pub clock: Clock,
    }

    impl AppState {
        pub async fn new() -> Self {
            let settings = Settings::new_from_file();
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
            let settings = Settings::new_testing();
            let f = move || {
                let time = time.clone();
                async move {
                    let t = *(time.lock().await);
                    t
                }
            };
            let clock = Clock::new(f);

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
        pub invite_exp_s: u64,
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
                    invite_exp_s: 1,
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
    use std::{pin::Pin, sync::Arc, time::Duration};

    #[derive(Clone)]
    pub struct Clock {
        ticker: Arc<
            dyn Fn() -> Pin<Box<dyn Future<Output = Duration> + Sync + Send + 'static>>
                + Sync
                + Send
                + 'static,
        >,
    }

    impl Clock {
        pub fn new<
            F: Fn() -> Fut + Send + Sync + Clone + 'static,
            Fut: Future<Output = Duration> + Send + Sync + 'static,
        >(
            ticker: F,
        ) -> Self {
            let fut = Arc::new(move || {
                let ticker = (ticker.clone())();
                let f: Pin<Box<dyn Future<Output = Duration> + Sync + Send + 'static>> =
                    Box::pin(ticker);
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
pub enum ServerReq {
    Login {
        email: String,
        password: String,
    },
    GetUser {
        username: String,
    },
    DecodeInvite {
        token: String,
    },
    GetInvite {
        email: String,
    },
    Register {
        username: String,
        invite_token: String,
        password: String,
    },
    GetPosts {
        time: u128,
        limit: u32,
    },
    AddPost {
        title: String,
        description: String,
        files: Vec<ServerReqImg>,
    },
    None,
}

#[cfg(feature = "ssr")]
impl<S> axum::extract::FromRequest<S> for ServerReq
where
    S: Send + Sync,
{
    type Rejection = ServerErr;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let multipart = axum::extract::Multipart::from_request(req, state)
            .await
            .map_err(|err| ServerDesErr::ServerDesGettingMultipartErr(err.to_string()))?;
        recv(multipart).await
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
pub struct ServerReqImg {
    pub path: String,
    pub data: Vec<u8>,
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
    SetAuthCookie { cookie: String },
    User { username: String },
    InviteToken(InviteToken),
    Posts(Vec<Post>),
    Ok,
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
    #[error("client err {0}")]
    ClientErr(#[from] ClientErr),

    #[error("auth err {0}")]
    ServerAuthErr(#[from] ServerAuthErr),

    #[error("login err {0}")]
    ServerLoginErr(#[from] ServerLoginErr),

    #[error("get user err {0}")]
    ServerGetUserErr(#[from] ServerGetUserErr),

    #[error("decode invite err {0}")]
    ServerDecodeInviteErr(#[from] ServerDecodeInviteErr),

    #[error("get invite err {0}")]
    ServerInviteErr(#[from] ServerInviteErr),

    #[error("add post err {0}")]
    ServerAddPostErr(#[from] ServerAddPostErr),

    #[error("registration err {0}")]
    ServerRegistrationErr(#[from] ServerRegistrationErr),

    #[error("add deserialization err {0}")]
    ServerDesErr(#[from] ServerDesErr),

    #[error("database err")]
    ServerDbErr,
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
pub enum ServerLoginErr {
    #[error("wrong credentials")]
    WrongCredentials,

    #[error("create cookie err {0}")]
    ServerCreateCookieErr(String),
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
pub enum ServerDesErr {
    #[error("wrong variant")]
    ServerWrongInput(String),

    #[error("failed to run field to bytes")]
    ServerDesFieldToBytesErr,

    #[error("failed to run rkyv access")]
    ServerDesRkyvAccessErr,

    #[error("failed to run rkyv deserialization")]
    ServerDesRkyvErr,

    #[error("failed to get next field from multipart")]
    ServerDesNextFieldErr,

    #[error("failed to get multipart from request {0}")]
    ServerDesGettingMultipartErr(String),
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
pub enum ClientErr {
    #[error("failed to deserialize req {0}")]
    ClientDesErr(String),

    #[error("failed to send req {0}")]
    ClientSendErr(String),
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
pub enum ServerGetUserErr {
    #[error("user not found")]
    NotFound,
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
pub enum ServerAuthErr {
    #[error("unauthorized no cookie")]
    ServerUnauthorizedNoCookie,

    #[error("unauthorized invalid cookie")]
    ServerUnauthorizedInvalidCookie,
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
pub enum ServerInviteErr {
    #[error("jwt error")]
    ServerJWT,
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
pub enum ServerAddPostErr {
    #[error("failed to create dir {0}")]
    ServerDirCreationFailed(String),

    #[error("file system err {0}")]
    ServerFSErr(String),

    #[error("invalid title {0}")]
    InvalidTitle(String),

    #[error("invalid description {0}")]
    InvalidDescription(String),

    #[error("img proccesing error {0:#?}")]
    ServerImgErr(Vec<ServerErrImgMeta>),
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
pub enum ServerDecodeInviteErr {
    #[error("invite not found")]
    InviteNotFound,

    #[error("jwt err {0}")]
    JWT(String),
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
pub enum ServerRegistrationErr {
    #[error("invalid registration input")]
    ServerRegistrationInvalidInput {
        username: Option<String>,
        email: Option<String>,
        password: Option<String>,
    },

    #[error("jwt error {0}")]
    ServerJWT(String),

    #[error("create cookie err")]
    ServerCreateCookieErr,

    #[error("jwt expired error")]
    TokenNotFound,

    #[error("jwt expired error")]
    TokenExpired,

    #[error("jwt expired error")]
    TokenUsed,
    // #[error("email is already in use")]
    // ServerEmailTaken,
    //
    // #[error("username is already in use")]
    // ServerUsernameTaken,
    //
    // #[error("{0}")]
    // ServerEmailInvalid(String),
    //
    // #[error("{0}")]
    // ServerUsernameInvalid(String),
    //
    // #[error("{0}")]
    // ServerPasswordInvalid(String),

    // #[error("invite token not found")]
    // ServerInviteTokenNotFound,
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
pub struct ServerErrImgMeta {
    pub path: String,
    pub err: ServerErrImg,
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
    PartialEq,
)]
pub enum ServerErrImg {
    #[error("failed to read img metadata {0}")]
    ServerImgMetadataReadFail(String),

    #[error("unsupported format {0}")]
    ServerImgUnsupportedFormat(String),

    #[error("img decode failed {0}")]
    ServerImgDecodeFailed(String),

    #[error("failed to create img webp encoder {0}")]
    ServerImgWebPEncoderCreationFailed(String),

    #[error("failed to encode img as webp {0}")]
    ServerImgWebPEncodingFailed(String),
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
    pub fn new<S: Into<String>>(email: S, created_at: u128) -> Self {
        Self {
            email: email.into(),
            created_at,
            exp: 0,
        }
    }
}

#[cfg(feature = "ssr")]
pub fn create_cookie<Key: AsRef<[u8]>, S: Into<String>>(
    key: Key,
    username: S,
    time: std::time::Duration,
) -> Result<(String, String), jsonwebtoken::errors::Error> {
    use tracing::trace;
    let key = key.as_ref();
    let token = encode_token(key, AuthToken::new(username, time.as_nanos()))
        .inspect_err(|err| error!("jwt exploded {err}"))?;
    trace!("token created: {token:?}");
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
pub fn verify_password<T: AsRef<[u8]>, S2: AsRef<str>>(
    password: T,
    hash: S2,
) -> Result<(), argon2::password_hash::Error> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let password = password.as_ref();
    let hash = hash.as_ref();
    PasswordHash::new(hash).and_then(|hash| Argon2::default().verify_password(password, &hash))
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

pub trait Api {
    fn provide_builder(&self, path: impl AsRef<str>) -> RequestBuilder;
    fn provide_signal_result(&self) -> Option<RwSignal<Option<Result<ServerRes, ServerErr>>>> {
        None
    }
    fn provide_signal_busy(&self) -> Option<RwSignal<bool>> {
        None
    }

    fn login(&self, email: impl Into<String>, password: impl Into<String>) -> ApiReq {
        let email = email.into();
        let password = password.into();
        let builder = self.provide_builder(crate::path::PATH_API_LOGIN);
        let server_req = ServerReq::Login { email, password };
        let result_signal = self.provide_signal_result();
        let busy_signal = self.provide_signal_busy();
        ApiReq {
            builder,
            server_req,
            result: result_signal,
            busy: busy_signal,
        }
    }

    fn logout(&self) -> ApiReq {
        let builder = self.provide_builder(crate::path::PATH_API_LOGOUT);
        let server_req = ServerReq::None;
        let result_signal = self.provide_signal_result();
        let busy_signal = self.provide_signal_busy();
        ApiReq {
            builder,
            server_req,
            result: result_signal,
            busy: busy_signal,
        }
    }

    fn get_user(&self, username: impl Into<String>) -> ApiReq {
        let username = username.into();
        let builder = self.provide_builder(crate::path::PATH_API_USER);
        let server_req = ServerReq::GetUser { username };
        let result_signal = self.provide_signal_result();
        let busy_signal = self.provide_signal_busy();
        ApiReq {
            builder,
            server_req,
            result: result_signal,
            busy: busy_signal,
        }
    }

    fn profile(&self) -> ApiReq {
        let builder = self.provide_builder(crate::path::PATH_API_PROFILE);
        let server_req = ServerReq::None;
        let result_signal = self.provide_signal_result();
        let busy_signal = self.provide_signal_busy();
        ApiReq {
            builder,
            server_req,
            result: result_signal,
            busy: busy_signal,
        }
    }

    fn decode_invite(&self, token: impl Into<String>) -> ApiReq {
        let token = token.into();
        let builder = self.provide_builder(crate::path::PATH_API_INVITE_DECODE);
        let server_req = ServerReq::DecodeInvite { token };
        let result_signal = self.provide_signal_result();
        let busy_signal = self.provide_signal_busy();
        ApiReq {
            builder,
            server_req,
            result: result_signal,
            busy: busy_signal,
        }
    }

    fn get_posts_newer(&self, time: u128, limit: u32) -> ApiReq {
        let builder = self.provide_builder(crate::path::PATH_API_POST_GET_NEWER);
        let server_req = ServerReq::GetPosts { time, limit };
        let result_signal = self.provide_signal_result();
        let busy_signal = self.provide_signal_busy();
        ApiReq {
            builder,
            server_req,
            result: result_signal,
            busy: busy_signal,
        }
    }

    fn get_posts_older(&self, time: u128, limit: u32) -> ApiReq {
        let builder = self.provide_builder(crate::path::PATH_API_POST_GET_OLDER);
        let server_req = ServerReq::GetPosts { time, limit };
        let result_signal = self.provide_signal_result();
        let busy_signal = self.provide_signal_busy();
        ApiReq {
            builder,
            server_req,
            result: result_signal,
            busy: busy_signal,
        }
    }

    fn add_post(
        &self,
        title: impl Into<String>,
        description: impl Into<String>,
        files: Vec<ServerReqImg>,
    ) -> ApiReq {
        let title = title.into();
        let description = description.into();
        let builder = self.provide_builder(crate::path::PATH_API_POST_ADD);
        let server_req = ServerReq::AddPost {
            title,
            description,
            files,
        };
        let result_signal = self.provide_signal_result();
        let busy_signal = self.provide_signal_busy();
        ApiReq {
            builder,
            server_req,
            result: result_signal,
            busy: busy_signal,
        }
    }

    fn get_invite(&self, email: impl Into<String>) -> ApiReq {
        let email = email.into();
        let builder = self.provide_builder(crate::path::PATH_API_INVITE);
        let server_req = ServerReq::GetInvite { email };
        let result_signal = self.provide_signal_result();
        let busy_signal = self.provide_signal_busy();
        ApiReq {
            builder,
            server_req,
            result: result_signal,
            busy: busy_signal,
        }
    }

    fn register(
        &self,
        username: impl Into<String>,
        invite_token: impl Into<String>,
        password: impl Into<String>,
    ) -> ApiReq {
        let username = username.into();
        let invite_token = invite_token.into();
        let password = password.into();
        let builder = self.provide_builder(crate::path::PATH_API_REGISTER);
        let server_req = ServerReq::Register {
            username,
            invite_token,
            password,
        };
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
    pub result: Option<RwSignal<Option<Result<ServerRes, ServerErr>>>>,
    pub busy: Option<RwSignal<bool>>,
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
        if let Some(signal_busy) = signal_busy {
            signal_busy.set(true);
        }
        spawn_local(async move {
            let (_, result) = send(builder, req, None::<&str>).await;
            fut(result.clone()).await;
            if let Some(signal_result) = signal_result {
                signal_result.set(Some(result));
            }
            if let Some(signal_busy) = signal_busy {
                signal_busy.set(false);
            }
        });
    }

    pub async fn send_native(self) -> Result<ServerRes, ServerErr> {
        let req = self.server_req;
        let builder = self.builder;
        let (_, result) = send(builder, req, None::<&str>).await;
        result
    }

    pub async fn send_native_with_token(
        self,
        token: impl AsRef<str>,
    ) -> Result<ServerRes, ServerErr> {
        let req = self.server_req;
        let builder = self.builder;
        let (_, result) = send(builder, req, Some(token)).await;
        result
    }

    #[cfg(test)]
    pub async fn send_native_and_extract_auth(
        self,
        secret: impl Into<String>,
    ) -> (
        Option<String>,
        Option<jsonwebtoken::TokenData<AuthToken>>,
        Result<ServerRes, ServerErr>,
    ) {
        use axum_extra::extract::CookieJar;
        use http::header::SET_COOKIE;

        let secret = secret.into();
        let req = self.server_req;
        let builder = self.builder;
        let (headers, result) = send(builder, req, None::<&str>).await;
        let jar = CookieJar::from_headers(&headers);
        let token = headers
            .get(SET_COOKIE)
            .map(|v| cut_cookie_full_encoded(v.to_str().unwrap()).to_string());
        let decoded_token = token
            .clone()
            .and_then(|cookie| decode_token::<AuthToken>(secret, cookie, false).ok());
        (token, decoded_token, result)
    }
}

#[cfg(test)]
#[derive(Clone, Copy)]
pub struct ApiTest<'a> {
    pub server: &'a axum_test::TestServer,
}

#[cfg(test)]
impl<'a> ApiTest<'a> {
    pub fn new(server: &'a axum_test::TestServer) -> Self {
        Self { server }
    }
}

#[cfg(test)]
impl<'a> Api for ApiTest<'a> {
    fn provide_builder(&self, path: impl AsRef<str>) -> RequestBuilder {
        let path = path.as_ref();
        let url = format!("{}{path}", crate::path::PATH_API);
        self.server.reqwest_post(&url)
    }
}

#[derive(Clone, Copy, Default)]
pub struct ApiWebpTmp {}

impl Api for ApiWebpTmp {
    fn provide_builder(&self, path: impl AsRef<str>) -> RequestBuilder {
        let origin = location().origin().unwrap();
        let path = path.as_ref();
        let url = format!("{origin}{}{path}", crate::path::PATH_API);
        reqwest::Client::new().post(url)
    }
}

impl ApiWebpTmp {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Default)]
pub struct ApiNative {
    pub origin: String,
}

impl Api for ApiNative {
    fn provide_builder(&self, path: impl AsRef<str>) -> RequestBuilder {
        let origin = &self.origin;
        let path = path.as_ref();
        let url = format!("{origin}{}{path}", crate::path::PATH_API);
        reqwest::Client::new().post(url)
    }
}

impl ApiNative {
    pub fn new(origin: impl Into<String>) -> Self {
        Self {
            origin: origin.into()
        }

    }
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

    pub fn is_succ_tracked(&self) -> bool {
        self.result
            .with(|v| v.as_ref().map(|v| v.is_ok()).unwrap_or_default())
    }

    pub fn is_pending_tracked(&self) -> bool {
        self.busy.get()
    }
}

impl Api for ApiWeb {
    fn provide_builder(&self, path: impl AsRef<str>) -> RequestBuilder {
        let origin = location().origin().unwrap();
        let path = path.as_ref();
        let url = format!("{origin}{}{path}", crate::path::PATH_API);
        reqwest::Client::new().post(url)
    }

    fn provide_signal_result(&self) -> Option<RwSignal<Option<Result<ServerRes, ServerErr>>>> {
        Some(self.result)
    }

    fn provide_signal_busy(&self) -> Option<RwSignal<bool>> {
        Some(self.busy)
    }
}

#[cfg(feature = "ssr")]
pub async fn recv(mut multipart: axum::extract::Multipart) -> Result<ServerReq, ServerErr> {
    let mut bytes = bytes::Bytes::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ServerDesErr::ServerDesNextFieldErr)?
    {
        if field.name().map(|name| name == "data").unwrap_or_default() {
            bytes = field
                .bytes()
                .await
                .map_err(|_| ServerDesErr::ServerDesFieldToBytesErr)?;
        }
    }

    let archived = rkyv::access::<ArchivedServerReq, rkyv::rancor::Error>(&bytes)
        .inspect_err(|err| error!("{err} SERVER RECV:\n{bytes:X}"))
        .map_err(|_| ServerDesErr::ServerDesRkyvAccessErr)?;
    trace!("5");
    let client_input = rkyv::deserialize::<ServerReq, rkyv::rancor::Error>(archived)
        .inspect_err(|err| error!("{err} SERVER RECV:\n{bytes:X}"))
        .map_err(|_| ServerDesErr::ServerDesRkyvErr)?;

    debug!("SERVER RECV:\n {client_input:?} - {bytes:X}");

    Ok(client_input)
}

#[cfg(feature = "ssr")]
impl axum::response::IntoResponse for ServerErr {
    fn into_response(self) -> axum::response::Response {
        use axum_extra::extract::{CookieJar, cookie::Cookie};

        let status = match self {
            ServerErr::ServerDbErr
            | ServerErr::ServerRegistrationErr(ServerRegistrationErr::ServerCreateCookieErr)
            | ServerErr::ServerLoginErr(ServerLoginErr::ServerCreateCookieErr(_))
            | ServerErr::ServerInviteErr(ServerInviteErr::ServerJWT) => {
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            ServerErr::ServerDesErr(_)
            | ServerErr::ServerAddPostErr(ServerAddPostErr::InvalidTitle(_))
            | ServerErr::ServerAddPostErr(ServerAddPostErr::InvalidDescription(_))
            | ServerErr::ServerRegistrationErr(ServerRegistrationErr::TokenExpired)
            | ServerErr::ServerRegistrationErr(ServerRegistrationErr::TokenUsed)
            | ServerErr::ServerRegistrationErr(ServerRegistrationErr::TokenNotFound)
            | ServerErr::ServerRegistrationErr(ServerRegistrationErr::ServerJWT(_))
            | ServerErr::ServerDecodeInviteErr(ServerDecodeInviteErr::InviteNotFound)
            | ServerErr::ServerDecodeInviteErr(ServerDecodeInviteErr::JWT(_))
            | ServerErr::ServerAddPostErr(ServerAddPostErr::ServerImgErr(_))
            | ServerErr::ServerAddPostErr(ServerAddPostErr::ServerFSErr(_))
            | ServerErr::ServerAddPostErr(ServerAddPostErr::ServerDirCreationFailed(_))
            | ServerErr::ServerRegistrationErr(
                ServerRegistrationErr::ServerRegistrationInvalidInput { .. },
            ) => axum::http::StatusCode::BAD_REQUEST,
            ServerErr::ServerAuthErr(ServerAuthErr::ServerUnauthorizedNoCookie) => {
                axum::http::StatusCode::OK
            }
            ServerErr::ServerAuthErr(ServerAuthErr::ServerUnauthorizedInvalidCookie)
            | ServerErr::ServerLoginErr(ServerLoginErr::WrongCredentials) => {
                axum::http::StatusCode::UNAUTHORIZED
            }
            ServerErr::ServerGetUserErr(ServerGetUserErr::NotFound) => {
                axum::http::StatusCode::NOT_FOUND
            }
            ServerErr::ClientErr(_) => unreachable!(),
        };

        match self {
            ServerErr::ServerAuthErr(ServerAuthErr::ServerUnauthorizedInvalidCookie) => {
                let result: Result<ServerRes, ServerErr> = Err(self);
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&result).unwrap();
                let bytes = bytes.to_vec();
                let bytes: bytes::Bytes = bytes.into();
                let jar = CookieJar::new().add(Cookie::new(
                    AUTHORIZATION.as_str(),
                    "Bearer=DELETED; Secure; HttpOnly; expires=Thu, 01 Jan 1970 00:00:00 GMT",
                ));
                (status, jar, bytes).into_response()
            }
            server_err => {
                let result: Result<ServerRes, ServerErr> = Err(server_err);
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&result).unwrap();
                let bytes = bytes.to_vec();
                let bytes: bytes::Bytes = bytes.into();
                (status, bytes).into_response()
            }
        }
    }
}

#[cfg(feature = "ssr")]
impl axum::response::IntoResponse for ServerRes {
    fn into_response(self) -> axum::response::Response {
        use axum_extra::extract::{CookieJar, cookie::Cookie};

        match self {
            ServerRes::SetAuthCookie { cookie } => {
                let jar = CookieJar::new().add(Cookie::new(
                    http::header::AUTHORIZATION.as_str(),
                    cookie.clone(),
                ));

                let result: Result<ServerRes, ServerErr> = Ok(ServerRes::Ok);
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&result).unwrap();
                let bytes = bytes.to_vec();
                let bytes: bytes::Bytes = bytes.into();

                debug!("SERVER SEND:\n{result:?} - {bytes:X}");

                (jar, bytes).into_response()
            }
            res => {
                let result: Result<ServerRes, ServerErr> = Ok(res);
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&result).unwrap();
                let bytes = bytes.to_vec();
                let bytes: bytes::Bytes = bytes.into();
                debug!("SERVER SEND:\n{result:?} - {bytes:X}");

                bytes.into_response()
            }
        }
    }
}

pub async fn send(
    mut req_builder: RequestBuilder,
    req: ServerReq,
    token: Option<impl AsRef<str>>,
) -> (http::HeaderMap, Result<ServerRes, ServerErr>) {
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&req).unwrap();
    debug!(
        "CLIENT SEND:\n{req:?} - {:X}",
        bytes::Bytes::copy_from_slice(bytes.as_ref())
    );
    let part = reqwest::multipart::Part::bytes(bytes.to_vec());
    let form = reqwest::multipart::Form::new().part("data", part);
    if let Some(token) = token {
        trace!("cookie set");
        req_builder = req_builder.header(
            http::header::COOKIE,
            format!(
                "{}=Bearer%3D{}%3B%20Secure%3B%20HttpOnly",
                AUTHORIZATION,
                token.as_ref()
            ),
        );
    }
    let res = req_builder
        .multipart(form)
        .send()
        .await
        .inspect_err(|err| error!("client failed to send {err}"))
        .map_err(|err| ServerErr::from(ClientErr::ClientSendErr(err.to_string())));
    let res = match res {
        Ok(v) => v,
        Err(err) => return (http::HeaderMap::new(), Err(err)),
    };

    let status = res.status();
    let headers = res.headers().clone();

    let bytes = match res
        .bytes()
        .await
        .map_err(|err| ClientErr::ClientDesErr(err.to_string()))
        .inspect_err(|err| {
            error!("client byte stream status {status}\nheaders: {headers:#?}\nerr: {err}")
        }) {
        Ok(bytes) => bytes,
        Err(err) => {
            return (
                headers,
                Err(ServerErr::from(ClientErr::ClientDesErr(err.to_string()))),
            );
        }
    };

    let body = rkyv::access::<
        ArchivedResult<ArchivedServerRes, ArchivedServerErr>,
        rkyv::rancor::Error,
    >(bytes.as_ref())
    .and_then(|archive| {
        rkyv::deserialize::<Result<ServerRes, ServerErr>, rkyv::rancor::Error>(archive)
    })
    .map_err(|err| ServerErr::from(ClientErr::ClientDesErr(err.to_string())))
    .flatten();

    debug!(
        "CLIENT RECV:\nstatus: {status}\nclient received headers: {headers:#?}\n{body:?} - {bytes:X}"
    );

    (headers, body)
}

#[cfg(feature = "ssr")]
pub mod backend {
    use crate::api::app_state::AppState;
    use crate::api::{
        AuthToken, InviteToken, Post, ServerAddPostErr, ServerAuthErr, ServerDecodeInviteErr,
        ServerDesErr, ServerErr, ServerErrImg, ServerErrImgMeta, ServerGetUserErr, ServerInviteErr,
        ServerLoginErr, ServerRegistrationErr, ServerReq, ServerRes, create_cookie,
        cut_cookie_value_decoded, decode_token, encode_token, hash_password, verify_password,
    };
    use crate::db::invite::add_invite::AddInviteErr;
    use crate::db::invite::get_invite::GetInviteErr;
    use crate::db::post::PostFile;
    use crate::db::session::get_session::GetSessionErr;
    use crate::db::user::add_user::AddUserErr;
    use crate::db::user::get_user_by_username::GetUserByUsernameErr;
    use crate::valid::auth::{
        proccess_password, proccess_post_description, proccess_post_title, proccess_username,
    };
    use axum::Extension;
    use axum::extract::State;
    use axum::response::IntoResponse;
    use axum_extra::extract::CookieJar;
    use axum_extra::extract::cookie::Cookie;
    use gxhash::{gxhash64, gxhash128};
    use http::header::AUTHORIZATION;
    use image::{ImageFormat, ImageReader};
    use little_exif::{filetype::FileExtension, metadata::Metadata};
    use std::time::Duration;
    use std::{io::Cursor, path::Path, str::FromStr};
    use tokio::fs;
    use tracing::{debug, error, info, trace};

    pub async fn get_user(
        State(app_state): State<AppState>,
        req: ServerReq,
    ) -> Result<ServerRes, ServerErr> {
        let ServerReq::GetUser { username } = req else {
            return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                "expected GetUser, received: {req:?}"
            ))));
        };

        let user = app_state
            .db
            .get_user_by_username(username)
            .await
            .map_err(|err| match err {
                GetUserByUsernameErr::UserNotFound => ServerGetUserErr::NotFound.into(),
                _ => ServerErr::ServerDbErr,
            })?;

        Ok(ServerRes::User {
            username: user.username,
        })
    }

    pub async fn logout(
        State(app_state): State<AppState>,
        jar: CookieJar,
    ) -> Result<ServerRes, ServerErr> {
        let token = jar
            .get(AUTHORIZATION.as_str())
            .ok_or(ServerErr::ServerAuthErr(
                ServerAuthErr::ServerUnauthorizedNoCookie,
            ))
            .map(|v| cut_cookie_value_decoded(v.value()).to_string())?;

        app_state
            .db
            .get_session(&token)
            .await
            .map_err(|_err| ServerErr::from(ServerAuthErr::ServerUnauthorizedInvalidCookie))?;

        app_state
            .db
            .delete_session(token)
            .await
            .map_err(|_err| ServerErr::from(ServerAuthErr::ServerUnauthorizedInvalidCookie))?;

        Ok(ServerRes::Ok)
    }

    pub async fn decode_invite(
        State(app_state): State<AppState>,
        req: ServerReq,
    ) -> Result<ServerRes, ServerErr> {
        let ServerReq::DecodeInvite { token } = req else {
            return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                "expected Register, received: {req:?}"
            ))));
        };

        let token = decode_token::<InviteToken>(&app_state.settings.auth.secret, token, false)
            .map_err(|err| ServerDecodeInviteErr::JWT(err.to_string()))?;

        Ok(ServerRes::InviteToken(token.claims))
    }

    pub async fn profile(
        State(app_state): State<AppState>,
        auth_token: Extension<AuthToken>,
    ) -> Result<ServerRes, ServerErr> {
        Ok(ServerRes::User {
            username: auth_token.username.clone(),
        })
    }

    pub async fn register(
        State(app_state): State<AppState>,
        req: ServerReq,
    ) -> Result<ServerRes, ServerErr> {
        let ServerReq::Register {
            username,
            invite_token,
            password,
        } = req
        else {
            return Err(ServerDesErr::ServerWrongInput(format!(
                "expected Register, received: {req:?}"
            ))
            .into());
        };
        let time = app_state.clock.now().await;
        let time_ns = time.as_nanos();

        let invite_token_decoded = app_state
            .db
            .get_invite_by_token(&invite_token)
            .await
            .inspect_err(|err| error!("failed to run use_invite {err}"))
            .map_err(|err| match err {
                GetInviteErr::DB(_) => ServerErr::ServerDbErr,
                GetInviteErr::NotFound => ServerRegistrationErr::TokenNotFound.into(),
            })
            .and_then(|invite| {
                if invite.expires < time_ns {
                    return Err(ServerRegistrationErr::TokenExpired.into());
                }
                if invite.used {
                    return Err(ServerRegistrationErr::TokenUsed.into());
                }
                decode_token::<InviteToken>(&app_state.settings.auth.secret, &invite_token, false)
                    .map_err(|err| ServerRegistrationErr::ServerJWT(err.to_string()).into())
            })?;

        let email = invite_token_decoded.claims.email;
        let username = proccess_username(username);
        let password = proccess_password(password, None)
            .and_then(|pss| hash_password(pss).map_err(|_| "hasher error".to_string()));

        let (Ok(username), Ok(password)) = (&username, &password) else {
            return Err(ServerErr::from(
                ServerRegistrationErr::ServerRegistrationInvalidInput {
                    username: username.err(),
                    email: None,
                    password: password.err(),
                },
            ));
        };

        let user = app_state
            .db
            .add_user(username, email, password)
            .await
            .map_err(|err| match err {
                AddUserErr::EmailIsTaken(_) => {
                    ServerRegistrationErr::ServerRegistrationInvalidInput {
                        username: None,
                        email: Some("email is taken".to_string()),
                        password: None,
                    }
                    .into()
                }
                AddUserErr::UsernameIsTaken(_) => {
                    ServerRegistrationErr::ServerRegistrationInvalidInput {
                        username: Some("username is taken".to_string()),
                        email: None,
                        password: None,
                    }
                    .into()
                }
                _ => ServerErr::ServerDbErr,
            })?;

        let result = app_state
            .db
            .use_invite(&invite_token, time.as_nanos())
            .await
            .inspect_err(|err| error!("failed to run use_invite {err}"))
            .map_err(|err| ServerErr::ServerDbErr)?;

        let (token, cookie) = create_cookie(&app_state.settings.auth.secret, &user.username, time)
            .map_err(|_| ServerRegistrationErr::ServerCreateCookieErr)?;

        let _session = app_state
            .db
            .add_session(token, &user.username)
            .await
            .map_err(|err| ServerErr::ServerDbErr)?;

        Ok(ServerRes::SetAuthCookie { cookie })
    }

    pub async fn get_posts_newer(
        State(app_state): State<AppState>,
        req: ServerReq,
    ) -> Result<ServerRes, ServerErr> {
        let ServerReq::GetPosts { time, limit } = req else {
            return Err(ServerDesErr::ServerWrongInput(format!(
                "expected GetPostAfter, received: {req:?}"
            ))
            .into());
        };
        let posts = app_state
            .db
            .get_post_newer(time, limit)
            .await
            .map_err(|_| ServerErr::ServerDbErr)?
            .into_iter()
            .map(|post| {
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

    pub async fn get_posts_older(
        State(app_state): State<AppState>,
        req: ServerReq,
    ) -> Result<ServerRes, ServerErr> {
        let ServerReq::GetPosts { time, limit } = req else {
            return Err(ServerDesErr::ServerWrongInput(format!(
                "expected GetPostAfter, received: {req:?}"
            ))
            .into());
        };
        let posts = app_state
            .db
            .get_post_older(time, limit)
            .await
            .map_err(|_| ServerErr::ServerDbErr)?
            .into_iter()
            .map(|post| {
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

    pub async fn add_post(
        State(app_state): State<AppState>,
        auth_token: axum::Extension<AuthToken>,
        req: ServerReq,
    ) -> Result<ServerRes, ServerErr> {
        let ServerReq::AddPost {
            title,
            description,
            files,
        } = req
        else {
            return Err(ServerDesErr::ServerWrongInput(format!(
                "expected AddPost, received: {req:?}"
            ))
            .into());
        };
        let time = app_state.clock.now().await;

        let title = proccess_post_title(title)
            .map_err(|err| ServerAddPostErr::InvalidTitle(err.to_string()))?;
        let description = proccess_post_description(description)
            .map_err(|err| ServerAddPostErr::InvalidDescription(err.to_string()))?;

        let (files, errs) = files
            .into_iter()
            .map(|v| {
                let path = v.path;
                let img_data_for_thumbnail = v.data.clone();
                let img_data_for_org = v.data;
                ImageReader::new(Cursor::new(img_data_for_thumbnail))
                    .with_guessed_format()
                    .inspect_err(|err| error!("error guesing the format {err}"))
                    .map_err(|err| ServerErrImg::ServerImgUnsupportedFormat(err.to_string()))
                    .and_then(|v| {
                        let img_format = v.format().ok_or(
                            ServerErrImg::ServerImgUnsupportedFormat("uwknown".to_string()),
                        )?;
                        v.decode()
                            .inspect_err(|err| error!("error decoding img {err}"))
                            .map_err(|err| ServerErrImg::ServerImgDecodeFailed(err.to_string()))
                            .map(|img| (img_format, img))
                    })
                    .and_then(|(img_format, img)| {
                        let width = img.width();
                        let height = img.height();
                        webp::Encoder::from_image(&img)
                            .inspect_err(|err| error!("failed to create webp encoder {err}"))
                            .map_err(|err| {
                                ServerErrImg::ServerImgWebPEncoderCreationFailed(err.to_string())
                            })
                            .and_then(|encoder| {
                                encoder
                                    .encode_simple(false, 90.0)
                                    .inspect_err(|err| {
                                        error!("failed to create webp encoder {err:?}")
                                    })
                                    .map_err(|err| {
                                        ServerErrImg::ServerImgWebPEncodingFailed(format!(
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
                                ServerErrImg::ServerImgUnsupportedFormat(img_format.to_string())
                            })
                            .and_then(|img_format| {
                                little_exif::metadata::Metadata::clear_metadata(
                                    &mut img_data_org,
                                    img_format,
                                )
                                .inspect_err(|err| error!("failed to read metadata {err:?}"))
                                .map_err(|err| {
                                    ServerErrImg::ServerImgMetadataReadFail(err.to_string())
                                })
                            })
                            .map(|_| {
                                (
                                    PostFile {
                                        extension: img_format.to_string(),
                                        hash: format!("{:X}", gxhash128(&img_data_org, 0)),
                                        width,
                                        height,
                                    },
                                    img_data_org,
                                    img_data_thumbnail.to_vec(),
                                )
                            })
                    })
                    .map_err(|err| ServerErrImgMeta { path, err })
            })
            .fold(
                (
                    Vec::<(PostFile, Vec<u8>, Vec<u8>)>::new(),
                    Vec::<ServerErrImgMeta>::new(),
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
        if !errs.is_empty() {
            return Err(ServerAddPostErr::ServerImgErr(errs).into());
        }

        let root_path = Path::new(&app_state.settings.site.files_path);
        let mut output_imgs = Vec::<Post>::new();
        for file in &files {
            let file_path = root_path.join(format!("{}.{}", &file.0.hash, &file.0.extension));
            if file_path.exists() {
                trace!(
                    "file already exists {}",
                    file_path.to_str().unwrap_or("err")
                );
                output_imgs.push(Post {
                    hash: file.0.hash.clone(),
                    extension: file.0.extension.clone(),
                    width: file.0.width,
                    height: file.0.height,
                    created_at: time.as_nanos(),
                });
                continue;
            }

            trace!("saving {}", file_path.to_str().unwrap_or("err"));
            (match fs::write(&file_path, &file.1).await {
                Ok(v) => Ok(v),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    fs::create_dir_all(&root_path)
                        .await
                        .inspect(|_| trace!("created img output dir {:?}", &file_path))
                        .inspect_err(|err| error!("error creating img output dir {err}"))
                        .map_err(|err| {
                            ServerAddPostErr::ServerDirCreationFailed(err.to_string())
                        })?;
                    fs::write(&file_path, &file.1).await
                }
                Err(err) => {
                    //
                    Err(err)
                }
            })
            .inspect_err(|err| error!("failed to save img to disk {err:?}"))
            .map_err(|err| ServerAddPostErr::ServerFSErr(err.to_string()))?;
            output_imgs.push(Post {
                hash: file.0.hash.clone(),
                extension: file.0.extension.clone(),
                width: file.0.width,
                height: file.0.height,
                created_at: time.as_nanos(),
            });
        }

        let post_files = files.into_iter().map(|v| v.0).collect::<Vec<PostFile>>();
        app_state
            .db
            .add_post(
                time.as_nanos(),
                &auth_token.username,
                &title,
                &description,
                post_files,
            )
            .await
            .inspect_err(|err| error!("failed to save images {err:?}"))
            .map_err(|_| ServerErr::ServerDbErr)?;

        Ok(ServerRes::Posts(output_imgs))
    }

    pub async fn get_invite(
        State(app_state): State<AppState>,
        req: ServerReq,
    ) -> Result<ServerRes, ServerErr> {
        let ServerReq::GetInvite { email } = req else {
            return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                "expected AddPost, received: {req:?}"
            ))));
        };

        let time = app_state.clock.now().await;
        let exp = time + Duration::from_secs(app_state.settings.auth.invite_exp_s);
        let invite = InviteToken::new(email.clone(), time.as_nanos());
        let invite_token = encode_token(&app_state.settings.auth.secret, invite)
            .map_err(|_| ServerInviteErr::ServerJWT)?;

        trace!("invite token created: {invite_token}");

        let invite = app_state
            .db
            .add_invite(time.clone().as_nanos(), invite_token, email, exp.as_nanos())
            .await;
        let invite = match invite {
            Err(AddInviteErr::EmailIsTaken(_)) => {
                return Ok(ServerRes::Ok);
            }
            invite => invite.map_err(|_| ServerErr::ServerDbErr),
        }?;
        trace!("result {invite:?}");

        let link = format!(
            "{}{}",
            &app_state.settings.site.address,
            crate::path::link_reg(&invite.token_raw),
        );
        trace!("{link}");

        Ok(ServerRes::Ok)
    }

    pub async fn login(
        State(app_state): State<AppState>,
        req: ServerReq,
    ) -> Result<ServerRes, ServerErr> {
        let ServerReq::Login { email, password } = req else {
            return Err(ServerDesErr::ServerWrongInput(format!(
                "expected Login, received: {req:?}"
            ))
            .into());
        };
        let time = app_state.clock.now().await;
        let user = app_state
            .db
            .get_user_by_email(email)
            .await
            .inspect_err(|err| trace!("user not found - {err}"))
            .map_err(|_| ServerErr::ServerLoginErr(ServerLoginErr::WrongCredentials))?;

        verify_password(password, user.password)
            .inspect_err(|err| trace!("passwords verification failed {err}"))
            .map_err(|_| ServerErr::ServerLoginErr(ServerLoginErr::WrongCredentials))?;

        let (token, cookie) = create_cookie(&app_state.settings.auth.secret, &user.username, time)
            .map_err(|err| {
                ServerErr::ServerLoginErr(ServerLoginErr::ServerCreateCookieErr(err.to_string()))
            })?;

        let _session = app_state
            .db
            .add_session(token, &user.username)
            .await
            .map_err(|err| ServerErr::ServerDbErr)?;

        Ok(ServerRes::SetAuthCookie { cookie })
    }

    pub async fn auth_middleware(
        State(app_state): State<AppState>,
        mut req: axum::extract::Request,
        next: axum::middleware::Next,
    ) -> axum::response::Response {
        let result = {
            let headers = req.headers();
            let jar = CookieJar::from_headers(headers);
            check_auth(&app_state, &jar).await
        };
        match result {
            Ok(token) => {
                {
                    let extensions = req.extensions_mut();
                    extensions.insert(token);
                }
                let response = next.run(req).await;
                return response;
            }
            Err(err) => {
                return err.into_response();
            }
        }
    }

    pub async fn check_auth(app_state: &AppState, jar: &CookieJar) -> Result<AuthToken, ServerErr>
    where
        ServerErr: std::error::Error + 'static,
    {
        let token = jar
            .get(AUTHORIZATION.as_str())
            .ok_or(ServerAuthErr::ServerUnauthorizedNoCookie)
            .map(|v| cut_cookie_value_decoded(v.value()).to_string())?;

        let _session = app_state
            .db
            .get_session(&token)
            .await
            .map_err(|err| match err {
                GetSessionErr::NotFound => {
                    ServerErr::from(ServerAuthErr::ServerUnauthorizedInvalidCookie)
                }
                _ => ServerErr::ServerDbErr,
            })?;

        let token = match decode_token::<AuthToken>(&app_state.settings.auth.secret, &token, false)
        {
            Ok(v) => v,
            Err(err) => {
                error!("invalid token was stored {err}");
                app_state
                    .db
                    .delete_session(token)
                    .await
                    .map_err(|err| ServerErr::ServerDbErr)?;
                return Err(ServerErr::from(
                    ServerAuthErr::ServerUnauthorizedInvalidCookie,
                ));
            }
        };

        Ok(token.claims)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::fs;

    use axum_test::TestServer;
    use gxhash::gxhash128;
    use pretty_assertions::assert_eq;
    use test_log::test;
    use tokio::sync::Mutex;
    use tracing::trace;

    use crate::api::app_state::AppState;
    use crate::api::{
        Api, ApiTest, InviteToken, ServerErr, ServerLoginErr, ServerRegistrationErr, ServerReqImg,
        ServerRes, encode_token,
    };
    use crate::server::create_api_router;

    #[test(tokio::test)]
    async fn full_api_test() {
        let current_time = Duration::from_nanos(1);
        let time_mut = Arc::new(Mutex::new(current_time));
        let app_state = AppState::new_testng(time_mut.clone()).await;
        let my_app = create_api_router(app_state.clone()).with_state(app_state.clone());

        let time = app_state.clock.now().await.as_nanos();

        let server = TestServer::builder()
            .http_transport()
            .build(my_app)
            .unwrap();

        let api = ApiTest::new(&server);

        let mut imgbuf = image::ImageBuffer::new(250, 250);
        // Iterate over the coordinates and pixels of the image
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let r = (0.3 * x as f32) as u8;
            let b = (0.3 * y as f32) as u8;
            *pixel = image::Rgb([r, 0, b]);
        }

        let path = "../target/tmp/img.png";
        imgbuf.save(path).unwrap();
        let img = tokio::fs::read(path).await.unwrap();

        let result = api.get_invite("hey1@hey.com").send_native().await.unwrap();

        trace!("{result:#?}");

        let invite = app_state
            .db
            .get_invite("hey1@hey.com", current_time.as_nanos())
            .await
            .unwrap();
        trace!("good invite {invite:#?}");

        let bad_invite_token = encode_token(
            &app_state.settings.auth.secret,
            InviteToken::new("hey1@hey.com", time),
        )
        .unwrap();

        let bad_invite = app_state
            .db
            .add_invite(time, &bad_invite_token, "hey1@hey.com", time + 1)
            .await
            .unwrap();
        trace!("bad invite added: {bad_invite:#?}");

        {
            *time_mut.lock().await = Duration::from_secs(10);
            let result = api
                .register("hey", &invite.token_raw, "*wowowowwoW12222pp")
                .send_native()
                .await;

            assert!(matches!(
                result,
                Err(ServerErr::ServerRegistrationErr(
                    ServerRegistrationErr::TokenExpired
                ))
            ));
            *time_mut.lock().await = Duration::from_nanos(1);
            // match result {
            //      => {
            //         assert!(username.is_some());
            //         assert!(email.is_none());
            //         assert!(password.is_some());
            //     }
            //     etc => panic!("expexted register err, got: {etc:?}"),
            // }
        }
        {
            let result = api
                .register("he", &invite.token_raw, "wowowowwoW12222pp")
                .send_native()
                .await;

            match result {
                Err(ServerErr::ServerRegistrationErr(
                    ServerRegistrationErr::ServerRegistrationInvalidInput {
                        username,
                        email,
                        password,
                    },
                )) => {
                    assert!(username.is_some());
                    assert!(email.is_none());
                    assert!(password.is_some());
                }
                etc => panic!("expexted register err, got: {etc:?}"),
            }
        }

        let (token, decoded_token, result) = api
            .register("hey", &invite.token_raw, "wowowowwoW12222pp*")
            .send_native_and_extract_auth(&app_state.settings.auth.secret)
            .await;

        let result = api.get_invite("hey1@hey.com").send_native().await.unwrap();
        assert_eq!(result, ServerRes::Ok);

        let token_raw = token.unwrap();

        let all_invites = app_state.db.get_all_invites().await.unwrap();

        trace!("all invites: {all_invites:#?}");

        {
            let result = api
                .register("he", &bad_invite_token, "wowowowwoW12222pp")
                .send_native()
                .await;

            assert!(matches!(
                result,
                Err(ServerErr::ServerRegistrationErr(
                    ServerRegistrationErr::TokenUsed,
                ))
            ));
        }

        let all_users = app_state.db.get_all_user().await.unwrap();

        trace!("ALL USERS {all_users:#?}");
        assert!(all_users.len() == 1);

        trace!("{token_raw:#?}");

        let result = api
            .logout()
            .send_native_with_token(&token_raw)
            .await
            .unwrap();

        assert_eq!(result, ServerRes::Ok);

        let result = api
            .login("hey1@hey.com3", "wowowowwoW12222pp*")
            .send_native()
            .await;

        assert!(matches!(
            result,
            Err(ServerErr::ServerLoginErr(ServerLoginErr::WrongCredentials))
        ));

        let (token, decoded_token, result) = api
            .login("hey1@hey.com", "wowowowwoW12222pp*")
            .send_native_and_extract_auth(&app_state.settings.auth.secret)
            .await;

        let token_raw = token.unwrap();

        let result = api
            .add_post(
                "title1",
                "wow",
                Vec::from([ServerReqImg {
                    path: path.to_string(),
                    data: img.clone(),
                }]),
            )
            .send_native_with_token(token_raw.clone())
            .await
            .unwrap();
        trace!("{result:#?}");

        let result = api.get_posts_older(2, 25).send_native().await.unwrap();
        match result {
            crate::api::ServerRes::Posts(posts) => {
                assert!(posts.len() == 1);
            }
            wrong => {
                panic!("{}", format!("expected posts, got {:?}", wrong));
            }
        }

        let result = api.get_posts_older(1, 25).send_native().await.unwrap();
        match result {
            crate::api::ServerRes::Posts(posts) => {
                assert!(posts.len() == 0);
            }
            wrong => {
                panic!("{}", format!("expected posts, got {:?}", wrong));
            }
        }

        *time_mut.lock().await = Duration::from_nanos(2);

        let result = api
            .add_post(
                "title2",
                "wow",
                Vec::from([ServerReqImg {
                    path: path.to_string(),
                    data: img.clone(),
                }]),
            )
            .send_native_with_token(token_raw.clone())
            .await
            .unwrap();

        *time_mut.lock().await = Duration::from_nanos(3);

        let result = api
            .add_post(
                "title3",
                "wow",
                Vec::from([ServerReqImg {
                    path: path.to_string(),
                    data: img.clone(),
                }]),
            )
            .send_native_with_token(token_raw.clone())
            .await
            .unwrap();

        let result = api.get_posts_older(2, 25).send_native().await.unwrap();
        match result {
            crate::api::ServerRes::Posts(posts) => {
                assert!(posts.len() == 1);
                assert_eq!(posts[0].created_at, 1);
            }
            wrong => {
                panic!("{}", format!("expected posts, got {:?}", wrong));
            }
        }

        let result = api.get_posts_newer(2, 25).send_native().await.unwrap();
        match result {
            crate::api::ServerRes::Posts(posts) => {
                assert!(posts.len() == 1);
                assert_eq!(posts[0].created_at, 3);
            }
            wrong => {
                panic!("{}", format!("expected posts, got {:?}", wrong));
            }
        }

        *time_mut.lock().await = Duration::from_nanos(4);

        let result = api
            .add_post(
                "title4",
                "wow",
                Vec::from([ServerReqImg {
                    path: path.to_string(),
                    data: img.clone(),
                }]),
            )
            .send_native_with_token(token_raw.clone())
            .await
            .unwrap();

        let result = api.get_posts_newer(2, 25).send_native().await.unwrap();
        match result {
            crate::api::ServerRes::Posts(posts) => {
                assert!(posts.len() == 2);
                assert_eq!(posts[0].created_at, 4);
                assert_eq!(posts[1].created_at, 3);
            }
            wrong => {
                panic!("{}", format!("expected posts, got {:?}", wrong));
            }
        }

        let result = api.get_posts_older(3, 25).send_native().await.unwrap();
        match result {
            crate::api::ServerRes::Posts(posts) => {
                assert!(posts.len() == 2);
                assert_eq!(posts[0].created_at, 2);
                assert_eq!(posts[1].created_at, 1);
            }
            wrong => {
                panic!("{}", format!("expected posts, got {:?}", wrong));
            }
        }

        let result = api.get_posts_newer(2, 1).send_native().await.unwrap();
        match result {
            crate::api::ServerRes::Posts(posts) => {
                assert!(posts.len() == 1);
                assert_eq!(posts[0].created_at, 3);
            }
            wrong => {
                panic!("{}", format!("expected posts, got {:?}", wrong));
            }
        }
    }
}
