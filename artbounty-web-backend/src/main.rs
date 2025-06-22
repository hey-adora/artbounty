use std::sync::LazyLock;

use artbounty_web_frontend::{app::App, shell};
use axum::Router;
use leptos::{logging, prelude::*};
use leptos_axum::{LeptosRoutes, generate_route_list};
use surrealdb::{Surreal, engine::remote::ws};
use tower_http::compression::CompressionLayer;
use tracing::{info, trace, trace_span};

static DB: LazyLock<Surreal<ws::Client>> = LazyLock::new(Surreal::init);

#[allow(clippy::needless_return)]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true),
        )
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .unwrap();

    trace!("started!");

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let comppression_layer = CompressionLayer::new().zstd(true).gzip(true).deflate(true);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options)
        .layer(comppression_layer);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

pub mod db {
    use surrealdb::{Connection, Surreal, engine::local::Mem, opt::IntoEndpoint};
    use thiserror::Error;
    use tracing::trace;

    pub struct DB<C: Connection> {
        pub db: Surreal<C>,
    }

    impl<C: Connection> DB<C> {
        pub async fn new<P>(
            address: impl IntoEndpoint<P, Client = C>,
        ) -> Result<Self, surrealdb::Error> {
            let db = Surreal::new(address).await?;
            Ok(Self { db })
        }
        pub async fn migrate(&self) -> Result<(), surrealdb::Error> {
            let db = &self.db;
            db.use_ns("artbounty").use_db("web").await?;
            // let result = db
            //     .query(
            //         r#"
            //     LET $latest_migration_version = (SELECT * FROM migration ORDER BY version DESC);
            //     RETURN IF $latest_migration_version == 0 {
            //         "on latest"
            //     } ELSE {
            //         DEFINE TABLE migration SCHEMAFULL;
            //         DEFINE FIELD version ON TABLE migration TYPE string;
            //         -- DEFINE FIELD modified_at ON TABLE migration TYPE datetime DEFAULT time::now();
            //         -- DEFINE FIELD created_at ON TABLE migration TYPE datetime DEFAULT time::now();

            //         -- DEFINE INDEX unique_version_index ON TABLE migration COLUMNS version UNIQUE;

            //         CREATE migration SET version = "v0"

            //         -- SELECT * FROM migration
            //     };
            // "#,
            //     )
            //     .await?;
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
                            DEFINE INDEX idx_migration_username ON TABLE user COLUMNS username UNIQUE;
                            DEFINE INDEX idx_migration_email ON TABLE user COLUMNS email UNIQUE;

                            CREATE migration SET version = 0;
                        };
                    };

                    SELECT * FROM migration;
            "#,
                )
                .await?;
            // let result = db
            //     .query(
            //         r#"
            //         DEFINE TABLE migration SCHEMAFULL;
            //         DEFINE FIELD version ON TABLE migration TYPE int;

            //         CREATE migration SET version = 0;
            //         LET $latest_migration = (SELECT * FROM migration ORDER BY version DESC)[0];
            //         $latest_migration
            // "#,
            //     )
            //     .await?;
            trace!("{:#?}", result);
            result.check()?;
            Ok(())
        }

        pub fn add_user(&self) {}
    }

    // #[derive(Debug, Error)]
    //"migration 0 executed"
    // pub enum GeneralError {
    //     #[error("data store disconnected")]
    //     Disconnect(#[from] io::Error),
    // }
}

#[cfg(test)]
mod backend_tests {
    use serde::{Deserialize, Serialize};
    use surrealdb::RecordId;
    use surrealdb::Surreal;

    // For an in memory database
    use surrealdb::engine::local::Mem;
    use test_log::test;
    use tracing::trace;

    use crate::db::DB;

    // #[derive(Debug, Serialize, Deserialize)]
    // struct User<'a> {
    //     username: &'a str,
    //     // name: name<'a>,
    //     // marketing: bool,
    // }
    #[derive(Debug, Serialize, Deserialize)]
    struct User {
        username: String,
        // name: name<'a>,
        // marketing: bool,
    }

    #[test(tokio::test)]
    async fn hello() {
        let db = DB::new::<Mem>(()).await.unwrap();
        db.migrate().await.unwrap();
        // db.migrate().await.unwrap();
        // let db = Surreal::new::<Mem>(()).await.unwrap();
        // trace!("hello");
        // db.use_ns("test").use_db("test").await.unwrap();
        // let mut result = db
        //     .query(
        //         r#"
        //         DEFINE TABLE user SCHEMAFULL;
        //         DEFINE FIELD username ON TABLE user TYPE string;
        //         CREATE user SET username = 'hey';
        //         LET $hello = (SELECT count() FROM migration)[0];
        //         $hello;
        //     "#,
        //     )
        //     .await
        //     .unwrap();
        // // let users: Vec<User> = result.take(3).unwrap();
        // trace!("{:#?}", result);
        // // db.create("user").content(User { username: "Hey" }).await.unwrap();
    }
}
