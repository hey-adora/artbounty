use crate::app::pages::admin::overview::Overview;
use crate::app::pages::admin::ws_live::WsLive;
use crate::app::pages::admin::ws_old::WsOld;
use crate::app::pages::admin::throttle_cached::ThrottleCached;
use crate::app::pages::admin::Admin;
use crate::app::pages::login::Login;
use crate::app::pages::register::Register;
use crate::app::utils::PageUrl;
use global_state::GlobalState;
use artcord_state::global;

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use cfg_if::cfg_if;
use pages::account::Account;
use pages::home::HomePage;
use pages::main_gallery::MainGalleryPage;
use pages::not_found::NotFound;
use pages::user_gallery::UserGalleryPage;
use tracing::debug;
use tracing::{error, trace};

pub mod components;
pub mod global_state;
pub mod pages;
pub mod hooks;
pub mod utils;

#[component]
pub fn App() -> impl IntoView {
  
    provide_meta_context();
    provide_context(GlobalState::new());

    // // #[cfg(feature = "development")]
    // // {
    // //     let debug_ws = WsRuntime::<global::DebugServerMsg, global::DebugClientMsg>::new();
    // //     debug_ws.connect(3001).unwrap();
    // //     let debug_ch = debug_ws.channel().key(0).start();
    // //     debug_ch.recv().start(|msg, _| {
    // //         window().location().reload().unwrap();
    // //     });
    // // }

    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    global_state.ws.connect(3420).unwrap();

    view! {
        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <meta name="description" content="Art Community!"/>
        <meta name="keywords" content="artcord,art,gallery,server,discord,community"/>
        <meta name="twitter:title" content="ArtCord"/>
        <meta name="twitter:description" content="Art Community!"/>
        <meta name="twitter:image" content="/assets/overview.webp"/>
        <meta name="twitter:card" content="summary_large_image"/>
        <meta http-equiv="Cache-Control" content="no-cache, no-store, must-revalidate"/>
        <meta http-equiv="Pragma" content="no-cache"/>
        <meta http-equiv="Expires" content="0"/>

        <Stylesheet id="leptos" href="/pkg/leptos_start5.css"/>
        <Title text="ArtCord"/>
        <Body  class=move || format!("text-low-purple    bg-fixed bg-sword-lady  bg-[right_65%_bottom_0] md:bg-center bg-cover bg-no-repeat  bg-dark-night2 {}", if global_state.nav_open.get() == true { "overflow-hidden w-screen h-[dvh]" } else { "" })  />
        <Router>
                // {
                //     PageUrl::update_current_page_url();
                // }

                <Routes>
                    <Route path=PageUrl::Home view=HomePage/>
                    <Route path=PageUrl::MainGallery view=MainGalleryPage/>
                    <Route path=PageUrl::UserGallery view=UserGalleryPage/>
                    <Route path="/account" view=Account/>
                    <Route path=PageUrl::AdminDash view=Admin >
                        <Route path="" view=Overview/>
                        <Route path=PageUrl::AdminThrottleCached view=ThrottleCached/>
                        <Route path=PageUrl::AdminDashWsLive view=WsLive/>
                        <Route path=PageUrl::AdminDashWsOld view=WsOld/>
                    </Route>
                    <Route path=PageUrl::NotFound view=NotFound/>
                    <ProtectedRoute condition=move || !global_state.auth_is_logged_out() redirect_path="/" path="/login" view=Login/>
                    <ProtectedRoute condition=move || !global_state.auth_is_logged_out() redirect_path="/"  path="/register" view=Register/>
                </Routes>
        </Router>
    }
}
