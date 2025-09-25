use std::str::FromStr;
use std::sync::LazyLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};
pub use surrealdb::Connection;
use surrealdb::method::Query;
use surrealdb::opt::IntoQuery;
// pub use surrealdb::engine::local;
use surrealdb::RecordId;
use surrealdb::engine::local::SurrealKv;
use surrealdb::engine::local::{self, Mem};
use surrealdb::{Surreal, opt::IntoEndpoint};
use thiserror::Error;
use tracing::{error, trace};

// pub static DB: LazyLock<Db<local::Db>> = LazyLock::new(Db::init);
derive_alias! {
    #[derive(Save!)] = #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)];
}

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

pub fn surreal_time_from_duration(time: Duration) -> surrealdb::Datetime {
    surrealdb::Datetime::from(chrono::DateTime::from_timestamp_nanos(
        time.as_nanos() as i64
    ))
}

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

#[derive(Save!)]
pub struct DBUser {
    pub id: RecordId,
    pub username: String,
    pub email: String,
    pub password: String,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Save!)]
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

#[derive(Save!)]
pub struct DBUserPostFile {
    pub extension: String,
    pub hash: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Save!)]
pub struct Invite {
    pub id: RecordId,
    pub token_raw: String,
    pub email: String,
    pub expires: u128,
    pub used: bool,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(Save!)]
pub struct Session {
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
pub enum AddInviteErr {
    #[error("DB error {0}")]
    DB(#[from] surrealdb::Error),

    #[error("account with \"{0}\" email already exists")]
    EmailIsTaken(String),
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

impl Db<local::Db> {
    pub async fn connect(&self) {
        // TODO make path as env
        let db = &self.db;
        // db.connect::<SurrealKv>("db5").await.unwrap();
        db.use_ns("artbounty").use_db("web").await.unwrap();
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
                    "
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
                            -- invite 
                            DEFINE TABLE invite SCHEMAFULL;
                            DEFINE FIELD token_raw ON TABLE invite TYPE string;
                            DEFINE FIELD email ON TABLE invite TYPE string;
                            DEFINE FIELD expires ON TABLE invite TYPE number;
                            DEFINE FIELD used ON TABLE invite TYPE bool DEFAULT false;
                            DEFINE FIELD modified_at ON TABLE invite TYPE number;
                            DEFINE FIELD created_at ON TABLE invite TYPE number;
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

                            CREATE migration SET version = 0, modified_at = $time, created_at = $time;
                        };
                    };

                    SELECT * FROM migration;
                ",
                )
                .bind(("time", time))
                .await.inspect_err(|result| trace!("DB RESULT {:#?}", result) )?;
        result.check()?;
        Ok(())
    }

    pub async fn get_post_str(&self, post_id: impl AsRef<str>) -> Result<DBUserPost, DB404Err> {
        let post_id = RecordId::from_str(post_id.as_ref())?;
        self.get_post(post_id).await
    }

    pub async fn get_post(&self, post_id: RecordId) -> Result<DBUserPost, DB404Err> {
        self.db
            .query("SELECT *, user.* FROM ONLY $post_id;")
            .bind(("post_id", post_id))
            .await
            .check_good(DB404Err::from)
            .and_then_take_or(0, DB404Err::NotFound)
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

    pub async fn add_invite(
        &self,
        time: u128,
        token_raw: impl Into<String>,
        email: impl Into<String>,
        expires: u128,
    ) -> Result<Invite, AddInviteErr> {
        let token_raw = token_raw.into();
        let email: String = email.into();

        self.db.query(
            r#"
             LET $prev_token = SELECT * FROM ONLY invite WHERE email = $email AND used = false AND expires >= $time ORDER BY created_at DESC;
             IF $prev_token {
                return $prev_token;
             } ELSE {
                LET $user_email = SELECT email FROM ONLY user WHERE email = $email;
                LET $result = CREATE invite SET
                   token_raw = $token_raw,
                   email = if $user_email { null } else { $email },
                   expires = $expires,
                   used = false,
                   modified_at = $time,
                   created_at = $time;
                return $result;
             }
            "#,
        )
        .bind(("token_raw", token_raw))
        .bind(("email", email.clone()))
        .bind(("expires", expires))
        .bind(("time", time))
        .await
        .check_good(|err| match err {
            err if err.field_value_null("email") => AddInviteErr::EmailIsTaken(email),
            err => err.into(),
        })
        .and_then_take_expect(1)
    }

    pub async fn get_all_invites(&self) -> Result<Vec<Invite>, DB404Err> {
        self.db
            .query("SELECT * FROM invite;")
            .await
            .check_good(DB404Err::from)
            .and_then_take_all(0)
    }
    pub async fn get_invite_by_token(&self, token: impl Into<String>) -> Result<Invite, DB404Err> {
        self.db
            .query("SELECT * FROM ONLY invite WHERE token_raw = $invite_token;")
            .bind(("invite_token", token.into()))
            .await
            .check_good(DB404Err::from)
            .and_then_take_or(0, DB404Err::NotFound)
    }

    pub async fn get_invite<Email: Into<String>>(
        &self,
        email: Email,
        time: u128,
    ) -> Result<Invite, DB404Err> {
        self.get_invites(email, time)
            .await
            .map_err(DB404Err::from)
            .and_then(|v| v.first().cloned().ok_or(DB404Err::NotFound))
    }

    pub async fn get_invites<Email: Into<String>>(
        &self,
        email: Email,
        time: u128,
    ) -> Result<Vec<Invite>, DB404Err> {
        self.db.query("SELECT * FROM invite WHERE email = $email AND used = false AND expires >= $time ORDER BY created_at DESC;")
            .bind(("email", email.into()))
            .bind(("time", time))
        .await
            .check_good(DB404Err::from)
            .and_then_take_all(0)
    }

    pub async fn use_invite<TokenRaw: Into<String>>(
        &self,
        token_raw: TokenRaw,
        time: u128,
    ) -> Result<Invite, DB404Err> {
        self.db
            .query(
                "UPDATE invite SET modified_at = $time, used = true WHERE token_raw = $token_raw AND expires >= $time;",
            )
            .bind(("token_raw", token_raw.into()))
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
    ) -> Result<Session, AddSessionErr> {
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

    pub async fn delete_session<S: Into<String>>(&self, token: S) -> Result<(), surrealdb::Error> {
        self.db
            .query("DELETE session WHERE access_token = $access_token;")
            .bind(("access_token", token.into()))
            .await
            .check_good(surrealdb::Error::from)
            .map(|_| ())
    }

    pub async fn get_session<S: Into<String>>(&self, token: S) -> Result<Session, DB404Err> {
        self.db
            .query("SELECT *, user.* FROM session WHERE access_token = $access_token;")
            .bind(("access_token", token.into()))
            .await
            .check_good(DB404Err::from)
            .and_then_take_or(0, DB404Err::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use pretty_assertions::assert_eq;
    use surrealdb::engine::local::Mem;
    use test_log::test;
    use tracing::trace;

    use crate::db::{AddInviteErr, AddSessionErr, AddUserErr, DB404Err, DBUserPostFile, Db};

    #[test(tokio::test)]
    async fn db_post() {
        let db = Db::new::<Mem>(()).await.unwrap();
        let time: u128 = 0;
        db.migrate(time).await.unwrap();
        db.add_user(time, "hey", "hey@hey.com", "123")
            .await
            .unwrap();

        let post = db
            .add_post(
                time,
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

        let post = db.get_post(posts[0].id.clone()).await.unwrap();
        assert_eq!(post.title, "title2");

        let post = db.get_post_str("wow:wow").await;
        trace!("result: {post:#?}");
        assert!(matches!(post, Err(DB404Err::NotFound)));

        // let posts = db.get_post_older(1, 1).await.unwrap();
        // assert_eq!(posts.len(), 2);

        // let post2 = db.get_post_older(2, 25).await.unwrap();
        // assert_eq!(post, post2);
        // let posts3 = db.get_post_older(1, 25).await.unwrap();
        // assert_eq!(posts, posts3);
    }

    #[test(tokio::test)]
    async fn db_invite() {
        let db = Db::new::<Mem>(()).await.unwrap();
        let time = Duration::from_nanos(0);
        let time = time.as_nanos();
        db.migrate(time).await.unwrap();

        let invite = db.add_invite(0, "wowza", "hey@hey.com", 0).await.unwrap();
        trace!("{invite:#?}");
        let invite = db.add_invite(1, "wowza1", "hey@hey.com", 2).await.unwrap();
        trace!("{invite:#?}");
        let invite = db.add_invite(1, "wowza2", "hey@hey.com", 0).await.unwrap();
        trace!("{invite:#?}");
        let invite = db.get_invite("hey@hey.com", 1).await;
        trace!("{invite:#?}");
        assert_eq!(invite.unwrap().token_raw, "wowza1");
        let invite = db.get_invite_by_token("wowza1").await;
        trace!("{invite:#?}");
        assert_eq!(invite.unwrap().token_raw, "wowza1");
        let invite = db.get_invite("hey1@hey.com", 0).await;
        trace!("{invite:#?}");
        assert!(matches!(invite, Err(DB404Err::NotFound)));
        let invites = db.get_invites("hey@hey.com", 1).await.unwrap();
        assert_eq!(invites.len(), 1);
        let result = db.use_invite("wowza", 1).await;
        assert!(matches!(result, Err(DB404Err::NotFound)));
        let _invite = db.use_invite("wowza1", 1).await.unwrap();
        let invite = db.get_invite("hey@hey.com", 1).await;
        assert!(matches!(invite, Err(DB404Err::NotFound)));

        let user = db.add_user(0, "hey1", "hey1@hey.com", "123").await.unwrap();
        let invite2 = db.add_invite(0, "wowza", "hey1@hey.com", 0).await;
        trace!("{invite2:#?}");
        assert!(matches!(invite2, Err(AddInviteErr::EmailIsTaken(_))));
    }

    #[test(tokio::test)]
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

        let user = db.get_user_by_email("hey@hey.com").await.unwrap();
        trace!("found {user:#?}");

        let user = db.get_user_by_email("hey2@hey.com").await;
        trace!("found {user:#?}");
        assert!(matches!(user, Err(DB404Err::NotFound)));

        let password = db.get_user_password("hey@hey.com").await.unwrap();
        trace!("found {user:#?}");
        assert_eq!(password, "hey");

        let result = db.get_user_password("hey2@hey.com").await;
        assert!(matches!(result, Err(DB404Err::NotFound)));
    }

    #[test(tokio::test)]
    async fn db_session() {
        let db = Db::new::<Mem>(()).await.unwrap();
        db.migrate(0).await.unwrap();
        let user = db.add_user(0, "hey", "hey@hey.com", "hey").await.unwrap();
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
    }
}
