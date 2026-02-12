use http::HeaderMap;
use http::header::{AUTHORIZATION, SET_COOKIE};
use leptos::prelude::*;
// use regex::Regex;
use reqwest::RequestBuilder;
use rkyv::result::ArchivedResult;
use std::fmt::Display;
use std::str::FromStr;
use thiserror::Error;
use tracing::{debug, error, trace};
use wasm_bindgen_futures::spawn_local;

use crate::api::shared::post_comment::UserPostComment;
use crate::path::{
    link_settings_form_email_completed, link_settings_form_email_current_click,
    link_settings_form_email_current_send, link_settings_form_email_final_confirm,
    link_settings_form_email_new_click, link_settings_form_email_new_send,
};

#[cfg(feature = "ssr")]
pub mod backend;

pub mod shared;

#[cfg(feature = "ssr")]
pub mod app_state {
    use std::{sync::Arc, time::Duration};

    use rand::distr::{Alphanumeric, SampleString};
    use surrealdb::RecordId;
    use tokio::sync::{Mutex, RwLock};
    use tracing::trace;

    use crate::{
        api::{
            EmailChangeNewErr, EmailChangeStage, EmailToken, PasswordChangeStage, ServerErr,
            ServerTokenErr, clock::Clock, encode_token, settings::Settings,
        },
        db::{self, DB404Err, DBSentEmailReason, DBUser, DbEngine},
        get_timestamp,
        path::{
            link_login_form_password_confirm, link_settings_form_email_current_confirm,
            link_settings_form_email_new_confirm, link_settings_form_password,
            link_settings_form_password_confirm,
        },
        view::app::hook::{
            use_email_change::EmailChangeFormStage, use_password_change::ChangePasswordFormStage,
        },
    };

    #[derive(Clone)]
    pub struct AppState {
        pub db: DbEngine,
        pub settings: Arc<RwLock<Settings>>,
        pub clock: Clock,
    }

    impl AppState {
        pub async fn new(time: u128) -> Self {
            let settings = Settings::new_from_file();
            let db = db::new_local(time, &settings.db.path).await;
            let f = move || async move { get_timestamp() };
            let clock = Clock::new(f);

            Self {
                db,
                settings: Arc::new(RwLock::new(settings)),
                clock,
            }
        }

        pub async fn new_testng(time: Arc<Mutex<u128>>, invite_exp_ns: u128) -> Self {
            let db = db::new_mem(*time.lock().await).await;
            let settings = Settings::new_testing(invite_exp_ns);
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
                settings: Arc::new(RwLock::new(settings)),
                clock,
            }
        }

        pub async fn get_address(&self) -> String {
            self.settings.read().await.site.address.clone()
        }

        pub async fn get_invite_exp_ns(&self) -> u128 {
            self.settings.read().await.auth.invite_exp_ns.into()
        }

        pub async fn set_invite_exp_ns(&self, duration_ns: u128) {
            self.settings.write().await.auth.invite_exp_ns = duration_ns as u64;
        }

        pub async fn get_secret(&self) -> String {
            self.settings.read().await.auth.secret.clone()
        }

        pub async fn get_file_path(&self) -> String {
            self.settings.read().await.site.files_path.clone()
        }

        pub async fn time(&self) -> u128 {
            self.clock.now().await
        }

        pub async fn new_token(
            &self,
            email: impl Into<String>,
        ) -> Result<(String, u128), ServerErr> {
            let time = self.time().await;
            let exp = time + self.get_invite_exp_ns().await;
            let key = self.gen_key().await;
            let confirm_token = EmailToken::new(key, email.into(), time);
            let confirm_token = encode_token(self.get_secret().await, confirm_token)
                .map_err(|_| ServerErr::from(ServerTokenErr::ServerJWT))?;

            Ok((confirm_token, exp))
        }

        pub async fn new_exp(&self) -> u128 {
            let time = self.time().await;
            let exp = time + self.get_invite_exp_ns().await;
            exp
        }

        // pub async fn new_token_v2(&self) -> (String, u128) {
        //     let time = self.time().await;
        //     let exp = time + self.get_invite_exp_ns().await;
        //     let key = self.gen_key().await;
        //
        //     Ok((key, exp))
        // }

        pub async fn gen_key(&self) -> String {
            rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 16)
        }

        //         pub async fn get_email_change(
        //             &self,
        //             time: u128,
        //             db_user: &DBUser,
        //         ) -> Result<DBEmailChange, ServerErr> {
        //             let result = self.db.get_email_change(time, db_user.id.clone()).await.map_err(|e| {
        //                 match e {
        // DB404Err::NotFound
        //                 }
        //             });
        //             match result {
        //                 Ok(v) => Ok(v),
        //                 Err(DB404Err::NotFound) => Ok(EmailChangeStage::None),
        //                 Err(DB404Err::DB(_)) => Err(ServerErr::DbErr),
        //             }
        //         }

        // pub async fn get_email_change_status_by_current_token(
        //     &self,
        //     time: u128,
        //     db_user: &DBUser,
        //     confirm_token: impl Into<String>,
        // ) -> Result<Option<(DBEmailChange, EmailChangeStage)>, ServerErr> {
        //     let result = self.db.get_email_change_by_current_token(time, db_user.id.clone(), confirm_token).await;
        //     match result {
        //         Ok(v) => {
        //             let stage = EmailChangeStage::from(&v);
        //             Ok(Some((v, stage)))
        //         }
        //         Err(DB404Err::NotFound) => Ok(None),
        //         Err(DB404Err::DB(_)) => Err(ServerErr::DbErr),
        //     }
        // }

        // pub async fn get_email_change_status_unwrap<F, R>(
        //     &self,
        //     time: u128,
        //     db_user: &DBUser,
        //     invalid_err: F,
        // ) -> Result<(DBEmailChange, EmailChangeStage), ServerErr>
        // where
        //     F: Fn(String) -> R,
        //     R: Into<ServerErr>,
        // {
        //     self.get_email_change_status(time, &db_user)
        //         .await?
        //         .ok_or(invalid_err("email change not started".to_string()).into())
        // }

        // pub async fn get_email_change_status_compare<F, R>(
        //     &self,
        //     time: u128,
        //     db_user: &DBUser,
        //     expected_stage: impl FnOnce(&EmailChangeStage) -> bool,
        //     invalid_stage_err: F,
        // ) -> Result<EmailChangeStage, ServerErr>
        // where
        //     F: Fn(String) -> R,
        //     R: Into<ServerErr>,
        // {
        //     let foo = EmailChangeStage::ConfirmEmail { .. } as u8;
        //     let stage = self.get_email_change_status(time, &db_user).await?;
        //     // .ok_or(invalid_stage_err("email change not started".to_string()).into())?;
        //     // let a = EmailChangeStage::from(email_change);
        //     let result = expected_stage(&stage);
        //     trace!("2 stage {stage}");
        //     // stage.is_stage(expected_stage, invalid_stage_err)?;
        //     if result {
        //         Ok(stage)
        //     } else {
        //         Err(invalid_stage_err(format!("unexpected stage {:?}", stage)).into())
        //     }
        // }

        // pub async fn get_email_change_status_compare_current<F, R>(
        //     &self,
        //     time: u128,
        //     db_user: &DBUser,
        //     current_token: impl AsRef<str>,
        //     expected_stage: EmailChangeStage,
        //     invalid_stage_err: F,
        //     invalid_token_err: impl Into<ServerErr>,
        // ) -> Result<(DBEmailChange, EmailChangeStage), ServerErr>
        // where
        //     F: Fn(String) -> R,
        //     R: Into<ServerErr>,
        // {
        //     let (email_change, stage) = self.get_email_change_status(time, &db_user)
        //         .await?
        //         .ok_or(invalid_stage_err("email change not started".to_string()).into())?;
        //     stage.is_stage(expected_stage, invalid_stage_err)?;
        //
        //     if email_change.current.token_raw == current_token.as_ref() {
        //         return Err(invalid_token_err.into());
        //     }
        //
        //     Ok((email_change, stage))
        // }

        // pub async fn get_email_change_status_by_current_token_unwrap<F, R>(
        //     &self,
        //     time: u128,
        //     db_user: &DBUser,
        //     confirm_token: impl Into<String>,
        //     invalid_err: impl Into<ServerErr>,
        // ) -> Result<(DBEmailChange, EmailChangeStage), ServerErr>
        // where
        //     F: Fn(String) -> R,
        //     R: Into<ServerErr>,
        // {
        //     self.get_email_change_status(time, &db_user)
        //         .await?
        //         .ok_or(invalid_err.into())
        // }

        // pub async fn get_email_change(
        //     &self,
        //     time: u128,
        //     user_id: RecordId,
        //     not_found: impl Into<ServerErr>,
        // ) -> Result<DBEmailChange, ServerErr> {
        //     self.db
        //         .get_email_change(time, user_id)
        //         .await
        //         .map_err(|err| match err {
        //             DB404Err::NotFound => not_found.into(),
        //             DB404Err::DB(_) => ServerErr::DbErr,
        //         })
        // }

        // pub async fn add_email_change(
        //     &self,
        //     time: u128,
        //     db_user: &DBUser,
        // ) -> Result<DBEmailChange, ServerErr> {
        //     let (confirm_token, exp) = self.new_token(&db_user.email).await?;
        //
        //     self.db
        //         .add_email_change(
        //             time,
        //             db_user.id.clone(),
        //             db_user.email.clone(),
        //             confirm_token.clone(),
        //             exp,
        //         )
        //         .await
        //         .map_err(|_| ServerErr::DbErr)
        // }

        pub async fn send_email_change(
            &self,
            time: u128,
            to_email: impl Into<String>,
            id: &RecordId,
            confim_token: impl Into<String>,
            old_email: impl Into<String>,
            expires: impl Into<u128>,
        ) -> Result<(), ServerErr> {
            let id = id.key().to_string();
            let link = link_settings_form_email_current_confirm(
                id,
                expires.into(),
                old_email.into(),
                confim_token.into(),
                None,
                None,
            );
            let link = format!("{}{}", &self.get_address().await, link);
            self.db
                .add_sent_email(
                    time,
                    link.clone(),
                    to_email.into(),
                    DBSentEmailReason::ConfirmEmailChange,
                )
                .await
                .map_err(|_| ServerErr::DbErr)?;
            trace!("{link}");

            Ok(())
        }

        pub async fn send_email_change_password(
            &self,
            time: u128,
            to_email: impl Into<String>,
            confim_key: impl Into<String>,
        ) -> Result<(), ServerErr> {
            let to_email = to_email.into();
            let link = link_settings_form_password_confirm(to_email.clone(), confim_key);
            let link = format!("{}{}", &self.get_address().await, link);

            self.db
                .add_sent_email(
                    time,
                    link.clone(),
                    to_email,
                    DBSentEmailReason::ConfirmEmailChange,
                )
                .await
                .map_err(|_| ServerErr::DbErr)?;
            trace!("{link}");

            Ok(())
        }

        pub async fn send_email_reset_password(
            &self,
            time: u128,
            to_email: impl Into<String>,
            confim_key: impl Into<String>,
        ) -> Result<(), ServerErr> {
            let to_email = to_email.into();
            let link = link_login_form_password_confirm(to_email.clone(), confim_key);
            let link = format!("{}{}", &self.get_address().await, link);

            self.db
                .add_sent_email(
                    time,
                    link.clone(),
                    to_email,
                    DBSentEmailReason::ConfirmEmailChange,
                )
                .await
                .map_err(|_| ServerErr::DbErr)?;
            trace!("{link}");

            Ok(())
        }

        pub async fn send_email_new(
            &self,
            time: u128,
            to_email: impl Into<String>,
            id: &RecordId,
            confim_token: impl Into<String>,
            old_email: impl Into<String>,
            expires: impl Into<u128>,
        ) -> Result<(), ServerErr> {
            let to_email = to_email.into();
            let id = id.key().to_string();
            let link = link_settings_form_email_new_confirm(
                id,
                expires.into(),
                old_email.into(),
                to_email.clone(),
                confim_token.into(),
                None,
                None,
            );
            let link = format!("{}{}", &self.get_address().await, link,);
            self.db
                .add_sent_email(
                    time,
                    link.clone(),
                    to_email,
                    DBSentEmailReason::ConfirmEmailChangeNewEmail,
                )
                .await
                .map_err(|_| ServerErr::DbErr)?;
            trace!("{link}");

            Ok(())
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
        pub invite_exp_ns: u64,
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

        pub fn new_testing(invite_exp_ns: u128) -> Self {
            Self {
                site: Site {
                    address: "http://localhost:3000".to_string(),
                    files_path: "../target/tmp/files".to_string(),
                },
                auth: Auth {
                    secret: "secret".to_string(),
                    invite_exp_ns: invite_exp_ns as u64,
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
            dyn Fn() -> Pin<Box<dyn Future<Output = u128> + Sync + Send + 'static>>
                + Sync
                + Send
                + 'static,
        >,
    }

    impl Clock {
        pub fn new<
            F: Fn() -> Fut + Send + Sync + Clone + 'static,
            Fut: Future<Output = u128> + Send + Sync + 'static,
        >(
            ticker: F,
        ) -> Self {
            let fut = Arc::new(move || {
                let ticker = (ticker.clone())();
                let f: Pin<Box<dyn Future<Output = u128> + Sync + Send + 'static>> =
                    Box::pin(ticker);
                f
            });

            Self { ticker: fut }
        }

        pub async fn now(&self) -> u128 {
            let mut fut = (self.ticker)();
            let fut = fut.as_mut();
            let duration = fut.await;
            duration
        }

        // pub async fn set(&self, time: u128) {
        //
        // }
    }

    // #[cfg(feature = "ssr")]
    // pub fn get_nanos() -> u128 {
    //     use std::time::{SystemTime, UNIX_EPOCH};
    //     SystemTime::now()
    //         .duration_since(UNIX_EPOCH)
    //         .unwrap()
    //         .as_nanos()
    // }
}

// derive_alias! {
//     #[derive(Com!)] = #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)];
// }

#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum ServerReq {
    Login {
        email: String,
        password: String,
    },
    ChangeEmail {
        new_email: String,
        confirm_token: String,
    },
    ChangeUsername {
        username: String,
        password: String,
    },
    ChangePassword {
        confirm_key: String,
        new_password: String,
    },
    GetUser {
        username: String,
    },
    ConfirmToken {
        token: String,
    },
    // ConfirmKind {
    //     kind: EmailConfirmTokenKind,
    // },
    // ConfirmToken {
    //     confirm_token: String,
    // },
    // SendEmailInvite {
    //     email: String,
    // },
    Id {
        id: String,
    },
    EmailAddressWithId {
        id: String,
        email: String,
    },
    EmailAddress {
        email: String,
    },
    Register {
        username: String,
        invite_token: String,
        password: String,
    },
    AddPostComment {
        post_id: String,
        text: String,
    },
    CommentId {
        comment_id: String,
    },
    PostId {
        post_id: String,
    },
    GetPosts {
        time: u128,
        limit: u32,
    },
    GetUserPosts {
        time: u128,
        limit: u32,
        username: String,
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
        let headers = format!("{:#?}", req.headers());
        let multipart = axum::extract::Multipart::from_request(req, state)
            .await
            .map_err(|err| ServerDesErr::ServerDesGettingMultipartErr(err.to_string()))?;
        recv(headers, multipart).await
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct ServerReqImg {
    pub path: String,
    pub data: Vec<u8>,
}

// TODO make sure the "id" fields only send key value
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum ServerRes {
    SetAuthCookie { token: String },
    DeleteAuthCookie,
    Condition(bool),
    User { username: String },
    Acc { username: String, email: String },
    InviteToken(EmailToken),
    Comments(Vec<UserPostComment>),
    Comment(UserPostComment),
    Posts(Vec<UserPost>),
    Post(UserPost),
    EmailChangeStage(EmailChangeStage),
    PasswordChangeStage(PasswordChangeStage),
    Ok,
}

#[derive(
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum ServerErr {
    #[error("client err {0}")]
    ClientErr(#[from] ClientErr),

    #[error("auth err {0}")]
    AuthErr(#[from] ServerAuthErr),

    #[error("login err {0}")]
    LoginErr(#[from] ServerLoginErr),

    // #[error("get user err {0}")]
    // ServerGetUserErr(#[from] ServerGetErr),
    #[error("decode invite err {0}")]
    DecodeInviteErr(#[from] ServerDecodeInviteErr),

    #[error("get invite err {0}")]
    TokenErr(#[from] ServerTokenErr),

    #[error("add post err {0}")]
    PostLikeErr(#[from] PostLikeErr),

    #[error("add post err {0}")]
    AddPostErr(#[from] ServerAddPostErr),

    #[error("registration err {0}")]
    RegistrationErr(#[from] ServerRegistrationErr),

    #[error("change username err {0}")]
    ChangeUsernameErr(#[from] ChangeUsernameErr),

    #[error("change username err {0}")]
    ChangePasswordErr(#[from] ChangePasswordErr),

    #[error("change email err {0}")]
    EmailChangeNew(#[from] EmailChangeNewErr),

    #[error("change email err {0}")]
    EmailChangeToken(#[from] EmailChangeTokenErr),

    #[error("change email err {0}")]
    EmailChange(#[from] EmailChangeErr),

    #[error("add deserialization err {0}")]
    DesErr(#[from] ServerDesErr),

    #[error("failed to get {0}")]
    NotFoundErr(#[from] Server404Err),

    // #[error("confirm action err {0}")]
    // ConfirmEmailChange(#[from] ConfirmEmailChangeErr),
    #[error("internal server err")]
    InternalServerErr,

    #[error("database err")]
    DbErr,
}

#[derive(
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum ServerLoginErr {
    #[error("wrong credentials")]
    WrongCredentials,

    #[error("create cookie err {0}")]
    ServerCreateCookieErr(String),
}

#[derive(
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum ServerDesErr {
    #[error("wrong variant")]
    ServerWrongInput(String),

    #[error("failed to run field to bytes")]
    ServerMutlipartAccessErr,

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
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum ClientErr {
    #[error("failed to deserialize req {0}")]
    ClientDesErr(String),

    #[error("failed to send req {0}")]
    ClientSendErr(String),
}

// #[derive(Error, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
// pub enum ServerGetUserErr {
//     #[error("user not found")]
//     NotFound,
// }

#[derive(
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum Server404Err {
    #[error("not found")]
    NotFound,
}

#[derive(
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum ServerAuthErr {
    #[error("unauthorized no cookie")]
    ServerUnauthorizedNoCookie,

    #[error("unauthorized invalid cookie")]
    ServerUnauthorizedInvalidCookie,
}

#[derive(
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum ServerTokenErr {
    #[error("jwt error")]
    ServerJWT,
}

#[derive(
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
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
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum ServerDecodeInviteErr {
    #[error("invite not found")]
    InviteNotFound,

    #[error("jwt err {0}")]
    JWT(String),
}

#[derive(
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
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

// #[derive(Error, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
// pub enum ChangeEmailErr {
//     #[error("email \"{0}\" is taken")]
//     EmailIsTaken(String),
//
//     #[error("invalid or expired token")]
//     InvalidToken,
//
//     #[error("user not found")]
//     NotFound,
// }
#[derive(
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum PostLikeErr {
    #[error("post \"{0}\" was already liked")]
    PostAlreadyLiked(String),

    #[error("post \"{0}\" not found")]
    PostNotFound(String),
}

#[derive(
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum ChangePasswordErr {
    #[error("invalid password {0}")]
    InvalidPassword(String),

    #[error("confirm key is invalid/expired")]
    NotFound,
}

#[derive(
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum ChangeUsernameErr {
    #[error("username \"{0}\" is taken")]
    UsernameIsTaken(String),

    #[error("wrong credentials")]
    WrongCredentials,

    #[error("user not found")]
    NotFound,
}

#[derive(
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum EmailChangeNewErr {
    #[error("email \"{0}\" is taken")]
    EmailIsTaken(String),

    #[error("token is invalid")]
    TokenInvalid,

    #[error("invalid stage: {0}")]
    InvalidStage(String),
}

#[derive(
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum EmailChangeTokenErr {
    #[error("token is invalid")]
    TokenInvalid,

    #[error("invalid stage: {0}")]
    InvalidStage(String),
}

#[derive(
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum EmailChangeErr {
    #[error("invalid stage: {0}")]
    InvalidStage(String),
}

// #[derive(Error, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
// pub enum ConfirmEmailChangeErr {
//     #[error("token not found")]
//     NotFound,
//
//     #[error("token already confirmed")]
//     AlreadyConfirmed,
// }

// #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
// pub enum EmailConfirmTokenKind {
//     ChangeEmail,
// }

#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct User {
    pub username: String,
    pub created_at: u128,
}

#[cfg(feature = "ssr")]
impl From<crate::db::DBUser> for User {
    fn from(value: crate::db::DBUser) -> Self {
        Self {
            username: value.username,
            created_at: value.created_at,
        }
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct UserPost {
    pub id: String,
    pub user: User,
    pub show: bool,
    pub title: String,
    pub description: String,
    pub favorites: u64,
    pub file: Vec<UserPostFile>,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    PartialOrd,
    strum::EnumString,
    strum::EnumIter,
    strum::Display,
    strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
#[repr(u8)]
pub enum PasswordChangeStage {
    Confirm,
    // ConfirmNewPss,
    Complete,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    PartialOrd,
    strum::EnumString,
    strum::EnumIter,
    strum::Display,
    strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
#[repr(u8)]
pub enum EmailChangeStage {
    // None,
    ConfirmEmail {
        id: String,
        old_email: String,
        expires: u128,
    },
    EnterNewEmail {
        id: String,
        old_email: String,
        expires: u128,
    },
    ConfirmNewEmail {
        id: String,
        old_email: String,
        new_email: String,
        expires: u128,
    },
    ReadyToComplete {
        id: String,
        old_email: String,
        new_email: String,
        expires: u128,
    },
    Complete {
        id: String,
        old_email: String,
        new_email: String,
        expires: u128,
    },
    Cancelled {
        id: String,
        old_email: String,
        expires: u128,
    },
}

impl EmailChangeStage {
    pub fn link(self, stage_error: Option<String>, general_info: Option<String>) -> String {
        match self {
            EmailChangeStage::Cancelled {
                id,
                old_email,
                expires,
            } => link_settings_form_email_current_send(old_email, stage_error, general_info),
            EmailChangeStage::ConfirmEmail {
                id,
                old_email,
                expires,
            } => link_settings_form_email_current_click(
                id,
                expires,
                old_email,
                stage_error,
                general_info,
            ),
            EmailChangeStage::EnterNewEmail {
                id,
                old_email,
                expires,
            } => {
                link_settings_form_email_new_send(id, expires, old_email, stage_error, general_info)
            }
            EmailChangeStage::ConfirmNewEmail {
                id,
                old_email,
                new_email,
                expires,
            } => link_settings_form_email_new_click(
                id,
                expires,
                old_email,
                new_email,
                stage_error,
                general_info,
            ),
            EmailChangeStage::ReadyToComplete {
                id,
                old_email,
                new_email,
                expires,
            } => link_settings_form_email_final_confirm(
                id,
                expires,
                old_email,
                new_email,
                stage_error,
                general_info,
            ),
            EmailChangeStage::Complete {
                id,
                old_email,
                new_email,
                expires,
            } => link_settings_form_email_completed(
                id,
                old_email,
                new_email,
                stage_error,
                general_info,
            ),
        }
    }
}

// #[cfg(feature = "ssr")]
// impl EmailChangeStage {
//     pub fn is_stage<F, R>(&self, expected: Self, invalid_stage_err: F) -> Result<(), ServerErr>
//     where
//         F: Fn(String) -> R,
//         R: Into<ServerErr>,
//     {
//         match self {
//             stage if *stage == expected => Ok(()),
//             stage => {
//                 Err(invalid_stage_err(format!("expected {:?}, got: {:?}", expected, stage)).into())
//             }
//         }
//     }
// }

#[cfg(feature = "ssr")]
impl From<&crate::db::email_change::DBEmailChange> for EmailChangeStage {
    fn from(value: &crate::db::email_change::DBEmailChange) -> Self {
        let output = if value.completed
            // && value.current.token_used
            && !value.new.as_ref().map(|v| v.token_used).unwrap_or_default()
        {
            EmailChangeStage::Cancelled {
                id: value.id.key().to_string(),
                old_email: value.current.email.clone(),
                expires: value.expires,
            }
        } else if value.completed {
            EmailChangeStage::Complete {
                id: value.id.key().to_string(),
                old_email: value.current.email.clone(),
                new_email: value.new.as_ref().unwrap().email.clone(),
                expires: value.expires,
            }
        } else if let Some(new) = &value.new
            && new.token_used
        {
            EmailChangeStage::ReadyToComplete {
                id: value.id.key().to_string(),
                old_email: value.current.email.clone(),
                new_email: new.email.clone(),
                expires: value.expires,
            }
        } else if let Some(new) = &value.new {
            EmailChangeStage::ConfirmNewEmail {
                id: value.id.key().to_string(),
                old_email: value.current.email.clone(),
                new_email: new.email.clone(),
                expires: value.expires,
            }
        } else if value.current.token_used {
            EmailChangeStage::EnterNewEmail {
                id: value.id.key().to_string(),
                old_email: value.current.email.clone(),
                expires: value.expires,
            }
        } else {
            EmailChangeStage::ConfirmEmail {
                id: value.id.key().to_string(),
                old_email: value.current.email.clone(),
                expires: value.expires,
            }
        };

        debug!("email change stage converter: input {value:#?}, output: {output:?}");
        output
    }
}

#[cfg(feature = "ssr")]
impl From<crate::db::DBUserPost> for UserPost {
    fn from(value: crate::db::DBUserPost) -> Self {
        Self {
            id: value.id.key().to_string(),
            user: value.user.into(),
            file: value.file.into_iter().map(UserPostFile::from).collect(),
            title: value.title,
            show: value.show,
            description: value.description,
            favorites: value.favorites,
            modified_at: value.modified_at,
            created_at: value.created_at,
        }
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct UserPostFile {
    pub extension: String,
    pub hash: String,
    pub width: u32,
    pub height: u32,
}

#[cfg(feature = "ssr")]
impl From<crate::db::DBUserPostFile> for UserPostFile {
    fn from(value: crate::db::DBUserPostFile) -> Self {
        Self {
            extension: value.extension,
            hash: value.hash,
            width: value.width,
            height: value.height,
        }
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct ServerErrImgMeta {
    pub path: String,
    pub err: ServerErrImg,
}

#[derive(
    Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
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
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
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
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct EmailToken {
    pub key: String,
    pub email: String,
    pub created_at: u128,
    pub exp: u64,
}

impl EmailToken {
    pub fn new(key: impl Into<String>, email: impl Into<String>, created_at: u128) -> Self {
        Self {
            key: key.into(),
            email: email.into(),
            created_at,
            exp: 0,
        }
    }
}

// #[cfg(feature = "ssr")]
// pub fn create_cookie<Key: AsRef<[u8]>, S: Into<String>>(
//     key: Key,
//     username: S,
//     time: std::time::Duration,
// ) -> Result<(String, String), jsonwebtoken::errors::Error> {
//     use tracing::trace;
//     let key = key.as_ref();
//     let token = encode_token(key, AuthToken::new(username, time.as_nanos()))
//         .inspect_err(|err| error!("jwt exploded {err}"))?;
//     trace!("token created: {token:?}");
//     let cookie = format!("Bearer={token}; HttpOnly; Secure");
//     trace!("cookie created: {cookie:?}");
//     Ok((token, cookie))
// }
const COOKIE_PREFIX: &'static str = "Bearer ";
const COOKIE_PREFIX_FULL: &'static str = "authorization=Bearer ";
const COOKIE_POSTFIX: &'static str = "; HttpOnly; Secure";
const COOKIE_DELETED: &'static str =
    "authorization=Bearer DELETED; Secure; HttpOnly; expires=Thu, 01 Jan 1970 00:00:00 GMT";

// pub fn cut_cookie<'a>(v: &'a str, start: &str, end: &str) -> &'a str {
//     let pos_pre = v.find(start);
//     let pos_pos = v.find(end);
//     match (pos_pre, pos_pos) {
//         (Some(pre), Some(pos)) if pre < pos => &v[pre..pos],
//         (Some(pre), None) => &v[pre..],
//         (None, Some(pos)) => &v[..pos],
//         _ => v,
//     }
//     // let start_len = start.len();
//     // let v_len = v.len();
//     // let end_len = end.len();
//     // if v_len <= end_len {
//     //     return v;
//     // }
//     // let final_len = v_len - end_len;
//     // if final_len <= start_len {
//     //     return v;
//     // }
//     // &v[start_len..final_len]
// }

// pub fn auth_token_get_from_set(headers: &mut HeaderMap) -> Option<String> {
//     headers
//         .get(SET_COOKIE)
//         .inspect(|v| trace!("extract auth value raw {v:?}"))
//         .map(|v| cut_cookie(v.to_str().unwrap(), COOKIE_PREFIX_FULL, COOKIE_POSTFIX).to_string())
//         .inspect(|v| trace!("extract auth value cut {v:?}"))
// }
pub fn auth_token_get(
    headers: &HeaderMap,
    header_name: http::header::HeaderName,
) -> Option<String> {
    // let rex = Regex::new(r"[a-zA-Z\d\-_]+\.[a-zA-Z\d\-_]+\.[a-zA-Z\d\-_]+").unwrap();
    headers
        .get(header_name)
        .inspect(|v| trace!("extract auth value raw {v:?}"))
        // .and_then(|v| Some(v.to_str()))
        // .and_then(|v| rex.find(v.to_str().unwrap()))
        .and_then(|v| v.to_str().ok().and_then(|v| extract_auth_token(v)))
        // .map(|v| v.as_str().to_string())
        // .map(|v| cut_cookie(v.to_str().unwrap(), COOKIE_PREFIX_FULL, COOKIE_POSTFIX).to_string())
        .inspect(|v| trace!("extract auth value cut {v:?}"))
}

fn extract_auth_token(input: impl AsRef<str>) -> Option<String> {
    let input = input.as_ref();
    let input_len = input.len();

    let mut start = 0;
    let mut end = 0_usize;
    let mut stage = 0_usize;
    for (i, c) in input.chars().map(|v| v).enumerate() {
        if (c >= '0' && c <= '9')
            || (c >= 'A' && c <= 'Z')
            || (c >= 'a' && c <= 'z')
            || c == '-'
            || c == '_'
        {
            if stage == 0 {
                stage = 1;
                start = i;
            }
            end = i;
            trace!("0 {c} stage {stage} cursor {start} end {end}");
            continue;
        }

        if c == '.' {
            stage += 1;
            trace!("1 {c} stage {stage} cursor {start} end {end}");
            continue;
        }

        if stage == 3 {
            // end = i;
            // stage = 4;
            trace!("2 {c} stage {stage} cursor {start} end {end}");
            break;
        }

        if stage > 0 {
            stage = 0;
        }

        start = i;

        trace!("3 {c} stage {stage} cursor {start} end {end}");
    }

    if stage == 3 && start < end && end < input.len() {
        Some(input[start..=end].to_string())
    } else {
        None
    }
}

#[cfg(test)]
#[test]
fn extract_auth_token_test() {
    crate::init_test_log();

    // let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJ1c2VybmFtZSI6InByaW1lMSIsImNyZWF0ZWRfYXQiOjE3Njg3MzA1NjYwOTYzNTYxMTYsImV4cCI6MH0.naD94yClraAw9nEj-k6_zfXzad1EJ815C07IMCmTJX7yWIg78jFe2Up2EZcYt6q_Vug8AD0dwQzZ8w7pqAee-w";
    let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJ1c2VybmFtZSI6InByaW1lIiwiY3JlYXRlZF9hdCI6MTc3MDEzNjY3NTMxMTY3Njc4OCwiZXhwIjowfQ.L8rQBPtF7kYpSPGKSV8pVG9tsWuE9agDz7sKquCPMXEy5Qy2q3PsiON3Tt3Z1-q8y1We6nnsRKbK79vZtPKSBA";
    let input = format!("authorization=Bearer {token}");
    let output = extract_auth_token(&input);
    assert_eq!(Some(token.to_string()), output);

    let input = format!("authorization=Bearer {token};gj04j3t");
    let output = extract_auth_token(&input);
    assert_eq!(Some(token.to_string()), output);

    let input = format!("{token}");
    let output = extract_auth_token(&input);
    assert_eq!(Some(token.to_string()), output);
}

// pub fn auth_token_get_short(
//     headers: &HeaderMap,
//     header_name: http::header::HeaderName,
// ) -> Option<String> {
//
//     headers
//         .get(header_name)
//         .inspect(|v| trace!("extract auth value raw {v:?}"))
//         .map(|v| cut_cookie(v.to_str().unwrap(), COOKIE_PREFIX_FULL, "").to_string())
//         .inspect(|v| trace!("extract auth value cut {v:?}"))
// }

pub fn create_auth_header(token: impl AsRef<str>) -> String {
    let cookie = format!(
        "{}={}{}{}",
        AUTHORIZATION,
        COOKIE_PREFIX,
        token.as_ref(),
        COOKIE_POSTFIX,
    );

    trace!("cookie set {cookie}");
    cookie
}

pub fn create_deleted_cookie() -> HeaderMap {
    let cookie = COOKIE_DELETED.to_string();
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());
    headers
}
pub fn create_auth_cookie(token: impl AsRef<str>) -> HeaderMap {
    let cookie = format!("{}{}{}", COOKIE_PREFIX_FULL, token.as_ref(), COOKIE_POSTFIX);
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());
    trace!("set auth cookie {}", cookie);
    headers
}

// pub fn cut_cookie_value_decoded(v: &str) -> &str {
//     cut_cookie(v, COOKIE_PREFIX, "")
// }
//
// pub fn cut_cookie_full_encoded(v: &str) -> &str {
//     cut_cookie(v, COOKIE_PREFIX_FULL, COOKIE_POSTFIX)
// }

// pub fn cut_cookie_full_with_expiration_encoded(v: &str) -> &str {
//     let start = "authorization=Bearer%3D";
//     let end =
//         "%3B%20Secure%3B%20HttpOnly%3B%20expires%3DThu%2C%2001%20Jan%201970%2000%3A00%3A00%20GMT";
//     cut_cookie(v, start, end)
// }

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

    fn into_req(&self, url: impl AsRef<str>, req: ServerReq) -> ApiReq {
        ApiReq::from_api(self, url, req)
    }

    // change password

    fn send_change_password(&self, email: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_CHANGE_PASSWORD_SEND,
            ServerReq::EmailAddress {
                email: email.into(),
            },
        )
    }

    fn confirm_change_password(
        &self,
        new_password: impl Into<String>,
        confirm_key: impl Into<String>,
    ) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_CHANGE_PASSWORD_CONFIRM,
            ServerReq::ChangePassword {
                confirm_key: confirm_key.into(),
                new_password: new_password.into(),
            },
        )
    }
    // post comment
    fn add_post_comment(&self, post_id: impl Into<String>, text: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_POST_COMMENT_ADD,
            ServerReq::AddPostComment {
                post_id: post_id.into(),
                text: text.into(),
            },
        )
    }

    fn get_post_comment(&self, post_id: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_POST_COMMENT_GET,
            ServerReq::PostId {
                post_id: post_id.into(),
            },
        )
    }

    fn delete_post_comment(&self, comment_id: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_POST_COMMENT_DELETE,
            ServerReq::CommentId {
                comment_id: comment_id.into(),
            },
        )
    }

    //

    // post like
    fn add_post_like(&self, post_id: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_POST_LIKE_ADD,
            ServerReq::PostId {
                post_id: post_id.into(),
            },
        )
    }

    fn check_post_like(&self, post_id: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_POST_LIKE_CHECK,
            ServerReq::PostId {
                post_id: post_id.into(),
            },
        )
    }

    fn delete_post_like(&self, post_id: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_POST_LIKE_DELETE,
            ServerReq::PostId {
                post_id: post_id.into(),
            },
        )
    }

    //

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
        let builder = self.provide_builder(crate::path::PATH_API_ACC);
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
        let server_req = ServerReq::ConfirmToken { token };
        let result_signal = self.provide_signal_result();
        let busy_signal = self.provide_signal_busy();
        ApiReq {
            builder,
            server_req,
            result: result_signal,
            busy: busy_signal,
        }
    }

    fn get_post(&self, post_id: impl Into<String>) -> ApiReq {
        let builder = self.provide_builder(crate::path::PATH_API_POST_GET);
        let server_req = ServerReq::PostId {
            post_id: post_id.into(),
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

    fn change_username(
        &self,
        password: impl Into<String>,
        new_username: impl Into<String>,
    ) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_CHANGE_USERNAME,
            ServerReq::ChangeUsername {
                username: new_username.into(),
                password: password.into(),
            },
        )
    }

    fn get_user_posts_newer(&self, time: u128, limit: u32, username: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_USER_POST_GET_NEWER,
            ServerReq::GetUserPosts {
                time,
                limit,
                username: username.into(),
            },
        )
    }

    fn get_user_posts_older(&self, time: u128, limit: u32, username: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_USER_POST_GET_OLDER,
            ServerReq::GetUserPosts {
                time,
                limit,
                username: username.into(),
            },
        )
    }

    fn get_user_posts_older_or_equal(
        &self,
        time: u128,
        limit: u32,
        username: impl Into<String>,
    ) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_USER_POST_GET_OLDER_OR_EQUAL,
            ServerReq::GetUserPosts {
                time,
                limit,
                username: username.into(),
            },
        )
    }

    fn get_user_posts_newer_or_equal(
        &self,
        time: u128,
        limit: u32,
        username: impl Into<String>,
    ) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_USER_POST_GET_NEWER_OR_EQUAL,
            ServerReq::GetUserPosts {
                time,
                limit,
                username: username.into(),
            },
        )
    }

    fn get_posts_newer_or_equal(&self, time: u128, limit: u32) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_POST_GET_NEWER_OR_EQUAL,
            ServerReq::GetPosts { time, limit },
        )
    }

    fn get_posts_older_or_equal(&self, time: u128, limit: u32) -> ApiReq {
        let builder = self.provide_builder(crate::path::PATH_API_POST_GET_OLDER_OR_EQUAL);
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

    fn send_email_invite(&self, email: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_SEND_EMAIL_INVITE,
            ServerReq::EmailAddress {
                email: email.into(),
            },
        )
    }

    // fn send_email_new(&self, email: impl Into<String>) -> ApiReq {
    //     self.into_req(
    //         crate::path::PATH_API_EMAIL_NEW,
    //         ServerReq::SendEmailChange {
    //             new_email: email.into(),
    //         },
    //     )
    // }

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

    // change email

    fn send_email_change(&self) -> ApiReq {
        self.into_req(crate::path::PATH_API_SEND_EMAIL_CHANGE, ServerReq::None)
    }

    fn confirm_email_change(&self, confirm_token: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_CONFIRM_EMAIL_CHANGE,
            ServerReq::ConfirmToken {
                token: confirm_token.into(),
            },
        )
    }

    fn send_email_new(&self, id: impl Into<String>, email: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_SEND_EMAIL_NEW,
            ServerReq::EmailAddressWithId {
                id: id.into(),
                email: email.into(),
            },
        )
    }

    fn confirm_email_new(&self, confirm_token: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_CONFIRM_EMAIL_NEW,
            ServerReq::ConfirmToken {
                token: confirm_token.into(),
            },
        )
    }

    fn change_email(&self, id: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_CHANGE_EMAIL,
            ServerReq::Id { id: id.into() },
        )
    }

    fn resend_email_change(&self, id: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_RESEND_EMAIL_CHANGE,
            ServerReq::Id { id: id.into() },
        )
    }

    fn resend_email_new(&self, id: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_RESEND_EMAIL_NEW,
            ServerReq::Id { id: id.into() },
        )
    }

    fn change_email_status(&self, id: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_CHANGE_EMAIL_STATUS,
            ServerReq::Id { id: id.into() },
        )
    }

    fn cancel_email_change(&self, id: impl Into<String>) -> ApiReq {
        self.into_req(
            crate::path::PATH_API_CANCEL_EMAIL_CHANGE,
            ServerReq::Id { id: id.into() },
        )
    }
}

pub struct ApiReq {
    pub builder: RequestBuilder,
    pub server_req: ServerReq,
    pub result: Option<RwSignal<Option<Result<ServerRes, ServerErr>>>>,
    pub busy: Option<RwSignal<bool>>,
}

impl ApiReq {
    pub fn from_api<A>(api: &A, url: impl AsRef<str>, req: ServerReq) -> Self
    where
        A: Api + ?Sized,
    {
        let builder = api.provide_builder(url.as_ref());
        let result_signal = api.provide_signal_result();
        let busy_signal = api.provide_signal_busy();
        ApiReq {
            builder,
            server_req: req,
            result: result_signal,
            busy: busy_signal,
        }
    }

    pub fn send_web<F, Fut>(self, fut: F)
    where
        F: FnOnce(Result<ServerRes, ServerErr>) -> Fut + 'static,
        Fut: Future<Output = ()>,
    {
        let req = self.server_req;
        let builder = self.builder;
        let signal_busy = self.busy;
        let signal_result = self.result;
        if let Some(signal_busy) = signal_busy {
            if signal_busy.get_untracked() {
                tracing::warn!("trying to send while still pending");
                return;
            }
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
        auth_token: impl AsRef<str>,
    ) -> Result<ServerRes, ServerErr> {
        let req = self.server_req;
        let builder = self.builder;
        let (_, result) = send(builder, req, Some(auth_token)).await;
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
        // use axum_extra::extract::{CookieJar, cookie::Cookie};
        use http::header::SET_COOKIE;

        let secret = secret.into();
        let req = self.server_req;
        let builder = self.builder;
        let (mut headers, result) = send(builder, req, None::<&str>).await;
        // let jar = CookieJar::from_headers(&headers);
        let token = auth_token_get(&mut headers, SET_COOKIE);
        let decoded_token = token
            .clone()
            .and_then(|cookie| decode_token::<AuthToken>(secret, cookie, false).ok());
        (token, decoded_token, result)
    }
}

#[cfg(test)]
pub struct ApiTest {
    pub server: axum_test::TestServer,
}

#[cfg(test)]
impl ApiTest {
    pub fn new(server: axum_test::TestServer) -> Self {
        Self { server }
    }
}

#[cfg(test)]
impl Api for ApiTest {
    fn provide_builder(&self, path: impl AsRef<str>) -> RequestBuilder {
        let path = path.as_ref();
        let url = format!("{}{path}", crate::path::PATH_API);
        self.server.reqwest_post(&url)
    }
}

#[derive(Clone, Copy, Default)]
pub struct ApiWebTmp {}

impl Api for ApiWebTmp {
    fn provide_builder(&self, path: impl AsRef<str>) -> RequestBuilder {
        let origin = location().origin().unwrap();
        let path = path.as_ref();
        let url = format!("{origin}{}{path}", crate::path::PATH_API);
        reqwest::Client::new().post(url)
    }
}

impl ApiWebTmp {
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
            origin: origin.into(),
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
pub async fn recv(
    headers: impl AsRef<str>,
    mut multipart: axum::extract::Multipart,
) -> Result<ServerReq, ServerErr> {
    let mut bytes = Default::default();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ServerDesErr::ServerDesNextFieldErr)?
    {
        if field.name().map(|name| name == "data").unwrap_or_default() {
            bytes = field
                .bytes()
                .await
                .inspect_err(|err| error!("multipart accesing data field failed: {err}"))
                .map_err(|_| ServerDesErr::ServerMutlipartAccessErr)?;
        }
    }

    let archived = rkyv::access::<ArchivedServerReq, rkyv::rancor::Error>(&bytes)
        .inspect_err(|err| error!("{err} SERVER RECV:\n{bytes:X}"))
        .map_err(|_| ServerDesErr::ServerDesRkyvAccessErr)?;
    trace!("5");
    let client_input = rkyv::deserialize::<ServerReq, rkyv::rancor::Error>(archived)
        .inspect_err(|err| error!("{err} SERVER RECV:\n{bytes:X}"))
        .map_err(|_| ServerDesErr::ServerDesRkyvErr)?;

    debug!(
        "SERVER RECV:\n{}\n {client_input:?} - {bytes:X}",
        headers.as_ref()
    );

    Ok(client_input)
}

#[cfg(feature = "ssr")]
impl axum::response::IntoResponse for ServerErr {
    fn into_response(self) -> axum::response::Response {
        // use axum_extra::extract::{CookieJar, cookie::Cookie};

        let status = axum::http::StatusCode::INTERNAL_SERVER_ERROR;
        // let status = match self {
        //     ServerErr::ServerDbErr
        //     | ServerErr::ServerRegistrationErr(ServerRegistrationErr::ServerCreateCookieErr)
        //     | ServerErr::ServerLoginErr(ServerLoginErr::ServerCreateCookieErr(_))
        //     | ServerErr::ServerTokenErr(ServerTokenErr::ServerJWT) => {
        //         axum::http::StatusCode::INTERNAL_SERVER_ERROR
        //     }
        //     ServerErr::ServerDesErr(_)
        //     | ServerErr::ServerAddPostErr(ServerAddPostErr::InvalidTitle(_))
        //     | ServerErr::ServerAddPostErr(ServerAddPostErr::InvalidDescription(_))
        //     | ServerErr::ServerRegistrationErr(ServerRegistrationErr::TokenExpired)
        //     | ServerErr::ServerRegistrationErr(ServerRegistrationErr::TokenUsed)
        //     | ServerErr::ServerRegistrationErr(ServerRegistrationErr::TokenNotFound)
        //     | ServerErr::ServerRegistrationErr(ServerRegistrationErr::ServerJWT(_))
        //     | ServerErr::ServerDecodeInviteErr(ServerDecodeInviteErr::InviteNotFound)
        //     | ServerErr::ServerDecodeInviteErr(ServerDecodeInviteErr::JWT(_))
        //     | ServerErr::ServerAddPostErr(ServerAddPostErr::ServerImgErr(_))
        //     | ServerErr::ServerAddPostErr(ServerAddPostErr::ServerFSErr(_))
        //     | ServerErr::ServerAddPostErr(ServerAddPostErr::ServerDirCreationFailed(_))
        //     | ServerErr::ChangeUsernameErr(ChangeUsernameErr::UsernameIsTaken(_))
        //     | ServerErr::ChangeEmailErr(ChangeEmailErr::EmailIsTaken(_))
        //     | ServerErr::ServerRegistrationErr(
        //         ServerRegistrationErr::ServerRegistrationInvalidInput { .. },
        //     ) => axum::http::StatusCode::BAD_REQUEST,
        //     ServerErr::ServerAuthErr(ServerAuthErr::ServerUnauthorizedNoCookie) => {
        //         axum::http::StatusCode::OK
        //     }
        //     ServerErr::ServerAuthErr(ServerAuthErr::ServerUnauthorizedInvalidCookie)
        //     | ServerErr::ChangeUsernameErr(ChangeUsernameErr::WrongCredentials)
        //     | ServerErr::ChangeEmailErr(ChangeEmailErr::InvalidToken)
        //     | ServerErr::ServerLoginErr(ServerLoginErr::WrongCredentials) => {
        //         axum::http::StatusCode::UNAUTHORIZED
        //     }
        //     ServerErr::ServerGetErr(ServerGetErr::NotFound)
        //     | ServerErr::ChangeEmailErr(ChangeEmailErr::NotFound)
        //     | ServerErr::ChangeUsernameErr(ChangeUsernameErr::NotFound) => {
        //         axum::http::StatusCode::NOT_FOUND
        //     }
        //     ServerErr::ClientErr(_) => unreachable!(),
        // };

        match self {
            ServerErr::AuthErr(ServerAuthErr::ServerUnauthorizedInvalidCookie) => {
                let result: Result<ServerRes, ServerErr> = Err(self);
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&result).unwrap();
                let bytes = bytes.to_vec();
                // let bytes: bytes::Bytes = bytes.into();
                let headers = create_deleted_cookie();
                // let jar = CookieJar::new().add(Cookie::new(AUTHORIZATION.as_str(), COOKIE_DELETED));
                (status, headers, bytes).into_response()
            }
            server_err => {
                let result: Result<ServerRes, ServerErr> = Err(server_err);
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&result).unwrap();
                let bytes = bytes.to_vec();
                // let bytes: bytes::Bytes = bytes.into();
                (status, bytes).into_response()
            }
        }
    }
}

#[cfg(feature = "ssr")]
impl axum::response::IntoResponse for ServerRes {
    fn into_response(self) -> axum::response::Response {
        // use axum_extra::extract::{CookieJar, cookie::Cookie};

        match self {
            ServerRes::DeleteAuthCookie => {
                let result: Result<ServerRes, ServerErr> = Ok(ServerRes::Ok);
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&result).unwrap();
                let bytes = bytes.to_vec();
                // let bytes: bytes::Bytes = bytes.into();
                let headers = create_deleted_cookie();
                (headers, bytes).into_response()
            }
            ServerRes::SetAuthCookie { token } => {
                let result: Result<ServerRes, ServerErr> = Ok(ServerRes::Ok);
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&result).unwrap();
                let bytes = bytes.to_vec();
                // let bytes: bytes::Bytes = bytes.into();
                let headers = create_auth_cookie(token);

                debug!("SERVER SEND:\n{result:?} - {bytes:?}");

                (headers, bytes).into_response()
            }
            res => {
                let result: Result<ServerRes, ServerErr> = Ok(res);
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&result).unwrap();
                let bytes = bytes.to_vec();
                // let bytes: bytes::Bytes = bytes.into();
                debug!("SERVER SEND:\n{result:?} - {bytes:?}");

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
    // http::header::REFERER
    debug!("CLIENT SEND:\n{req:?} - {:?}", bytes.as_ref());
    let part = reqwest::multipart::Part::bytes(bytes.to_vec());
    let form = reqwest::multipart::Form::new().part("data", part);
    if let Some(token) = token {
        let cookie = create_auth_header(token);
        req_builder = req_builder.header(http::header::COOKIE, cookie);
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

#[cfg(test)]
pub mod tests {
    use axum::Router;
    use std::path::Path;
    use std::sync::Arc;
    use std::time::Duration;
    use surrealdb::RecordId;
    use tokio::fs::{self, create_dir_all};

    use axum_test::TestServer;
    use gxhash::gxhash128;
    // use pretty_assertions::assert_eq;
    // use test_log::test;
    use tokio::sync::Mutex;
    use tracing::{debug, error, trace};

    use crate::api::app_state::AppState;
    use crate::api::shared::post_comment::UserPostComment;
    use crate::api::{
        Api, ApiTest, EmailChangeErr, EmailChangeNewErr, EmailChangeStage, EmailChangeTokenErr,
        EmailToken, PostLikeErr, Server404Err, ServerAuthErr, ServerErr, ServerLoginErr,
        ServerRegistrationErr, ServerReqImg, ServerRes, UserPost, encode_token,
    };
    use crate::db::post_comment::DBPostComment;
    use crate::db::email_change::create_email_change_id;
    use crate::db::{DBUser, EmailIsTakenErr, email_change::DBEmailChange};
    // use crate::db::DBEmailTokenKind;
    use crate::server::create_api_router;
    // use tracing_appender::rolling;

    pub struct ApiTestApp {
        pub state: AppState,
        // pub router: Router,
        // pub server: TestServer,
        pub time: Arc<Mutex<u128>>,
        pub api: ApiTest,
    }

    #[derive(thiserror::Error, Debug)]
    enum TestErr {
        // #[error("assert failed {0}")]
        // Assert(String),
        #[error("assert failed")]
        Assert,
    }

    impl ApiTestApp {
        pub async fn new(invite_exp_ns: u128) -> Self {
            let _ = tracing_subscriber::fmt()
                .event_format(
                    tracing_subscriber::fmt::format()
                        .with_file(true)
                        .with_line_number(true),
                )
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                // .with_writer(file)
                .try_init();

            let time_mut = Arc::new(Mutex::new(0));
            let app_state = AppState::new_testng(time_mut.clone(), invite_exp_ns).await;
            let my_app = create_api_router(app_state.clone()).with_state(app_state.clone());
            let server = TestServer::builder()
                .http_transport()
                .build(my_app)
                .unwrap();
            let api = ApiTest::new(server);
            Self {
                state: app_state,
                // router: my_app,
                // server,
                time: time_mut,
                api,
            }
        }

        pub async fn set_time(&self, time: u128) {
            *self.time.lock().await = time;
        }

        // pub async fn set_exp(&mut self, duration_ns: u128) {
        //     self.state.set_invite_exp_ns(duration_ns);
        // }

        pub async fn add_post(
            &self,
            time: u128,
            auth_token: impl Into<String>,
        ) -> Option<UserPost> {
            self.set_time(time).await;
            let auth_token = auth_token.into();

            let mut imgbuf = image::ImageBuffer::new(250, 250);
            // Iterate over the coordinates and pixels of the image
            for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
                let r = (0.3 * x as f32) as u8;
                let b = (0.3 * y as f32) as u8;
                *pixel = image::Rgb([r, 0, b]);
            }

            create_dir_all("../target/tmp/").await.unwrap();
            let path = "../target/tmp/img.png";
            imgbuf.save(path).unwrap();

            let img = tokio::fs::read(path).await.unwrap();
            let result = self
                .api
                .add_post(
                    "title1",
                    "wow",
                    Vec::from([ServerReqImg {
                        path: path.to_string(),
                        data: img.clone(),
                    }]),
                )
                .send_native_with_token(auth_token.clone())
                .await;
            trace!("{result:#?}");

            // let result_posts = self.api.get_posts_older(2, 25).send_native().await.unwrap();

            match result {
                Ok(crate::api::ServerRes::Post(post)) => Some(post),
                _ => None,
            }
            // let matched = matches!(result, Ok(crate::api::ServerRes::Post(_)));
            // let matched = match result {
            //     Ok(crate::api::ServerRes::Posts(posts)) => posts.len() == 1,
            //     wrong => false,
            // };

            // if matched { Some(()) } else { None }
        }

        // pub async fn add_post_comment(
        //     &self,
        //     server_time: u128,
        //     auth_token: impl AsRef<str>,
        //     post_id: impl Into<String>,
        //     text: impl Into<String>,
        // ) -> Option<UserPostComment> {
        //     self.set_time(server_time).await;
        //     let result = self
        //         .api
        //         .add_post_comment(post_id, text)
        //         .send_native_with_token(auth_token)
        //         .await;
        //
        //     let Ok(ServerRes::Comment(comment)) = result else {
        //
        //         return None;
        //     };
        //
        //     Some(comment)
        // }

        pub async fn expect_posts(
            &self,
            server_time: u128,
            post_count_newer: usize,
            post_count_newer_or_equal: usize,
            post_count_older: usize,
            post_count_older_or_equal: usize,
        ) -> Option<()> {
            let (matched_newer, len_newer) = match self
                .api
                .get_posts_newer(server_time, 1000)
                .send_native()
                .await
            {
                Ok(crate::api::ServerRes::Posts(posts)) => {
                    let len = posts.len();
                    let result = len == post_count_newer;
                    (result, len)
                }
                wrong => (false, 0),
            };

            let (matched_newer_or_equal, len_newer_or_equal) = match self
                .api
                .get_posts_newer_or_equal(server_time, 1000)
                .send_native()
                .await
            {
                Ok(crate::api::ServerRes::Posts(posts)) => {
                    let len = posts.len();
                    let result = len == post_count_newer_or_equal;
                    (result, len)
                }
                wrong => (false, 0),
            };

            let (matched_older, len_older) = match self
                .api
                .get_posts_older(server_time, 1000)
                .send_native()
                .await
            {
                Ok(crate::api::ServerRes::Posts(posts)) => {
                    let len = posts.len();
                    let result = len == post_count_older;
                    (result, len)
                }
                wrong => (false, 0),
            };

            let (matched_older_or_equal, len_older_or_equal) = match self
                .api
                .get_posts_older_or_equal(server_time, 1000)
                .send_native()
                .await
            {
                Ok(crate::api::ServerRes::Posts(posts)) => {
                    let len = posts.len();
                    let result = len == post_count_older_or_equal;
                    (result, len)
                }
                wrong => (false, 0),
            };

            if !matched_newer {
                error!("expected newer len to be {post_count_newer}, got {len_newer}.");
            }
            if !matched_newer_or_equal {
                error!(
                    "expected newer_or_equal len to be {post_count_newer_or_equal}, got {len_newer_or_equal}."
                );
            }
            if !matched_older {
                error!("expected older len to be {post_count_older}, got {len_older}.");
            }
            if !matched_older_or_equal {
                error!(
                    "expected older_or_equal len to be {post_count_older_or_equal}, got {len_older_or_equal}."
                );
            }

            if matched_newer && matched_newer_or_equal && matched_older && matched_older_or_equal {
                Some(())
            } else {
                None
            }
        }
        pub async fn delete_post_like(
            &self,
            server_time: u128,
            auth_token: impl AsRef<str>,
            post_id: impl Into<String>,
        ) -> Option<()> {
            self.set_time(server_time).await;

            let result = self
                .api
                .delete_post_like(post_id)
                .send_native_with_token(auth_token)
                .await;

            if result == Ok(ServerRes::Ok) {
                Some(())
            } else {
                None
            }
        }

        pub async fn add_post_like(
            &self,
            server_time: u128,
            auth_token: impl AsRef<str>,
            post_id: impl Into<String>,
        ) -> Option<()> {
            self.set_time(server_time).await;
            let result = self
                .api
                .add_post_like(post_id)
                .send_native_with_token(auth_token)
                .await;

            if result == Ok(ServerRes::Ok) {
                Some(())
            } else {
                None
            }
        }

        pub async fn add_post_like_err_already_liked(
            &self,
            server_time: u128,
            auth_token: impl AsRef<str>,
            post_id: impl Into<String>,
        ) -> Option<()> {
            self.set_time(server_time).await;
            let result = self
                .api
                .add_post_like(post_id)
                .send_native_with_token(auth_token)
                .await;

            if matches!(
                result,
                Err(ServerErr::PostLikeErr(PostLikeErr::PostAlreadyLiked(_)))
            ) {
                Some(())
            } else {
                None
            }
        }

        pub async fn add_post_like_err_not_found(
            &self,
            server_time: u128,
            auth_token: impl AsRef<str>,
            post_id: impl Into<String>,
        ) -> Option<()> {
            self.set_time(server_time).await;
            let result = self
                .api
                .add_post_like(post_id)
                .send_native_with_token(auth_token)
                .await;
            if matches!(
                result,
                Err(ServerErr::PostLikeErr(PostLikeErr::PostNotFound(_)))
            ) {
                Some(())
            } else {
                None
            }
        }

        pub async fn check_post_like(
            &self,
            server_time: u128,
            auth_token: impl AsRef<str>,
            post_id: impl Into<String>,
            condition: bool,
        ) -> Option<()> {
            self.set_time(server_time).await;
            let result = self
                .api
                .check_post_like(post_id)
                .send_native_with_token(auth_token)
                .await
                .unwrap();
            if result == ServerRes::Condition(condition) {
                Some(())
            } else {
                None
            }
        }

        pub async fn register(
            &self,
            server_time: u128,
            username: impl Into<String>,
            email: impl Into<String>,
            password: impl Into<String>,
        ) -> Option<String> {
            self.set_time(server_time).await;
            let secret = self.state.get_secret().await;

            let username = username.into();
            let email = email.into();
            let password = password.into();
            let time = self.state.clock.now().await;

            let result = self
                .api
                .send_email_invite(email.clone())
                .send_native()
                .await
                .unwrap();

            let all = self.state.db.get_invite_all().await.unwrap();
            trace!("----- ALL INVITES ------\n{all:#?}");

            let invite = self
                .state
                .db
                .get_invite_valid(
                    time,
                    // DBEmailTokenKind::RequestConfirmRegistrationEmail,
                    email,
                    // 0,
                )
                .await
                .unwrap();

            let (token, decoded_token, result) = self
                .api
                .register(username, &invite.token_raw, password)
                .send_native_and_extract_auth(&secret)
                .await;

            token
        }

        pub async fn register_fail_expired_taken(
            &self,
            server_time: u128,
            leap_time: u128,
            username: impl Into<String>,
            email: impl Into<String>,
            password: impl Into<String>,
        ) -> Option<()> {
            self.set_time(server_time).await;
            let username = username.into();
            let email = email.into();
            let password = password.into();

            let result = self
                .api
                .send_email_invite(email.clone())
                .send_native()
                .await
                .unwrap();

            let all = self.state.db.get_invite_all().await.unwrap();
            trace!("----- ALL INVITES ------\n{all:#?}");

            let invite = self
                .state
                .db
                .get_invite_valid(server_time, email)
                .await
                .unwrap();

            self.set_time(leap_time).await;

            let result = self
                .api
                .register(username, &invite.token_raw, password)
                .send_native()
                .await;

            let matched = matches!(
                result,
                Err(ServerErr::RegistrationErr(
                    ServerRegistrationErr::TokenExpired
                ))
            );
            self.set_time(server_time).await;

            if matched { Some(()) } else { None }
        }

        pub async fn register_fail_404(
            &self,
            server_time: u128,
            username: impl Into<String>,
        ) -> Option<()> {
            self.set_time(server_time).await;
            let username = username.into();
            let email = format!("{username}@hey.com");
            let password = "passworD1%%%";

            let result = self
                .api
                .register(username, "404", password)
                .send_native()
                .await;

            let matched = matches!(
                result,
                Err(ServerErr::RegistrationErr(
                    ServerRegistrationErr::TokenNotFound
                ))
            );

            if matched { Some(()) } else { None }
        }
        pub async fn register_fail_invalid(
            &self,
            server_time: u128,
            username: impl Into<String>,
            email: impl Into<String>,
            password: impl Into<String>,
        ) -> Option<()> {
            self.set_time(server_time).await;
            let username = username.into();
            let email = email.into();
            let password = password.into();

            let result = self
                .api
                .send_email_invite(email.clone())
                .send_native()
                .await
                .unwrap();
            let invite = self
                .state
                .db
                .get_invite_valid(server_time, email)
                .await
                .unwrap();

            let result = self
                .api
                .register(username, &invite.token_raw, password)
                .send_native()
                .await;

            trace!("recv: {result:?}");

            let matched = match result {
                Err(ServerErr::RegistrationErr(
                    ServerRegistrationErr::ServerRegistrationInvalidInput {
                        username,
                        email,
                        password,
                    },
                )) => username.is_some() && email.is_none() && password.is_some(),
                etc => false,
            };

            if matched { Some(()) } else { None }
        }

        pub async fn is_logged_in(
            &self,
            server_time: u128,
            auth_token: impl AsRef<str>,
        ) -> Option<()> {
            self.set_time(server_time).await;

            let result = self.api.profile().send_native_with_token(&auth_token).await;
            let matched = match result {
                Ok(ServerRes::Acc { username, email }) => true,
                _ => false,
            };

            if matched { Some(()) } else { None }
        }

        pub async fn is_logged_out(
            &self,
            server_time: u128,
            auth_token: impl AsRef<str>,
        ) -> Option<()> {
            self.set_time(server_time).await;

            let result = self.api.profile().send_native_with_token(&auth_token).await;
            let matched = result
                == Err(ServerErr::AuthErr(
                    ServerAuthErr::ServerUnauthorizedInvalidCookie,
                ));

            if matched { Some(()) } else { None }
        }

        pub async fn logout(&self, server_time: u128, auth_token: impl AsRef<str>) -> Option<()> {
            self.set_time(server_time).await;

            let result = self.api.logout().send_native_with_token(&auth_token).await;

            let matched = result == Ok(ServerRes::Ok);

            let result = self.api.profile().send_native_with_token(&auth_token).await;
            let matched_profile = result == Err(ServerErr::DbErr);

            if matched { Some(()) } else { None }
        }

        pub async fn login(
            &self,
            server_time: u128,
            email: impl Into<String>,
            password: impl Into<String>,
        ) -> Option<String> {
            self.set_time(server_time).await;

            let email = email.into();
            let password = password.into();
            let secret = self.state.get_secret().await;

            let (token, decoded_token, result) = self
                .api
                .login(email, password)
                .send_native_and_extract_auth(&secret)
                .await;

            token
        }

        pub async fn req_email_change(
            &self,
            server_time: u128,
            auth_token: impl AsRef<str>,
            expires: u128,
        ) -> Option<String> {
            self.set_time(server_time).await;

            let result = self
                .api
                .send_email_change()
                .send_native_with_token(auth_token.as_ref())
                .await;

            let matched = matches!(
                result,
                Ok(ServerRes::EmailChangeStage(
                    EmailChangeStage::ConfirmEmail { .. }
                ))
            );
            // let matched = result
            //     == Ok(ServerRes::EmailChangeStage(EmailChangeStage::ConfirmEmail {
            //     expires
            // }));

            if matched {
                match result {
                    Ok(ServerRes::EmailChangeStage(EmailChangeStage::ConfirmEmail {
                        id,
                        old_email,
                        expires,
                    })) => Some(id),
                    _ => unreachable!(),
                }
            } else {
                None
            }
        }

        pub async fn req_email_new(
            &self,
            server_time: u128,
            id: impl Into<String>,
            auth_token: impl AsRef<str>,
            new_email: impl AsRef<str>,
            expires: u128,
        ) -> Option<()> {
            self.set_time(server_time).await;

            let result = self
                .api
                .send_email_new(id, new_email.as_ref())
                .send_native_with_token(auth_token.as_ref())
                .await;

            let matched = matches!(
                result,
                Ok(ServerRes::EmailChangeStage(
                    EmailChangeStage::ConfirmNewEmail { .. }
                ))
            );

            if matched { Some(()) } else { None }
        }

        pub async fn req_email_new_fail_taken(
            &self,
            server_time: u128,
            id: impl Into<String>,
            auth_token: impl AsRef<str>,
            new_email: impl AsRef<str>,
        ) -> Option<()> {
            self.set_time(server_time).await;

            let result = self
                .api
                .send_email_new(id, new_email.as_ref())
                .send_native_with_token(auth_token.as_ref())
                .await;

            let matched = matches!(
                result,
                Err(ServerErr::EmailChangeNew(EmailChangeNewErr::EmailIsTaken(
                    _
                )))
            );

            if matched { Some(()) } else { None }
        }

        pub async fn req_email_new_fail_stage(
            &self,
            server_time: u128,
            id: impl Into<String>,
            auth_token: impl AsRef<str>,
            new_email: impl AsRef<str>,
        ) -> Option<()> {
            self.set_time(server_time).await;

            let result = self
                .api
                .send_email_new(id, new_email.as_ref())
                .send_native_with_token(auth_token.as_ref())
                .await;

            let matched = matches!(
                result,
                Err(ServerErr::EmailChange(EmailChangeErr::InvalidStage(_)))
            );

            if !matched {
                error!("RESULT: {result:?} EXPETED INVALID STAGE");
            }

            if matched { Some(()) } else { None }
        }

        pub async fn req_email_new_fail_invalid(
            &self,
            server_time: u128,
            id: impl Into<String>,
            auth_token: impl AsRef<str>,
            new_email: impl AsRef<str>,
        ) -> Option<()> {
            self.set_time(server_time).await;

            let result = self
                .api
                .send_email_new(id, new_email.as_ref())
                .send_native_with_token(auth_token.as_ref())
                .await;

            let matched = matches!(
                result,
                Err(ServerErr::EmailChangeNew(EmailChangeNewErr::TokenInvalid))
            );

            if matched { Some(()) } else { None }
        }

        pub async fn req_email_change_complete(
            &self,
            server_time: u128,
            id: impl Into<String>,
            auth_token: impl AsRef<str>,
            new_email: impl AsRef<str>,
            expires: u128,
        ) -> Option<()> {
            self.set_time(server_time).await;

            let result = self
                .api
                .change_email(id)
                .send_native_with_token(auth_token.as_ref())
                .await;

            let matched = matches!(
                result,
                Ok(ServerRes::EmailChangeStage(
                    EmailChangeStage::Complete { .. }
                ))
            );

            if matched { Some(()) } else { None }
        }

        pub async fn confirm_email_change(
            &self,
            server_time: u128,
            id: String,
            auth_token: impl AsRef<str>,
            db_user: &DBUser,
            expires: u128,
        ) -> Option<()> {
            self.set_time(server_time).await;

            let confirm_token = self
                .state
                .db
                .get_email_change(0, create_email_change_id(id))
                .await
                .unwrap();

            let result = self
                .api
                .confirm_email_change(confirm_token.current.token_raw.clone())
                .send_native_with_token(auth_token.as_ref())
                .await;

            let matched = matches!(
                result,
                Ok(ServerRes::EmailChangeStage(
                    EmailChangeStage::EnterNewEmail { .. }
                ))
            );

            if matched { Some(()) } else { None }
        }

        // pub async fn confirm_email_change_fail_stage(
        //     &self,
        //     server_time: u128,
        //     auth_token: impl AsRef<str>,
        //     db_user: &DBUser,
        // ) -> Option<()> {
        //     self.set_time(server_time).await;
        //
        //     let result = self
        //         .api
        //         .confirm_email_change("invalid")
        //         .send_native_with_token(auth_token.as_ref())
        //         .await;
        //
        //     let matched = matches!(
        //         result,
        //         Err(ServerErr::EmailChangeToken(
        //             EmailChangeTokenErr::InvalidStage(_)
        //         ))
        //     );
        //
        //     if matched { Some(()) } else { None }
        // }

        pub async fn confirm_email_change_fail_invalid(
            &self,
            server_time: u128,
            auth_token: impl AsRef<str>,
            db_user: &DBUser,
        ) -> Option<()> {
            self.set_time(server_time).await;

            let confirm_token = self.state.db.get_email_change(0, db_user.id.clone()).await;

            let result = self
                .api
                .confirm_email_change(
                    confirm_token
                        .map(|v| v.current.token_raw)
                        .unwrap_or(String::from("invalid")),
                )
                .send_native_with_token(auth_token.as_ref())
                .await;

            let matched = matches!(
                result,
                Err(ServerErr::EmailChangeToken(
                    EmailChangeTokenErr::TokenInvalid
                ))
            );

            if matched { Some(()) } else { None }
        }
        pub async fn confirm_email_new(
            &self,
            serevr_time: u128,
            id: String,
            auth_token: impl AsRef<str>,
            db_user: &DBUser,
            new_email: impl AsRef<str>,
            expires: u128,
        ) -> Option<()> {
            self.set_time(serevr_time).await;

            let confirm_token = self
                .state
                .db
                .get_email_change(serevr_time, create_email_change_id(id))
                .await
                .unwrap();

            let result = self
                .api
                .confirm_email_new(confirm_token.new.clone().unwrap().token_raw)
                .send_native_with_token(auth_token.as_ref())
                .await;

            let matched = matches!(
                result,
                Ok(ServerRes::EmailChangeStage(
                    EmailChangeStage::ReadyToComplete { .. }
                ))
            );

            if matched { Some(()) } else { None }
        }

        pub async fn confirm_email_new_fail_stage(
            &self,
            server_time: u128,
            id: impl Into<String>,
            auth_token: impl AsRef<str>,
        ) -> anyhow::Result<()> {
            self.set_time(server_time).await;

            let confirm_token = self
                .state
                .db
                .get_email_change(0, create_email_change_id(id.into()))
                .await?;

            let result = self
                .api
                .confirm_email_new(confirm_token.new.clone().unwrap().token_raw)
                .send_native_with_token(auth_token.as_ref())
                .await;

            let matched = matches!(
                result,
                Err(ServerErr::EmailChangeToken(
                    EmailChangeTokenErr::InvalidStage(_)
                ))
            );

            if !matched {
                error!("RESULT: {result:?} EXPETED INVALID STAGE");
            }

            if matched {
                Ok(())
            } else {
                Err(TestErr::Assert.into())
            }
        }

        pub async fn confirm_email_new_fail_invalid(
            &self,
            server_time: u128,
            auth_token: impl AsRef<str>,
            db_user: &DBUser,
        ) -> Option<()> {
            self.set_time(server_time).await;

            let result = self
                .api
                .confirm_email_new("invalid")
                .send_native_with_token(auth_token.as_ref())
                .await;

            let matched = matches!(
                result,
                Err(ServerErr::EmailChangeToken(
                    EmailChangeTokenErr::TokenInvalid
                ))
            );

            if matched { Some(()) } else { None }
        }

        pub async fn status_email_change(
            &self,
            server_time: u128,
            id: impl Into<String>,
            auth_token: impl AsRef<str>,
            expected_status: impl FnOnce(EmailChangeStage) -> bool,
            expected_email: Option<String>,
            expires: Option<u128>,
        ) -> Option<()> {
            self.set_time(server_time).await;

            let result = self
                .api
                .change_email_status(id)
                .send_native_with_token(auth_token.as_ref())
                .await;
            let matched = match result {
                Ok(ServerRes::EmailChangeStage(stage)) => expected_status(stage),
                _ => false,
            };

            if matched { Some(()) } else { None }
        }

        pub async fn status_email_change_404(
            &self,
            server_time: u128,
            id: impl Into<String>,
            auth_token: impl AsRef<str>,
        ) -> Option<()> {
            self.set_time(server_time).await;

            let result = self
                .api
                .change_email_status(id)
                .send_native_with_token(auth_token.as_ref())
                .await;
            let matched = matches!(result, Err(ServerErr::NotFoundErr(Server404Err::NotFound)));

            if matched { Some(()) } else { None }
        }

        pub async fn resend_change(
            &self,
            server_time: u128,
            id: impl Into<String>,
            auth_token: impl AsRef<str>,
            expected_rec_email: impl Into<String>,
            expected_new_email: Option<String>,
            expires: u128,
        ) -> Option<()> {
            self.set_time(server_time).await;

            let result = self
                .api
                .resend_email_change(id)
                .send_native_with_token(auth_token.as_ref())
                .await;

            let matched = matches!(
                result,
                Ok(ServerRes::EmailChangeStage(
                    EmailChangeStage::ConfirmEmail { .. }
                ))
            );
            // let matched = result
            //     == Ok(ServerRes::EmailChangeStage {
            //         stage: Some(EmailChangeStage::ConfirmEmail),
            //         new_email: expected_new_email,
            //         expires: Some(expires),
            //     });

            let db_result = self
                .state
                .db
                .get_sent_email_by_email_latest(expected_rec_email.into())
                .await;

            if matched && db_result.is_ok() {
                Some(())
            } else {
                None
            }
        }

        pub async fn resend_new(
            &self,
            server_time: u128,
            id: impl Into<String>,
            auth_token: impl AsRef<str>,
            expected_rec_email: impl Into<String>,
            expected_new_email: Option<String>,
            expires: u128,
        ) -> Option<()> {
            self.set_time(server_time).await;

            let result = self
                .api
                .resend_email_new(id)
                .send_native_with_token(auth_token.as_ref())
                .await;

            let matched = matches!(
                result,
                Ok(ServerRes::EmailChangeStage(
                    EmailChangeStage::ConfirmNewEmail { .. }
                ))
            );
            // let matched = result
            //     == Ok(ServerRes::EmailChangeStage {
            //         stage: Some(EmailChangeStage::ConfirmNewEmail),
            //         new_email: expected_new_email,
            //         expires: Some(expires),
            //     });

            // let db_result = self
            //     .state
            //     .db
            //     .get_sent_email_by_email_latest(expected_rec_email.into())
            //     .await;

            // let
            // let db_matched = db_result.map(|v| v.body.contains("a"));

            if matched { Some(()) } else { None }
        }

        pub async fn cancel_email_change(
            &self,
            server_time: u128,
            id: impl Into<String>,
            auth_token: impl AsRef<str>,
        ) -> anyhow::Result<()> {
            self.set_time(server_time).await;

            let result = self
                .api
                .cancel_email_change(id)
                .send_native_with_token(auth_token)
                .await?;

            Ok(())
        }
    }

    // async fn register(
    //     app_state: AppState,
    //     api: &ApiTest<'_>,
    //     username: impl Into<String>,
    // ) -> String {
    // }
    #[tokio::test]
    async fn api_change_password_test() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;

        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789B")
            .await
            .unwrap();

        let result = app
            .api
            .send_change_password("hey@heyadora.com")
            .send_native()
            .await;
        assert!(matches!(result, Ok(ServerRes::Ok)));

        let confirm_token = app
            .state
            .db
            .get_confirm_email_latest(0, "hey@heyadora.com")
            .await
            .unwrap();

        let result = app
            .api
            .confirm_change_password("pas$word123456789A", confirm_token.id.key().to_string())
            .send_native()
            .await;

        assert!(matches!(result, Ok(ServerRes::Ok)));

        let result = app.login(0, "hey@heyadora.com", "pas$word123456789B").await;
        assert!(result.is_none());

        let result = app.login(1, "hey@heyadora.com", "pas$word123456789A").await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn api_email_change_test() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;
        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();
        let auth_token3 = app
            .register(0, "hey3", "hey3@heyadora.com", "pas$word123456789")
            .await
            .unwrap();
        let db_user = app.state.db.get_user_by_username("hey").await.unwrap();

        app.status_email_change_404(0, "invalid", &auth_token)
            .await
            .unwrap();
        app.req_email_new_fail_invalid(0, "invalid", &auth_token, "hey3@hey.com")
            .await;
        app.confirm_email_new_fail_invalid(0, &auth_token, &db_user)
            .await;
        app.confirm_email_change_fail_invalid(0, &auth_token, &db_user)
            .await;

        // ### START
        let id = app.req_email_change(0, &auth_token, 1).await.unwrap();
        // app.req_email_change_fail_stage(0, id.clone(), &auth_token)
        //     .await
        //     .unwrap();
        app.cancel_email_change(0, id.clone(), &auth_token)
            .await
            .unwrap();
        app.status_email_change(
            0,
            id.clone(),
            &auth_token,
            |v| v.is_cancelled(),
            None,
            Some(1),
        )
        .await
        .unwrap();
        let id = app.req_email_change(0, &auth_token, 1).await.unwrap();

        app.req_email_new_fail_stage(0, id.clone(), &auth_token, "hey3@hey.com")
            .await
            .unwrap();
        app.status_email_change(
            0,
            id.clone(),
            &auth_token,
            |v| v.is_confirm_email(),
            None,
            Some(1),
        )
        .await
        .unwrap();
        app.resend_change(0, id.clone(), &auth_token, "hey@heyadora.com", None, 1)
            .await
            .unwrap();

        // ###
        app.confirm_email_change(0, id.clone(), &auth_token, &db_user, 1)
            .await
            .unwrap();

        // app.req_email_change_fail_stage(0, id.clone(), &auth_token)
        //     .await
        //     .unwrap();
        app.req_email_new_fail_taken(0, id.clone(), &auth_token, "hey3@heyadora.com")
            .await
            .unwrap();
        app.status_email_change(
            0,
            id.clone(),
            &auth_token,
            |v| v.is_enter_new_email(),
            None,
            Some(1),
        )
        .await
        .unwrap();

        // ###
        app.req_email_new(0, id.clone(), &auth_token, "hey2@hey.com", 1)
            .await
            .unwrap();

        app.confirm_email_new_fail_invalid(0, &auth_token, &db_user)
            .await
            .unwrap();
        app.status_email_change(
            0,
            id.clone(),
            &auth_token,
            |v| v.is_confirm_new_email(),
            Some("hey2@hey.com".to_string()),
            Some(1),
        )
        .await
        .unwrap();
        app.resend_new(
            0,
            id.clone(),
            &auth_token,
            "hey@heyadora.com",
            Some("hey2@hey.com".to_string()),
            1,
        )
        .await
        .unwrap();

        // ###
        app.confirm_email_new(0, id.clone(), &auth_token, &db_user, "hey2@hey.com", 1)
            .await
            .unwrap();

        app.confirm_email_new_fail_stage(0, &id, &auth_token)
            .await
            .unwrap();
        app.status_email_change(
            0,
            id.clone(),
            &auth_token,
            |v| v.is_ready_to_complete(),
            Some("hey2@hey.com".to_string()),
            Some(1),
        )
        .await
        .unwrap();

        // ###
        app.req_email_change_complete(0, id.clone(), &auth_token, "hey2@hey.com", 1)
            .await
            .unwrap();

        app.confirm_email_change_fail_invalid(0, &auth_token, &db_user)
            .await
            .unwrap();
        app.confirm_email_new_fail_invalid(0, &auth_token, &db_user)
            .await
            .unwrap();
        app.req_email_new_fail_stage(0, id.clone(), &auth_token, "hey2@hey.com")
            .await
            .unwrap();
        app.status_email_change(0, id.clone(), &auth_token, |v| v.is_complete(), None, None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn api_auth_test() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;
        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();
        app.is_logged_in(0, &auth_token).await.unwrap();
        app.register_fail_expired_taken(0, 2, "hey2", "hey2@heyadora.com", "pas$word123456789")
            .await
            .unwrap();
        app.register_fail_404(0, "hey2").await.unwrap();
        app.register_fail_invalid(0, "pr", "prime@heyadora.com", "wowowowwoW12222pp")
            .await
            .unwrap();
        app.logout(0, &auth_token).await.unwrap();
        app.is_logged_out(0, &auth_token).await.unwrap();
        let auth_token = app
            .login(0, "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();
        app.is_logged_in(0, &auth_token).await.unwrap();
    }


    #[tokio::test]
    async fn api_post_like_test() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;

        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let post = app.add_post(0, &auth_token).await.unwrap();
        debug!("wtf is that {post:#?}");

        app.check_post_like(0, &auth_token, post.id.clone(), false)
            .await
            .unwrap();
        app.add_post_like(0, &auth_token, post.id.clone())
            .await
            .unwrap();
        app.check_post_like(0, &auth_token, post.id.clone(), true)
            .await
            .unwrap();
        app.add_post_like_err_already_liked(0, &auth_token, post.id.clone())
            .await
            .unwrap();
        app.add_post_like_err_not_found(0, &auth_token, "none")
            .await
            .unwrap();
        app.delete_post_like(0, &auth_token, post.id.clone())
            .await
            .unwrap();
        app.check_post_like(0, &auth_token, post.id.clone(), false)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn api_post_test() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;
        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        app.add_post(0, &auth_token).await.unwrap();
        app.expect_posts(0, 0, 1, 0, 1).await.unwrap();
        app.add_post(1, &auth_token).await.unwrap();
        app.expect_posts(0, 1, 2, 0, 1).await.unwrap();
        app.expect_posts(1, 0, 1, 1, 2).await.unwrap();
        app.expect_posts(2, 0, 0, 2, 2).await.unwrap();
    }

    // #[test(tokio::test)]
    // #[tokio::test]
    // async fn full_api_test() {
    //     let file = rolling::daily("./logs", "log");
    //     let _ = tracing_subscriber::fmt()
    //         .event_format(
    //             tracing_subscriber::fmt::format()
    //                 .with_file(true)
    //                 .with_line_number(true),
    //         )
    //         .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    //         // .with_writer(file)
    //         .try_init();
    //
    //     let time_mut = Arc::new(Mutex::new(1));
    //     let app_state = AppState::new_testng(time_mut.clone(), 1).await;
    //     let my_app = create_api_router(app_state.clone()).with_state(app_state.clone());
    //
    //     let time = app_state.clock.now().await;
    //     let secret = app_state.get_secret().await;
    //
    //     let server = TestServer::builder()
    //         .http_transport()
    //         .build(my_app)
    //         .unwrap();
    //
    //     let api = ApiTest::new(server);
    //
    //     let mut imgbuf = image::ImageBuffer::new(250, 250);
    //     // Iterate over the coordinates and pixels of the image
    //     for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
    //         let r = (0.3 * x as f32) as u8;
    //         let b = (0.3 * y as f32) as u8;
    //         *pixel = image::Rgb([r, 0, b]);
    //     }
    //
    //     let path = "../target/tmp/img.png";
    //     imgbuf.save(path).unwrap();
    //     let img = tokio::fs::read(path).await.unwrap();
    //
    //     let result = api
    //         .send_email_invite("hey1@hey.com")
    //         .send_native()
    //         .await
    //         .unwrap();
    //     trace!("{result:#?}");
    //
    //     let invite = app_state
    //         .db
    //         .get_invite_valid(
    //             time,
    //             // DBEmailTokenKind::RequestConfirmRegistrationEmail,
    //             "hey1@hey.com",
    //             // 0,
    //         )
    //         .await
    //         .unwrap();
    //
    //     trace!("good invite {invite:#?}");
    //
    //     let bad_invite_token =
    //         encode_token(&secret, EmailToken::new("123", "hey1@hey.com", time)).unwrap();
    //
    //     let bad_invite = app_state
    //         .db
    //         .add_invite(
    //             time,
    //             // DBEmailTokenKind::RequestConfirmRegistrationEmail,
    //             &bad_invite_token,
    //             "hey1@hey.com",
    //             time + 1,
    //         )
    //         .await
    //         .unwrap();
    //     trace!("bad invite added: {bad_invite:#?}");
    //
    //     {
    //         *time_mut.lock().await = Duration::from_secs(10).as_nanos();
    //         let result = api
    //             .register("hey", &invite.token_raw, "*wowowowwoW12222pp")
    //             .send_native()
    //             .await;
    //
    //         assert!(matches!(
    //             result,
    //             Err(ServerErr::RegistrationErr(
    //                 ServerRegistrationErr::TokenExpired
    //             ))
    //         ));
    //         *time_mut.lock().await = Duration::from_nanos(0).as_nanos();
    //         // match result {
    //         //      => {
    //         //         assert!(username.is_some());
    //         //         assert!(email.is_none());
    //         //         assert!(password.is_some());
    //         //     }
    //         //     etc => panic!("expexted register err, got: {etc:?}"),
    //         // }
    //     }
    //     {
    //         let result = api
    //             .register("he", &invite.token_raw, "wowowowwoW12222pp")
    //             .send_native()
    //             .await;
    //
    //         match result {
    //             Err(ServerErr::RegistrationErr(
    //                 ServerRegistrationErr::ServerRegistrationInvalidInput {
    //                     username,
    //                     email,
    //                     password,
    //                 },
    //             )) => {
    //                 assert!(username.is_some());
    //                 assert!(email.is_none());
    //                 assert!(password.is_some());
    //             }
    //             etc => panic!("expexted register err, got: {etc:?}"),
    //         }
    //     }
    //
    //     let (token, decoded_token, result) = api
    //         .register("hey", &invite.token_raw, "wowowowwoW12222pp*")
    //         .send_native_and_extract_auth(&secret)
    //         .await;
    //
    //     let result = api
    //         .send_email_invite("hey1@hey.com")
    //         .send_native()
    //         .await
    //         .unwrap();
    //     assert_eq!(result, ServerRes::Ok);
    //
    //     let token_raw = token.unwrap();
    //
    //     let all_invites = app_state.db.get_invite_all().await.unwrap();
    //
    //     trace!("all invites: {all_invites:#?}");
    //
    //     {
    //         let result = api
    //             .register("he", &bad_invite_token, "wowowowwoW12222pp")
    //             .send_native()
    //             .await;
    //
    //         assert!(matches!(
    //             result,
    //             Err(ServerErr::RegistrationErr(
    //                 ServerRegistrationErr::TokenNotFound,
    //             ))
    //         ));
    //     }
    //
    //     let all_users = app_state.db.get_all_user().await.unwrap();
    //     let all_sessions = app_state.db.get_session_all().await.unwrap();
    //
    //     trace!("ALL USERS {all_users:#?}");
    //     assert!(all_users.len() == 1);
    //
    //     trace!("ALL SESSIONS {all_sessions:#?}");
    //     assert!(all_users.len() == 1);
    //
    //     trace!("{token_raw:#?}");
    //
    //     let result = api
    //         .logout()
    //         .send_native_with_token(&token_raw)
    //         .await
    //         .unwrap();
    //
    //     assert_eq!(result, ServerRes::Ok);
    //
    //     let result = api
    //         .login("hey1@hey.com3", "wowowowwoW12222pp*")
    //         .send_native()
    //         .await;
    //
    //     assert!(matches!(
    //         result,
    //         Err(ServerErr::LoginErr(ServerLoginErr::WrongCredentials))
    //     ));
    //
    //     let (token, decoded_token, result) = api
    //         .login("hey1@hey.com", "wowowowwoW12222pp*")
    //         .send_native_and_extract_auth(&secret)
    //         .await;
    //
    //     let token_raw = token.unwrap();
    //
    //     let result = api
    //         .add_post(
    //             "title1",
    //             "wow",
    //             Vec::from([ServerReqImg {
    //                 path: path.to_string(),
    //                 data: img.clone(),
    //             }]),
    //         )
    //         .send_native_with_token(token_raw.clone())
    //         .await
    //         .unwrap();
    //     trace!("{result:#?}");
    //
    //     let result = api.get_posts_older(2, 25).send_native().await.unwrap();
    //     match result {
    //         crate::api::ServerRes::Posts(posts) => {
    //             assert!(posts.len() == 1);
    //         }
    //         wrong => {
    //             panic!("{}", format!("expected posts, got {:?}", wrong));
    //         }
    //     }
    //
    //     let result = api.get_posts_older(1, 25).send_native().await.unwrap();
    //     match result {
    //         crate::api::ServerRes::Posts(posts) => {
    //             assert!(posts.len() == 0);
    //         }
    //         wrong => {
    //             panic!("{}", format!("expected posts, got {:?}", wrong));
    //         }
    //     }
    //
    //     *time_mut.lock().await = Duration::from_nanos(2).as_nanos();
    //
    //     let result = api
    //         .add_post(
    //             "title2",
    //             "wow",
    //             Vec::from([ServerReqImg {
    //                 path: path.to_string(),
    //                 data: img.clone(),
    //             }]),
    //         )
    //         .send_native_with_token(token_raw.clone())
    //         .await
    //         .unwrap();
    //
    //     *time_mut.lock().await = Duration::from_nanos(3).as_nanos();
    //
    //     let result = api
    //         .add_post(
    //             "title3",
    //             "wow",
    //             Vec::from([ServerReqImg {
    //                 path: path.to_string(),
    //                 data: img.clone(),
    //             }]),
    //         )
    //         .send_native_with_token(token_raw.clone())
    //         .await
    //         .unwrap();
    //
    //     let result = api.get_posts_older(2, 25).send_native().await.unwrap();
    //     match result {
    //         crate::api::ServerRes::Posts(posts) => {
    //             assert!(posts.len() == 1);
    //             assert_eq!(posts[0].created_at, 1);
    //         }
    //         wrong => {
    //             panic!("{}", format!("expected posts, got {:?}", wrong));
    //         }
    //     }
    //
    //     let result = api.get_posts_newer(2, 25).send_native().await.unwrap();
    //     match result {
    //         crate::api::ServerRes::Posts(posts) => {
    //             assert!(posts.len() == 1);
    //             assert_eq!(posts[0].created_at, 3);
    //         }
    //         wrong => {
    //             panic!("{}", format!("expected posts, got {:?}", wrong));
    //         }
    //     }
    //
    //     *time_mut.lock().await = Duration::from_nanos(4).as_nanos();
    //
    //     let result = api
    //         .add_post(
    //             "title4",
    //             "wow",
    //             Vec::from([ServerReqImg {
    //                 path: path.to_string(),
    //                 data: img.clone(),
    //             }]),
    //         )
    //         .send_native_with_token(token_raw.clone())
    //         .await
    //         .unwrap();
    //
    //     let result = api.get_posts_newer(2, 25).send_native().await.unwrap();
    //     match result {
    //         crate::api::ServerRes::Posts(posts) => {
    //             assert!(posts.len() == 2);
    //             assert_eq!(posts[0].created_at, 4);
    //             assert_eq!(posts[1].created_at, 3);
    //         }
    //         wrong => {
    //             panic!("{}", format!("expected posts, got {:?}", wrong));
    //         }
    //     }
    //
    //     let result = api.get_posts_older(3, 25).send_native().await.unwrap();
    //     match result {
    //         crate::api::ServerRes::Posts(posts) => {
    //             assert!(posts.len() == 2);
    //             assert_eq!(posts[0].created_at, 2);
    //             assert_eq!(posts[1].created_at, 1);
    //         }
    //         wrong => {
    //             panic!("{}", format!("expected posts, got {:?}", wrong));
    //         }
    //     }
    //
    //     let result = api.get_posts_newer(2, 1).send_native().await.unwrap();
    //     match result {
    //         crate::api::ServerRes::Posts(posts) => {
    //             assert!(posts.len() == 1);
    //             assert_eq!(posts[0].created_at, 3);
    //         }
    //         wrong => {
    //             panic!("{}", format!("expected posts, got {:?}", wrong));
    //         }
    //     }
    //
    //     let result = api
    //         .get_user_posts_newer(2, 10, "hey")
    //         .send_native()
    //         .await
    //         .unwrap();
    //     match result {
    //         crate::api::ServerRes::Posts(posts) => {
    //             assert!(posts.len() == 2);
    //         }
    //         wrong => {
    //             panic!("{}", format!("expected posts, got {:?}", wrong));
    //         }
    //     }
    //
    //     let result = api
    //         .get_user_posts_newer_or_equal(2, 10, "hey")
    //         .send_native()
    //         .await
    //         .unwrap();
    //     match result {
    //         crate::api::ServerRes::Posts(posts) => {
    //             assert_eq!(posts.len(), 3);
    //         }
    //         wrong => {
    //             panic!("{}", format!("expected posts, got {:?}", wrong));
    //         }
    //     }
    //
    //     let result = api
    //         .get_user_posts_older(2, 10, "hey")
    //         .send_native()
    //         .await
    //         .unwrap();
    //     match result {
    //         crate::api::ServerRes::Posts(posts) => {
    //             assert_eq!(posts.len(), 1);
    //         }
    //         wrong => {
    //             panic!("{}", format!("expected posts, got {:?}", wrong));
    //         }
    //     }
    //
    //     let result = api
    //         .get_user_posts_older_or_equal(2, 10, "hey")
    //         .send_native()
    //         .await
    //         .unwrap();
    //     match result {
    //         crate::api::ServerRes::Posts(posts) => {
    //             assert_eq!(posts.len(), 2);
    //         }
    //         wrong => {
    //             panic!("{}", format!("expected posts, got {:?}", wrong));
    //         }
    //     }
    //
    //     let result = api
    //         .send_email_invite("hey2@hey.com")
    //         .send_native()
    //         .await
    //         .unwrap();
    //     trace!("{result:#?}");
    //
    //     *time_mut.lock().await = Duration::from_nanos(5).as_nanos();
    //
    //     let invite2 = app_state
    //         .db
    //         .get_invite_valid(
    //             *time_mut.lock().await,
    //             // DBEmailTokenKind::RequestConfirmRegistrationEmail,
    //             "hey2@hey.com",
    //             // 0,
    //         )
    //         .await
    //         .unwrap();
    //
    //     let (token2, decoded_token2, result2) = api
    //         .register("hey2", &invite2.token_raw, "wowowowwoW12222pp*")
    //         .send_native_and_extract_auth(&secret)
    //         .await;
    //
    //     let token_raw2 = token2.unwrap();
    //
    //     *time_mut.lock().await = Duration::from_nanos(6).as_nanos();
    //
    //     let result = api
    //         .add_post(
    //             "420",
    //             "wow",
    //             Vec::from([ServerReqImg {
    //                 path: path.to_string(),
    //                 data: img.clone(),
    //             }]),
    //         )
    //         .send_native_with_token(token_raw2.clone())
    //         .await
    //         .unwrap();
    //
    //     let result = api
    //         .get_user_posts_newer(2, 10, "hey2")
    //         .send_native()
    //         .await
    //         .unwrap();
    //     match result {
    //         crate::api::ServerRes::Posts(posts) => {
    //             assert_eq!(posts.len(), 1);
    //         }
    //         wrong => {
    //             panic!("{}", format!("expected posts, got {:?}", wrong));
    //         }
    //     }
    //
    //     let result = api
    //         .get_user_posts_older(7, 10, "hey2")
    //         .send_native()
    //         .await
    //         .unwrap();
    //     match result {
    //         crate::api::ServerRes::Posts(posts) => {
    //             assert_eq!(posts.len(), 1);
    //         }
    //         wrong => {
    //             panic!("{}", format!("expected posts, got {:?}", wrong));
    //         }
    //     }
    //
    //     let result = api.get_user_posts_newer(2, 10, "hey99").send_native().await;
    //     assert!(matches!(
    //         result,
    //         Err(ServerErr::GetErr(ServerGetErr::NotFound))
    //     ));
    //
    //     let result = api
    //         .get_user_posts_newer_or_equal(2, 10, "hey99")
    //         .send_native()
    //         .await;
    //     assert!(matches!(
    //         result,
    //         Err(ServerErr::GetErr(ServerGetErr::NotFound))
    //     ));
    //
    //     let result = api
    //         .change_username("wowowowwoW12222pp*", "bye")
    //         .send_native_with_token(token_raw.clone())
    //         .await
    //         .unwrap();
    //     match result {
    //         crate::api::ServerRes::User { username } => {
    //             assert_eq!(username, "bye");
    //         }
    //         wrong => {
    //             panic!("{}", format!("expected User, got {:?}", wrong));
    //         }
    //     }
    //
    //     let result = api
    //         .profile()
    //         .send_native_with_token(token_raw.clone())
    //         .await
    //         .unwrap();
    //
    //     match result {
    //         crate::api::ServerRes::Acc { username, email } => {
    //             assert_eq!(username, "bye");
    //         }
    //         wrong => {
    //             panic!("{}", format!("expected Acc, got {:?}", wrong));
    //         }
    //     }
    // }
}
