use std::sync::LazyLock;

use artbounty_db::db::{DB, Db};
use artbounty_frontend::{app::App, shell};
use axum::{Router, routing::post};
use leptos::{logging, prelude::*};
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::compression::CompressionLayer;
use tracing::{info, trace, trace_span};

// static DB: LazyLock<Surreal<ws::Client>> = LazyLock::new(Surreal::init);

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

    DB.connect().await;
    DB.migrate().await.unwrap();
    // let db = Db::new_kv().await.unwrap();

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
        // .route("/api/register", post(api::register::create))
        .with_state(leptos_options)
        .layer(comppression_layer);

    // .layer(artbounty_api::middleware::auth::AuthLayer);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

// pub mod api {
//     pub mod register {
//         pub fn create() {}
//     }
// }
