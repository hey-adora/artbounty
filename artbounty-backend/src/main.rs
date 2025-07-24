use artbounty_api::{api, app_state::AppState};
use artbounty_db::db::DbEngine;
use artbounty_frontend::{app::App, shell};
use axum::{
    Router,
    extract::{Multipart, Query, State},
    http::Method,
    response::IntoResponse,
    routing::post,
};
use leptos::{logging, prelude::*};
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::{
    compression::CompressionLayer,
    cors::{self, CorsLayer},
};
use tracing::trace;

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

    // DB.connect().await;
    // DB.migrate().await.unwrap();
    // let db = Db::<local::SurrealKv>::new().await.unwrap();
    let db = artbounty_db::db::new_local().await;

    //
    let conf = get_configuration(Some("Cargo.toml")).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let comppression_layer = CompressionLayer::new().zstd(true).gzip(true).deflate(true);
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

    let api_router = Router::new()
        .route(
            artbounty_api::auth::api::login::PATH,
            post(artbounty_api::auth::api::login::server),
        )
        .with_state(app_state);
    // let api2_router = Router::new().route("/api/login", post(async | State(db): State<DbEngine>, m: Multipart| { "".into_response() } )).with_state(db);
    // let api2_router = Router::new().route("/api/login", post(async |m: Query<i32>, State(db): State<DbEngine>| { "" } )).with_state(db);
    // .layer(middleware::from_fn(middleware2::auth::auth));
    // .layer(axum::middleware::map_response(
    //     async |res: axum::http::Response<axum::body::Body>| {
    //         trace!("777");
    //         res
    //     },
    // ));

    let app = Router::new()
        // .leptos_routes(&leptos_options, routes, {
        //     let leptos_options = leptos_options.clone();
        //     move || shell(leptos_options.clone())
        // })
        // .merge(api_router)
        // .fallback(leptos_axum::file_and_error_handler(shell))
        // // .route("/api/register", post(api::register::create))
        // .with_state(leptos_options)
        .merge(leptos_router)
        .merge(api_router)
        // .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(cors)
        .layer(comppression_layer);

    // .layer(artbounty_api::middleware::auth::AuthLayer);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

// pub mod wow {
//     use rkyv::{
//         Archive, Archived, Serialize, access, bytecheck::CheckBytes, rancor::Error, to_bytes,
//     };

//     #[derive(Archive, Serialize)]
//     struct Example {
//         name: String,
//         value: i32,
//     }

//     fn wow() {
//         let value = Example {
//             name: "pi".to_string(),
//             value: 31415926,
//         };

//         let bytes = to_bytes::<Error>(&value).unwrap();
//         let archived = access::<ArchivedExample, Error>(&bytes).unwrap();

//         assert_eq!(archived.name, "pi");
//         assert_eq!(archived.value, 31415926);
//     }
// }
pub mod api2 {}
pub mod middleware2 {
    pub mod auth {
        use axum::body::Body;
        use tracing::trace;

        pub async fn auth(
            _req: axum::extract::Request,
            _next: axum::middleware::Next,
        ) -> axum::response::Response {
            // let response = next.run(req).await;
            // let bob = Body::empty();
            let r2 = axum::response::Response::builder()
                .status(403)
                .body(Body::empty())
                .unwrap();
            trace!("hello666");

            r2
        }

        // pub async fn out(
        //     res: axum::http::Response<axum::body::Body>,
        // ) -> axum::http::Response<axum::body::Body> {
        //     res
        // }
    }
}

// pub mod api {
//     pub mod register {
//         pub fn create() {}
//     }
// }
