// #![feature(test)]
// #![feature(strict_overflow_ops)]
// #![feature(lazy_get)]
// #![feature(thread_local)]
// extern crate test;

use leptos::prelude::*;
use leptos_meta::MetaTags;
// use server_fn::codec::Rkyv;

// extern crate rustc_lexer;

use app::App;

pub mod app;
pub mod logger;
pub mod toolbox;
pub mod utils;

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
            <body class="bg-gray-950">
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
