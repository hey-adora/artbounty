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
            db.connect::<SurrealKv>("db5").await.unwrap();
            // #[cfg(feature = "test")]
            // {
            //     trace!("USING MEM DB");
            //     db.connect::<Mem>(()).await.unwrap();
            // }

            // #[cfg(not(feature = "test"))]
            // {
            //     trace!("USING FILE DB");
            //     db.connect::<SurrealKv>("db5").await.unwrap();
            // }
            // cfg_if! {
            //     if #[cfg(feature = "test")] {
            //         trace!("USING MEM DB");
            //         db.connect::<Mem>(()).await.unwrap();
            //     } else {
            //         trace!("USING FILE DB");
            //         db.connect::<SurrealKv>("db5").await.unwrap();
            //     }
            // }
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
                            DEFINE INDEX idx_stat_country ON TABLE session COLUMNS access_token UNIQUE;

                            CREATE migration SET version = 0;
                        };
                    };

                    SELECT * FROM migration;
                ",
                )
                .await.inspect(|result| trace!("DB RESULT {:#?}", result) )?;
            result
                .check()
                .inspect(|result| trace!("RESULT CHECK {:#?}", result))?;
            Ok(())
        }

        // pub async fn verify_user(&self, email: String, password: String) {
        //     let db = &self.db;
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

        pub async fn get_user_password_hash<S: Into<String>>(
            &self,
            email: S,
        ) -> Result<String, GetUserPasswordErr> {
            let db = &self.db;
            let email = email.into();
            // let mut result = db
            //     .query(
            //         r#"
            //          LET $value = SELECT password FROM user WHERE email = $email;
            //          $value.password
            //     "#,
            //     )
            //     .bind(("email", email))
            //     .await?;
            let mut result = db
                .query(
                    r#"
                    (SELECT password FROM user WHERE email = $email).password
                "#,
                )
                .bind(("email", email))
                .await?;

            // let mut result = result.check().map_err(|err| match err {
            //     surrealdb::Error::Db(surrealdb::error::Db::IndexExists { index, .. })
            //         if index == "idx_session_token" =>
            //     {
            //         AddSessionErr::TokenExists
            //     }
            //     err => err.into(),
            // })?;

            let mut password = result
                .take::<Option<String>>(0)?
                .ok_or(GetUserPasswordErr::UserNotFound)?;
            trace!("result: {password}");
            Ok(password)
        }

        pub async fn add_user(
            &self,
            username: String,
            email: String,
            password: String,
        ) -> Result<User, AddUserErr> {
            let db = &self.db;
            trace!("add_user input: username {username} email: {email} password: {password}");
            // let password = {
            //     let salt = SaltString::generate(&mut OsRng);
            //     let argon2 = Argon2::default();
            //     let password_hash = argon2
            //         .hash_password(password.as_bytes(), &salt)?
            //         .to_string();
            //     password_hash
            // };
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
                // surrealdb::Error::Db(surrealdb::error::Db::FieldValue { value, check, .. })
                //     if check == "string::is::email($value)" =>
                // {
                //     AddUserErr::EmailInvalid(value)
                // }
                // surrealdb::Error::Db(surrealdb::error::Db::FieldValue { value, check, .. })
                //     if check == "string::is::alphanum($value)" =>
                // {
                //     AddUserErr::UsernameInvalid(value)
                // }
                surrealdb::Error::Db(surrealdb::error::Db::IndexExists {
                    index, value, ..
                }) if index == "idx_user_email" => AddUserErr::EmailIsTaken(value),
                surrealdb::Error::Db(surrealdb::error::Db::IndexExists {
                    index, value, ..
                }) if index == "idx_user_username" => AddUserErr::UsernameIsTaken(value),
                err => err.into(),
            })?;
            let mut user = result
                .take::<Option<User>>(0)?
                .ok_or(AddUserErr::NotFound)?;

            Ok(user)
        }

        pub async fn add_session<S: Into<String>>(
            &self,
            token: S,
            username: S,
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
                .bind(("username", username))
                .await?;

            trace!("result: {result:#?}");

            let mut result = result.check().map_err(|err| match err {
                surrealdb::Error::Db(surrealdb::error::Db::IndexExists { index, .. })
                    if index == "idx_session_access_token" =>
                {
                    AddSessionErr::TokenExists
                }
                err => err.into(),
            });

            trace!("result2: {result:#?}");
            let mut result = result?;

            let mut session = result
                .take::<Option<Session>>(1)?
                .ok_or(AddSessionErr::NotFound)?;

            Ok(session)
        }

        pub async fn delete_session<S: Into<String>>(
            &self,
            token: S,
        ) -> Result<(), DeleteSessionErr> {
            let db = &self.db;
            let token: String = token.into();
            let mut result = db
                .query(
                    r#"
                     DELETE session WHERE access_token = $access_token;
                "#,
                )
                .bind(("access_token", token))
                .await?;
            trace!("result: {result:#?}");

            let mut result = result
                .check()
                .inspect(|result| trace!("result2: {result:#?}"))?;
            // trace!("result2: {result:#?}");
            // let mut result = result?;

            // let mut session = result
            //     .take::<Option<Session>>(0)?
            //     .ok_or(DeleteSessionErr::NotFound)?;

            Ok(())
        }

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
                .await?;
            trace!("result: {result:#?}");

            let mut result = result
                .check()
                .inspect(|result| trace!("result2: {result:#?}"))?;

            let mut session = result
                .take::<Option<Session>>(0)?
                .ok_or(GetSessionErr::NotFound)?;

            Ok(session)
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

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Session {
        pub id: RecordId,
        pub access_token: String,
        pub user_id: RecordId,
        pub modified_at: Datetime,
        pub created_at: Datetime,
    }

    #[derive(Debug, Error)]
    pub enum AddUserErr {
        #[error("DB error {0}")]
        DB(#[from] surrealdb::Error),

        #[error("not found")]
        NotFound,

        // #[error("hashing error {0}")]
        // Hash(#[from] password_hash::Error),

        // #[error("invalid email \"{0}\"")]
        // EmailInvalid(String),
        #[error("email {0} is taken")]
        EmailIsTaken(String),

        // #[error("username \"{0}\" is invalid")]
        // UsernameInvalid(String),
        #[error("username {0} is taken")]
        UsernameIsTaken(String),
    }

    #[derive(Debug, Error)]
    pub enum AddSessionErr {
        #[error("DB error {0}")]
        DB(#[from] surrealdb::Error),

        #[error("not found")]
        NotFound,

        #[error("token already exists")]
        TokenExists,
    }

    #[derive(Debug, Error)]
    pub enum GetSessionErr {
        #[error("DB error {0}")]
        DB(#[from] surrealdb::Error),

        #[error("not found")]
        NotFound,
    }

    #[derive(Debug, Error)]
    pub enum DeleteSessionErr {
        #[error("DB error {0}")]
        DB(#[from] surrealdb::Error),

        #[error("not found")]
        NotFound,
    }

    #[derive(Debug, Error)]
    pub enum GetUserPasswordErr {
        #[error("DB error {0}")]
        DB(#[from] surrealdb::Error),

        #[error("user not found")]
        UserNotFound,
    }

    #[derive(Debug, Error)]
    pub enum GetUserByEmailErr {
        #[error("DB error {0}")]
        DB(#[from] surrealdb::Error),

        #[error("user not found")]
        UserNotFound,
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
    use surrealdb::Datetime;
    use surrealdb::engine::local::Mem;
    use test_log::test;
    use tracing::trace;

    // use crate::db::AddUserErr;
    use crate::db::User;
    use crate::db::{AddUserErr, Db};

    // #[test(tokio::test)]
    // async fn test_time() {
    //     let a = Datetime::default();
    //     let b = a.to_string();
    //     // let c: u128 = a.try_into().unwrap();
    //     trace!("{b}");
    //     // let b = RecordI
    // }
    #[test(tokio::test)]
    async fn test_get_user_by_email() {
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
        let user = db.get_user_by_email("hey@hey.com").await;
        trace!("user: {user:?}");
        assert!(user.is_ok());
    }

    #[test(tokio::test)]
    async fn test_get_user_password() {
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
        let password = db.get_user_password_hash("hey@hey.com").await;
        trace!("pss: {password:?}");
        assert!(password.is_ok());
    }

    #[test(tokio::test)]
    async fn test_add_session() {
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

        let session = db.add_session("token", "hey").await;
        trace!("session: {session:?}");
        assert!(session.is_ok());

        let session = db.add_session("token", "hey").await;
        trace!("session: {session:?}");
        assert!(session.is_err());
    }

    #[test(tokio::test)]
    async fn test_delete_session() {
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

        let session = db.add_session("token", "hey").await;
        trace!("session: {session:?}");
        assert!(session.is_ok());

        let session = db.delete_session("token").await;
        trace!("session: {session:?}");
        assert!(session.is_ok());

        let session = db.get_session("token").await;
        trace!("session: {session:?}");
        assert!(session.is_err());
    }

    #[test(tokio::test)]
    async fn test_get_session() {
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

        let session = db.get_session("token").await;
        trace!("session: {session:?}");
        assert!(session.is_err());

        let session = db.add_session("token", "hey").await;
        trace!("session: {session:?}");
        assert!(session.is_ok());

        let session = db.get_session("token").await;
        trace!("session: {session:?}");
        assert!(session.is_ok());
    }

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

        // let user: core::result::Result<User, AddUserErr> = db
        //     .add_user(
        //         "hey2".to_string(),
        //         "heyhey.com".to_string(),
        //         "hey".to_string(),
        //     )
        //     .await;
        // trace!("{user:#?}");
        // assert!(matches!(user, Err(AddUserErr::EmailInvalid(_))));

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

        // let user = db
        //     .add_user(
        //         "hey$%".to_string(),
        //         "hey3@hey.com".to_string(),
        //         "hey".to_string(),
        //     )
        //     .await;
        // trace!("{user:#?}");
        // assert!(matches!(user, Err(AddUserErr::UsernameInvalid(_))));
    }
}
