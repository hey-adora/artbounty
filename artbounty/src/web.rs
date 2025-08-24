pub mod path {
    use leptos::prelude::*;
    use leptos_router::{OptionalParamSegment, ParamSegment, StaticSegment, WildcardSegment, path};

    pub const PATH_API: &'static str = "/api";
    pub const PATH_HOME: &'static str = "/";
    pub const PATH_HOME_BS: () = path!("/");
    pub const PATH_U_USER: &'static str = "/u/:user";
    pub const PATH_LOGIN: &'static str = "/login";
    pub const PATH_LOGIN_BS: (StaticSegment<&'static str>,) = path!("/login");
    pub const PATH_REGISTER: &'static str = "/register";

    #[derive(Debug, Clone, PartialEq, strum::EnumString, strum::Display)]
    #[strum(serialize_all = "lowercase")]
    pub enum RegKind {
        Reg,
        CheckEmail,
        // Loading,
    }

    pub fn link_check_email<Email: AsRef<str>>(email: Email) -> String {
        format!(
            "{}?kind={}&email={}",
            PATH_REGISTER,
            RegKind::CheckEmail,
            email.as_ref()
        )
    }

    pub fn link_reg<Token: AsRef<str>>(token: Token) -> String {
        format!(
            "{}?kind={}&token={}",
            PATH_REGISTER,
            RegKind::Reg,
            token.as_ref()
        )
    }
}

#[cfg(feature = "ssr")]
pub async fn server() {
    use axum::{
        Router,
        extract::{Multipart, Query, Request, State},
        http::Method,
        middleware::{self, Next},
    };
    use leptos::config::get_configuration;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use log::trace;
    use tower_http::{
        compression::{CompressionLayer, DefaultPredicate, predicate},
        cors::{self, CorsLayer},
        services::ServeDir,
    };
    use tracing::info;

    use crate::{controller::app_state::AppState, view::{app::App, shell}};

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

    let api_router = artbounty_api::router::new().with_state(app_state.clone());

    let app = Router::new()
        .nest_service("/file", ServeDir::new(&app_state.settings.site.files_path))
        .merge(leptos_router)
        .merge(api_router)
        .layer(cors)
        .layer(comppression_layer);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

pub mod router {

    // pub fn link_reg<Token: AsRef<str>>(token: Token) -> String {
    //     format!("{}?kind={}&token={}", PATH, RegKind::Reg, token.as_ref())
    // }

    #[cfg(feature = "ssr")]
    pub fn new() -> axum::Router<crate::app_state::AppState> {
        // tracing_subscriber::fmt()
        //     .event_format(
        //         tracing_subscriber::fmt::format()
        //             .with_file(true)
        //             .with_line_number(true),
        //     )
        //     .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        //     .try_init()
        //     .unwrap();

        use axum::{Router, routing::post};
        let routes = Router::new()
            .route(
                crate::auth::api::login::PATH,
                post(crate::auth::api::login::server),
            )
            .route(
                crate::auth::api::register::PATH,
                post(crate::auth::api::register::server),
            )
            .route(
                crate::auth::api::invite_decode::PATH,
                post(crate::auth::api::invite_decode::server),
            )
            .route(
                crate::auth::api::invite::PATH,
                post(crate::auth::api::invite::server),
            )
            .route(
                crate::auth::api::profile::PATH,
                post(crate::auth::api::profile::server),
            )
            .route(
                crate::auth::api::user::PATH,
                post(crate::auth::api::user::server),
            )
            .route(
                crate::auth::api::logout::PATH,
                post(crate::auth::api::logout::server),
            )
            .route(
                crate::post::api::add::PATH,
                post(crate::post::api::add::server),
            )
            .route(
                crate::post::api::get_after::PATH,
                post(crate::post::api::get_after::server),
            );
        Router::new().nest(API_PATH, routes)
    }
}

pub mod fe_router {
    use leptos::prelude::*;
    use leptos_meta::MetaTags;

    pub fn shell(options: LeptosOptions) -> impl IntoView {
        view! {
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1" />

                    <AutoReload options=options.clone() />
                    <HydrationScripts options />
                    <MetaTags/>

                    <meta name="color-scheme" content="dark light" />
                    <link rel="shortcut icon" type="image/ico" href="/favicon.ico" />
                    <link rel="stylesheet" id="leptos" href="/pkg/artbounty_1.css" />
                </head>
                <body class="bg-main-dark">
                    <App />
                </body>
            </html>
        }
    }

    #[cfg(feature = "hydrate")]
    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn hydrate() {
        console_error_panic_hook::set_once();
        logger::simple_web_logger_init();
        tracing::debug!("yo wtf");
        leptos::mount::hydrate_body(App);
    }
    pub mod home {
        pub const PATH: &'static str = "";
    }

    pub mod user {
        pub const PATH: &'static str = "/u/:user";
    }

    pub mod login {
        pub const PATH: &'static str = "/login";
    }

    pub mod registration {
        pub const PATH: &'static str = "/register";

        #[derive(Debug, Clone, PartialEq, strum::EnumString, strum::Display)]
        #[strum(serialize_all = "lowercase")]
        pub enum RegKind {
            Reg,
            CheckEmail,
            // Loading,
        }

        pub fn link_check_email<Email: AsRef<str>>(email: Email) -> String {
            format!(
                "{}?kind={}&email={}",
                PATH,
                RegKind::CheckEmail,
                email.as_ref()
            )
        }

        pub fn link_reg<Token: AsRef<str>>(token: Token) -> String {
            format!("{}?kind={}&token={}", PATH, RegKind::Reg, token.as_ref())
        }
    }
}
