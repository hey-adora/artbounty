pub mod profile {}
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

    use artbounty_api::api;
    use artbounty_shared::auth::{proccess_email, proccess_password, proccess_username};
    use leptos::tachys::reactive_graph::bind::GetValue;
    use leptos::{html::Input, prelude::*};
    use web_sys::SubmitEvent;
    use crate::toolbox::prelude::*;

    use crate::app::components::nav::Nav;
    #[component]
    pub fn Page() -> impl IntoView {
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
        let api_invite = artbounty_api::auth::api::invite::client.ground();
        let api_register = artbounty_api::auth::api::register::client.ground();
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

        let on_register = move |e: SubmitEvent| {
            e.prevent_default();
            let (Some(username), Some(email), Some(password), Some(password_confirmation)) = (
                register_username.get(),
                register_email.get(),
                register_password.get(),
                register_password_confirmation.get(),
            ) else {
                return;
            };

            let username = proccess_username(username.value());
            let email = proccess_email(email.value());
            let password = proccess_password(password.value(), Some(password_confirmation.value()));

            register_username_err.set(username.clone().err().unwrap_or_default());
            register_email_err.set(email.clone().err().unwrap_or_default());
            register_password_err.set(password.clone().err().unwrap_or_default());
            register_general_err.set(String::new());

            let (Ok(username), Ok(email), Ok(password)) = (username, email, password) else {
                return;
            };

            // todo!("create register dispatch");

            // api_register.dispatch(artbounty_api::auth::api::register::Input {
            //     email,
            //     password,
            //     username,
            // });

            // trace!("register dispatched");
            // register.dispatch(api::register::Register {
            //     email,
            //     password,
            //     username,
            // });
        };

        // Effect::new(move || {
        //     let result = register.value();
        //     let Some(result) = result.get() else {
        //         trace!("does anything work?");
        //         return;
        //     };
        //     trace!("no");
        //     match result {
        //         Ok(_) => {
        //             trace!("ok???");
        //             //
        //         }
        //         Err(ServerFnError::WrappedServerError(
        //             api::register::RegisterErr::EmailInvalid(err),
        //         )) => {
        //             email_err.set(err);
        //         }
        //         Err(ServerFnError::WrappedServerError(api::register::RegisterErr::EmailTaken)) => {
        //             email_err.set("email is taken".to_string());
        //         }
        //         Err(ServerFnError::WrappedServerError(
        //             api::register::RegisterErr::UsernameInvalid(err),
        //         )) => {
        //             username_err.set(err);
        //         }
        //         Err(ServerFnError::WrappedServerError(
        //             api::register::RegisterErr::PasswordInvalid(err),
        //         )) => {
        //             password_err.set(err);
        //         }
        //         Err(ServerFnError::WrappedServerError(
        //             api::register::RegisterErr::UsernameTaken,
        //         )) => {
        //             username_err.set("username is taken".to_string());
        //         }
        //         Err(_) => {
        //             general_err.set("serevr err".to_string());
        //         }
        //     }
        // });

        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] min-h-[100dvh]">
                <Nav/>
                <div class=move || format!("grid  text-white {}", if api_register.is_pending() || api_register.is_complete() {"items-center"} else {"justify-stretch"})>
                    <div class=move||format!("mx-auto text-[1.5rem] {}", if api_register.is_pending() {""} else {"hidden"})>
                        <h1>"LOADING..."</h1>
                    </div>
                    <div class=move||format!("mx-auto flex flex-col gap-2 text-center {}", if api_register.is_complete() {""} else {"hidden"})>
                        <h1 class="text-[1.5rem]">"VERIFY EMAIL"</h1>
                        <p class="max-w-[30rem]">"Verification email was sent to \""{move || register_username.get().map(|v| v.value()).unwrap_or(String::from("error"))}"\" click the confirmtion link in the email."</p>
                        <a href="/login" class="underline">"Go to Login"</a>
                    </div>
                    <form method="POST" action="" on:submit=on_register class=move || format!("flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full {}", if api_register.is_pending() || api_register.is_complete() {"hidden"} else {""})>
                        <h1 class="text-[1.5rem]  text-center my-[4rem]">"REGISTRATION"</h1>
                        <div class=move||format!("text-red-600 {}", if register_general_err.with(|v| v.is_empty()) {"hidden"} else {""})>{move || { register_general_err.get() }}</div>
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
                                <input placeholder="alice@mail.com" id="email" node_ref=register_email type="text" class="border-b-2 border-white w-full mt-1 " />
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
                            <a href="/login" class="underline">"or Login"</a>
                        </div>
                    </form>
                </div>
            </main>
        }
    }
}

pub mod login {
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
        let main_ref = NodeRef::new();
        let input_email: NodeRef<Input> = NodeRef::new();
        let input_password: NodeRef<Input> = NodeRef::new();
        let general_err = RwSignal::new(String::new());
        let email_err = RwSignal::new(String::new());

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
                Err(ResErr::ClientErr(err)) => {
                    general_err.set(format!("Error sending request \"{err}\"."));
                }
                Err(ResErr::ServerErr(artbounty_api::auth::api::login::ServerErr::Incorrect)) => {
                    general_err.set("Email or Password is incorrect.".to_string());
                }
                Err(ResErr::ServerErr(artbounty_api::auth::api::login::ServerErr::ServerErr)) | Err(ResErr::ServerErr(artbounty_api::auth::api::login::ServerErr::CreateCookieErr)) => {
                    general_err.set("Server error.".to_string());
                }
                _ => {
                    general_err.set("Unknown error.".to_string());
                }
            }
        });
        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] min-h-[100dvh]">
                <Nav/>
                <div class=move || format!("grid  text-white {}", if api_login.is_pending() || api_login.is_complete() {"items-center"} else {"justify-stretch"})>
                    <div class=move||format!("mx-auto text-[1.5rem] {}", if api_login.is_pending() {""} else {"hidden"})>
                        <h1>"LOADING..."</h1>
                    </div>
                    <form method="POST" action="" on:submit=on_login class=move || format!("flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full {}", if api_login.is_pending() || api_login.is_complete() {"hidden"} else {""})>
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
