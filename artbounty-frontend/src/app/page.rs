pub mod home {
    use std::rc::Rc;

    use crate::{
        app::{
            GlobalState,
            components::{
                gallery::{Gallery, Img},
                nav::Nav,
            },
        },
        toolbox::prelude::*,
    };
    use leptos::prelude::*;
    use reactive_stores::Store;
    use tracing::trace;
    use web_sys::{HtmlDivElement, HtmlElement};

    #[component]
    pub fn Page() -> impl IntoView {
        let main_ref = NodeRef::new();
        let global_state = expect_context::<GlobalState>();
        let fake_imgs = RwSignal::new(Vec::<Img>::new());
        // let imgs = global_state.imgs;

        main_ref.on_file_drop(async |event, data| {
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
    use leptos::{
        html::{Input, div, h1, main},
        prelude::*,
        task::spawn_local,
    };
    use tracing::{debug, trace};
    use web_sys::SubmitEvent;

    use crate::app::components::nav::Nav;
    #[component]
    pub fn Page() -> impl IntoView {
        let main_ref = NodeRef::new();
        let input_username: NodeRef<Input> = NodeRef::new();
        let input_email: NodeRef<Input> = NodeRef::new();
        let input_password: NodeRef<Input> = NodeRef::new();
        let input_password_confirmation: NodeRef<Input> = NodeRef::new();
        let username_err = RwSignal::new(String::new());
        let email_err = RwSignal::new(String::new());
        let password_err = RwSignal::new(String::new());
        // let register = ServerAction::<api::register::Register>::new();
        let on_register = move |e: SubmitEvent| {
            e.prevent_default();
            let (Some(username), Some(email), Some(password), Some(password_confirmation)) = (
                input_username.get(),
                input_email.get(),
                input_password.get(),
                input_password_confirmation.get(),
            ) else {
                return;
            };

            let username = proccess_username(username.value());
            let email = proccess_email(email.value());
            let password = proccess_password(password.value(), password_confirmation.value());

            if let Err(err) = &username {
                username_err.set(err.clone());
            }

            if let Err(err) = &email {
                email_err.set(err.clone());
            }

            if let Err(err) = &password {
                password_err.set(err.clone());
            }

            let (Ok(username), Ok(email), Ok(password)) = (username, email, password) else {
                return;
            };

            // if !username.is_alphanumerc() {}

            // let email = password.value();
            // let password = password.value();
            // let password_confirmation = password_confirmation.value();

            // register.dispatch(api::register::Create {  });
            trace!("oh hello");
            spawn_local(async move {
                let data = api::register::register(username, email, password).await;
                trace!("register result: {data:#?}");
            });
        };
        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] h-screen  ">
                <Nav/>
                <div class="grid place-items-center text-white">
                    <div class="bg-gray-900  flex flex-col gap-4 px-3 py-4">
                        <h1 class="text-2xl font-bold">"Register"</h1>
                        <form method="POST" action="" on:submit=on_register class="flex flex-col gap-2">
                            <div class="flex flex-col gap-0">
                                <label for="username">"Username"</label>
                                <Show when=move || username_err.with(|err|!err.is_empty())>{move || username_err.get()}</Show>
                                <input id="username" node_ref=input_username type="text" class="border-b-2 border-white" />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="email">"Email"</label>
                                <input id="email" node_ref=input_email type="text" class="border-b-2 border-white" />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="password">"Password"</label>
                                <input id="password" node_ref=input_password type="password" class="border-b-2 border-white" />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="password_confirmation">"Password Confirmation"</label>
                                <input id="password_confirmation" node_ref=input_password_confirmation type="password" class="border-b-2 border-white" />
                            </div>
                            <input type="submit" value="Register" class="border-2 border-white mt-2"/>
                        </form>
                    </div>
                </div>
            </main>
        }
    }
}

pub mod login {
    use crate::{
        app::{
            GlobalState,
            components::{gallery::Gallery, nav::Nav},
        },
        toolbox::prelude::*,
    };
    use artbounty_api::api;
    use leptos::{
        html::{Input, div, h1, main},
        prelude::*,
        task::spawn_local,
    };
    use reactive_stores::Store;
    use tracing::{debug, trace};
    use web_sys::{HtmlDivElement, HtmlElement, SubmitEvent};

    #[component]
    pub fn Page() -> impl IntoView {
        let main_ref = NodeRef::new();
        let input_email: NodeRef<Input> = NodeRef::new();
        let input_password: NodeRef<Input> = NodeRef::new();
        // let input_password_confirmation: NodeRef<Input> = NodeRef::new();

        // let data = OnceResource::new(api::register::create());
        let global_state = expect_context::<GlobalState>();
        // let g = Action::new(|username: String, password: String, password_confirm: String| {})
        let register = ServerAction::<api::register::Register>::new();
        let on_login = move |e: SubmitEvent| {
            e.prevent_default();
            let (Some(email), Some(password)) = (input_email.get(), input_password.get()) else {
                return;
            };

            let email = email.value();
            let password = password.value();

            // register.dispatch(api::register::Create {  });
            trace!("oh hello");
            spawn_local(async move {
                let data = api::login::login("hey@hey.com".to_string(), "hey".to_string()).await;
                trace!("register result: {data:#?}");
            });
        };
        // let imgs = global_state.imgs;

        // let wowza = (0..100).map(|i| div().child(i)).collect_view();

        // let v2 = main().class("grid grid-rows-[auto_1fr] h-screen  ").child((
        //     Nav(),
        //     div().class("grid place-items-center text-white").child(
        //         div()
        //             .class("bg-gray-900  flex flex-col gap-4 px-3 py-4")
        //             .child((h1().class("text-2xl font-bold").child("Register"), wowza)),
        //     ),
        // ));
        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] h-screen  ">
                <Nav/>
                <div class="grid place-items-center text-white">
                    <div class="bg-gray-900  flex flex-col gap-4 px-3 py-4">
                        <h1 class="text-2xl font-bold">"Login"</h1>
                        <form method="POST" action="" on:submit=on_login class="flex flex-col gap-2">
                            <div class="flex flex-col gap-0">
                                <label>"Email"</label>
                                <input node_ref=input_email type="email" class="border-b-2 border-white" />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label>"Password"</label>
                                <input node_ref=input_password type="password" class="border-b-2 border-white" />
                            </div>
                            <input type="submit" value="Login" class="border-2 border-white mt-2"/>
                        </form>
                    </div>
                </div>
            </main>
        }
    }

    // pub mod api2 {
    //     pub mod register {
    //         use leptos::{prelude::*, server};

    //         #[server]
    //         pub async fn create() -> Result<usize, ServerFnError> {
    //             Ok(69)
    //         }
    //     }
    // }

    // pub mod api {
    //     pub mod register {
    //         use leptos::{prelude::*, server};

    //         #[server]
    //         pub async fn create() -> Result<usize, ServerFnError> {
    //             Ok(69)
    //         }
    //     }
    // }
}
