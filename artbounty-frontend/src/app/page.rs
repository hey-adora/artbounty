pub mod profile {
    use std::rc::Rc;

    use crate::app::components::{
        gallery::{Gallery, Img},
        nav::Nav,
    };
    use crate::toolbox::prelude::*;
    use artbounty_api::utils::ResErr;
    use leptos::Params;
    use leptos::prelude::*;
    use leptos_router::{hooks::use_params, params::Params};

    use leptos_router::hooks::use_query;
    use tracing::trace;

    #[derive(Params, PartialEq, Clone)]
    pub struct UserParams {
        pub username: Option<String>,
    }

    #[component]
    pub fn Page() -> impl IntoView {
        let main_ref = NodeRef::new();
        let api_user = artbounty_api::auth::api::user::client.ground();
        let param = use_params::<UserParams>();
        let param_username = move || param.read().as_ref().ok().and_then(|v| v.username.clone());
        // let user = RwSignal::<Option<artbounty_api::auth::api::user::ServerOutput>>::new(None);
        // let user = move |callback: fn(&artbounty_api::auth::api::user::ServerOutput) -> String| {
        //     api_user
        //         .inner
        //         .with(|v| {
        //             v.value.as_ref().map(|v| match v {
        //                 Ok(v) => callback(v),
        //                 Err(ResErr::ServerErr(artbounty_api::auth::api::user::ServerErr::NotFound)) =>,
        //                 Err(err) => "err".to_string(),
        //             })
        //         })
        //         .unwrap_or("loading...".to_string())
        // };
        let user_username = RwSignal::new("loading...".to_string());

        Effect::new(move || {
            let Some(username) = param_username() else {
                return;
            };
            api_user.dispatch(artbounty_api::auth::api::user::Input { username });
        });

        Effect::new(move || {
            let result = api_user.value();
            trace!("user received1 {result:?}");
            let Some(result) = result else {
                trace!("user received2 {result:?}");
                return;
            };
            trace!("user received?");

            match result {
                Ok(v) => {
                    user_username.set(v.username);
                }
                Err(ResErr::ServerErr(artbounty_api::auth::api::user::ServerErr::NotFound)) => {
                    user_username.set("User Not Found".to_string());
                }
                Err(err) => {
                    user_username.set(err.to_string());
                }
            }
        });

        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] h-screen">
                <Nav/>
                <div>
                    <h1>{move || user_username.get()}</h1>
                </div>
            </main>
        }
    }
}
pub mod home {
    use std::rc::Rc;

    use crate::app::components::{
        gallery::{Gallery, Img},
        nav::Nav,
    };
    use crate::toolbox::prelude::*;
    use leptos::prelude::*;

    use tracing::trace;

    #[component]
    pub fn Page() -> impl IntoView {
        let main_ref = NodeRef::new();
        let fake_imgs = RwSignal::new(Vec::<Img>::new());

        main_ref.on_file_drop(async |_event, data| {
            for file in data.get_files() {
                let stream = file.get_file_stream()?;
                let mut data = Vec::<u8>::new();
                while let Some(chunk) = stream.get_stream_chunk().await? {
                    chunk.push_to_vec(&mut data);
                }
                let data_str = String::from_utf8_lossy(&data);
                trace!("file: {}", data_str);
            }

            Ok(())
        });

        Effect::new(move || {
            let imgs = Img::rand_vec(200);
            fake_imgs.set(imgs);
        });

        let fetch_init = Rc::new(move |count| -> Vec<Img> {
            trace!("gog1");
            if count == 0 || count > fake_imgs.with(|v| v.len()) {
                Vec::new()
            } else {
                fake_imgs.with(|v| v[..count].to_vec())
            }
        });

        let fetch_bottom = move |count: usize, last_img: Img| -> Vec<Img> {
            trace!("gogbtm");

            fake_imgs
                .with_untracked(|imgs| {
                    imgs.iter()
                        .position(|img| img.id == last_img.id)
                        .and_then(|pos_start| {
                            let len = imgs.len();
                            if len == pos_start + 1 {
                                return None;
                            }
                            let pos_end = pos_start + count;
                            let pos_end = if pos_start + count > len {
                                len
                            } else {
                                pos_end
                            };
                            Some(imgs[pos_start..pos_end].to_vec())
                        })
                })
                .unwrap_or_default()
        };
        let fetch_top = move |count: usize, last_img: Img| -> Vec<Img> {
            trace!("gogtop");

            fake_imgs
                .with_untracked(|imgs| {
                    imgs.iter()
                        .position(|img| img.id == last_img.id)
                        .and_then(|pos_end| {
                            trace!("FETCH_TOP: POS_END {pos_end}");
                            if pos_end == 0 {
                                return None;
                            }
                            let pos_start = pos_end.saturating_sub(count);
                            Some(imgs[pos_start..pos_end].to_vec())
                        })
                })
                .unwrap_or_default()
        };

        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] h-screen">
                <Nav/>
                <Gallery fetch_init fetch_bottom fetch_top />
            </main>
        }
    }
}

pub mod register {

    use crate::app::{Acc, GlobalState};
    use crate::toolbox::prelude::*;
    use artbounty_api::utils::ResErr;
    use artbounty_api::{api, auth};
    use artbounty_shared::auth::{proccess_email, proccess_password, proccess_username};
    use artbounty_shared::fe_router::registration::{self, RegKind};
    use leptos::Params;
    use leptos::tachys::reactive_graph::bind::GetValue;
    use leptos::{html::Input, prelude::*};
    use leptos_router::hooks::use_query;
    use leptos_router::params::Params;
    use tracing::trace;
    use web_sys::SubmitEvent;

    #[derive(Params, PartialEq, Clone)]
    pub struct RegParams {
        pub token: Option<String>,
        pub email: Option<String>,
        pub loading: Option<bool>,
        pub kind: Option<RegKind>,
    }

    // #[derive(Debug, Clone, PartialEq, strum::EnumString, strum::Display)]
    // #[strum(serialize_all = "lowercase")]
    // pub enum RegKind {
    //     Reg,
    //     CheckEmail,
    //     // Loading,
    // }

    // #[derive(Params, PartialEq)]
    // struct InviteParams {
    //     pub email: Option<String>,
    // }

    // #[derive(Params, PartialEq)]
    // struct RegParams2 {
    //     pub token: String,
    // }

    use crate::app::components::nav::Nav;
    #[component]
    pub fn Page() -> impl IntoView {
        let global_state = expect_context::<GlobalState>();
        let main_ref = NodeRef::new();
        let invite_email: NodeRef<Input> = NodeRef::new();
        let register_username: NodeRef<Input> = NodeRef::new();
        let register_email: NodeRef<Input> = NodeRef::new();
        let register_password: NodeRef<Input> = NodeRef::new();
        let register_password_confirmation: NodeRef<Input> = NodeRef::new();
        let register_username_err = RwSignal::new(String::new());
        let register_email_err = RwSignal::new(String::new());
        let register_password_err = RwSignal::new(String::new());
        let register_general_err = RwSignal::new(String::new());
        let invite_general_err = RwSignal::new(String::new());
        let invite_email_err = RwSignal::new(String::new());
        let invite_email: NodeRef<Input> = NodeRef::new();
        let api_invite = artbounty_api::auth::api::invite::client.ground();
        let api_invite_decode = artbounty_api::auth::api::invite_decode::client.ground();
        let api_register = artbounty_api::auth::api::register::client.ground();
        let query = use_query::<RegParams>();
        // let invite_query = use_query::<RegParams>();
        // let query2 = use_query::<RegParams2>();
        let navigate = leptos_router::hooks::use_navigate();

        let get_query_token = move || query.read().as_ref().ok().and_then(|v| v.token.clone());
        let get_query_email = move || query.read().as_ref().ok().and_then(|v| v.email.clone());
        // let get_query_loading = move || query.read().as_ref().ok().and_then(|v| v.loading.clone());
        let get_query_kind = move || query.read().as_ref().ok().and_then(|v| v.kind.clone());
        let query_kind_is_check_email = move || {
            get_query_kind()
                .map(|v| v == RegKind::CheckEmail)
                .unwrap_or_default()
        };
        let query_kind_is_reg = move || {
            get_query_kind()
                .map(|v| v == RegKind::Reg)
                .unwrap_or_default()
        };
        // let query_kind_is_loading = move || get_query_kind().map(|v| v == RegKind::Loading).unwrap_or_default();
        let get_query_email_or_err = move || get_query_email().unwrap_or(String::from("error"));
        let is_loading = move || {
            api_register.is_pending() || api_invite.is_pending() || api_invite_decode.is_pending()
        };
        // let get_invite_decoded_email = move || {
        //     api_invite_decode
        //         .value()
        //         .and_then(|v| v.ok())
        //         .map(|v| v.email)
        // };

        // let has_token2 = move || { query2.read().as_ref().ok().map(|v| v.token.clone()) };
        // Effect::new(move || {
        //     let token = get_token();
        //     // let token2 = has_token2();
        //     trace!("token??? = {:?}", token);
        //     // trace!("token???2 = {:?}", token2);
        // });

        // let api_register2 = api_register.clone();
        // let api_register3 = api_register.clone();
        // let api_register4 = api_register.clone();
        // let api_register5 = api_register.clone();
        // let api_register6 = api_register.clone();
        // let registration_completed = move || false;
        // let registration_pending = move || false;
        // let registration_result = move || Option::<artbounty_api::auth::api::register>::None;
        // let register = artbounty_api::auth::api::login::client.ground();
        // let register = ServerAction::<api::register::Register>::new();
        // let registration_completed = move || {
        //     register
        //         .value()
        //         .with(|v| v.as_ref().map(|v| v.is_ok()))
        //         .unwrap_or_default()
        // };
        // let registration_pending = move || register.pending().get();
        // let registration_result = move || register.value().get().and_then(|v| v.ok());

        let on_invite = move |e: SubmitEvent| {
            e.prevent_default();

            let Some(email) = invite_email.get() else {
                return;
            };

            let email = proccess_email(email.value());

            invite_email_err.set(email.clone().err().unwrap_or_default());
            invite_general_err.set(String::new());

            let Ok(email) = email else {
                return;
            };

            api_invite.dispatch(auth::api::invite::Input { email });
        };
        let on_register = move |e: SubmitEvent| {
            e.prevent_default();
            let (Some(username), Some(password), Some(password_confirmation)) = (
                register_username.get(),
                // register_email.get(),
                register_password.get(),
                register_password_confirmation.get(),
            ) else {
                return;
            };

            let username = proccess_username(username.value());
            // let email = proccess_email(email.value());
            let password = proccess_password(password.value(), Some(password_confirmation.value()));
            let token = get_query_token();

            register_username_err.set(username.clone().err().unwrap_or_default());
            // register_email_err.set(email.clone().err().unwrap_or_default());
            register_password_err.set(password.clone().err().unwrap_or_default());
            register_general_err.set(if token.is_some() {
                String::new()
            } else {
                String::from("token is missing from; invalid link")
            });

            let (Ok(username), Ok(password), Some(token)) = (username, password, token) else {
                return;
            };

            // todo!("create register dispatch");

            api_register.dispatch(artbounty_api::auth::api::register::Input {
                email_token: token,
                password,
                username,
            });

            // trace!("register dispatched");
            // register.dispatch(api::register::Register {
            //     email,
            //     password,
            //     username,
            // });
        };
        Effect::new({
            let navigate = navigate.clone();
            move || {
                let (Some(result), Some(email)) =
                    (api_invite.value(), invite_email.get().map(|v| v.value()))
                else {
                    return;
                };

                match result {
                    Ok(_) => {
                        navigate(&registration::link_check_email(email), Default::default());
                    }
                    // Err(ResErr::ServerErr(artbounty_api::auth::api::invite::ServerErr::))
                    Err(err) => {
                        invite_general_err.set(err.to_string());
                    }
                }
            }
        });

        Effect::new(move || {
            let Some(token) = get_query_token() else {
                return;
            };

            api_invite_decode.dispatch(artbounty_api::auth::api::invite_decode::Input { token });
        });

        //
        Effect::new(move || {
            let Some(result) = api_register.value() else {
                trace!("does anything work?");
                return;
            };
            trace!("no");
            match result {
                Ok(res) => {
                    trace!("ok???");
                    global_state.acc.set(Some(Acc {
                        username: res.username,
                    }));
                    navigate("/", Default::default());
                    //
                }
                Err(ResErr::ServerErr(
                    artbounty_api::auth::api::register::ServerErr::EmailInvalid(err),
                )) => {
                    register_email_err.set(err);
                }
                Err(ResErr::ServerErr(
                    artbounty_api::auth::api::register::ServerErr::EmailTaken,
                )) => {
                    register_email_err.set("email is taken".to_string());
                }
                Err(ResErr::ServerErr(
                    artbounty_api::auth::api::register::ServerErr::UsernameInvalid(err),
                )) => {
                    register_username_err.set(err);
                }
                Err(ResErr::ServerErr(
                    artbounty_api::auth::api::register::ServerErr::PasswordInvalid(err),
                )) => {
                    register_password_err.set(err);
                }
                Err(ResErr::ServerErr(
                    artbounty_api::auth::api::register::ServerErr::UsernameTaken,
                )) => {
                    register_username_err.set("username is taken".to_string());
                }
                Err(err) => {
                    register_general_err.set(err.to_string());
                }
            }
        });

        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] min-h-[100dvh]">
                <Nav/>
                // <div class=move || format!("grid  text-white {}", if api_register.is_pending() || api_register.is_complete() || api_invite.is_complete() || api_invite.is_pending() || get_query_token().is_some() || get_query_email().is_some() {"items-center"} else {"justify-stretch"})>
                <div class=move || format!("grid  text-white {}", if is_loading() {"items-center"} else {"justify-stretch"})>
                    <div class=move||format!("mx-auto text-[1.5rem] {}", if is_loading() {""} else {"hidden"})>
                        <h1>"LOADING..."</h1>
                    </div>
                    <div class=move||format!("mx-auto flex flex-col gap-2 text-center {}", if query_kind_is_check_email() && !is_loading() {""} else {"hidden"})>
                        <h1 class="text-[1.5rem] my-[4rem]">"VERIFY EMAIL"</h1>
                        <p class="max-w-[30rem]">"Verification email was sent to \""{get_query_email_or_err}"\" click the confirmtion link in the email."</p>
                        // <a href="/login" class="underline">"Go to Login"</a>
                    </div>
                    // <form method="POST" action="" on:submit=on_invite class=move || format!("flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full {}", if api_invite.is_pending() || api_invite.is_complete() || get_query_token().is_some() || get_query_email().is_some() {"hidden"} else {""})>
                    <form method="POST" action="" on:submit=on_invite class=move || format!("flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full {}", if get_query_kind().is_none() && !is_loading() {""} else {"hidden"})>
                        <h1 class="text-[1.5rem]  text-center my-[4rem]">"REGISTRATION"</h1>
                        <div class=move||format!("text-red-600 text-center {}", if invite_general_err.with(|v| v.is_empty()) {"hidden"} else {""})>{move || { invite_general_err.get() }}</div>
                        <div class="flex flex-col gap-0">
                            <label for="email" class="text-[1.2rem] ">"Email"</label>
                            <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if invite_email_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
                                <ul class="list-disc ml-[1rem]">
                                    {move || invite_email_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                </ul>
                            </div>
                            <input placeholder="alice@mail.com" id="email" node_ref=invite_email type="text" class="border-b-2 border-white w-full mt-1 " />
                        </div>
                        <div class="flex flex-col gap-[1.3rem] mx-auto my-[4rem] text-center">
                            <input type="submit" value="Register" class="border-2 border-white text-[1.3rem] font-bold px-4 py-1 hover:bg-white hover:text-gray-950"/>
                        </div>
                    </form>
                    <form method="POST" action="" on:submit=on_register class=move || format!("flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full {}", if query_kind_is_reg() && !is_loading() {""} else {"hidden"})>
                        <h1 class="text-[1.5rem]  text-center my-[4rem]">"FINISH REGISTRATION"</h1>
                        <div class=move||format!("text-red-600 text-center {}", if register_general_err.with(|v| v.is_empty()) {"hidden"} else {""})>{move || { register_general_err.get() }}</div>
                        <div class="flex flex-col justify-center gap-[3rem]">
                            <div class="flex flex-col gap-0">
                                <label for="username" class="text-[1.2rem] ">"Username"</label>
                                <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if register_username_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
                                    <ul class="list-disc ml-[1rem]">
                                        {move || register_username_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </div>
                                <input placeholder="Alice" id="username" node_ref=register_username type="text" class="border-b-2 border-white w-full mt-1 " />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="email" class="text-[1.2rem] ">"Email"</label>
                                <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if register_email_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
                                    <ul class="list-disc ml-[1rem]">
                                        {move || register_email_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </div>
                                <input value=move|| api_invite_decode.value().and_then(|v|v.ok()).map(|v|v.email).unwrap_or_default() readonly placeholder="loading..." id="email" node_ref=register_email type="text" class="border-b-2 border-white w-full mt-1 " />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="password" class="text-[1.2rem] ">"Password"</label>
                                <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if register_password_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
                                    <ul class="list-disc ml-[1rem]">
                                        {move || register_password_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </div>
                                <input id="password" node_ref=register_password type="password" class="border-b-2 border-white w-full mt-1 " />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="password_confirmation" class="text-[1.3rem] ">"Password Confirmation"</label>
                                <input id="password_confirmation" node_ref=register_password_confirmation type="password" class="border-b-2 border-white w-full mt-1 " />
                            </div>
                        </div>
                        <div class="flex flex-col gap-[1.3rem] mx-auto my-[4rem] text-center">
                            <input type="submit" value="Register" class="border-2 border-white text-[1.3rem] font-bold px-4 py-1 hover:bg-white hover:text-gray-950"/>
                            // <a href="/login" class="underline">"or Login"</a>
                        </div>
                    </form>
                </div>
            </main>
        }
    }

    #[cfg(test)]
    mod fe {
        use log::trace;
        use pretty_assertions::assert_eq;
        use std::str::FromStr;
        use test_log::test;

        use crate::logger::simple_shell_logger_init;

        use super::RegKind;

        #[test]
        pub fn reg_kind() {
            let kind = RegKind::Reg;
            let kind_s = kind.to_string();
            trace!("kind as str: {kind_s}");
            let kind_b = RegKind::from_str(&kind_s).unwrap();
            assert_eq!(kind, kind_b);
        }
    }
}

pub mod login {
    use crate::app::{Acc, GlobalState};
    use crate::toolbox::prelude::*;
    use crate::{app::components::nav::Nav, toolbox::api::ground};
    use artbounty_api::api;
    use artbounty_api::utils::ResErr;
    use artbounty_shared::auth::proccess_email;
    use leptos::{html::Input, prelude::*};

    use tracing::trace;
    use web_sys::SubmitEvent;

    #[component]
    pub fn Page() -> impl IntoView {
        let global_state = expect_context::<GlobalState>();
        let main_ref = NodeRef::new();
        let input_email: NodeRef<Input> = NodeRef::new();
        let input_password: NodeRef<Input> = NodeRef::new();
        let general_err = RwSignal::new(String::new());
        let email_err = RwSignal::new(String::new());
        let navigate = leptos_router::hooks::use_navigate();

        let api_login = artbounty_api::auth::api::login::client.ground();
        let on_login = move |e: SubmitEvent| {
            e.prevent_default();
            let (Some(email), Some(password)) = (input_email.get(), input_password.get()) else {
                return;
            };

            let email = proccess_email(email.value());
            let password = password.value();
            // let password = proccess_password(password.value(), None); NEVER PUT PASSWORD VERIFICATION ON LOGIN; if password verification rules ever change the old accounts wont be able to login.

            email_err.set(email.clone().err().unwrap_or_default());
            // password_err.set(password.clone().err().unwrap_or_default());
            general_err.set(String::new());

            let Ok(email) = email else {
                return;
            };

            trace!("lohin dispatched");
            api_login.dispatch(artbounty_api::auth::api::login::Input { email, password });
        };
        // let login_completed = {let login = login.clone(); move || login.is_complete()};
        // let login_pending = {let login = login.clone(); move || login.is_pending()};

        Effect::new(move || {
            let Some(result) = api_login.value() else {
                trace!("does anything work?");
                return;
            };
            trace!("received {result:#?}");
            match result {
                Ok(res) => {
                    global_state.acc.set(Some(Acc {
                        username: res.username,
                    }));

                    navigate("/", Default::default());
                }
                Err(ResErr::ClientErr(err)) => {
                    general_err.set(format!("Error sending request \"{err}\"."));
                }
                Err(ResErr::ServerErr(artbounty_api::auth::api::login::ServerErr::Incorrect)) => {
                    general_err.set("Email or Password is incorrect.".to_string());
                }
                Err(ResErr::ServerErr(artbounty_api::auth::api::login::ServerErr::ServerErr))
                | Err(ResErr::ServerErr(
                    artbounty_api::auth::api::login::ServerErr::CreateCookieErr,
                )) => {
                    general_err.set("Server error.".to_string());
                }
                Err(err) => {
                    general_err.set(err.to_string());
                }
            }
        });
        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] min-h-[100dvh]">
                <Nav/>
                <div class=move || format!("grid  text-white {}", if api_login.is_pending() {"items-center"} else {"justify-stretch"})>
                    <div class=move||format!("mx-auto text-[1.5rem] {}", if api_login.is_pending() {""} else {"hidden"})>
                        <h1>"LOADING..."</h1>
                    </div>
                    <form method="POST" action="" on:submit=on_login class=move || format!("flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full {}", if api_login.is_pending() || api_login.is_succ() {"hidden"} else {""})>
                        <h1 class="text-[1.5rem]  text-center my-[4rem]">"LOGIN"</h1>
                        <div class=move||format!("text-red-600 {}", if general_err.with(|v| v.is_empty()) {"hidden"} else {""})>{move || { general_err.get() }}</div>
                        <div class="flex flex-col justify-center gap-[3rem]">
                            <div class="flex flex-col gap-0">
                                <label for="email" class="text-[1.2rem] ">"Email"</label>
                                <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if email_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
                                    <ul class="list-disc ml-[1rem]">
                                        {move || email_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </div>
                                <input placeholder="alice@mail.com" id="email" node_ref=input_email type="email" class="border-b-2 border-white" />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="password" class="text-[1.2rem] ">"Password"</label>
                                // <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if password_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
                                //     <ul class="list-disc ml-[1rem]">
                                //         {move || password_err.get().trim().split("\n").filter(|v| v.len() > 1).into_iter().map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                //     </ul>
                                // </div>
                                <input id="password" node_ref=input_password type="password" class="border-b-2 border-white" />
                            </div>
                        </div>
                        <div class="flex flex-col gap-[1.3rem] mx-auto my-[4rem] text-center">
                            <input type="submit" value="Login" class="border-2 border-white text-[1.3rem] font-bold px-4 py-1 hover:bg-white hover:text-gray-950"/>
                            <a href="/register" class="underline">"or Register"</a>
                        </div>
                    </form>
                </div>
            </main>
        }
    }
}
