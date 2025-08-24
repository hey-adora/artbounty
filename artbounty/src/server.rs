use leptos::{logging, prelude::*};
use tracing::trace;

use crate::{
    controller,
    path::{
        PATH_API, PATH_API_INVITE, PATH_API_INVITE_DECODE, PATH_API_LOGIN, PATH_API_LOGOUT,
        PATH_API_POST_ADD, PATH_API_POST_GET_AFTER, PATH_API_PROFILE, PATH_API_REGISTER,
        PATH_API_USER,
    },
};

#[cfg(feature = "ssr")]
pub async fn server() {
    use axum::{
        Router,
        extract::{Multipart, Query, Request, State},
        response::IntoResponse,
        http::Method,
        middleware::{self, Next},
        routing::post,
    };
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tower_http::{
        compression::{CompressionLayer, DefaultPredicate, predicate},
        cors::{self, CorsLayer},
        services::ServeDir,
    };

    use crate::{
        controller::app_state::AppState,
        view::{app::App, shell},
    };

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

    let api_router = create_api_router().with_state(app_state.clone());

    let app = Router::new()
        .nest_service("/file", ServeDir::new(&app_state.settings.site.files_path))
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

#[cfg(feature = "ssr")]
pub fn create_api_router() -> axum::Router<crate::server::controller::app_state::AppState> {
    use axum::{
        Router,
        extract::{Multipart, Query, Request, State},
        http::Method,
        middleware::{self, Next},
        routing::post,
    };
    let api_router = Router::new()
        .route(PATH_API_LOGIN, post(controller::auth::route::login::server))
        .route(
            PATH_API_REGISTER,
            post(controller::auth::route::register::server),
        )
        .route(
            PATH_API_INVITE_DECODE,
            post(controller::auth::route::invite_decode::server),
        )
        .route(PATH_API_INVITE, post(controller::auth::route::invite::server))
        .route(PATH_API_PROFILE, post(controller::auth::route::profile::server))
        .route(PATH_API_USER, post(controller::auth::route::user::server))
        .route(PATH_API_LOGOUT, post(controller::auth::route::logout::server))
        .route(PATH_API_POST_ADD, post(controller::post::route::add::server))
        .route(
            PATH_API_POST_GET_AFTER,
            post(controller::post::route::get_after::server),
        );
    Router::new().nest(PATH_API, api_router)
}

// use artbounty_api::app_state::AppState;
// use artbounty_db::db::DbEngine;
// use artbounty_frontend::{app::App, shell};
// use axum::{
//     Router,
//     extract::{Multipart, Query, Request, State},
//     http::Method,
//     middleware::{self, Next},
// };
// use leptos::{logging, prelude::*};
// use leptos_axum::{LeptosRoutes, generate_route_list};
// use tower_http::{
//     compression::{CompressionLayer, DefaultPredicate, predicate},
//     cors::{self, CorsLayer},
//     services::ServeDir,
// };
// use tracing::trace;
//
// #[allow(clippy::needless_return)]
// #[tokio::main]
// async fn main() {
//     tracing_subscriber::fmt()
//         .event_format(
//             tracing_subscriber::fmt::format()
//                 .with_file(true)
//                 .with_line_number(true),
//         )
//         .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
//         .try_init()
//         .unwrap();
//
//     trace!("started!");
//
//     let conf = get_configuration(Some("Cargo.toml")).unwrap();
//     let leptos_options = conf.leptos_options;
//     let addr = leptos_options.site_addr;
//     let routes = generate_route_list(App);
//
//     let comppression_layer = CompressionLayer::new()
//         .br(true)
//         .zstd(true)
//         .gzip(true)
//         .deflate(true)
//         .compress_when(predicate::SizeAbove::new(0));
//     let app_state = AppState::new().await;
//
//     let cors = CorsLayer::new()
//         // allow `GET` and `POST` when accessing the resource
//         .allow_methods([Method::GET, Method::POST])
//         // allow requests from any origin
//         .allow_origin(cors::Any);
//
//     let leptos_router = Router::new()
//         .leptos_routes(&leptos_options, routes, {
//             let leptos_options = leptos_options.clone();
//             move || shell(leptos_options.clone())
//         })
//         .fallback(leptos_axum::file_and_error_handler(shell))
//         .with_state(leptos_options);
//
//     let api_router = artbounty_api::router::new().with_state(app_state.clone());
//
//     let app = Router::new()
//         .nest_service("/file", ServeDir::new(&app_state.settings.site.files_path))
//         .merge(leptos_router)
//         .merge(api_router)
//         .layer(cors)
//         .layer(comppression_layer);
//
//     let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
//     logging::log!("listening on http://{}", &addr);
//     axum::serve(listener, app.into_make_service())
//         .await
//         .unwrap();
// }
//
// pub mod api2 {}
// pub mod middleware2 {
//     pub mod auth {
//         use axum::body::Body;
//         use tracing::trace;
//
//         pub async fn auth(
//             _req: axum::extract::Request,
//             _next: axum::middleware::Next,
//         ) -> axum::response::Response {
//             let r2 = axum::response::Response::builder()
//                 .status(403)
//                 .body(Body::empty())
//                 .unwrap();
//             trace!("hello666");
//
//             r2
//         }
//     }
// }
