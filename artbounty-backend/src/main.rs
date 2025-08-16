use artbounty_api::{app_state::AppState};
use artbounty_db::db::DbEngine;
use artbounty_frontend::{app::App, shell};
use axum::{
    Router,
    extract::{Multipart, Query, Request, State},
    http::Method,
    middleware::{self, Next},
};
use leptos::{logging, prelude::*};
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::{
    compression::{CompressionLayer, DefaultPredicate, predicate},
    cors::{self, CorsLayer},
};
use tracing::trace;

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

    let conf = get_configuration(Some("Cargo.toml")).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let comppression_layer = CompressionLayer::new()
        .br(true)
        .zstd(true)
        .gzip(true)
        .deflate(true)
        .compress_when(predicate::SizeAbove::new(0));
    let app_state = AppState::new().await;

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(cors::Any);

    let leptos_router = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let api_router = artbounty_api::router::new().with_state(app_state);

    let app = Router::new()
        .merge(leptos_router)
        .merge(api_router)
        .layer(cors)
        .layer(comppression_layer);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

pub mod api2 {}
pub mod middleware2 {
    pub mod auth {
        use axum::body::Body;
        use tracing::trace;

        pub async fn auth(
            _req: axum::extract::Request,
            _next: axum::middleware::Next,
        ) -> axum::response::Response {
            let r2 = axum::response::Response::builder()
                .status(403)
                .body(Body::empty())
                .unwrap();
            trace!("hello666");

            r2
        }
    }
}
