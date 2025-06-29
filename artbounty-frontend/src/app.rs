use components::gallery::Img;
use indextree::Arena;
use indextree::NodeId;
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use page::{home, login, register};
use reactive_stores::Store;
use tracing::trace;

use crate::toolbox::prelude::*;

pub mod components;
pub mod page;

#[derive(Clone, Default, Debug)]
pub struct GlobalState {
    // imgs: RwSignal<Vec<Img>>,
    // id: RwSignal<usize>,
    // tree: RwSignal<Arena<usize>>,
    // current: RwSignal<Option<NodeId>>,
}

#[component]
pub fn App() -> impl IntoView {
    provide_context(GlobalState::default());

    view! {
        <Router>
            <Routes fallback=|| "not found">
                <Route path=path!("") view=home::Page />
                <Route path=path!("/login") view=login::Page />
                <Route path=path!("/register") view=register::Page />
            </Routes>
        </Router>
    }
}
