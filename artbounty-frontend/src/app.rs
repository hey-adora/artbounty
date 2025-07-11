use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use page::{home, login, register};


pub mod components;
pub mod page;

#[derive(Clone, Default, Debug)]
pub struct GlobalState {
}

#[component]
pub fn App() -> impl IntoView {
    provide_context(GlobalState::default());
    // let profile = ServerAction::<api::profile::Profile>::new();
    // Effect::new(move || {
    //     profile.dispatch(api::profile::Profile {});
    // });

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
