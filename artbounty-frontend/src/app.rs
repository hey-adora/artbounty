use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use log::trace;
use page::{home, login, profile, register};

use crate::toolbox::prelude::*;

pub mod components;
pub mod page;

#[derive(Clone, Copy, Default, Debug)]
pub struct GlobalState {
    pub acc: RwSignal<Option<Acc>>,
    pub acc_pending: RwSignal<bool>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            acc_pending: RwSignal::new(true),
            ..Default::default()
        }
    }
    pub fn is_logged_in(&self) -> bool {
        self.acc.with(|v| v.is_some())
    }
    pub fn acc_pending(&self) -> bool {
        self.acc_pending.get()
    }
    pub fn logout(&self) {
        self.acc.set(None);
    }
}

#[derive(Clone, Default, Debug)]
pub struct Acc {
    pub username: String,
}

#[component]
pub fn App() -> impl IntoView {
    provide_context(GlobalState::new());
    let global_state = expect_context::<GlobalState>();
    // let a = 77;

    let api_profile = artbounty_api::auth::api::profile::client.ground();
    // let profile = ServerAction::<api::profile::Profile>::new();
    Effect::new(move || {
        api_profile.dispatch(artbounty_api::auth::api::profile::Input {});
    });

    Effect::new(move || {
        let Some(result) = api_profile.value() else {
            return;
        };
        match result {
            Ok(res) => {
                global_state.acc.set(Some(Acc {
                    username: res.username,
                }));
            }
            Err(err) => {
                trace!("profile err: {err}");
            }
        }
        global_state.acc_pending.set(false);
    });

    view! {
        <Router>
            <Routes fallback=|| "not found">
                <Route path=path!("") view=home::Page />
                <Route path=path!("/u/:username") view=profile::Page />
                <ProtectedRoute path=path!("/login") condition=move||Some(!global_state.is_logged_in()) redirect_path=|| "/" view=login::Page />
                <ProtectedRoute path=path!("/register") condition=move||Some(!global_state.is_logged_in()) redirect_path=|| "/" view=register::Page />
            </Routes>
        </Router>
    }
}
