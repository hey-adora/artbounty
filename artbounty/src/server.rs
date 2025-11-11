use leptos::{logging, prelude::*};
use tracing::trace;

use crate::path::{
    PATH_API, PATH_API_ACC, PATH_API_INVITE_DECODE, PATH_API_LOGIN, PATH_API_LOGOUT,
    PATH_API_POST_ADD, PATH_API_POST_GET_OLDER, PATH_API_REGISTER, PATH_API_SEND_EMAIL_INVITE,
    PATH_API_USER,
};

#[cfg(feature = "ssr")]
pub async fn server() {
    use axum::{
        Router,
        extract::{Multipart, Query, Request, State},
        http::Method,
        middleware::{self, Next},
        response::IntoResponse,
        routing::post,
    };
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tower_http::{
        compression::{CompressionLayer, DefaultPredicate, predicate},
        cors::{self, CorsLayer},
        services::ServeDir,
    };

    use crate::{
        api::{
            app_state::{self, AppState},
            clock::get_timestamp,
        },
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

    let time = get_timestamp().as_nanos();

    let comppression_layer = CompressionLayer::new()
        .br(true)
        .zstd(true)
        .gzip(true)
        .deflate(true)
        .compress_when(predicate::SizeAbove::new(0));
    let app_state = AppState::new(time).await;
    let file_path = app_state.get_file_path().await;

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

    let api_router = create_api_router(app_state.clone()).with_state(app_state.clone());

    let app = Router::new()
        .nest_service("/file", ServeDir::new(&file_path))
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
pub fn create_api_router(
    app_state: crate::api::app_state::AppState,
) -> axum::Router<crate::api::app_state::AppState> {
    use axum::{
        Router,
        extract::{Multipart, Query, Request, State},
        http::Method,
        middleware::{self, Next},
        routing::post,
    };

    use crate::{
        api::{self, backend::auth_middleware},
        path::{
            PATH_API_CANCEL_EMAIL_CHANGE, PATH_API_CHANGE_EMAIL, PATH_API_CHANGE_EMAIL_STATUS,
            PATH_API_CHANGE_USERNAME, PATH_API_CONFIRM_EMAIL_CHANGE, PATH_API_CONFIRM_EMAIL_NEW,
            PATH_API_POST_GET, PATH_API_POST_GET_NEWER, PATH_API_POST_GET_NEWER_OR_EQUAL,
            PATH_API_POST_GET_OLDER_OR_EQUAL, PATH_API_RESEND_EMAIL_CHANGE,
            PATH_API_RESEND_EMAIL_NEW, PATH_API_SEND_EMAIL_CHANGE, PATH_API_SEND_EMAIL_NEW,
            PATH_API_USER_POST_GET_NEWER, PATH_API_USER_POST_GET_NEWER_OR_EQUAL,
            PATH_API_USER_POST_GET_OLDER, PATH_API_USER_POST_GET_OLDER_OR_EQUAL,
        },
    };

    // use crate::api::{self, auth_middleware};
    let api_router_public = Router::new()
        .route(PATH_API_LOGIN, post(api::backend::login))
        .route(PATH_API_REGISTER, post(api::backend::register))
        .route(
            PATH_API_INVITE_DECODE,
            post(api::backend::decode_email_token),
        )
        .route(
            PATH_API_SEND_EMAIL_INVITE,
            post(api::backend::send_email_invite),
        )
        .route(PATH_API_USER, post(api::backend::get_user))
        .route(PATH_API_LOGOUT, post(api::backend::logout))
        .route(PATH_API_POST_GET, post(api::backend::get_post))
        .route(PATH_API_POST_GET_OLDER, post(api::backend::get_posts_older))
        .route(PATH_API_POST_GET_NEWER, post(api::backend::get_posts_newer))
        .route(
            PATH_API_POST_GET_OLDER_OR_EQUAL,
            post(api::backend::get_posts_older_or_equal),
        )
        .route(
            PATH_API_POST_GET_NEWER_OR_EQUAL,
            post(api::backend::get_posts_newer_or_equal),
        )
        .route(
            PATH_API_USER_POST_GET_OLDER,
            post(api::backend::get_posts_older_for_user),
        )
        .route(
            PATH_API_USER_POST_GET_NEWER,
            post(api::backend::get_posts_newer_for_user),
        )
        .route(
            PATH_API_USER_POST_GET_OLDER_OR_EQUAL,
            post(api::backend::get_posts_older_or_equal_for_user),
        )
        .route(
            PATH_API_USER_POST_GET_NEWER_OR_EQUAL,
            post(api::backend::get_posts_newer_or_equal_for_user),
        );
    let api_router_auth = Router::new()
        .route(PATH_API_ACC, post(api::backend::get_account))
        .route(
            PATH_API_CHANGE_USERNAME,
            post(api::backend::change_username),
        )
        .route(PATH_API_CHANGE_EMAIL, post(api::backend::change_email))
        .route(
            PATH_API_RESEND_EMAIL_CHANGE,
            post(api::backend::resend_email_change),
        )
        .route(
            PATH_API_RESEND_EMAIL_NEW,
            post(api::backend::resend_email_new),
        )
        .route(
            PATH_API_SEND_EMAIL_CHANGE,
            post(api::backend::send_email_change),
        )
        .route(PATH_API_SEND_EMAIL_NEW, post(api::backend::send_email_new))
        .route(
            PATH_API_CHANGE_EMAIL_STATUS,
            post(api::backend::status_email_change),
        )
        // .route(PATH_API_CHANGE_EMAIL, post(api::backend::change_email))
        .route(
            PATH_API_CANCEL_EMAIL_CHANGE,
            post(api::backend::cancel_email_change),
        )
        .route(
            PATH_API_CONFIRM_EMAIL_CHANGE,
            post(api::backend::confirm_email_change),
        )
        .route(
            PATH_API_CONFIRM_EMAIL_NEW,
            post(api::backend::confirm_email_new),
        )
        // .route(
        //     PATH_API_EMAIL_CHANGE,
        //     post(api::backend::send_email_new),
        // )
        .route(PATH_API_POST_ADD, post(api::backend::add_post))
        .route_layer(middleware::from_fn_with_state(app_state, auth_middleware));
    let api_router = Router::new()
        .merge(api_router_public)
        .merge(api_router_auth);
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
