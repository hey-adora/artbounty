pub mod db {
    use std::sync::LazyLock;

    use argon2::Argon2;
    use argon2::PasswordHasher;
    use cfg_if::cfg_if;
    use password_hash::{SaltString, rand_core::OsRng};
    use serde::{Deserialize, Serialize};
    use surrealdb::RecordId;
    use surrealdb::engine::local;
    use surrealdb::engine::local::SurrealKv;
    use surrealdb::{Connection, Datetime, Surreal, engine::local::Mem, opt::IntoEndpoint};
    use thiserror::Error;
    use tracing::trace;

    pub static DB: LazyLock<Db<local::Db>> = LazyLock::new(Db::init);
    // pub type DbKv = Db<local::Db>;

    #[derive(Debug, Clone)]
    pub struct Db<C: Connection> {
        pub db: Surreal<C>,
    }

    // impl Db<local::Db> {
    //     pub fn init()
    //     pub async fn new_kv() -> Result<Self, surrealdb::Error> {
    //         Db::new::<SurrealKv>("db5").await
    //     }
    // }

    // async fn hello() {
    //     // use surrealdb::Surreal;
    //     // use surrealdb::engine::local::SurrealKv;
    // }
    // pub trait Db: Connection {
    // }
    impl Db<local::Db> {
        pub async fn connect(&self) {
            // TODO make path as env
            let db = &self.db;
            cfg_if! {
                if #[cfg(test)] {
                    db.connect::<Mem>(()).await.unwrap();
                } else {
                    db.connect::<SurrealKv>("db5").await.unwrap();
                }
            }
            db.use_ns("artbounty").use_db("web").await.unwrap();
        }
    }

    impl<C: Connection> Db<C> {
        fn init() -> Self {
            let db = Surreal::<C>::init();
            Self { db }
            // db.co
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
                            DEFINE FIELD email ON TABLE user TYPE string ASSERT string::is::email($value);
                            DEFINE FIELD password ON TABLE user TYPE string;
                            DEFINE FIELD modified_at ON TABLE user TYPE datetime DEFAULT time::now();
                            DEFINE FIELD created_at ON TABLE user TYPE datetime DEFAULT time::now();
                            DEFINE INDEX idx_user_username ON TABLE user COLUMNS username UNIQUE;
                            DEFINE INDEX idx_user_email ON TABLE user COLUMNS email UNIQUE;

                            CREATE migration SET version = 0;
                        };
                    };

                    SELECT * FROM migration;
                ",
                )
                .await?;
            trace!("{:#?}", result);
            result.check()?;
            Ok(())
        }

        pub async fn add_user(
            &self,
            username: String,
            email: String,
            password: String,
        ) -> Result<User, AddUserErr> {
            let db = &self.db;
            let password = {
                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::default();
                let password_hash = argon2
                    .hash_password(password.as_bytes(), &salt)?
                    .to_string();
                password_hash
            };
            let mut result = db
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
                .await?;
            trace!("{:#?}", result);
            let mut result = result.check().map_err(|err| match err {
                surrealdb::Error::Db(surrealdb::error::Db::FieldValue { value, check, .. })
                    if check == "string::is::email($value)" =>
                {
                    AddUserErr::Email(value)
                }
                surrealdb::Error::Db(surrealdb::error::Db::IndexExists {
                    index, value, ..
                }) if index == "idx_user_email" => AddUserErr::EmailIsTaken(value),
                surrealdb::Error::Db(surrealdb::error::Db::IndexExists {
                    index, value, ..
                }) if index == "idx_user_username" => AddUserErr::UsernameIsTaken(value),
                err => err.into(),
            })?;
            let mut user: Vec<User> = result.take(0)?;
            Ok(user[0].clone())
        }
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct User {
        pub id: RecordId,
        pub username: String,
        pub email: String,
        pub password: String,
        pub modified_at: Datetime,
        pub created_at: Datetime,
        // name: name<'a>,
        // marketing: bool,
    }

    #[derive(Debug, Error)]
    pub enum AddUserErr {
        #[error("DB error {0}")]
        DB(#[from] surrealdb::Error),

        #[error("hashing error {0}")]
        Hash(#[from] password_hash::Error),

        #[error("invalid email {0}")]
        Email(String),

        #[error("email {0} is taken")]
        EmailIsTaken(String),

        #[error("username {0} is taken")]
        UsernameIsTaken(String),
    }
}

#[cfg(test)]
mod database_tests {
    // use rkyv::result::ArchivedResult;
    // use serde::{Deserialize, Serialize};

    // // use pretty_assertions::{assert_eq, assert_ne};
    // use surrealdb::RecordId;
    // use surrealdb::Surreal;

    // // For an in memory database
    use surrealdb::engine::local::Mem;
    use test_log::test;
    use tracing::trace;

    // use crate::db::AddUserErr;
    use crate::db::User;
    use crate::db::{AddUserErr, Db};

    #[test(tokio::test)]
    async fn register() {
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

        let user: core::result::Result<User, AddUserErr> = db
            .add_user(
                "hey2".to_string(),
                "heyhey.com".to_string(),
                "hey".to_string(),
            )
            .await;
        trace!("{user:#?}");
        assert!(matches!(user, Err(AddUserErr::Email(_))));

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
}
