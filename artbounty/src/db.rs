use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;

pub use surrealdb::Connection;
use surrealdb::engine::local::SurrealKv;
use surrealdb::engine::local::{self, Mem};
use surrealdb::{RecordId, RecordIdKey};
use surrealdb::{Surreal, opt::IntoEndpoint};
use thiserror::Error;
use tracing::{error, trace};

use crate::db::post::create_post_id;

// pub static DB: LazyLock<Db<local::Db>> = LazyLock::new(Db::init);
// derive_alias! {
//     #[derive(Save!)] = #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)];
// }

pub type DbEngine = Db<local::Db>;
pub async fn new_local(time: u128, path: impl AsRef<str>) -> Db<local::Db> {
    let db = Db::<local::Db>::new::<SurrealKv>(path.as_ref())
        .await
        .unwrap();
    db.connect().await;
    db.migrate(time).await.unwrap();

    db
}

pub async fn new_mem(time: u128) -> Db<local::Db> {
    let db = Db::<local::Db>::new::<Mem>(()).await.unwrap();
    db.connect().await;
    db.migrate(time).await.unwrap();

    db
}

// pub fn surreal_time_from_duration(time: Duration) -> surrealdb::Datetime {
//     surrealdb::Datetime::from(chrono::DateTime::from_timestamp_nanos(
//         time.as_nanos() as i64
//     ))
// }

pub trait SurrealCheckUtils {
    fn check_good<ERR: std::error::Error + From<surrealdb::Error>>(
        self,
        f: impl FnOnce(surrealdb::Error) -> ERR,
    ) -> Result<surrealdb::Response, ERR>;
}

pub trait SurrealSerializeUtils<ERR: std::error::Error + From<surrealdb::Error>> {
    fn and_then_take_all<Value: serde::de::DeserializeOwned + std::fmt::Debug>(
        self,
        index: usize,
    ) -> Result<Vec<Value>, ERR>;
    fn and_then_take_or<Value: serde::de::DeserializeOwned + std::fmt::Debug>(
        self,
        index: usize,
        err: ERR,
    ) -> Result<Value, ERR>;
    fn and_then_take_expect<Value: serde::de::DeserializeOwned + std::fmt::Debug>(
        self,
        index: usize,
    ) -> Result<Value, ERR>;
}

impl SurrealCheckUtils for Result<surrealdb::Response, surrealdb::Error> {
    fn check_good<ERR: std::error::Error + From<surrealdb::Error>>(
        self,
        f: impl FnOnce(surrealdb::Error) -> ERR,
    ) -> Result<surrealdb::Response, ERR> {
        self.inspect_err(|err| error!("db error: {err}"))
            .inspect(|e| trace!("result {e:#?}"))?
            .check()
            .inspect_err(|err| error!("db check error: {err}"))
            .map_err(f)
    }
}

impl<ERR: std::error::Error + From<surrealdb::Error>> SurrealSerializeUtils<ERR>
    for Result<surrealdb::Response, ERR>
{
    fn and_then_take_all<Value: serde::de::DeserializeOwned + std::fmt::Debug>(
        self,
        index: usize,
    ) -> Result<Vec<Value>, ERR> {
        self.and_then(|mut result| {
            result
                .take::<Vec<Value>>(index)
                .inspect_err(|err| error!("unexpected err {err}"))
                .inspect(|v| trace!("db serialized to: {v:#?}"))
                .map_err(ERR::from)
        })
    }

    fn and_then_take_or<Value: serde::de::DeserializeOwned + std::fmt::Debug>(
        self,
        index: usize,
        err: ERR,
    ) -> Result<Value, ERR> {
        self.and_then(|mut result| {
            result
                .take::<Option<Value>>(index)
                .inspect_err(|err| error!("unexpected err {err}"))
                .inspect(|v| trace!("db serialized to: {v:#?}"))
                .map_err(ERR::from)
                .and_then(|v| v.ok_or(err))
        })
    }

    fn and_then_take_expect<Value: serde::de::DeserializeOwned + std::fmt::Debug>(
        self,
        index: usize,
    ) -> Result<Value, ERR> {
        self.and_then(|mut result| {
            result
                .take::<Option<Value>>(index)
                .inspect_err(|err| error!("unexpected err {err}"))
                .inspect(|v| trace!("db serialized to: {v:#?}"))
                .map_err(ERR::from)
                .map(|v| v.expect("must exist"))
        })
    }
}

pub trait SurrealErrUtils {
    fn index_exists(&self, index_name: impl AsRef<str>) -> bool;
    fn field_value_null(&self, field_name: impl AsRef<str>) -> bool;
}

impl SurrealErrUtils for surrealdb::Error {
    fn index_exists(&self, index_name: impl AsRef<str>) -> bool {
        match self {
            surrealdb::Error::Db(surrealdb::error::Db::IndexExists { index, value, .. })
                if index == index_name.as_ref() =>
            {
                true
            }

            _ => false,
        }
    }

    fn field_value_null(&self, field_name: impl AsRef<str>) -> bool {
        match self {
            surrealdb::Error::Db(surrealdb::error::Db::FieldCheck {
                thing,
                value,
                field,
                check,
            }) if value == "NULL"
                || value == "NONE"
                    && field
                        .first()
                        .map(|f| f.to_string())
                        .inspect(|f| trace!("field: {f}"))
                        .map(|f| &f[1..] == field_name.as_ref())
                        .unwrap_or_default() =>
            {
                true
            }

            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Db<C: Connection> {
    pub db: Surreal<C>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DBUser {
    pub id: RecordId,
    pub username: String,
    pub email: String,
    pub password: String,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DBUserPost {
    pub id: RecordId,
    pub user: DBUser,
    pub show: bool,
    pub title: String,
    pub description: String,
    pub favorites: u64,
    pub file: Vec<DBUserPostFile>,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DBUserPostFile {
    pub extension: String,
    pub hash: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DBInvite {
    pub id: RecordId,
    pub token_raw: String,
    // pub kind: String,
    pub email: String,
    pub expires: u128,
    pub used: bool,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DBSentEmail {
    pub id: RecordId,
    pub body: String,
    pub to_email: String,
    pub reason: String,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DBSentEmailReason {
    ConfirmPasswordChange,
    ConfirmEmailChange,
    ConfirmEmailChangeNewEmail,
}

impl Display for DBSentEmailReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            DBSentEmailReason::ConfirmPasswordChange => "confirm_password_change",
            DBSentEmailReason::ConfirmEmailChange => "confirm_email_change",
            DBSentEmailReason::ConfirmEmailChangeNewEmail => "confirm_email_change_new_email",
        };

        write!(f, "{}", text)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DBSession {
    pub id: RecordId,
    pub access_token: String,
    pub user: DBUser,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Debug, Error)]
pub enum AddPostErr {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),

    #[error("user \"{0}\" not found")]
    UserNotFound(String),
}

#[derive(Debug, Error)]
pub enum AddUserErr {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),

    #[error("email {0} is taken")]
    EmailIsTaken(String),

    #[error("username {0} is taken")]
    UsernameIsTaken(String),
}

#[derive(Debug, Error)]
pub enum GetAllUsers {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),
}

#[derive(Debug, Error)]
pub enum AddSessionErr {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),

    #[error("user \"{0}\" not found")]
    UserNotFound(String),

    #[error("token already exists")]
    TokenExists,
}

#[derive(Debug, Error)]
pub enum DB404Err {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),

    #[error("user not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum DBPostLikeErr {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),

    #[error("post was already liked")]
    PostWasAlreadyLiked,

    #[error("post \"{0}\" was not found")]
    PostNotFound(String),
}

#[derive(Debug, Error)]
pub enum EmailIsTakenErr {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),

    #[error("account with \"{0}\" email already exists")]
    EmailIsTaken(String),
}

#[derive(Debug, Error)]
pub enum DBChangeUsernameErr {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),

    #[error("username {0} is taken")]
    UsernameIsTaken(String),

    #[error("user not found")]
    NotFound,
}

impl Db<local::Db> {
    pub async fn connect(&self) {
        // TODO make path as env
        let db = &self.db;
        // db.connect::<SurrealKv>("db5").await.unwrap();
        db.use_ns("artbounty").use_db("web").await.unwrap();
    }
}

pub fn create_user_id(id: impl Into<String>) -> RecordId {
    RecordId::from_table_key("user", id.into())
}

pub mod post_like {
    use crate::db::DB404Err;
    use crate::db::DBPostLikeErr;
    use crate::db::DBUser;
    use crate::db::DBUserPost;
    use crate::db::SurrealCheckUtils;
    use crate::db::SurrealErrUtils;
    use crate::db::SurrealSerializeUtils;
    use crate::db::post::create_post_id;

    use super::Db;
    pub use surrealdb::Connection;
    use surrealdb::RecordId;
    use surrealdb::RecordIdKey;

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct DBPostLike {
        pub id: RecordId,
        pub user: RecordId,
        pub post: RecordId,
        pub modified_at: u128,
        pub created_at: u128,
    }

    pub fn create_post_like_id(id: impl Into<RecordIdKey>) -> RecordId {
        RecordId::from_table_key("post_like", id)
    }

    impl<C: Connection> Db<C> {
        pub async fn add_post_like(
            &self,
            time: u128,
            user_id: RecordId,
            post_id: impl Into<RecordIdKey>,
        ) -> Result<DBPostLike, DBPostLikeErr> {
            let post_id = post_id.into();
            self.db
                .query(
                    r#"
                 LET $post = SELECT id FROM ONLY $post_id;
                 CREATE post_like SET
                    user = $user_id,
                    post = $post.id,
                    modified_at = $time,
                    created_at = $time
                 RETURN *;
                "#,
                )
                .bind(("time", time))
                .bind(("user_id", user_id))
                .bind(("post_id", create_post_id(post_id.clone())))
                .await
                .check_good(|err| match err {
                    err if err.index_exists("idx_user_post") => DBPostLikeErr::PostWasAlreadyLiked,
                    err if err.field_value_null("post") => {
                        DBPostLikeErr::PostNotFound(post_id.to_string())
                    }
                    err => err.into(),
                })
                .and_then_take_expect(1)
        }

        //
        pub async fn delete_post_like(
            &self,
            user: RecordId,
            post_id: impl Into<RecordIdKey>,
        ) -> Result<(), surrealdb::Error> {
            self.db
                .query(
                    r#"
                        DELETE post_like WHERE
                            user = $user_id AND
                            post = $post_id;
                    "#,
                )
                .bind(("user_id", user))
                .bind(("post_id", create_post_id(post_id.into())))
                .await
                .check_good(surrealdb::Error::from)
                // .check_good(Surreal::from)
                .map(|_| ())
            // .and_then_take_or(0, DB404Err::NotFound)
        }

        pub async fn get_post_like(
            &self,
            time: u128,
            user: RecordId,
            post: RecordId,
        ) -> Result<DBPostLike, DB404Err> {
            self.db
                .query(
                    r#"
                        SELECT * FROM ONLY post_like WHERE
                                user = $user_user AND
                                post = $user_post
                    "#,
                )
                .bind(("time", time))
                .bind(("user_user", user))
                .bind(("user_post", post))
                .await
                .check_good(DB404Err::from)
                .and_then_take_or(0, DB404Err::NotFound)
        }

        pub async fn check_post_like(
            &self,
            time: u128,
            user: RecordId,
            post_id: impl Into<RecordIdKey>,
        ) -> Result<RecordId, DB404Err> {
            let post_id = post_id.into();
            self.db
                .query(
                    r#"
                        LET $result = SELECT id FROM ONLY post_like WHERE
                                user = $user_id AND
                                post = $post_id;
                        return $result.id;
                    "#,
                )
                .bind(("time", time))
                .bind(("user_id", user))
                .bind(("post_id", create_post_id(post_id.clone())))
                .await
                .check_good(DB404Err::from)
                .and_then_take_or(1, DB404Err::NotFound)
        }
    }

    #[cfg(test)]
    mod tests {
        use std::time::Duration;

        // use pretty_assertions::assert_eq;
        use surrealdb::{RecordId, engine::local::Mem};
        // use test_log::test;
        use tracing::trace;

        use crate::{
            api::{ChangeUsernameErr, ServerRes},
            db::{
                AddSessionErr, AddUserErr, DB404Err, DBChangeUsernameErr, DBPostLikeErr,
                DBSentEmailReason, DBUserPostFile, Db, EmailIsTakenErr,
                post_like::create_post_like_id,
            },
        };

        #[tokio::test]
        async fn db_post_like() {
            crate::init_test_log();
            let db = Db::new::<Mem>(()).await.unwrap();
            db.migrate(0).await.unwrap();

            let user = db.add_user(0, "hey1", "hey1@hey.com", "123").await.unwrap();
            let post = db
                .add_post(0, "hey1", "title", "description", 0, vec![])
                .await
                .unwrap();

            // TODO add more failure tests

            // let result = db.add_post_like(0, user.id.clone(), post.id.clone()).await;
            let result = db.add_post_like(0, user.id.clone(), "wtf").await;
            assert!(matches!(result, Err(DBPostLikeErr::PostNotFound(_))));

            let result = db.delete_post_like(user.id.clone(), "wtf").await;
            assert!(result.is_ok());

            let result = db
                .add_post_like(0, user.id.clone(), post.id.key().clone())
                .await;
            assert!(result.is_ok());

            let result = db
                .delete_post_like(user.id.clone(), post.id.key().clone())
                .await;
            assert!(result.is_ok());

            let result = db
                .add_post_like(0, user.id.clone(), post.id.key().clone())
                .await;
            assert!(result.is_ok());

            let result = db
                .add_post_like(0, user.id.clone(), post.id.key().clone())
                .await;
            assert!(matches!(result, Err(DBPostLikeErr::PostWasAlreadyLiked)));

            // let result = db.add_post_like(0, user.id.clone(), post.id.clone()).await;
            // assert!(matches!(result, Err(DBPostLikeErr::PostWasAlreadyLiked)));

            let result = db.get_post_like(0, user.id.clone(), post.id.clone()).await;
            assert!(result.is_ok());

            let result = db
                .check_post_like(0, user.id.clone(), post.id.key().clone())
                .await;
            assert!(result.is_ok());

            let result = db.check_post_like(0, user.id.clone(), "none").await;
            assert!(matches!(result, Err(DB404Err::NotFound)));
        }
    }
}

pub mod confirm_email {
    use crate::db::DB404Err;
    use crate::db::SurrealCheckUtils;
    use crate::db::SurrealSerializeUtils;

    use super::Db;
    pub use surrealdb::Connection;
    use surrealdb::RecordId;
    use surrealdb::RecordIdKey;

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct DBConfirmEmail {
        pub id: RecordId,
        pub to_email: String,
        // pub token: String,
        pub completed: bool,
        pub expires: u128,
        pub modified_at: u128,
        pub created_at: u128,
    }

    // #[derive(Save!)]
    // pub enum DBSalvation {
    //     Fox,
    //     Bunny {
    //         honey: String,
    //     },
    //     Funny(usize),
    // }
    //
    // fn from_evil<S: Serializer>(v: &DBSalvation, serializer: S) -> Result<S::Ok, S::Error> {
    //     let v: [(&'de str, Option<usize>, String)] =  match v {
    //         DBSalvation::Fox => ["fox", None::<usize>, String::new()],
    //         DBSalvation::Bunny { honey } => ["bunny", None::<usize>, honey.clone()],
    //         DBSalvation::Funny(v) => ["funny", Some(*v), String::new()],
    //     };
    //     v.serialize(serializer)
    // }
    //
    // fn to_evil<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DBSalvation, D::Error> {
    //     <[(&'de str, Option<usize>, String)]>::deserialize(deserializer).map(|v| match v {
    //         ("fox", None, _) => DBSalvation::Fox,
    //         ("bunny", None, v) => DBSalvation::Bunny { honey: v },
    //         ("funny", Some(n), _) => DBSalvation::Funny(n),
    //         _ => unreachable!("cant happen")
    //     })
    //
    //     // Option::<String>::deserialize(deserializer)
    //     //     .and_then(|str| Ok(str.and_then(|str| Some(Evil { two: str }))))
    // }

    impl<C: Connection> Db<C> {
        pub async fn add_confirm_email(
            &self,
            time: u128,
            to_email: impl Into<String>,
            // token: impl Into<String>,
            expires: u128,
        ) -> Result<DBConfirmEmail, surrealdb::Error> {
            self.db
                .query(
                    r#"
                 CREATE confirm_email SET
                    to_email = $to_email,
                    completed = false,
                    expires = $exp,
                    modified_at = $time,
                    created_at = $time;
                "#,
                )
                .bind(("time", time))
                .bind(("exp", expires))
                .bind(("to_email", to_email.into()))
                // .bind(("email_token", token.into()))
                .await
                // .check_good(|err| match err {
                //     err if err.index_exists("idx_user_email") => AddUserErr::EmailIsTaken(email),
                //     err if err.index_exists("idx_user_username") => {
                //         AddUserErr::UsernameIsTaken(username)
                //     }
                //     err => err.into(),
                // })
                .and_then_take_expect(0)
        }

        pub async fn update_confirm_email_by_key(
            &self,
            time: u128,
            confirm_email_key: impl Into<RecordIdKey>,
        ) -> Result<DBConfirmEmail, DB404Err> {
            let id = RecordId::from_table_key("confirm_email", confirm_email_key);
            // let a = id.key().to_string()
            self.db
                .query(
                    "UPDATE confirm_email SET modified_at = $time, completed = true WHERE id = $confirm_email_id AND completed = false AND expires >= $time",
                )
                .bind(("confirm_email_id", id))
                .bind(("time", time))
                .await
                .check_good(DB404Err::from)
                .and_then_take_or(0, DB404Err::NotFound)
        }
        pub async fn get_confirm_email_by_key(
            &self,
            time: u128,
            confirm_email_key: impl Into<RecordIdKey>,
        ) -> Result<DBConfirmEmail, DB404Err> {
            let id = RecordId::from_table_key("confirm_email", confirm_email_key);
            self.db
                .query(
                    r#"
                        SELECT * FROM ONLY confirm_email WHERE
                                expires >= $time AND
                                completed = false AND
                                id = $confirm_email_id 
                    "#,
                )
                .bind(("time", time))
                .bind(("confirm_email_id", id))
                .await
                .check_good(DB404Err::from)
                .and_then_take_or(0, DB404Err::NotFound)
        }

        pub async fn get_confirm_email_latest(
            &self,
            time: u128,
            email: impl Into<String>,
        ) -> Result<DBConfirmEmail, DB404Err> {
            self.db
                .query(
                    r#"
                        SELECT * FROM ONLY confirm_email WHERE
                                expires >= $time AND
                                completed = false AND
                                to_email = $email
                                ORDER BY created_at DESC
                    "#,
                )
                .bind(("time", time))
                .bind(("email", email.into()))
                .await
                .check_good(DB404Err::from)
                .and_then_take_or(0, DB404Err::NotFound)
        }
    }

    #[cfg(test)]
    mod tests {
        use std::time::Duration;

        // use pretty_assertions::assert_eq;
        use surrealdb::engine::local::Mem;
        // use test_log::test;
        use tracing::trace;

        use crate::{
            api::ChangeUsernameErr,
            db::{
                AddSessionErr, AddUserErr, DB404Err, DBChangeUsernameErr, DBSentEmailReason,
                DBUserPostFile, Db, EmailIsTakenErr,
            },
        };

        #[tokio::test]
        async fn db_confirm_email() {
            crate::init_test_log();

            let db = Db::new::<Mem>(()).await.unwrap();
            db.migrate(0).await.unwrap();

            let result = db.get_confirm_email_latest(0, "prime@heyadora.com").await;
            assert!(result.is_err());

            let result = db.add_confirm_email(0, "prime@heyadora.com", 1).await;
            assert!(result.is_ok());
            let key = result.unwrap().id.key().to_string();

            let result = db.get_confirm_email_latest(0, "prime@heyadora.com").await;
            assert!(result.is_ok());

            // must fail because cant have dublicate tokens
            // let result = db
            //     .add_confirm_email(0, "prime@heyadora.com", &key, 1)
            //     .await;
            // assert!(result.is_err());

            let result = db.get_confirm_email_by_key(0, &key).await;
            assert!(result.is_ok());

            let result = db.get_confirm_email_by_key(1, &key).await;
            assert!(result.is_ok());

            // must fail because token is expires
            let result = db.get_confirm_email_by_key(2, &key).await;
            assert!(result.is_err());

            // must fail because token is expires
            let result = db.update_confirm_email_by_key(2, &key).await;
            assert!(result.is_err());

            let result = db.update_confirm_email_by_key(0, &key).await;
            assert!(result.is_ok());

            // must fail because token is completed/used
            let result = db.get_confirm_email_by_key(1, &key).await;
            assert!(result.is_err());

            // must fail because token is completed/used
            let result = db.update_confirm_email_by_key(0, &key).await;
            assert!(result.is_err());
        }
    }
}

pub mod post {
    use surrealdb::{RecordId, RecordIdKey};

    pub fn create_post_id(id: impl Into<RecordIdKey>) -> RecordId {
        RecordId::from_table_key("post", id.into())
    }
}

pub mod email_change {
    use crate::db::DB404Err;
    use crate::db::DBUser;
    use crate::db::EmailIsTakenErr;
    use crate::db::SurrealCheckUtils;
    use crate::db::SurrealErrUtils;
    use crate::db::SurrealSerializeUtils;

    use super::Db;
    // use super::Save;
    pub use surrealdb::Connection;
    use surrealdb::RecordId;
    use surrealdb::engine::local::SurrealKv;
    use surrealdb::engine::local::{self, Mem};
    use surrealdb::{Surreal, opt::IntoEndpoint};
    use thiserror::Error;
    use tracing::{error, trace};

    pub fn create_email_change_id(id: impl Into<String>) -> RecordId {
        RecordId::from_table_key("email_change", id.into())
    }

    #[derive(Debug, Error)]
    pub enum DBChangeEmailErr {
        #[error("DB error {0}")]
        DB(#[from] surrealdb::Error),

        #[error("email \"{0}\" is taken")]
        EmailIsTaken(String),

        #[error("user not found")]
        NotFound,
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct DBEmailChange {
        pub id: RecordId,
        pub user: DBUser,
        // pub stage: DBEmailChangeStage,
        pub current: DBEmailChangeToken,
        pub new: Option<DBEmailChangeToken>,
        pub completed: bool,
        pub expires: u128,
        pub modified_at: u128,
        pub created_at: u128,
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct DBEmailChangeToken {
        pub email: String,
        pub token_raw: String,
        pub token_used: bool,
        // pub token_expires: u128,
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    pub enum DBEmailChangeStage {
        ConfirmCurrentEmail {
            email_current_token: String,
        },
        EnterNewEmail {
            email_current_token: String,
            email_new_address: String,
        },
        ConfirmNewEmail {
            email_current_token: String,
            email_new_token: String,
            email_new_address: String,
        },
        ReadyToComplete {
            email_current_token: String,
            email_new_token: String,
            email_new_address: String,
        },
        Complete {
            email_current_token: String,
            email_new_token: String,
            email_new_address: String,
        },
        Cancelled,
    }

    // #[derive(Save!)]
    // pub enum DBEmailTokenKind {
    //     RequestConfirmRegistrationEmail,
    //     RequestChangeEmail,
    //     RequestConfirmNewEmail,
    // }
    //
    // impl Display for DBEmailTokenKind {
    //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //         let name = match self {
    //             Self::RequestConfirmRegistrationEmail => "registration",
    //             Self::RequestChangeEmail => "change_email",
    //             Self::RequestConfirmNewEmail => "confirm_new_email",
    //         };
    //         write!(f, "{}", name)
    //     }
    // }

    // impl From<EmailConfirmTokenKind> for DBEmailTokenKind {
    //     fn from(value: EmailConfirmTokenKind) -> Self {
    //         match value {
    //             EmailConfirmTokenKind::ChangeEmail => DBEmailTokenKind::RequestChangeEmail,
    //         }
    //     }
    // }

    impl<C: Connection> Db<C> {
        pub async fn add_email_change(
            &self,
            time: u128,
            user: RecordId,
            user_email: impl Into<String>,
            token_raw: impl Into<String>,
            expires: u128,
            // where_used: u64,
        ) -> Result<DBEmailChange, surrealdb::Error> {
            let token_raw = token_raw.into();
            let user_email: String = user_email.into();

            self.db
                .query(
                    r#"
                CREATE email_change SET
                   user = $user,
                   current.email = $user_email,
                   current.token_raw = $token_raw,
                   current.token_used = false,
                   new = NONE,
                   completed = false,
                   expires = $expires,
                   modified_at = $time,
                   created_at = $time 
                RETURN *, user.*;
            "#,
                )
                // .bind(("kind", kind.into().to_string()))
                .bind(("user", user))
                .bind(("token_raw", token_raw))
                .bind(("user_email", user_email.clone()))
                .bind(("expires", expires))
                // .bind(("where_used", where_used))
                .bind(("time", time))
                .await
                .check_good(surrealdb::Error::from)
                .and_then_take_expect(0)
        }

        pub async fn update_email_change_confirm_current(
            &self,
            time: u128,
            email_change: RecordId,
        ) -> Result<DBEmailChange, surrealdb::Error> {
            self.db
            .query(
                r#"
                    UPDATE $email_change_id SET current.token_used = true, modified_at = $time RETURN *, user.*;
                "#,
            )
            .bind(("email_change_id", email_change))
            .bind(("time", time))
            .await
            .check_good(surrealdb::Error::from)
            .and_then_take_expect(0)
        }

        pub async fn update_email_change_add_new(
            &self,
            time: u128,
            email_change: RecordId,
            new_email: impl Into<String>,
            token_raw: impl Into<String>,
        ) -> Result<DBEmailChange, EmailIsTakenErr> {
            // let a = RecordId::from_table_key("user", "a");
            // let email_change = RecordId::from_str(email_change.as_ref())?;
            let new_email = new_email.into();
            self.db
                .query(
                    r#"
                    LET $user_email = SELECT email FROM ONLY user WHERE email = $new_email;
                    UPDATE $email_change_id SET 
                        new.email = if $user_email { null } else { $new_email },
                        new.token_raw = $token_raw,
                        new.token_used = false,
                        modified_at = $time
                    RETURN *, user.*;
                "#,
                )
                .bind(("new_email", new_email.clone()))
                .bind(("token_raw", token_raw.into()))
                .bind(("email_change_id", email_change))
                .bind(("time", time))
                .await
                .check_good(|err| match err {
                    err if err.field_value_null("new.email") => {
                        EmailIsTakenErr::EmailIsTaken(new_email)
                    }
                    err => err.into(),
                })
                .and_then_take_expect(1)
        }

        pub async fn update_email_change_confirm_new(
            &self,
            time: u128,
            email_change: RecordId,
        ) -> Result<DBEmailChange, surrealdb::Error> {
            self.db
            .query("UPDATE $email_change_id SET new.token_used = true, modified_at = $time RETURN *, user.*;",)
            .bind(("email_change_id", email_change))
            .bind(("time", time))
            .await
            .check_good(surrealdb::Error::from)
            .and_then_take_expect(0)
        }

        pub async fn update_email_change_complete(
            &self,
            time: u128,
            email_change: RecordId,
        ) -> Result<DBEmailChange, surrealdb::Error> {
            self.db
            .query("UPDATE $email_change_id SET completed = true, modified_at = $time RETURN *, user.*;",)
            .bind(("email_change_id", email_change))
            .bind(("time", time))
            .await
            .check_good(surrealdb::Error::from)
            .and_then_take_expect(0)
        }

        pub async fn update_user_email(
            &self,
            user: RecordId,
            new_email: impl Into<String>,
            time: u128,
        ) -> Result<DBUser, DBChangeEmailErr> {
            let new_email = new_email.into();
            self.db
                .query(
                    "UPDATE user SET modified_at = $time, email = $new_email WHERE id = $user_id;",
                )
                .bind(("user_id", user))
                .bind(("new_email", new_email.clone()))
                .bind(("time", time))
                .await
                .check_good(|err| match err {
                    err if err.index_exists("idx_user_email") => {
                        DBChangeEmailErr::EmailIsTaken(new_email)
                    }
                    err => err.into(),
                })
                .and_then_take_or(0, DBChangeEmailErr::NotFound)
        }

        pub async fn get_email_change_all(&self) -> Result<Vec<DBEmailChange>, surrealdb::Error> {
            self.db
                .query("SELECT *, user.* FROM email_change")
                .await
                .check_good(surrealdb::Error::from)
                .and_then_take_all(0)
        }

        pub async fn get_email_change(
            &self,
            time: u128,
            email_change: RecordId,
        ) -> Result<DBEmailChange, DB404Err> {
            self.db
                .query(
                    r#"
                    SELECT *, user.* FROM ONLY $email_change_id;
                "#,
                )
                .bind(("email_change_id", email_change))
                .bind(("time", time))
                .await
                .check_good(DB404Err::from)
                .and_then_take_or(0, DB404Err::NotFound)
        }

        pub async fn get_email_change_by_current_token(
            &self,
            time: u128,
            user: RecordId,
            token_raw: impl Into<String>,
        ) -> Result<DBEmailChange, DB404Err> {
            self.db
                .query(
                    r#"
                    SELECT *, user.* FROM ONLY email_change WHERE 
                                user = $user_id AND
                                expires >= $time AND
                                completed = false AND
                                current.token_raw = $token_raw 
                                ORDER BY created_at DESC;
                "#,
                )
                .bind(("token_raw", token_raw.into()))
                .bind(("user_id", user))
                .bind(("time", time))
                .await
                .check_good(DB404Err::from)
                .and_then_take_or(0, DB404Err::NotFound)
        }

        pub async fn get_email_change_by_new_token(
            &self,
            time: u128,
            user: RecordId,
            token_raw: impl Into<String>,
        ) -> Result<DBEmailChange, DB404Err> {
            self.db
                .query(
                    r#"
                    SELECT *, user.* FROM ONLY email_change WHERE 
                                user = $user_id AND
                                expires >= $time AND
                                completed = false AND
                                new.token_raw = $token_raw 
                                ORDER BY created_at DESC;
                "#,
                )
                .bind(("token_raw", token_raw.into()))
                .bind(("user_id", user))
                .bind(("time", time))
                .await
                .check_good(DB404Err::from)
                .and_then_take_or(0, DB404Err::NotFound)
        }
    }

    #[cfg(test)]
    mod tests {
        use std::time::Duration;

        // use pretty_assertions::assert_eq;
        use surrealdb::engine::local::Mem;
        // use test_log::test;
        use tracing::trace;

        use crate::db::{DB404Err, Db, EmailIsTakenErr, email_change::DBChangeEmailErr};

        #[tokio::test]
        async fn db_email_change() {
            crate::init_test_log();

            let db = Db::new::<Mem>(()).await.unwrap();
            db.migrate(0).await.unwrap();

            let user = db.add_user(0, "hey1", "hey1@hey.com", "123").await.unwrap();
            let user_3 = db.add_user(0, "hey3", "hey3@hey.com", "123").await.unwrap();

            let email_change = db
                .add_email_change(0, user.id.clone(), user.email.clone(), "token", 1)
                .await
                .unwrap();

            // confirm current token
            {
                let email_change = db
                    .get_email_change(0, email_change.id.clone())
                    .await
                    .unwrap();
                let result = db
                    .update_email_change_confirm_current(0, email_change.id.clone())
                    .await
                    .unwrap();
            }

            // error check: cant allow to use email that is already used by a user
            {
                let email_change = db
                    .get_email_change(0, email_change.id.clone())
                    .await
                    .unwrap();
                let result = db
                    .update_email_change_add_new(
                        0,
                        email_change.id.clone(),
                        "hey3@hey.com",
                        "token2",
                    )
                    .await;
                assert!(matches!(result, Err(EmailIsTakenErr::EmailIsTaken(_))));
            }

            // add new email stage
            {
                let email_change = db
                    .get_email_change(0, email_change.id.clone())
                    .await
                    .unwrap();
                let result = db
                    .update_email_change_add_new(
                        0,
                        email_change.id.clone(),
                        "hey2@hey.com",
                        "token2",
                    )
                    .await
                    .unwrap();
            }

            // confirm new email
            {
                let email_change = db
                    .get_email_change(0, email_change.id.clone())
                    .await
                    .unwrap();
                let result = db
                    .update_email_change_confirm_new(0, email_change.id.clone())
                    .await
                    .unwrap();
            }

            // complete
            {
                let email_change = db
                    .get_email_change(0, email_change.id.clone())
                    .await
                    .unwrap();
                let result = db
                    .update_email_change_complete(0, email_change.id.clone())
                    .await
                    .unwrap();
            }
        }

        #[tokio::test]
        async fn update_user_email() {
            let db = Db::new::<Mem>(()).await.unwrap();
            db.migrate(0).await.unwrap();

            let user = db.add_user(0, "hey1", "hey1@hey.com", "123").await.unwrap();
            let user2 = db.add_user(0, "hey3", "hey3@hey.com", "123").await.unwrap();
            let _result = db
                .update_user_email(user.id.clone(), "hey2@hey.com", 0)
                .await
                .unwrap();
            let user = db.get_user_by_email("hey2@hey.com").await.unwrap();
            assert_eq!(user.username, "hey1");
            assert_eq!(user.email, "hey2@hey.com");

            let result = db.get_user_by_email("hey1@hey.com").await;
            assert!(matches!(result, Err(DB404Err::NotFound)));

            let result = db
                .update_user_email(user.id.clone(), "hey3@hey.com", 0)
                .await;
            assert!(matches!(result, Err(DBChangeEmailErr::EmailIsTaken(_))));
        }
    }
}

impl<C: Connection> Db<C> {
    fn init() -> Self {
        let db = Surreal::<C>::init();
        Self { db }
    }
    pub async fn new<P>(
        address: impl IntoEndpoint<P, Client = C>,
    ) -> Result<Self, surrealdb::Error> {
        let db = Surreal::new(address).await?;
        db.use_ns("artbounty").use_db("web").await?;
        Ok(Self { db })
    }

    pub async fn migrate(&self, time: u128) -> Result<(), surrealdb::Error> {
        let db = &self.db;
        let result = db
                .query(
                    r#"
                    FOR $i in 0..2 {
                        LET $latest_migration = (SELECT version FROM migration ORDER BY version DESC)[0];
                        IF $latest_migration.version == 1 {
                            -- latest
                            BREAK;
                        } ELSE IF $latest_migration.version == 0 {
                            -- latest
                            BREAK;
                        } ELSE {
                            -- migration
                            DEFINE TABLE migration SCHEMAFULL;
                            DEFINE FIELD version ON TABLE migration TYPE int;
                            DEFINE FIELD modified_at ON TABLE migration TYPE number;
                            DEFINE FIELD created_at ON TABLE migration TYPE number;
                            DEFINE INDEX idx_migration_version ON TABLE migration COLUMNS version UNIQUE;
                            -- user
                            DEFINE TABLE user SCHEMAFULL;
                            DEFINE FIELD username ON TABLE user TYPE string;
                            DEFINE FIELD email ON TABLE user TYPE string;
                            DEFINE FIELD password ON TABLE user TYPE string;
                            DEFINE FIELD modified_at ON TABLE user TYPE number;
                            DEFINE FIELD created_at ON TABLE user TYPE number;
                            DEFINE INDEX idx_user_username ON TABLE user COLUMNS username UNIQUE;
                            DEFINE INDEX idx_user_email ON TABLE user COLUMNS email UNIQUE;
                            -- session
                            DEFINE TABLE session SCHEMAFULL;
                            DEFINE FIELD access_token ON TABLE session TYPE string;
                            DEFINE FIELD user ON TABLE session TYPE record<user>;
                            DEFINE FIELD modified_at ON TABLE session TYPE number;
                            DEFINE FIELD created_at ON TABLE session TYPE number;
                            DEFINE INDEX idx_session_access_token ON TABLE session COLUMNS access_token UNIQUE;
                            -- stats
                            DEFINE TABLE stat SCHEMAFULL;
                            DEFINE FIELD country ON TABLE stat TYPE string;
                            DEFINE FIELD modified_at ON TABLE stat TYPE number;
                            DEFINE FIELD created_at ON TABLE stat TYPE number;
                            DEFINE INDEX idx_stat_country ON TABLE stat COLUMNS country UNIQUE;
                            -- sent_email 
                            DEFINE TABLE sent_email SCHEMAFULL;
                            DEFINE FIELD body ON TABLE sent_email TYPE string;
                            DEFINE FIELD to_email ON TABLE sent_email TYPE string;
                            DEFINE FIELD reason ON TABLE sent_email TYPE string;
                            DEFINE FIELD modified_at ON TABLE sent_email TYPE number;
                            DEFINE FIELD created_at ON TABLE sent_email TYPE number;
                            -- invite 
                            DEFINE TABLE invite SCHEMAFULL;
                            DEFINE FIELD token_raw ON TABLE invite TYPE string;
                            -- DEFINE FIELD kind ON TABLE invite TYPE string;
                            DEFINE FIELD email ON TABLE invite TYPE string;
                            DEFINE FIELD expires ON TABLE invite TYPE number;
                            DEFINE FIELD used ON TABLE invite TYPE bool;
                            DEFINE FIELD modified_at ON TABLE invite TYPE number;
                            DEFINE FIELD created_at ON TABLE invite TYPE number;
                            DEFINE INDEX idx_invite_token_raw ON TABLE invite COLUMNS token_raw UNIQUE;

                            -- confirm email
                            DEFINE TABLE confirm_email SCHEMAFULL;
                            DEFINE FIELD to_email ON TABLE confirm_email TYPE string;
                            -- DEFINE FIELD token ON TABLE confirm_email TYPE string;
                            DEFINE FIELD completed ON TABLE confirm_email TYPE bool;
                            DEFINE FIELD expires ON TABLE confirm_email TYPE number;
                            DEFINE FIELD modified_at ON TABLE confirm_email TYPE number;
                            DEFINE FIELD created_at ON TABLE confirm_email TYPE number;

                            -- DEFINE INDEX idx_confirm_email_token ON TABLE confirm_email COLUMNS token UNIQUE;

                            -- email change
                            DEFINE TABLE email_change SCHEMAFULL;
                            DEFINE FIELD user ON TABLE email_change TYPE record<user>;
                            -- DEFINE FIELD stage ON TABLE email_change TYPE object;

                            DEFINE FIELD current ON TABLE email_change TYPE object;
                            DEFINE FIELD current.email ON TABLE email_change TYPE string;
                            DEFINE FIELD current.token_raw ON TABLE email_change TYPE string;
                            DEFINE FIELD current.token_used ON TABLE email_change TYPE bool;
                            DEFINE FIELD new ON TABLE email_change TYPE option<object>;
                            DEFINE FIELD new.email ON TABLE email_change TYPE string;
                            DEFINE FIELD new.token_raw ON TABLE email_change TYPE string;
                            DEFINE FIELD new.token_used ON TABLE email_change TYPE bool;
                            DEFINE FIELD completed ON TABLE email_change TYPE bool;

                            DEFINE FIELD expires ON TABLE email_change TYPE number;
                            DEFINE FIELD modified_at ON TABLE email_change TYPE number;
                            DEFINE FIELD created_at ON TABLE email_change TYPE number;
                            -- post 
                            DEFINE TABLE post SCHEMAFULL;
                            DEFINE FIELD user ON TABLE post TYPE record<user>;
                            DEFINE FIELD show ON TABLE post TYPE bool;
                            DEFINE FIELD title ON TABLE post TYPE string;
                            DEFINE FIELD description ON TABLE post TYPE string;
                            DEFINE FIELD favorites ON TABLE post TYPE number;
                            DEFINE FIELD file ON TABLE post TYPE array<object>;
                            DEFINE FIELD file.*.extension ON TABLE post TYPE string;
                            DEFINE FIELD file.*.hash ON TABLE post TYPE string;
                            DEFINE FIELD file.*.width ON TABLE post TYPE int;
                            DEFINE FIELD file.*.height ON TABLE post TYPE int;
                            DEFINE FIELD modified_at ON TABLE post TYPE number;
                            DEFINE FIELD created_at ON TABLE post TYPE number;
                            -- DEFINE INDEX idx_post_hash ON TABLE post COLUMNS hash UNIQUE;

                            --post like 
                            DEFINE TABLE post_like SCHEMAFULL;
                            DEFINE FIELD user ON TABLE post_like TYPE record<user>;
                            DEFINE FIELD post ON TABLE post_like TYPE record<post>;
                            DEFINE FIELD modified_at ON TABLE post_like TYPE number;
                            DEFINE FIELD created_at ON TABLE post_like TYPE number;
                            DEFINE INDEX idx_user_post ON TABLE post_like COLUMNS user, post UNIQUE;

                            CREATE migration SET version = 0, modified_at = $time, created_at = $time;
                        };
                    };

                    SELECT * FROM migration;
                "#,
                )
                .bind(("time", time))
                .await.inspect_err(|result| trace!("DB RESULT {:#?}", result) )?;
        result.check()?;
        Ok(())
    }

    // pub async fn get_post_str(&self, post_key: impl AsRef<str>) -> Result<DBUserPost, DB404Err> {
    //     let post_id = RecordId::from_str(post_key.as_ref())?;
    //     self.get_post(post_id).await
    // }

    pub async fn get_post(&self, post_key: impl Into<RecordIdKey>) -> Result<DBUserPost, DB404Err> {
        self.db
            .query("SELECT *, user.* FROM ONLY $post_id;")
            .bind(("post_id", create_post_id(post_key)))
            .await
            .check_good(DB404Err::from)
            .and_then_take_or(0, DB404Err::NotFound)
    }

    pub async fn get_post_newer_or_equal_for_user(
        &self,
        time: u128,
        limit: u32,
        user: RecordId,
    ) -> Result<Vec<DBUserPost>, surrealdb::Error> {
        self.db.query("(SELECT *, user.* FROM post WHERE created_at >= $created_at AND user = $user ORDER BY created_at ASC LIMIT $post_limit).reverse()")
            .bind(("post_limit", limit))
            .bind(("created_at", time))
            .bind(("user", user))
            .await
            .check_good(surrealdb::Error::from)
            .and_then_take_all(0)
    }

    pub async fn get_post_older_or_equal_for_user(
        &self,
        time: u128,
        limit: u32,
        user: RecordId,
    ) -> Result<Vec<DBUserPost>, surrealdb::Error> {
        self.db.query("SELECT *, user.* FROM post WHERE created_at <= $created_at AND user = $user ORDER BY created_at DESC LIMIT $post_limit")
            .bind(("post_limit", limit))
            .bind(("created_at", time))
            .bind(("user", user))
            .await
            .check_good(surrealdb::Error::from)
            .and_then_take_all(0)
    }

    pub async fn get_post_newer_for_user(
        &self,
        time: u128,
        limit: u32,
        user: RecordId,
    ) -> Result<Vec<DBUserPost>, surrealdb::Error> {
        self.db.query("(SELECT *, user.* FROM post WHERE created_at > $created_at AND user = $user ORDER BY created_at ASC LIMIT $post_limit).reverse()")
            .bind(("post_limit", limit))
            .bind(("created_at", time))
            .bind(("user", user))
            .await
            .check_good(surrealdb::Error::from)
            .and_then_take_all(0)
    }

    pub async fn get_post_older_for_user(
        &self,
        time: u128,
        limit: u32,
        user: RecordId,
    ) -> Result<Vec<DBUserPost>, surrealdb::Error> {
        self.db.query("SELECT *, user.* FROM post WHERE created_at < $created_at AND user = $user ORDER BY created_at DESC LIMIT $post_limit")
            .bind(("post_limit", limit))
            .bind(("created_at", time))
            .bind(("user", user))
            .await
            .check_good(surrealdb::Error::from)
            .and_then_take_all(0)
    }

    pub async fn get_post_newer_or_equal(
        &self,
        time: u128,
        limit: u32,
    ) -> Result<Vec<DBUserPost>, surrealdb::Error> {
        self.db.query("(SELECT *, user.* FROM post WHERE created_at >= $created_at ORDER BY created_at ASC LIMIT $post_limit).reverse()")
            .bind(("post_limit", limit))
            .bind(("created_at", time))
            .await
            .check_good(surrealdb::Error::from)
            .and_then_take_all(0)
    }

    pub async fn get_post_older_or_equal(
        &self,
        time: u128,
        limit: u32,
    ) -> Result<Vec<DBUserPost>, surrealdb::Error> {
        self.db.query("SELECT *, user.* FROM post WHERE created_at <= $created_at ORDER BY created_at DESC LIMIT $post_limit")
            .bind(("post_limit", limit))
            .bind(("created_at", time))
            .await
            .check_good(surrealdb::Error::from)
            .and_then_take_all(0)
    }

    pub async fn get_post_newer(
        &self,
        time: u128,
        limit: u32,
    ) -> Result<Vec<DBUserPost>, surrealdb::Error> {
        self.db.query("(SELECT *, user.* FROM post WHERE created_at > $created_at ORDER BY created_at ASC LIMIT $post_limit).reverse()")
            .bind(("post_limit", limit))
            .bind(("created_at", time))
            .await
            .check_good(surrealdb::Error::from)
            .and_then_take_all(0)
    }

    pub async fn get_post_older(
        &self,
        time: u128,
        limit: u32,
    ) -> Result<Vec<DBUserPost>, surrealdb::Error> {
        self.db.query("SELECT *, user.* FROM post WHERE created_at < $created_at ORDER BY created_at DESC LIMIT $post_limit")
            .bind(("post_limit", limit))
            .bind(("created_at", time))
            .await
            .check_good(surrealdb::Error::from)
            .and_then_take_all(0)
    }

    pub async fn add_post(
        &self,
        time: u128,
        username: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        favorites: u64,
        files: Vec<DBUserPostFile>,
    ) -> Result<DBUserPost, AddPostErr> {
        let username = username.into();
        let title = title.into();
        let description = description.into();

        self.db
            .query(
                r#"
             LET $user = SELECT id FROM ONLY user WHERE username = $username;
             LET $post = CREATE post SET
                user = $user.id,
                show = true,
                title = $title,
                description = $description,
                favorites = $favorites,
                file = $files,
                modified_at = $time,
                created_at = $time;
             SELECT *, user.* FROM $post.id;
            "#,
            )
            .bind(("files", files))
            .bind(("username", username.clone()))
            .bind(("title", title))
            .bind(("description", description))
            .bind(("favorites", favorites))
            .bind(("time", time))
            .await
            .check_good(|err| match err {
                err if err.field_value_null("user_id") => AddPostErr::UserNotFound(username),
                err => err.into(),
            })
            .and_then_take_expect(2)
    }

    // pub async fn add_invite(
    //     &self,
    //     time: u128,
    //     // kind: impl Into<DBEmailTokenKind>,
    //     token_raw: impl Into<String>,
    //     email: impl Into<String>,
    //     expires: u128,
    // ) -> Result<DBInvite, surrealdb::Error> {
    //     let email: String = email.into();
    //
    //     self.db.query(
    //         r#"
    //          LET $prev_token = SELECT * FROM ONLY invite WHERE email = $email AND kind = $kind AND used = 0 AND expires >= $time ORDER BY created_at DESC;
    //          IF $prev_token {
    //             return $prev_token;
    //          } ELSE {
    //             LET $result = CREATE email_confirm SET
    //                token_raw = $token_raw,
    //                kind = $kind,
    //                email = $email,
    //                expires = $expires,
    //                used = 0,
    //                modified_at = $time,
    //                created_at = $time;
    //             return $result;
    //          }
    //         "#,
    //     )
    //     .bind(("kind", kind.into().to_string()))
    //     .bind(("token_raw", token_raw.into()))
    //     .bind(("email", email.clone()))
    //     .bind(("expires", expires))
    //     .bind(("time", time))
    //     .await
    //     .check_good(surrealdb::Error::from)
    //     .and_then_take_expect(1)
    // }
    // current.email = $user_email AND
    // current.token_used = false AND
    // current.token_expires >= $time AND
    // (new = NONE OR (
    // new.token_used = false AND
    // new.token_expires >= $time
    // ))
    // ORDER BY created_at DESC;
    pub async fn add_sent_email(
        &self,
        time: u128,
        body: impl Into<String>,
        to_email: impl Into<String>,
        reason: DBSentEmailReason,
    ) -> Result<DBSentEmail, surrealdb::Error> {
        self.db
            .query(
                r#"
               CREATE sent_email SET
                   body = $body,
                   to_email = $to_email,
                   reason = $reason,
                   modified_at = $time,
                   created_at = $time;
            "#,
            )
            .bind(("body", body.into()))
            .bind(("to_email", to_email.into()))
            .bind(("reason", reason.to_string()))
            .bind(("time", time))
            .await
            .check_good(surrealdb::Error::from)
            .and_then_take_expect(0)
    }

    pub async fn get_sent_email_by_email(
        &self,
        to_email: impl Into<String>,
    ) -> Result<Vec<DBSentEmail>, surrealdb::Error> {
        self.db
            .query(
                r#"
                SELECT * FROM sent_email WHERE to_email = $to_email ORDER BY created_at DESC;
            "#,
            )
            .bind(("to_email", to_email.into()))
            .await
            .check_good(surrealdb::Error::from)
            .and_then_take_all(0)
    }

    pub async fn get_sent_email_by_email_latest(
        &self,
        to_email: impl Into<String>,
    ) -> Result<DBSentEmail, DB404Err> {
        self.db
            .query(
                r#"
                SELECT * FROM ONLY sent_email WHERE to_email = $to_email ORDER BY created_at DESC LIMIT 1;
            "#,
            )
            .bind(("to_email", to_email.into()))
            .await
            .check_good(DB404Err::from)
            .and_then_take_or(0, DB404Err::NotFound)
    }

    pub async fn add_invite(
        &self,
        time: u128,
        // kind: impl Into<DBEmailTokenKind>,
        token_raw: impl Into<String>,
        email: impl Into<String>,
        expires: u128,
        // where_used: u64,
    ) -> Result<DBInvite, EmailIsTakenErr> {
        let token_raw = token_raw.into();
        let email: String = email.into();

        self.db.query(
            r#"
             LET $prev_token = SELECT * FROM ONLY invite WHERE email = $email AND kind = $kind AND used = false AND expires >= $time ORDER BY created_at DESC;
             IF $prev_token {
                return $prev_token;
             } ELSE {
                LET $user_email = SELECT email FROM ONLY user WHERE email = $email;
                LET $result = CREATE invite SET
                   token_raw = $token_raw,
                   kind = $kind,
                   email = if $user_email { null } else { $email },
                   expires = $expires,
                   used = false,
                   modified_at = $time,
                   created_at = $time;
                return $result;
             }
            "#,
        )
        // .bind(("kind", kind.into().to_string()))
        .bind(("token_raw", token_raw))
        .bind(("email", email.clone()))
        .bind(("expires", expires))
        // .bind(("where_used", where_used))
        .bind(("time", time))
        .await
        .check_good(|err| match err {
            err if err.field_value_null("email") => EmailIsTakenErr::EmailIsTaken(email),
            err => err.into(),
        })
        .and_then_take_expect(1)
    }

    pub async fn get_invite_all(&self) -> Result<Vec<DBInvite>, DB404Err> {
        self.db
            .query("SELECT * FROM invite;")
            .await
            .check_good(DB404Err::from)
            .and_then_take_all(0)
    }

    pub async fn get_invite_all_valid<Email: Into<String>>(
        &self,
        time: u128,
        email: Email,
    ) -> Result<Vec<DBInvite>, DB404Err> {
        self.db.query("SELECT * FROM invite WHERE email = $email AND used = false AND expires >= $time ORDER BY created_at DESC;")
            .bind(("email", email.into()))
            .bind(("time", time))
        .await
            .check_good(DB404Err::from)
            .and_then_take_all(0)
    }

    pub async fn get_invite_any_by_token(
        &self,
        // kind: impl Into<DBEmailTokenKind>,
        token: impl Into<String>,
    ) -> Result<DBInvite, DB404Err> {
        self.db
            .query("SELECT * FROM ONLY invite WHERE token_raw = $invite_token;")
            // .bind(("kind", kind.into().to_string()))
            .bind(("invite_token", token.into()))
            .await
            .check_good(DB404Err::from)
            .and_then_take_or(0, DB404Err::NotFound)
    }

    pub async fn get_invite_valid(
        &self,
        time: u128,
        // kind: impl Into<DBEmailTokenKind>,
        email: impl Into<String>,
        // used: u64,
    ) -> Result<DBInvite, DB404Err> {
        self.db.query("SELECT * FROM invite WHERE email = $email AND used = false AND expires >= $time ORDER BY created_at DESC;")
            // .bind(("kind", kind.into().to_string()))
            .bind(("email", email.into()))
            .bind(("time", time))
            // .bind(("used", used))
            .await
            .check_good(DB404Err::from)
            .and_then_take_all(0)
            .and_then(|v| v.first().cloned().ok_or(DB404Err::NotFound))
    }

    pub async fn update_invite_used(
        &self,
        time: u128,
        // kind: impl Into<DBEmailTokenKind>,
        token_raw: impl Into<String>,
        // where_used: u64,
        // set_used: u64,
    ) -> Result<DBInvite, DB404Err> {
        self.db
            .query(
                "UPDATE invite SET modified_at = $time, used = true WHERE token_raw = $token_raw AND used = false AND expires >= $time;",
                // "UPDATE email_confirm SET modified_at = $time, used = 1 WHERE kind = $kind AND token_raw = $token_raw AND expires >= $time;",
            )
            .bind(("token_raw", token_raw.into()))
            // .bind(("kind", kind.into().to_string()))
            .bind(("time", time))
            // .bind(("set_used", set_used))
            // .bind(("where_used", where_used))
            .await
            .check_good(DB404Err::from)
            .and_then_take_or(0, DB404Err::NotFound)
    }

    pub async fn update_user_username(
        &self,
        user: RecordId,
        new_username: impl Into<String>,
        time: u128,
    ) -> Result<DBUser, DBChangeUsernameErr> {
        let username = new_username.into();
        self.db
            .query(
                "UPDATE user SET modified_at = $time, username = $new_username WHERE id = $user_id;",
            )
            .bind(("user_id", user))
            .bind(("new_username", username.clone()))
            .bind(("time", time))
            .await
            .check_good(|err| match err {
                err if err.index_exists("idx_user_username") => DBChangeUsernameErr::UsernameIsTaken(username),
                err => err.into(),
            })
            .and_then_take_or(0, DBChangeUsernameErr::NotFound)
    }

    pub async fn update_user_password(
        &self,
        user: RecordId,
        new_password: impl Into<String>,
        time: u128,
    ) -> Result<DBUser, DB404Err> {
        self.db
            .query(
                "UPDATE user SET modified_at = $time, password = $new_password WHERE id = $user_id;",
            )
            .bind(("user_id", user))
            .bind(("new_password", new_password.into()))
            .bind(("time", time))
            .await
            .check_good(DB404Err::from)
            .and_then_take_or(0, DB404Err::NotFound)
    }

    pub async fn update_user_password_by_email(
        &self,
        time: u128,
        email: impl Into<String>,
        new_password: impl Into<String>,
    ) -> Result<DBUser, DB404Err> {
        self.db
            .query(
                "UPDATE user SET modified_at = $time, password = $new_password WHERE email = $email;",
            )
            .bind(("email", email.into()))
            .bind(("new_password", new_password.into()))
            .bind(("time", time))
            .await
            .check_good(DB404Err::from)
            .and_then_take_or(0, DB404Err::NotFound)
    }

    pub async fn add_user<Username: Into<String>, Email: Into<String>, Password: Into<String>>(
        &self,
        time: u128,
        username: Username,
        email: Email,
        password: Password,
    ) -> Result<DBUser, AddUserErr> {
        let username = username.into();
        let email = email.into();
        let password = password.into();

        self.db
            .query(
                r#"
             CREATE user SET
                username = $username,
                email = $email,
                password = $password,
                modified_at = $time,
                created_at = $time;
            "#,
            )
            .bind(("time", time))
            .bind(("username", username.clone()))
            .bind(("email", email.clone()))
            .bind(("password", password))
            .await
            .check_good(|err| match err {
                err if err.index_exists("idx_user_email") => AddUserErr::EmailIsTaken(email),
                err if err.index_exists("idx_user_username") => {
                    AddUserErr::UsernameIsTaken(username)
                }
                err => err.into(),
            })
            .and_then_take_expect(0)
    }

    pub async fn get_user_by_username<Username: Into<String>>(
        &self,
        username: Username,
    ) -> Result<DBUser, DB404Err> {
        self.db
            .query("SELECT * FROM user WHERE username = $username;")
            .bind(("username", username.into()))
            .await
            .check_good(DB404Err::from)
            .and_then_take_or(0, DB404Err::NotFound)
    }

    pub async fn get_all_user(&self) -> Result<Vec<DBUser>, GetAllUsers> {
        self.db
            .query("SELECT * FROM user;")
            .await
            .check_good(GetAllUsers::from)
            .and_then_take_all(0)
    }
    pub async fn get_user_by_email(&self, email: impl Into<String>) -> Result<DBUser, DB404Err> {
        self.db
            .query("SELECT * FROM user WHERE email = $email;")
            .bind(("email", email.into()))
            .await
            .check_good(DB404Err::from)
            .and_then_take_or(0, DB404Err::NotFound)
    }

    pub async fn get_user_password<S: Into<String>>(&self, email: S) -> Result<String, DB404Err> {
        self.db
            .query("(SELECT password FROM user WHERE email = $email).password")
            .bind(("email", email.into()))
            .await
            .check_good(DB404Err::from)
            .and_then_take_or(0, DB404Err::NotFound)
    }

    pub async fn add_session(
        &self,
        time: u128,
        token: impl Into<String>,
        username: impl Into<String>,
    ) -> Result<DBSession, AddSessionErr> {
        let username: String = username.into();
        self.db
            .query(
                r#"
                     LET $user = SELECT * FROM ONLY user WHERE username = $username;
                     LET $user_session = CREATE session SET access_token = $access_token, user = $user.id, modified_at = $time, created_at = $time;
                     SELECT *, user.* FROM $user_session.id;
                "#,
            )
            .bind(("time", time))
            .bind(("access_token", token.into()))
            .bind(("username", username.clone()))
            .await
            .check_good(|err| match err {
                err if err.field_value_null("user") => AddSessionErr::UserNotFound(username),
                err if err.index_exists("idx_session_access_token") => AddSessionErr::TokenExists,
                err => err.into(),
            })
            .and_then_take_expect(2)
    }

    pub async fn delete_session_user(&self, user_id: RecordId) -> Result<(), surrealdb::Error> {
        self.db
            .query("DELETE session WHERE user = $user_id;")
            .bind(("user_id", user_id))
            .await
            .check_good(surrealdb::Error::from)
            .map(|_| ())
    }

    pub async fn delete_session<S: Into<String>>(&self, token: S) -> Result<(), surrealdb::Error> {
        self.db
            .query("DELETE session WHERE access_token = $access_token;")
            .bind(("access_token", token.into()))
            .await
            .check_good(surrealdb::Error::from)
            .map(|_| ())
    }

    pub async fn get_session<S: Into<String>>(&self, token: S) -> Result<DBSession, DB404Err> {
        self.db
            .query("SELECT *, user.* FROM session WHERE access_token = $access_token;")
            .bind(("access_token", token.into()))
            .await
            .check_good(DB404Err::from)
            .and_then_take_or(0, DB404Err::NotFound)
    }

    pub async fn get_session_all(&self) -> Result<Vec<DBSession>, DB404Err> {
        self.db
            .query("SELECT *, user.* FROM session")
            .await
            .check_good(DB404Err::from)
            .and_then_take_all(0)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    // use pretty_assertions::assert_eq;
    use surrealdb::engine::local::Mem;
    use tracing::trace;

    use crate::{
        api::ChangeUsernameErr,
        db::{
            AddSessionErr, AddUserErr, DB404Err, DBChangeUsernameErr, DBSentEmailReason,
            DBUserPostFile, Db, EmailIsTakenErr,
        },
    };

    #[tokio::test]
    async fn db_sent_email() {
        crate::init_test_log();

        let db = Db::new::<Mem>(()).await.unwrap();
        db.migrate(0).await.unwrap();

        let sent_email = db
            .add_sent_email(
                0,
                "wowza",
                "prime@heyadora.com",
                DBSentEmailReason::ConfirmEmailChangeNewEmail,
            )
            .await
            .unwrap();
        assert_eq!(sent_email.body, "wowza");

        let sent_email = db
            .add_sent_email(
                1,
                "wowza2",
                "prime@heyadora.com",
                DBSentEmailReason::ConfirmEmailChangeNewEmail,
            )
            .await
            .unwrap();
        assert_eq!(sent_email.body, "wowza2");

        let all_emails = db
            .get_sent_email_by_email("prime@heyadora.com")
            .await
            .unwrap();
        assert_eq!(all_emails[0].body, "wowza2");

        let latest_email = db
            .get_sent_email_by_email_latest("prime@heyadora.com")
            .await
            .unwrap();
        assert_eq!(latest_email.body, "wowza2");
    }

    #[tokio::test]
    async fn db_post() {
        let db = Db::new::<Mem>(()).await.unwrap();
        db.migrate(0).await.unwrap();
        let user = db.add_user(0, "hey", "hey@hey.com", "123").await.unwrap();
        let user2 = db.add_user(0, "hey2", "hey2@hey.com", "123").await.unwrap();

        let post = db
            .add_post(
                0,
                "hey",
                "title",
                "description",
                0,
                vec![
                    DBUserPostFile {
                        extension: ".png".to_string(),
                        hash: "A".to_string(),
                        width: 1,
                        height: 1,
                    },
                    DBUserPostFile {
                        extension: ".png".to_string(),
                        hash: "B".to_string(),
                        width: 1,
                        height: 1,
                    },
                ],
            )
            .await
            .unwrap();
        trace!("{post:#?}");
        assert!(post.file.len() == 2);
        assert_eq!(post.title, "title");
        assert_eq!(post.file[0].hash, "A");
        assert_eq!(post.file[1].hash, "B");

        for i in 1..=3 {
            let _post = db
                .add_post(
                    i,
                    "hey",
                    format!("title{i}"),
                    "description",
                    0,
                    vec![DBUserPostFile {
                        extension: ".png".to_string(),
                        hash: i.to_string(),
                        width: 1,
                        height: 1,
                    }],
                )
                .await
                .unwrap();
        }

        let posts = db.get_post_older(2, 3).await.unwrap();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].title, "title1");
        assert_eq!(posts[1].title, "title");

        let posts = db.get_post_older(2, 1).await.unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].title, "title1");

        let posts = db.get_post_newer(1, 3).await.unwrap();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].title, "title3");
        assert_eq!(posts[1].title, "title2");

        let posts = db.get_post_newer(1, 1).await.unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].title, "title2");

        let post = db.get_post(posts[0].id.key().clone()).await.unwrap();
        assert_eq!(post.title, "title2");

        let post = db.get_post("wow:wow").await;
        trace!("result: {post:#?}");
        assert!(matches!(post, Err(DB404Err::NotFound)));

        let posts = db
            .get_post_newer_or_equal_for_user(1, 3, user.id.clone())
            .await
            .unwrap();
        assert_eq!(posts.len(), 3);
        assert_eq!(posts[0].title, "title3");

        let posts = db
            .get_post_older_or_equal_for_user(1, 3, user.id.clone())
            .await
            .unwrap();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].title, "title1");

        let posts = db
            .get_post_newer_or_equal_for_user(1, 3, user2.id.clone())
            .await
            .unwrap();
        assert_eq!(posts.len(), 0);

        // let posts = db.get_post_older(1, 1).await.unwrap();
        // assert_eq!(posts.len(), 2);

        // let post2 = db.get_post_older(2, 25).await.unwrap();
        // assert_eq!(post, post2);
        // let posts3 = db.get_post_older(1, 25).await.unwrap();
        // assert_eq!(posts, posts3);
    }

    #[tokio::test]
    async fn db_email_token() {
        let db = Db::new::<Mem>(()).await.unwrap();
        let time = Duration::from_nanos(0);
        let time = time.as_nanos();
        db.migrate(time).await.unwrap();

        let invite = db
            .add_invite(
                0,
                // DBEmailTokenKind::RequestConfirmRegistrationEmail,
                "wowza",
                "hey@hey.com",
                0,
                // 0,
            )
            .await
            .unwrap();
        trace!("{invite:#?}");
        let invite = db
            .add_invite(
                1,
                // DBEmailTokenKind::RequestConfirmRegistrationEmail,
                "wowza1",
                "hey@hey.com",
                2,
                // 0
            )
            .await
            .unwrap();
        trace!("{invite:#?}");
        let invite = db
            .add_invite(
                1,
                // DBEmailTokenKind::RequestConfirmRegistrationEmail,
                "wowza2",
                "hey@hey.com",
                0,
                // 0
            )
            .await
            .unwrap();
        trace!("{invite:#?}");
        let invite = db
            .get_invite_valid(
                1,
                // DBEmailTokenKind::RequestConfirmRegistrationEmail,
                "hey@hey.com",
                // 0,
            )
            .await;
        trace!("{invite:#?}");
        assert_eq!(invite.unwrap().token_raw, "wowza1");
        let invite = db.get_invite_any_by_token("wowza1").await;
        trace!("{invite:#?}");
        assert_eq!(invite.unwrap().token_raw, "wowza1");
        let invite = db
            .get_invite_valid(
                0,
                // DBEmailTokenKind::RequestConfirmRegistrationEmail,
                "hey1@hey.com",
                // 0,
            )
            .await;
        trace!("{invite:#?}");
        assert!(matches!(invite, Err(DB404Err::NotFound)));
        let invites = db.get_invite_all_valid(1, "hey@hey.com").await.unwrap();
        assert_eq!(invites.len(), 1);
        // let result = db.update_email_confirm_used(1, "wowza", 0, 1).await;
        // assert!(matches!(result, Err(DB404Err::NotFound)));
        // // let result = db.update_email_confirm_used(1, "wowza1", 1).await;
        // // assert!(matches!(result, Err(DB404Err::NotFound)));
        // let _invite = db.update_email_confirm_used(1, "wowza1", 0,  1).await.unwrap();
        // let invite = db
        //     .get_invite_valid(
        //         1,
        //         // DBEmailTokenKind::RequestConfirmRegistrationEmail,
        //         "hey@hey.com",
        //         // 0,
        //     )
        //     .await;
        // assert!(matches!(invite, Err(DB404Err::NotFound)));

        let user = db.add_user(0, "hey1", "hey1@hey.com", "123").await.unwrap();
        let invite2 = db
            .add_invite(
                0,
                // DBEmailTokenKind::RequestConfirmRegistrationEmail,
                "wowza",
                "hey1@hey.com",
                0,
                // 0,
            )
            .await;
        trace!("{invite2:#?}");
        assert!(matches!(invite2, Err(EmailIsTakenErr::EmailIsTaken(_))));
    }

    #[tokio::test]
    async fn db_user() {
        let db = Db::new::<Mem>(()).await.unwrap();
        let time = 0;
        db.migrate(time).await.unwrap();
        let user = db
            .add_user(time, "hey", "hey@hey.com", "hey")
            .await
            .unwrap();
        trace!("{user:#?}");

        let user = db.add_user(time, "hey2", "hey@hey.com", "hey").await;
        trace!("{user:#?}");
        assert!(matches!(user, Err(AddUserErr::EmailIsTaken(_))));

        let user = db.add_user(time, "hey", "hey2@hey.com", "hey").await;
        trace!("{user:#?}");
        assert!(matches!(user, Err(AddUserErr::UsernameIsTaken(_))));

        let user = db.get_user_by_username("hey").await.unwrap();
        trace!("found {user:#?}");

        let user = db.get_user_by_username("hey2").await;
        trace!("found {user:#?}");
        assert!(matches!(user, Err(DB404Err::NotFound)));

        let user1 = db.get_user_by_email("hey@hey.com").await.unwrap();
        trace!("found {user1:#?}");

        let user = db.get_user_by_email("hey2@hey.com").await;
        trace!("found {user:#?}");
        assert!(matches!(user, Err(DB404Err::NotFound)));

        let password = db.get_user_password("hey@hey.com").await.unwrap();
        trace!("found {user:#?}");
        assert_eq!(password, "hey");

        let result = db.get_user_password("hey2@hey.com").await;
        assert!(matches!(result, Err(DB404Err::NotFound)));

        let result = db
            .update_user_username(user1.id.clone(), "hey5", time)
            .await
            .unwrap();
        assert_eq!(result.username, "hey5");

        let result = db.get_user_by_username("hey").await;
        assert!(matches!(result, Err(DB404Err::NotFound)));

        let user2 = db
            .add_user(time, "hey2", "hey2@hey.com", "hey")
            .await
            .unwrap();

        let result = db
            .update_user_username(user1.id.clone(), "hey2", time)
            .await;
        assert!(matches!(
            result,
            Err(DBChangeUsernameErr::UsernameIsTaken(_))
        ));

        let result = db
            .update_user_password(user1.id.clone(), "pass1", time)
            .await;

        assert!(result.is_ok());

        let result = db
            .update_user_password_by_email(time, "hey@hey.com", "pass3")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn db_session() {
        let db = Db::new::<Mem>(()).await.unwrap();
        db.migrate(0).await.unwrap();
        let user = db.add_user(0, "hey", "hey@hey.com", "hey").await.unwrap();
        let user2 = db
            .add_user(0, "hey11", "hey11@hey.com", "hey")
            .await
            .unwrap();

        trace!("created {user:#?}");
        let session = db.add_session(0, "token", "hey").await.unwrap();

        let session = db.add_session(0, "token", "hey").await;
        trace!("session: {session:?}");
        assert!(matches!(session, Err(AddSessionErr::TokenExists)));

        let session = db.add_session(0, "token", "hey2").await;
        trace!("session: {session:?}");
        assert!(matches!(session, Err(AddSessionErr::UserNotFound(_))));

        let session = db.get_session("token1").await;
        assert!(matches!(session, Err(DB404Err::NotFound)));

        let _session = db.get_session("token").await.unwrap();

        db.delete_session("token").await.unwrap();

        let session = db.get_session("token").await;
        assert!(matches!(session, Err(DB404Err::NotFound)));

        let session = db.add_session(0, "token", "hey").await.unwrap();
        let session = db.add_session(0, "token11", "hey11").await.unwrap();
        db.delete_session_user(user.id.clone()).await.unwrap();

        let session = db.get_session("token").await;
        assert!(matches!(session, Err(DB404Err::NotFound)));

        let session = db.get_session("token11").await.unwrap();
    }
}
