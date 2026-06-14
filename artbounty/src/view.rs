use std::{cell::RefCell, sync::RwLock};

use leptos;
use leptos::prelude::*;
use leptos_meta::MetaTags;
use std::sync::{Arc, LazyLock};
use wasm_bindgen::prelude::*;

use app::App;

pub mod app;
pub mod logger;
pub mod toolbox;

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

                <meta name="darkreader-lock" />
                <meta name="color-scheme" content="dark light" />
                <link rel="shortcut icon" type="image/ico" href="/favicon.ico" />
                <link rel="stylesheet" id="leptos" href="/pkg/artbounty_1.css" />
            </head>
            <body class="bg-base00 text-base05">
                // <span class="tailwidn_placeholder hidden animate-[glow_1s_linear]"/>
                <App />
            </body>
        </html>
    }
}

// pub static KILLME: Arc<RwSignal<bool>> = Arc::new(RwSignal::new(false));
// pub static KILLME: Arc<RwLock<bool>> = Arc::new(RwLock::new(false));
//

// #[cfg(feature = "testing")]
// #[wasm_bindgen]
// #[derive(Clone, Debug, Default)]
// pub struct DebugStateExport {
//     pub delayed_scroll: Vec<f64>,
// }

// thread_local! {
//
//     pub static KILLME: LazyLock<RwLock<bool>> = LazyLock::new(|| RwLock::new(false));
//     pub static KILLME2: RefCell<bool> = RefCell::new(false);
// }
// pub thread_local!(
// );

#[cfg(feature = "csr")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn csr() {
    console_error_panic_hook::set_once();
    logger::simple_web_logger_init();
    tracing::debug!("yo wtf");
    leptos::mount::mount_to_body(App);

    // let callback = move || {
    //     // use crate::{
    //     //     view::{app::GlobalState, toolbox::prelude::*},
    //     // };
    //     let wtf = KILLME.with(|v| {
    //         let a = *v.read().unwrap();
    //
    //         a
    //     });
    //
    //     // let wtf = KILLME.read().unwrap();
    //     // let global_state = expect_context::<GlobalState>();
    //     // let acc = global_state.acc.get_untracked();
    //     tracing::trace!("wowza {}", wtf);
    //     //
    // };
    // let closure = Closure::<dyn Fn()>::new(callback.clone()).into_js_value();
    //
    // closure
    // leptos::mount::hydrate_body(App);
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    logger::simple_web_logger_init();
    tracing::debug!("yo wtf");
    leptos::mount::hydrate_body(App);
}
