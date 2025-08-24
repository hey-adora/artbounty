use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
pub use surrealdb::Connection;
// pub use surrealdb::engine::local;
use surrealdb::RecordId;
use surrealdb::engine::local::SurrealKv;
use surrealdb::engine::local::{self, Mem};
use surrealdb::{Datetime, Surreal, opt::IntoEndpoint};
use thiserror::Error;
use tracing::{error, trace};

// pub static DB: LazyLock<Db<local::Db>> = LazyLock::new(Db::init);

pub type DbEngine = Db<local::Db>;
pub async fn new_local(path: impl AsRef<str>) -> Db<local::Db> {
    let db = Db::<local::Db>::new::<SurrealKv>(path.as_ref())
        .await
        .unwrap();
    db.connect().await;
    db.migrate().await.unwrap();

    db
}

pub async fn new_mem() -> Db<local::Db> {
    let db = Db::<local::Db>::new::<Mem>(()).await.unwrap();
    db.connect().await;
    db.migrate().await.unwrap();

    db
}

#[derive(Debug, Clone)]
pub struct Db<C: Connection> {
    pub db: Surreal<C>,
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
    pub async fn migrate(&self) -> Result<(), surrealdb::Error> {
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
                            DEFINE FIELD modified_at ON TABLE migration TYPE datetime DEFAULT time::now();
                            DEFINE FIELD created_at ON TABLE migration TYPE datetime DEFAULT time::now();
                            DEFINE INDEX idx_migration_version ON TABLE migration COLUMNS version UNIQUE;
                            -- user
                            DEFINE TABLE user SCHEMAFULL;
                            DEFINE FIELD username ON TABLE user TYPE string;
                            DEFINE FIELD email ON TABLE user TYPE string;
                            DEFINE FIELD password ON TABLE user TYPE string;
                            DEFINE FIELD modified_at ON TABLE user TYPE datetime DEFAULT time::now();
                            DEFINE FIELD created_at ON TABLE user TYPE datetime DEFAULT time::now();
                            DEFINE INDEX idx_user_username ON TABLE user COLUMNS username UNIQUE;
                            DEFINE INDEX idx_user_email ON TABLE user COLUMNS email UNIQUE;
                            -- session
                            DEFINE TABLE session SCHEMAFULL;
                            DEFINE FIELD access_token ON TABLE session TYPE string;
                            DEFINE FIELD user_id ON TABLE session TYPE record<user>;
                            DEFINE FIELD modified_at ON TABLE session TYPE datetime DEFAULT time::now();
                            DEFINE FIELD created_at ON TABLE session TYPE datetime DEFAULT time::now();
                            DEFINE INDEX idx_session_access_token ON TABLE session COLUMNS access_token UNIQUE;
                            -- stats
                            DEFINE TABLE stat SCHEMAFULL;
                            DEFINE FIELD country ON TABLE stat TYPE string;
                            DEFINE FIELD modified_at ON TABLE stat TYPE datetime DEFAULT time::now();
                            DEFINE FIELD created_at ON TABLE stat TYPE datetime DEFAULT time::now();
                            DEFINE INDEX idx_stat_country ON TABLE stat COLUMNS country UNIQUE;
                            -- invite 
                            DEFINE TABLE invite SCHEMAFULL;
                            DEFINE FIELD token_raw ON TABLE invite TYPE string;
                            DEFINE FIELD email ON TABLE invite TYPE string;
                            DEFINE FIELD expires ON TABLE invite TYPE datetime;
                            DEFINE FIELD used ON TABLE invite TYPE bool DEFAULT false;
                            DEFINE FIELD modified_at ON TABLE invite TYPE datetime DEFAULT time::now();
                            DEFINE FIELD created_at ON TABLE invite TYPE datetime DEFAULT time::now();
                            -- post 
                            DEFINE TABLE post SCHEMAFULL;
                            DEFINE FIELD user_id ON TABLE post TYPE record<user>;
                            DEFINE FIELD show ON TABLE post TYPE bool;
                            DEFINE FIELD title ON TABLE post TYPE string;
                            DEFINE FIELD description ON TABLE post TYPE string;
                            DEFINE FIELD file ON TABLE post TYPE array<object>;
                            DEFINE FIELD file.*.extension ON TABLE post TYPE string;
                            DEFINE FIELD file.*.hash ON TABLE post TYPE string;
                            DEFINE FIELD file.*.width ON TABLE post TYPE int;
                            DEFINE FIELD file.*.height ON TABLE post TYPE int;
                            DEFINE FIELD modified_at ON TABLE post TYPE datetime;
                            DEFINE FIELD created_at ON TABLE post TYPE datetime;
                            -- DEFINE INDEX idx_post_hash ON TABLE post COLUMNS hash UNIQUE;

                            CREATE migration SET version = 0;
                        };
                    };

                    SELECT * FROM migration;
                ",
                )
                .await.inspect_err(|result| trace!("DB RESULT {:#?}", result) )?;
        result.check()?;
        // .inspect(|result| trace!("RESULT CHECK {:#?}", result))?;
        Ok(())
    }
}
pub mod post {
    use serde::{Deserialize, Serialize};
    use surrealdb::{Datetime, RecordId};

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct Post {
        pub id: RecordId,
        pub user_id: RecordId,
        pub show: bool,
        pub title: String,
        pub file: Vec<PostFile>,
        pub modified_at: Datetime,
        pub created_at: Datetime,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct PostFile {
        pub extension: String,
        pub hash: String,
        pub width: u32,
        pub height: u32,
    }

    pub mod get_after {
        use std::time::Duration;

        use serde::{Deserialize, Serialize};
        use surrealdb::{Connection, Datetime, RecordId};
        use thiserror::Error;
        use tracing::{error, trace};

        use super::super::{Db, post::PostFile};

        use super::Post;

        impl<C: Connection> Db<C> {
            pub async fn get_post(&self, time: Duration) -> Result<Vec<Post>, GetPostErr> {
                let db = &self.db;
                // let username = username.into();
                // let title = title.into();
                // let description = description.into();
                let time = Datetime::from(chrono::DateTime::from_timestamp_nanos(
                    time.as_nanos() as i64
                ));

                let result = db
                        .query(
                            r#"
                            -- LET $user = SELECT id FROM ONLY user WHERE username = $username;
                            -- SELECT * FROM post WHERE created_at = $created_at AND user_id = $user.id
                            SELECT * FROM post WHERE created_at <= $created_at ORDER BY created_at DESC
                        "#,
                        )
                        // .bind(("files", files))
                        // .bind(("username", username))
                        // .bind(("title", title))
                        // .bind(("description", description))
                        // .bind(("modified_at", time.clone()))
                        .bind(("created_at", time))
                        .await
                        .inspect_err(|err| error!("get_post query {:#?}", err))?;

                trace!("{:#?}", result);
                let mut result = result.check().map_err(|err| match err {
                    err => {
                        error!("get_post res {:#?}", err);
                        GetPostErr::from(err)
                    }
                })?;
                let result = result
                    .take::<Vec<Post>>(0)
                    .inspect_err(|err| error!("get_post serialize error {:#?}", err))?;

                trace!("record created: {result:#?}");

                Ok(result)
            }
        }

        #[derive(Debug, Error)]
        pub enum GetPostErr {
            #[error("DB error {0}")]
            DB(#[from] surrealdb::Error),
            // #[error("account with \"{0}\" email already exists")]
            // EmailIsTaken(String),
        }

        #[cfg(test)]
        mod tests {
            use std::time::Duration;

            use pretty_assertions::assert_eq;
            use surrealdb::engine::local::Mem;
            use test_log::test;
            use tracing::trace;

            use super::super::super::{
                Db, invite::add_invite::AddInviteErr, post::PostFile, user::add_user::AddUserErr,
            };

            #[test(tokio::test)]
            async fn get_post() {
                let db = Db::new::<Mem>(()).await.unwrap();
                let time = Duration::from_nanos(0);
                db.migrate().await.unwrap();
                db.add_user("hey", "hey@hey.com", "123").await.unwrap();
                let posts = db
                    .add_post(
                        time.clone(),
                        "hey",
                        "title",
                        "description",
                        vec![
                            PostFile {
                                extension: ".png".to_string(),
                                hash: "A".to_string(),
                                width: 1,
                                height: 1,
                            },
                            PostFile {
                                extension: ".png".to_string(),
                                hash: "B".to_string(),
                                width: 1,
                                height: 1,
                            },
                        ],
                    )
                    .await
                    .unwrap();
                trace!("{posts:#?}");
                assert!(posts.len() == 1);
                let posts2 = db.get_post(Duration::from_nanos(0)).await.unwrap();
                assert_eq!(posts, posts2);
                let posts3 = db.get_post(Duration::from_nanos(1)).await.unwrap();
                assert_eq!(posts, posts3);
            }
        }
    }

    pub mod add {
        use std::time::Duration;

        use serde::{Deserialize, Serialize};
        use surrealdb::{Connection, Datetime, RecordId};
        use thiserror::Error;
        use tracing::{error, trace};

        use super::super::{Db, post::PostFile};

        use super::Post;

        impl<C: Connection> Db<C> {
            pub async fn add_post(
                &self,
                time: Duration,
                username: impl Into<String>,
                title: impl Into<String>,
                description: impl Into<String>,
                files: Vec<PostFile>,
            ) -> Result<Vec<Post>, AddPostErr> {
                let db = &self.db;
                let username = username.into();
                let title = title.into();
                let description = description.into();
                let time = Datetime::from(chrono::DateTime::from_timestamp_nanos(
                    time.as_nanos() as i64
                ));

                let result = db
                        .query(
                            r#"
                             LET $user = SELECT id FROM ONLY user WHERE username = $username;
                             --LET $len = $files.len();
                             --FOR $i in 0..1 {
                             --};
                             CREATE post SET
                                user_id = $user.id,
                                show = true,
                                title = $title,
                                description = $description,
                                file = $files,
                                -- extension = $files[$i].extension,
                                -- hash = $files[$i].hash,
                                -- width = $files[$i].width,
                                -- height = $files[$i].height,
                                modified_at = $modified_at,
                                created_at = $created_at;

                            -- SELECT * FROM post WHERE created_at = $created_at AND user_id = $user.id
                        "#,
                        )
                        .bind(("files", files))
                        .bind(("username", username))
                        .bind(("title", title))
                        .bind(("description", description))
                        .bind(("modified_at", time.clone()))
                        .bind(("created_at", time))
                        .await
                        .inspect_err(|err| error!("add_post query {:#?}", err))?;

                trace!("{:#?}", result);
                let mut result = result.check().map_err(|err| match err {
                    err => {
                        error!("add_post res {:#?}", err);
                        AddPostErr::from(err)
                    }
                })?;
                let result = result
                    .take::<Vec<Post>>(1)
                    .inspect_err(|err| error!("add_post serialize error {:#?}", err))?;

                trace!("record created: {result:#?}");

                Ok(result)
            }
        }

        #[derive(Debug, Error)]
        pub enum AddPostErr {
            #[error("DB error {0}")]
            DB(#[from] surrealdb::Error),
            // #[error("account with \"{0}\" email already exists")]
            // EmailIsTaken(String),
        }

        #[cfg(test)]
        mod tests {
            use std::time::Duration;

            use surrealdb::engine::local::Mem;
            use test_log::test;
            use tracing::trace;

            use super::super::super::{
                Db, invite::add_invite::AddInviteErr, post::PostFile, user::add_user::AddUserErr,
            };

            #[test(tokio::test)]
            async fn add_post() {
                let db = Db::new::<Mem>(()).await.unwrap();
                let time = Duration::from_nanos(0);
                db.migrate().await.unwrap();
                db.add_user("hey", "hey@hey.com", "123").await.unwrap();
                let posts = db
                    .add_post(
                        time.clone(),
                        "hey",
                        "title",
                        "description",
                        vec![
                            PostFile {
                                extension: ".png".to_string(),
                                hash: "A".to_string(),
                                width: 1,
                                height: 1,
                            },
                            PostFile {
                                extension: ".png".to_string(),
                                hash: "B".to_string(),
                                width: 1,
                                height: 1,
                            },
                        ],
                    )
                    .await
                    .unwrap();
                trace!("{posts:#?}");
                assert!(posts.len() == 1);
            }
        }
    }
}
pub mod invite {
    use serde::{Deserialize, Serialize};
    use surrealdb::{Datetime, RecordId};

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Invite {
        pub id: RecordId,
        pub token_raw: String,
        pub email: String,
        pub expires: Datetime,
        pub used: bool,
        pub modified_at: Datetime,
        pub created_at: Datetime,
    }

    pub mod add_invite {
        use std::time::Duration;

        use serde::{Deserialize, Serialize};
        use surrealdb::{Connection, Datetime, RecordId};
        use thiserror::Error;
        use tracing::{error, trace};

        use super::super::Db;

        use super::Invite;

        impl<C: Connection> Db<C> {
            pub async fn add_invite(
                &self,
                time: Duration,
                token_raw: impl Into<String>,
                email: impl Into<String>,
                expiration: Duration,
            ) -> Result<Invite, AddInviteErr> {
                let db = &self.db;
                let token_raw = token_raw.into();
                let email: String = email.into();
                let time = Datetime::from(chrono::DateTime::from_timestamp_nanos(
                    time.as_nanos() as i64
                ));
                let expires = Datetime::from(chrono::DateTime::from_timestamp_nanos(
                    expiration.as_nanos() as i64,
                ));

                let result = db
                    .query(
                        r#"
                             LET $user_email = SELECT email FROM ONLY user WHERE email = $email;
                             CREATE invite SET
                                token_raw = $token_raw,
                                email = if $user_email { null } else { $email },
                                expires = $expires,
                                used = false,
                                modified_at = $modified_at,
                                created_at = $created_at;
                        "#,
                    )
                    .bind(("token_raw", token_raw))
                    .bind(("email", email.clone()))
                    .bind(("expires", expires))
                    .bind(("modified_at", time.clone()))
                    .bind(("created_at", time))
                    .await
                    .inspect_err(|err| error!("add_invite query {:#?}", err))?;

                trace!("{:#?}", result);
                let mut result = result.check().map_err(|err| match err {
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
                                .map(|f| f == ".email")
                                .unwrap_or_default() =>
                    {
                        AddInviteErr::EmailIsTaken(email)
                    }
                    err => {
                        error!("add_invite res {:#?}", err);
                        AddInviteErr::from(err)
                    }
                })?;
                let invite = result
                    .take::<Option<Invite>>(1)
                    .inspect_err(|err| error!("add_invite serialize error {:#?}", err))?
                    .expect("record was just created");

                trace!("record created: {invite:#?}");

                Ok(invite)
            }
        }

        #[derive(Debug, Error)]
        pub enum AddInviteErr {
            #[error("DB error {0}")]
            DB(#[from] surrealdb::Error),

            #[error("account with \"{0}\" email already exists")]
            EmailIsTaken(String),
        }

        #[cfg(test)]
        mod tests {
            use std::time::Duration;

            use surrealdb::engine::local::Mem;
            use test_log::test;
            use tracing::trace;

            use super::super::super::{
                Db, invite::add_invite::AddInviteErr, user::add_user::AddUserErr,
            };

            #[test(tokio::test)]
            async fn add_invite() {
                let db = Db::new::<Mem>(()).await.unwrap();
                let time = Duration::from_nanos(0);
                db.migrate().await.unwrap();
                let invite = db
                    .add_invite(
                        time.clone(),
                        "wowza",
                        "hey@hey.com",
                        Duration::from_nanos(0),
                    )
                    .await
                    .unwrap();
                trace!("{invite:#?}");
                let user = db.add_user("hey1", "hey1@hey.com", "123").await.unwrap();
                let invite2 = db
                    .add_invite(
                        time.clone(),
                        "wowza",
                        "hey1@hey.com",
                        Duration::from_nanos(0),
                    )
                    .await;
                trace!("{invite2:#?}");
                assert!(matches!(invite2, Err(AddInviteErr::EmailIsTaken(_))));
            }
        }
    }

    pub mod get_invite {
        use std::time::Duration;

        use serde::{Deserialize, Serialize};
        use surrealdb::{Connection, Datetime, RecordId};
        use thiserror::Error;
        use tracing::{error, trace};

        use super::super::Db;

        use super::Invite;

        impl<C: Connection> Db<C> {
            pub async fn get_invite<Email: Into<String>>(
                &self,
                email: Email,
                time: Duration,
            ) -> Result<Invite, GetInviteErr> {
                let db = &self.db;
                let email = email.into();
                let time = Datetime::from(chrono::DateTime::from_timestamp_nanos(
                    time.as_nanos() as i64
                ));

                let result = db
                        .query(
                            r#"
                             SELECT * FROM invite WHERE email = $email AND used = false AND expires >= $time ORDER BY created_at DESC;
                        "#,
                        )
                        .bind(("email", email.clone()))
                        .bind(("time", time))
                        .await
                        .inspect_err(|err| trace!("add_invite query {:#?}", err))?;

                trace!("{:#?}", result);
                let mut result = result.check().map_err(|err| match err {
                    err => {
                        error!("add_invite res {:#?}", err);
                        GetInviteErr::from(err)
                    }
                })?;
                let invite = result
                    .take::<Vec<Invite>>(0)?
                    .first()
                    .cloned()
                    .ok_or(GetInviteErr::NotFound)?;

                trace!("record created: {invite:#?}");

                Ok(invite)
            }
        }

        #[derive(Debug, Error)]
        pub enum GetInviteErr {
            #[error("DB error {0}")]
            DB(#[from] surrealdb::Error),

            #[error("token not found")]
            NotFound,
        }

        #[cfg(test)]
        mod db {
            use std::time::Duration;

            use surrealdb::engine::local::Mem;
            use test_log::test;
            use tracing::trace;

            use super::super::super::{
                Db, invite::get_invite::GetInviteErr, user::add_user::AddUserErr,
            };

            #[test(tokio::test)]
            async fn get_invite() {
                let db = Db::new::<Mem>(()).await.unwrap();
                let time = Duration::from_nanos(0);
                db.migrate().await.unwrap();
                let invite = db
                    .add_invite(time, "wowza", "hey@hey.com", Duration::from_nanos(0))
                    .await
                    .unwrap();
                trace!("{invite:#?}");
                let invite = db
                    .add_invite(time, "wowza1", "hey@hey.com", Duration::from_nanos(2))
                    .await
                    .unwrap();
                trace!("{invite:#?}");
                let invite = db
                    .add_invite(time, "wowza2", "hey@hey.com", Duration::from_nanos(0))
                    .await
                    .unwrap();
                trace!("{invite:#?}");
                let invite = db.get_invite("hey@hey.com", Duration::from_nanos(1)).await;
                trace!("{invite:#?}");
                assert_eq!(invite.unwrap().token_raw, "wowza1");
                let invite = db.get_invite("hey1@hey.com", Duration::from_nanos(0)).await;
                trace!("{invite:#?}");
                assert!(matches!(invite, Err(GetInviteErr::NotFound)));
            }
        }
    }

    pub mod use_invite {
        use std::time::Duration;

        use chrono::{DateTime, Utc};
        use serde::{Deserialize, Serialize};
        use surrealdb::{Connection, Datetime, RecordId};
        use thiserror::Error;
        use tracing::{error, trace};

        use super::super::Db;

        use super::Invite;

        impl<C: Connection> Db<C> {
            pub async fn use_invite<TokenRaw: Into<String>>(
                &self,
                token_raw: TokenRaw,
                time: Duration,
                // time: DateTime<Utc>,
            ) -> Result<Invite, UseInviteErr> {
                let db = &self.db;
                let token_raw = token_raw.into();
                let time = Datetime::from(chrono::DateTime::from_timestamp_nanos(
                    time.as_nanos() as i64
                ));

                let result = db
                        .query(
                            r#"
                             UPDATE invite SET modified_at = $time, used = true WHERE token_raw = $token_raw;
                        "#,
                        )
                        .bind(("token_raw", token_raw))
                        .bind(("time", time))
                        .await?;
                // .inspect_err(|err| trace!("use_invite query {:#?}", err))?;

                trace!("{:#?}", result);

                let mut result = result.check().map_err(|err| match err {
                    err => {
                        error!("use_invite res {:#?}", err);
                        UseInviteErr::from(err)
                    }
                })?;
                let invite = result
                    .take::<Option<Invite>>(0)?
                    .ok_or(UseInviteErr::NotFound)?;

                // trace!("record created: {invite:#?}");

                Ok(invite)
            }
        }

        #[derive(Debug, Error)]
        pub enum UseInviteErr {
            #[error("DB error {0}")]
            DB(#[from] surrealdb::Error),

            #[error("token not found")]
            NotFound,
        }

        #[cfg(test)]
        mod tests {
            use std::time::Duration;

            use chrono::{DateTime, Utc};
            use surrealdb::engine::local::Mem;
            use test_log::test;
            use tracing::trace;

            use super::super::super::{
                Db,
                invite::{get_invite::GetInviteErr, use_invite::UseInviteErr},
                user::add_user::AddUserErr,
            };

            #[test(tokio::test)]
            async fn use_invite() {
                // let time = DateTime::<Utc>::default();
                let db = Db::new::<Mem>(()).await.unwrap();
                let time = Duration::from_nanos(0);
                db.migrate().await.unwrap();
                let invite = db
                    .add_invite(time, "wowza", "hey@hey.com", Duration::from_nanos(0))
                    .await
                    .unwrap();
                // trace!("{invite:#?}");
                let invite = db.get_invite("hey@hey.com", Duration::from_nanos(0)).await;
                assert!(matches!(invite, Ok(_)));
                db.use_invite("wowza", Duration::from_nanos(0))
                    .await
                    .unwrap();
                let invite = db.get_invite("hey@hey.com", Duration::from_nanos(0)).await;
                assert!(matches!(invite, Err(GetInviteErr::NotFound)));
            }
        }
    }
}

pub mod user {
    use serde::{Deserialize, Serialize};
    use surrealdb::{Datetime, RecordId};

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct User {
        pub id: RecordId,
        pub username: String,
        pub email: String,
        pub password: String,
        pub modified_at: Datetime,
        pub created_at: Datetime,
    }

    pub mod add_user {
        use serde::{Deserialize, Serialize};
        use surrealdb::{Connection, Datetime, RecordId};
        use thiserror::Error;
        use tracing::{error, trace};

        use super::super::Db;

        use super::User;

        impl<C: Connection> Db<C> {
            pub async fn add_user<
                Username: Into<String>,
                Email: Into<String>,
                Password: Into<String>,
            >(
                &self,
                username: Username,
                email: Email,
                password: Password,
            ) -> Result<User, AddUserErr> {
                let db = &self.db;
                let username = username.into();
                let email = email.into();
                let password = password.into();
                trace!("add_user input: username {username} email: {email} password: {password}");
                let result = db
                    .query(
                        r#"
                             CREATE user SET
                                username = $username,
                                email = $email,
                                password = $password;
                        "#,
                    )
                    .bind(("username", username))
                    .bind(("email", email))
                    .bind(("password", password))
                    .await
                    .inspect_err(|err| error!("add_user query {:#?}", err))?;
                trace!("{:#?}", result);
                let mut result = result.check().map_err(|err| match err {
                    surrealdb::Error::Db(surrealdb::error::Db::IndexExists {
                        index,
                        value,
                        ..
                    }) if index == "idx_user_email" => AddUserErr::EmailIsTaken(value),
                    surrealdb::Error::Db(surrealdb::error::Db::IndexExists {
                        index,
                        value,
                        ..
                    }) if index == "idx_user_username" => AddUserErr::UsernameIsTaken(value),
                    err => {
                        error!("add_user res {:#?}", err);
                        err.into()
                    }
                })?;
                let user = result
                    .take::<Option<User>>(0)?
                    .ok_or(AddUserErr::NotFound)?;

                trace!("user created: {user:#?}");

                Ok(user)
            }
        }

        #[derive(Debug, Error)]
        pub enum AddUserErr {
            #[error("DB error {0}")]
            DB(#[from] surrealdb::Error),

            #[error("not found")]
            NotFound,

            #[error("email {0} is taken")]
            EmailIsTaken(String),

            #[error("username {0} is taken")]
            UsernameIsTaken(String),
        }

        #[cfg(test)]
        mod tests {
            use surrealdb::engine::local::Mem;
            use test_log::test;
            use tracing::trace;

            use super::super::super::{Db, user::add_user::AddUserErr};

            #[test(tokio::test)]
            async fn add_user() {
                let db = Db::new::<Mem>(()).await.unwrap();
                db.migrate().await.unwrap();
                let user = db
                    .add_user(
                        "hey".to_string(),
                        "hey@hey.com".to_string(),
                        "hey".to_string(),
                    )
                    .await
                    .unwrap();
                trace!("{user:#?}");

                let user = db
                    .add_user(
                        "hey2".to_string(),
                        "hey@hey.com".to_string(),
                        "hey".to_string(),
                    )
                    .await;
                trace!("{user:#?}");
                assert!(matches!(user, Err(AddUserErr::EmailIsTaken(_))));

                let user = db
                    .add_user(
                        "hey".to_string(),
                        "hey2@hey.com".to_string(),
                        "hey".to_string(),
                    )
                    .await;
                trace!("{user:#?}");
                assert!(matches!(user, Err(AddUserErr::UsernameIsTaken(_))));
            }
            // #[test(tokio::test)]
            // async fn test_add_session() {
            //     let db = Db::new::<Mem>(()).await.unwrap();
            //     db.migrate().await.unwrap();
            //
            //     let _user = db
            //         .add_user(
            //             "hey".to_string(),
            //             "hey@hey.com".to_string(),
            //             "hey".to_string(),
            //         )
            //         .await
            //         .unwrap();
            //
            //     let session = db.add_session("token", "hey").await;
            //     trace!("session: {session:?}");
            //     assert!(session.is_ok());
            //
            //     let session = db.add_session("token", "hey").await;
            //     trace!("session: {session:?}");
            //     assert!(session.is_err());
            // }
        }
    }

    pub mod get_user_by_username {
        use surrealdb::Connection;
        use tracing::{error, trace};

        use super::super::Db;
        use thiserror::Error;

        use super::User;

        impl<C: Connection> Db<C> {
            pub async fn get_user_by_username<Username: Into<String>>(
                &self,
                username: Username,
            ) -> Result<User, GetUserByUsernameErr> {
                let db = &self.db;
                let username = username.into();

                let mut result = db
                    .query(
                        r#"
                            SELECT * FROM user WHERE username = $username;
                        "#,
                    )
                    .bind(("username", username))
                    .await
                    .inspect_err(|err| error!("get user by username error: {err}"))
                    .inspect(|e| trace!("result {e:#?}"))?
                    .check()
                    .inspect_err(|err| error!("get user by username check error: {err}"))?;
                result
                    .take::<Option<User>>(0)
                    .inspect_err(|err| error!("unexpected err {err}"))?
                    .ok_or(GetUserByUsernameErr::UserNotFound)
            }
        }

        #[derive(Debug, Error)]
        pub enum GetUserByUsernameErr {
            #[error("DB error {0}")]
            DB(#[from] surrealdb::Error),

            #[error("user not found")]
            UserNotFound,
        }

        #[cfg(test)]
        pub mod db {
            use surrealdb::{Connection, engine::local::Mem};
            use tracing::trace;

            use super::super::super::{
                Db,
                user::{
                    get_user_by_email::GetUserByEmailErr,
                    get_user_by_username::GetUserByUsernameErr,
                },
            };
            use test_log::test;
            use thiserror::Error;

            use super::User;

            #[test(tokio::test)]
            async fn get_user_by_email() {
                let db = Db::new::<Mem>(()).await.unwrap();
                db.migrate().await.unwrap();
                let user = db.add_user("hey", "hey@hey.com", "hey").await.unwrap();
                let user = db.get_user_by_username("hey").await.unwrap();
                trace!("found {user:#?}");
                let user = db.get_user_by_username("hey2").await;
                assert!(matches!(user, Err(GetUserByUsernameErr::UserNotFound)));
            }
        }
    }

    pub mod get_user_by_email {
        use surrealdb::Connection;

        use super::super::Db;
        use thiserror::Error;

        use super::User;

        impl<C: Connection> Db<C> {
            pub async fn get_user_by_email<S: Into<String>>(
                &self,
                email: S,
            ) -> Result<User, GetUserByEmailErr> {
                let db = &self.db;
                let email = email.into();

                let mut result = db
                    .query(
                        r#"
                            SELECT * FROM user WHERE email = $email;
                        "#,
                    )
                    .bind(("email", email))
                    .await?;
                result
                    .take::<Option<User>>(0)?
                    .ok_or(GetUserByEmailErr::UserNotFound)
            }
        }

        #[derive(Debug, Error)]
        pub enum GetUserByEmailErr {
            #[error("DB error {0}")]
            DB(#[from] surrealdb::Error),

            #[error("user not found")]
            UserNotFound,
        }

        #[cfg(test)]
        pub mod tests {
            use surrealdb::{Connection, engine::local::Mem};
            use tracing::trace;

            use super::super::super::{Db, user::get_user_by_email::GetUserByEmailErr};
            use test_log::test;
            use thiserror::Error;

            use super::User;

            #[test(tokio::test)]
            async fn get_user_by_email() {
                let db = Db::new::<Mem>(()).await.unwrap();
                db.migrate().await.unwrap();
                let user = db.add_user("hey", "hey@hey.com", "hey").await.unwrap();
                let user = db.get_user_by_email("hey@hey.com").await.unwrap();
                trace!("found {user:#?}");
                let user = db.get_user_by_email("hey2@hey.com").await;
                assert!(matches!(user, Err(GetUserByEmailErr::UserNotFound)));
            }
        }
    }

    pub mod get_user_password_hash {
        use surrealdb::Connection;
        use tracing::trace;

        use super::super::Db;
        use thiserror::Error;

        use super::User;

        impl<C: Connection> Db<C> {
            pub async fn get_user_password<S: Into<String>>(
                &self,
                email: S,
            ) -> Result<String, GetUserPasswordErr> {
                let db = &self.db;
                let email = email.into();
                let mut result = db
                    .query(
                        r#"
                            (SELECT password FROM user WHERE email = $email).password
                        "#,
                    )
                    .bind(("email", email))
                    .await?;

                let password = result
                    .take::<Option<String>>(0)?
                    .ok_or(GetUserPasswordErr::UserNotFound)?;
                trace!("result: {password}");
                Ok(password)
            }
        }

        #[derive(Debug, Error)]
        pub enum GetUserPasswordErr {
            #[error("DB error {0}")]
            DB(#[from] surrealdb::Error),

            #[error("user not found")]
            UserNotFound,
        }

        #[cfg(test)]
        pub mod tests {
            use surrealdb::{Connection, engine::local::Mem};
            use tracing::trace;

            use super::super::super::{
                Db,
                user::{
                    get_user_by_email::GetUserByEmailErr,
                    get_user_password_hash::GetUserPasswordErr,
                },
            };
            use test_log::test;
            use thiserror::Error;

            use super::User;

            #[test(tokio::test)]
            async fn get_user_password() {
                let db = Db::new::<Mem>(()).await.unwrap();
                db.migrate().await.unwrap();
                let user = db.add_user("hey", "hey@hey.com", "123").await.unwrap();
                let user = db.get_user_password("hey@hey.com").await.unwrap();
                trace!("found {user:#?}");
                let user = db.get_user_password("hey2@hey.com").await;
                assert!(matches!(user, Err(GetUserPasswordErr::UserNotFound)));
            }
        }
    }
}

pub mod session {
    use serde::{Deserialize, Serialize};
    use surrealdb::{Datetime, RecordId};

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Session {
        pub id: RecordId,
        pub access_token: String,
        pub user_id: RecordId,
        pub modified_at: Datetime,
        pub created_at: Datetime,
    }

    pub mod add_session {
        use surrealdb::Connection;
        use thiserror::Error;
        use tracing::{error, trace};

        use super::super::Db;

        use super::Session;

        impl<C: Connection> Db<C> {
            pub async fn add_session(
                &self,
                token: impl Into<String>,
                username: impl Into<String>,
            ) -> Result<Session, AddSessionErr> {
                let db = &self.db;
                let token: String = token.into();
                let username: String = username.into();
                let mut result = db
                    .query(
                        r#"
                             LET $user_id = SELECT id FROM ONLY user WHERE username = $username;
                             CREATE session SET access_token = $access_token, user_id = $user_id.id;
                        "#,
                    )
                    .bind(("access_token", token))
                    .bind(("username", username.clone()))
                    .await
                    .inspect_err(|err| error!("err {err}"))
                    .inspect(|result| trace!("result: {result:#?}"))?
                    .check()
                    .map_err(|err| match err {
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
                                    .map(|f| f == ".user_id")
                                    .unwrap_or_default() =>
                        {
                            AddSessionErr::UserNotFound(username)
                        }
                        surrealdb::Error::Db(surrealdb::error::Db::IndexExists {
                            index, ..
                        }) if index == "idx_session_access_token" => AddSessionErr::TokenExists,
                        err => {
                            error!("unexpected error {err}");
                            err.into()
                        }
                    })?;

                let session = result
                    .take::<Option<Session>>(1)
                    .inspect_err(|err| error!("add session error: {err}"))?
                    .expect("session was just created");

                Ok(session)
            }
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
        #[cfg(test)]
        pub mod db {
            use surrealdb::{Connection, engine::local::Mem};
            use tracing::trace;

            use super::super::super::{
                Db, session::add_session::AddSessionErr, user::get_user_by_email::GetUserByEmailErr,
            };
            use test_log::test;
            use thiserror::Error;

            #[test(tokio::test)]
            async fn add_session() {
                let db = Db::new::<Mem>(()).await.unwrap();
                db.migrate().await.unwrap();
                let user = db.add_user("hey", "hey@hey.com", "hey").await.unwrap();
                trace!("created {user:#?}");
                let session = db.add_session("token", "hey").await;
                trace!("session: {session:?}");
                assert!(session.is_ok());

                let session = db.add_session("token", "hey").await;
                trace!("session: {session:?}");
                assert!(matches!(session, Err(AddSessionErr::TokenExists)));

                let session = db.add_session("token", "hey2").await;
                trace!("session: {session:?}");
                assert!(matches!(session, Err(AddSessionErr::UserNotFound(_))));
            }
        }
    }

    pub mod delete_session {
        use surrealdb::Connection;
        use thiserror::Error;
        use tracing::{error, trace};

        use super::super::Db;

        use super::Session;

        impl<C: Connection> Db<C> {
            pub async fn delete_session<S: Into<String>>(
                &self,
                token: S,
            ) -> Result<(), DeleteSessionErr> {
                let db = &self.db;
                let token: String = token.into();
                let result = db
                    .query(
                        r#"
                             DELETE session WHERE access_token = $access_token;
                        "#,
                    )
                    .bind(("access_token", token))
                    .await
                    .inspect_err(|err| error!("delete session error: {err}"))
                    .inspect(|result| trace!("result: {result:#?}"))?
                    .check()
                    .inspect_err(|err| error!("unexpected error: {err}"))?;

                Ok(())
            }
        }
        #[derive(Debug, Error)]
        pub enum DeleteSessionErr {
            #[error("DB error {0}")]
            DB(#[from] surrealdb::Error),
            // #[error("not found")]
            // NotFound,
        }
        #[cfg(test)]
        mod db {
            use surrealdb::{Connection, engine::local::Mem};
            use tracing::trace;

            use super::super::super::{
                Db, session::delete_session::DeleteSessionErr,
                user::get_user_by_email::GetUserByEmailErr,
            };
            use test_log::test;
            use thiserror::Error;

            #[test(tokio::test)]
            async fn delete_session() {
                let db = Db::new::<Mem>(()).await.unwrap();
                db.migrate().await.unwrap();

                let _user = db
                    .add_user(
                        "hey".to_string(),
                        "hey@hey.com".to_string(),
                        "hey".to_string(),
                    )
                    .await
                    .unwrap();

                let session = db.add_session("token", "hey").await;
                trace!("session: {session:?}");
                assert!(session.is_ok());

                let session = db.delete_session("token").await;
                trace!("session: {session:?}");
                assert!(session.is_ok());

                let session = db.get_session("token").await;
                trace!("session: {session:?}");
                assert!(session.is_err());

                // let session = db.delete_session("token").await;
                // trace!("session: {session:?}");
                // assert!(matches!(session, Err(DeleteSessionErr::NotFound)));
            }
        }
    }

    pub mod get_session {
        use surrealdb::Connection;
        use thiserror::Error;
        use tracing::{error, trace};

        use super::super::Db;

        use super::Session;

        impl<C: Connection> Db<C> {
            pub async fn get_session<S: Into<String>>(
                &self,
                token: S,
            ) -> Result<Session, GetSessionErr> {
                let db = &self.db;
                let token: String = token.into();
                let mut result = db
                    .query(
                        r#"
                     SELECT * FROM session WHERE access_token = $access_token;
                "#,
                    )
                    .bind(("access_token", token))
                    .await
                    .inspect_err(|err| error!("get_session query {:#?}", err))?;
                trace!("result: {result:#?}");

                // let mut result = result
                //     .check()
                //     .inspect(|result| trace!("result2: {result:#?}"))?;

                let session = result
                    .take::<Option<Session>>(0)
                    .inspect_err(|err| error!("get session error: {err}"))?
                    .ok_or(GetSessionErr::NotFound)?;

                Ok(session)
            }
        }

        #[derive(Debug, Error)]
        pub enum GetSessionErr {
            #[error("DB error {0}")]
            DB(#[from] surrealdb::Error),

            #[error("not found")]
            NotFound,
        }

        #[cfg(test)]
        mod tests {
            use surrealdb::{Connection, engine::local::Mem};
            use tracing::trace;

            use super::super::super::{
                Db, session::get_session::GetSessionErr, user::get_user_by_email::GetUserByEmailErr,
            };
            use test_log::test;
            use thiserror::Error;

            #[test(tokio::test)]
            async fn get_session() {
                let db = Db::new::<Mem>(()).await.unwrap();
                db.migrate().await.unwrap();

                let _user = db
                    .add_user(
                        "hey".to_string(),
                        "hey@hey.com".to_string(),
                        "hey".to_string(),
                    )
                    .await
                    .unwrap();

                let session = db.get_session("token").await;
                trace!("session: {session:?}");
                assert!(matches!(session, Err(GetSessionErr::NotFound)));

                let session = db.add_session("token", "hey").await;
                trace!("session: {session:?}");
                assert!(session.is_ok());

                let session = db.get_session("token").await;
                trace!("session: {session:?}");
                assert!(session.is_ok());
            }
        }
    }
}
