use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use log::error;
use log::trace;
use page::{home, login, post, profile, register};
use tracing::info;

use crate::api::Api;
use crate::api::ApiWeb;
use crate::api::ApiWebpTmp;
use crate::api::ServerErr;
use crate::api::ServerRes;
use crate::path::link_user;
use crate::view::toolbox::prelude::*;

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
    pub fn get_username_untracked(&self) -> Option<String> {
        self.acc.with_untracked(|acc| acc.as_ref().map(|acc| acc.username.clone()))
    }
    pub fn update_auth(&self) {
        let this = self.clone();
        ApiWebpTmp::new()
            .profile()
            .send_web(move |result| async move {
                this.set_auth_from_res(result);
            });
    }
    pub async fn update_auth_now(&self) -> Result<ServerRes, ServerErr> {
        let result = ApiWebpTmp::new().profile().send_native().await;
        self.set_auth_from_res(result.clone());
        result
    }
    pub fn set_auth_from_res(&self, result: Result<ServerRes, ServerErr>) {
        match result {
            Ok(ServerRes::User { username }) => {
                info!("logged in as {username}");
                let r = self.acc.try_set(Some(Acc { username: username }));
                if r.is_some() {
                    error!("global state acc was disposed somehow");
                }
            }
            Ok(res) => {
                error!("expected User, received {res:?}");
            }
            Err(err) => {
                error!("{err}");
            }
        }
        let r = self.acc_pending.try_set(false);
        if r.is_some() {
            error!("global state acc was disposed somehow");
        }
    }
    pub fn is_logged_in(&self) -> bool {
        self.acc.with(|v| v.is_some())
    }
    pub fn acc_pending(&self) -> bool {
        self.acc_pending.get()
    }
    pub fn logout(&self) {
        let api = ApiWebpTmp::new();
        let acc = self.acc;
        api.logout().send_web(move |result| async move {
            match result {
                Ok(_) => {
                    let r = acc.try_set(None);
                    if r.is_some() {
                        error!("global state acc was disposed somehow");
                    }
                }
                Err(err) => error!("logout fail"),
            }
        });
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

    let api = ApiWeb::new();
    // // let profile = ServerAction::<api::profile::Profile>::new();
    Effect::new(move || {
        global_state.update_auth();
    });

    // Effect::new(move || {
    //     let Some(result) = api_profile.value_tracked() else {
    //         return;
    //     };
    //     match result {
    //         Ok(res) => {
    //             global_state.acc.set(Some(Acc {
    //                 username: res.username,
    //             }));
    //         }
    //         Err(err) => {
    //             trace!("profile err: {err}");
    //         }
    //     }
    //     global_state.acc_pending.set(false);
    // });
    let redirect_path = move || link_user(global_state.get_username_untracked().unwrap_or(String::from("/")));

    view! {
        <Router>
            <Routes fallback=|| "not found">
                <Route path=path!("") view=home::Page />
                <Route path=path!("/post") view=post::Page />
                <Route path=path!("/u/:username") view=profile::Page />
                <ProtectedRoute path=path!("/login") condition=move||Some(!global_state.is_logged_in()) redirect_path view=login::Page />
                <ProtectedRoute path=path!("/register") condition=move||Some(!global_state.is_logged_in()) redirect_path view=register::Page />
            </Routes>
        </Router>
    }
}
