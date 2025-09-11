pub mod post {
    use std::rc::Rc;

    use crate::api::{Api, ApiWeb, ServerAddPostErr, ServerErr, ServerReqImg};
    use crate::valid::auth::{proccess_post_description, proccess_post_title};
    use crate::view::app::components::nav::Nav;
    use crate::view::toolbox::prelude::*;
    use leptos::prelude::*;
    use leptos::{Params, task::spawn_local};
    use leptos_router::{hooks::use_params, params::Params};

    use leptos_router::hooks::use_query;
    use tracing::{error, trace};
    use web_sys::{HtmlInputElement, HtmlTextAreaElement, SubmitEvent};

    #[derive(Params, PartialEq, Clone)]
    pub struct UserParams {
        pub username: Option<String>,
    }

    #[component]
    pub fn Page() -> impl IntoView {
        let main_ref = NodeRef::new();
        let upload_title = NodeRef::new();
        let upload_title_err = RwSignal::new(String::new());
        let upload_image = NodeRef::new();
        let upload_image_err = RwSignal::new(String::new());
        let upload_description = NodeRef::new();
        let upload_description_err = RwSignal::new(String::new());
        let upload_tags = NodeRef::new();
        let upload_tags_err = RwSignal::new(String::new());
        let upload_general_err = RwSignal::new(String::new());
        let api = ApiWeb::new();
        // let api_post = controller::post::route::add::client.ground();
        let on_upload = move |e: SubmitEvent| {
            e.prevent_default();
            trace!("uploading...");
            let (Some(files), Some(title), Some(description), Some(tags)) = (
                (upload_image.get_untracked() as Option<HtmlInputElement>)
                    .and_then(|f: HtmlInputElement| f.files())
                    .map(|f| f.get_files()),
                upload_title.get_untracked() as Option<HtmlInputElement>,
                upload_description.get_untracked() as Option<HtmlTextAreaElement>,
                upload_tags.get_untracked() as Option<HtmlTextAreaElement>,
            ) else {
                return;
            };

            let title = proccess_post_title(title.value());
            let description = proccess_post_description(description.value());

            upload_title_err.set(title.clone().err().unwrap_or_default());
            upload_description_err.set(description.clone().err().unwrap_or_default());
            // upload_tags_err.set(description.clone().err().unwrap_or_default());
            upload_image_err.set(String::new());
            upload_general_err.set(String::new());
            let (Ok(title), Ok(description)) = (title, description) else {
                return;
            };
            spawn_local(async move {
                let mut files_data = Vec::<ServerReqImg>::new();
                'for_file: for file in files {
                    // let a = file.;
                    let stream = match file.get_file_stream() {
                        Ok(stream) => stream,
                        Err(err) => {
                            error!("error getting file stream \"{err}\"");
                            continue;
                        }
                    };

                    let mut data = Vec::<u8>::new();
                    while let Some(chunk) = match stream.get_stream_chunk().await {
                        Ok(chunk) => chunk,
                        Err(err) => {
                            error!("error getting file stream chunk \"{err}\"");
                            continue 'for_file;
                        }
                    } {
                        chunk.push_to_vec(&mut data);
                    }
                    // let data_str = String::from_utf8_lossy(&data);
                    let path = file.name();
                    // trace!("file: {:02X?}", data);
                    trace!("file: {}", path);
                    files_data.push(ServerReqImg { data, path });
                }
                trace!("files data read");
                api.add_post(title, description, files_data)
                    .send_web(move |res| async move {
                        match res {
                            Ok(_) => {
                                //
                            }
                            Err(ServerErr::ServerAddPostErr(ServerAddPostErr::ServerImgErr(
                                errs,
                            ))) => {
                                let msg = errs
                                    .clone()
                                    .into_iter()
                                    .map(|err| err.err.to_string())
                                    .collect::<Vec<String>>()
                                    .join("\n");
                                let _ = upload_image_err.try_set(msg);
                            }
                            Err(ServerErr::ServerAddPostErr(
                                ServerAddPostErr::ServerDirCreationFailed(err),
                            )) => {
                                let _ = upload_general_err.try_set(err.to_string());
                            }
                            Err(_) => {
                                //
                            }
                        };
                    });
                // api_post.dispatch_and_run(
                //     controller::post::route::add::Input {
                //         title,
                //         description,
                //         files: files_data,
                //     },
                //     move |result| {
                //         let result = result.clone();
                //         async move {
                //             match result {
                //                 Ok(_) => {
                //                     //
                //                 }
                //                 Err(ResErr::ServerErr(
                //                     controller::post::route::add::ServerErr::ImgErrors(errs),
                //                 )) => {
                //                     let msg = errs
                //                         .clone()
                //                         .into_iter()
                //                         .map(|err| err.to_string())
                //                         .collect::<Vec<String>>()
                //                         .join("\n");
                //                     let _ = upload_image_err.try_set(msg);
                //                 }
                //                 Err(ResErr::ServerErr(err)) => {
                //                     let _ = upload_general_err.try_set(err.to_string());
                //                 }
                //                 Err(_) => {
                //                     //
                //                 }
                //             };
                //         }
                //     },
                // );
            });
        };

        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] h-screen">
                <Nav/>
                <div>
                    <div class=move||format!("mx-auto text-[1.5rem] {}", if api.is_pending_tracked() {""} else {"hidden"})>
                        <h1>"LOADING..."</h1>
                    </div>
                    <form method="POST" action="" on:submit=on_upload class=move || format!("flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full {}", if !api.is_pending_tracked() {""} else {"hidden"})>
                        <h1 class="text-[1.5rem]  text-center my-[4rem]">"UPLOAD"</h1>
                        <div class=move||format!("text-red-600 text-center {}", if upload_general_err.with(|v| v.is_empty()) {"hidden"} else {""})>{move || { upload_general_err.get() }}</div>
                        <div class="flex flex-col gap-4">
                            <div class="flex flex-col gap-0">
                                <label for="title" class="text-[1.2rem] ">"Title"</label>
                                <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if upload_title_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
                                    <ul class="list-disc ml-[1rem]">
                                        {move || upload_title_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </div>
                                <input placeholder="Funny looking cat" id="title" name="name" node_ref=upload_title type="text" class="border-b-2 border-white w-full mt-1 " />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="image" class="text-[1.2rem] ">"Images"</label>
                                <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if upload_image_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
                                    <ul class="list-disc ml-[1rem]">
                                        {move || upload_image_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </div>
                                <input type="file" id="image" name="image" node_ref=upload_image multiple />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="description" class="text-[1.2rem] ">"Description"</label>
                                <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if upload_description_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
                                    <ul class="list-disc ml-[1rem]">
                                        {move || upload_description_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </div>
                                <textarea class="border-l-2 border-white pl-2 bg-main-light" node_ref=upload_description id="description" name="description" rows="4" cols="50">""</textarea>
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="tags" class="text-[1.2rem] ">"Tags"</label>
                                <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if upload_tags_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
                                    <ul class="list-disc ml-[1rem]">
                                        {move || upload_tags_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </div>
                                <textarea class="border-l-2 border-white pl-2 bg-main-light" node_ref=upload_tags id="tags" name="tags" rows="1" cols="50">""</textarea>
                            </div>
                        </div>
                        <div class="flex flex-col gap-[1.3rem] mx-auto my-[4rem] text-center">
                            <input type="submit" value="Post" class="border-2 border-white text-[1.3rem] font-bold px-4 py-1 hover:bg-white hover:text-gray-950"/>
                        </div>
                    </form>
                </div>
            </main>
        }
    }
}
pub mod profile {
    use crate::api::Api;
    use crate::api::ApiWeb;
    use crate::api::ServerErr;
    use crate::api::ServerGetUserErr;
    use crate::api::ServerRes;
    use crate::view::app::components::nav::Nav;
    use crate::view::toolbox::prelude::*;
    use leptos::Params;
    use leptos::prelude::*;
    use leptos_router::{hooks::use_params, params::Params};
    use std::rc::Rc;
    use tracing::error;

    use leptos_router::hooks::use_query;
    use tracing::trace;

    #[derive(Params, PartialEq, Clone)]
    pub struct UserParams {
        pub username: Option<String>,
    }

    #[component]
    pub fn Page() -> impl IntoView {
        let main_ref = NodeRef::new();
        // let api_user = controller::auth::route::user::client.ground();
        let api = ApiWeb::new();
        let param = use_params::<UserParams>();
        let param_username = move || param.read().as_ref().ok().and_then(|v| v.username.clone());
        let user_username = RwSignal::new("loading...".to_string());

        Effect::new(move || {
            let Some(username) = param_username() else {
                return;
            };
            api.get_user(username).send_web(move |result| async move {
                match result {
                    Ok(ServerRes::User { username }) => {
                        user_username.set(username);
                    }
                    Ok(res) => {
                        user_username.set(format!("expected Uesr, received {res:?}"));
                        error!("expected Uesr, received {res:?}");
                    }
                    Err(ServerErr::ServerGetUserErr(ServerGetUserErr::NotFound)) => {
                        user_username.set("Not Found".to_string());
                    }
                    Err(err) => {
                        user_username.set(err.to_string());
                        error!("get user err: {err}");
                    }
                }
            });
            // api_user.dispatch(controller::auth::route::user::Input { username });
        });
        //
        // Effect::new(move || {
        //     let result = api_user.value_tracked();
        //     trace!("user received1 {result:?}");
        //     let Some(result) = result else {
        //         trace!("user received2 {result:?}");
        //         return;
        //     };
        //     trace!("user received?");
        //
        //     match result {
        //         Ok(v) => {
        //             user_username.set(v.username);
        //         }
        //         Err(ResErr::ServerErr(controller::auth::route::user::ServerErr::NotFound)) => {
        //             user_username.set("User Not Found".to_string());
        //         }
        //         Err(err) => {
        //             user_username.set(err.to_string());
        //         }
        //     }
        // });

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

    use crate::view::{
        app::components::{
            gallery::{Gallery, Img},
            nav::Nav,
        },
        toolbox::prelude::*,
    };
    use leptos::prelude::*;

    use tracing::trace;

    #[component]
    pub fn Page() -> impl IntoView {
        let main_ref = NodeRef::new();
        let fake_imgs = RwSignal::new(Vec::<Img>::new());

        main_ref.use_file_drop(async |_event, data| {
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

        // let fetch_bottom = async move |count: usize, last_img: Img|  {
        //      {
        //         trace!("gogbtm");
        //
        //         // fake_imgs
        //         //     .with_untracked(|imgs| {
        //         //         imgs.iter()
        //         //             .position(|img| img.id == last_img.id)
        //         //             .and_then(|pos_start| {
        //         //                 let len = imgs.len();
        //         //                 if len == pos_start + 1 {
        //         //                     return None;
        //         //                 }
        //         //                 let pos_end = pos_start + count;
        //         //                 let pos_end = if pos_start + count > len {
        //         //                     len
        //         //                 } else {
        //         //                     pos_end
        //         //                 };
        //         //                 Some(imgs[pos_start..pos_end].to_vec())
        //         //             })
        //         //     })
        //         //     .unwrap_or_default()
        //         Vec::new()
        //     }
        // };
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

        // <AwaitProps<>
        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] h-screen">
                <Nav/>
                <Gallery row_height=250 />
                // <Gallery fetch_init fetch_bottom=|c, img| async {Vec::new()} fetch_top />
            </main>
        }
    }
}

pub mod register {

    use leptos::Params;
    use leptos::tachys::reactive_graph::bind::GetValue;
    use leptos::{html::Input, prelude::*};
    use leptos_router::NavigateOptions;
    use leptos_router::hooks::use_query;
    use leptos_router::params::Params;
    use web_sys::SubmitEvent;

    use crate::api::{Api, ApiWeb, ServerErr, ServerRegistrationErr, ServerRes};
    use crate::path::RegKind;
    use crate::path::{self, link_user};
    use crate::valid::auth::{proccess_email, proccess_password, proccess_username};
    use crate::view::app::components::nav::Nav;
    use crate::view::app::{Acc, GlobalState};
    use crate::view::toolbox::prelude::*;
    use tracing::{error, trace};

    #[derive(Params, PartialEq, Clone)]
    pub struct RegParams {
        pub token: Option<String>,
        pub email: Option<String>,
        pub loading: Option<bool>,
        pub kind: Option<RegKind>,
    }

    #[component]
    pub fn Page() -> impl IntoView {
        let global_state = expect_context::<GlobalState>();
        let main_ref = NodeRef::new();
        let invite_email: NodeRef<Input> = NodeRef::new();
        let register_username: NodeRef<Input> = NodeRef::new();
        let register_email: NodeRef<Input> = NodeRef::new();
        let register_email_decoded = RwSignal::new(String::new());
        let register_password: NodeRef<Input> = NodeRef::new();
        let register_password_confirmation: NodeRef<Input> = NodeRef::new();
        let register_username_err = RwSignal::new(String::new());
        let register_email_err = RwSignal::new(String::new());
        let register_password_err = RwSignal::new(String::new());
        let register_general_err = RwSignal::new(String::new());
        let invite_general_err = RwSignal::new(String::new());
        let invite_email_err = RwSignal::new(String::new());
        let invite_completed = RwSignal::new(String::new());
        let invite_email: NodeRef<Input> = NodeRef::new();
        // let api_invite = controller::auth::route::invite::client.ground();
        // let api_invite_decode = controller::auth::route::invite_decode::client.ground();
        // let api_register = controller::auth::route::register::client.ground();
        let api = ApiWeb::new();
        let api_invite_decode = ApiWeb::new();
        let query = use_query::<RegParams>();
        let navigate = leptos_router::hooks::use_navigate();

        let get_query_token = move || query.read().as_ref().ok().and_then(|v| v.token.clone());
        let get_query_email = move || query.read().as_ref().ok().and_then(|v| v.email.clone());
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
        let get_query_email_or_err = move || get_query_email().unwrap_or(String::from("error"));
        // let decoded_email = move || api.result.with(|v| match v);

        // Effect::new(move || {
        //     match api.result.get()
        // });
        // let is_loading = move || {
        //     api_register.is_pending_tracked() || api_invite.is_pending_tracked() || api_invite_decode.is_pending_tracked()
        // };

        let on_invite = {
            let navigate = navigate.clone();
            move |e: SubmitEvent| {
                e.prevent_default();
                let navigate = navigate.clone();

                let Some(email) = invite_email.get_untracked() else {
                    return;
                };

                let email = proccess_email(email.value());

                invite_email_err.set(email.clone().err().unwrap_or_default());
                invite_general_err.set(String::new());

                let Ok(email) = email else {
                    return;
                };
                let email_clone = email.clone();

                api.get_invite(email_clone).send_web(move |result| {
                    let email = email.clone();
                    let navigate = navigate.clone();

                    async move {
                        match result {
                            Ok(ServerRes::Ok) => {
                                // let result = api.profile().send_native().await;
                                invite_completed.set(email.clone());
                                navigate(
                                    &path::link_check_email(email),
                                    NavigateOptions {
                                        ..Default::default()
                                    },
                                );
                                // global_state.set_auth_from_res(result);
                            }
                            Ok(res) => {
                                error!("expected Ok, received {res:?}");
                            }

                            Err(err) => {
                                invite_general_err.set(err.to_string());
                                error!("get invite err: {err}");
                            }
                        }
                    }
                });

                // api_invite.dispatch(controller::auth::route::invite::Input { email });
            }
        };
        let on_register = move |e: SubmitEvent| {
            e.prevent_default();
            // let link = link_user("hey5");
            // trace!("navigating to {link}");
            // navigate(
            //     &link,
            //     NavigateOptions {
            //         // replace: true,
            //         ..Default::default()
            //     },
            // );
            // return;
            // let navigate = navigate.clone();
            let (Some(username), Some(password), Some(password_confirmation)) = (
                register_username.get_untracked(),
                // register_email.get(),
                register_password.get_untracked(),
                register_password_confirmation.get_untracked(),
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

            let (Ok(username), Ok(password), Some(invite_token)) = (username, password, token)
            else {
                return;
            };

            api.register(username, invite_token, password)
                .send_web(move |result| {
                    // let navigate = navigate.clone();
                    async move {
                        match result {
                            Ok(ServerRes::Ok) => {
                                let res = global_state.update_auth_now().await;
                                match res {
                                    Ok(ServerRes::User { username }) => {
                                        let _ = global_state.update_auth_now().await;
                                        // let link = link_user(username);
                                        // trace!("navigating to {link}");
                                        // navigate(
                                        //     &link,
                                        //     NavigateOptions {
                                        //         replace: true,
                                        //         ..Default::default()
                                        //     },
                                        // );
                                        //
                                        // navigate(&link, Default::default());
                                    }
                                    res => {
                                        error!("expected User, received {res:?}");
                                        // navigate("/", Default::default());
                                    }
                                }
                            }
                            Ok(res) => {
                                register_email_decoded
                                    .set(format!("error, expected OK, received: {res:?}"));
                            }
                            Err(ServerErr::ServerRegistrationErr(
                                ServerRegistrationErr::TokenExpired,
                            )) => {
                                register_general_err
                                    .set("This invite link is already expired.".to_string());
                            }
                            Err(ServerErr::ServerRegistrationErr(
                                ServerRegistrationErr::TokenUsed,
                            )) => {
                                register_general_err
                                    .set("This invite link was already used.".to_string());
                            }
                            Err(ServerErr::ServerRegistrationErr(
                                ServerRegistrationErr::TokenNotFound,
                            )) => {
                                register_general_err
                                    .set("This invite link is invalid.".to_string());
                            }
                            Err(err) => {
                                register_general_err.set(err.to_string());
                            }
                        }
                    }
                });
            // api_register.dispatch(controller::auth::route::register::Input {
            //     email_token: token,
            //     password,
            //     username,
            // });
        };
        // Effect::new({
        //     let navigate = navigate.clone();
        //     move || {
        //         let (Some(result), Some(email)) =
        //             (api_invite.value_tracked(), invite_email.get().map(|v| v.value()))
        //         else {
        //             return;
        //         };
        //
        //         match result {
        //             Ok(_) => {
        //                 navigate(&path::link_check_email(email), Default::default());
        //             }
        //             // Err(ResErr::ServerErr(artbounty_api::auth::api::invite::ServerErr::))
        //             Err(err) => {
        //                 invite_general_err.set(err.to_string());
        //             }
        //         }
        //     }
        // });

        Effect::new(move || {
            let Some(token) = get_query_token() else {
                return;
            };

            api_invite_decode
                .decode_invite(token)
                .send_web(move |result| async move {
                    match result {
                        Ok(ServerRes::InviteToken(token)) => {
                            register_email_decoded.set(token.email);
                        }
                        Ok(res) => {
                            register_email_decoded
                                .set(format!("error, expected OK, received: {res:?}"));
                        }
                        Err(err) => {
                            register_email_decoded.set(err.to_string());
                        }
                    }
                });

            // api_invite_decode.dispatch(controller::auth::route::invite_decode::Input { token });
        });

        //
        // Effect::new(move || {
        //     let Some(result) = api_register.value_tracked() else {
        //         trace!("does anything work?");
        //         return;
        //     };
        //     trace!("no");
        //     match result {
        //         Ok(res) => {
        //             trace!("ok???");
        //             global_state.acc.set(Some(Acc {
        //                 username: res.username,
        //             }));
        //             navigate("/", Default::default());
        //             //
        //         }
        //         Err(ResErr::ServerErr(
        //             controller::auth::route::register::ServerErr::EmailInvalid(err),
        //         )) => {
        //             register_email_err.set(err);
        //         }
        //         Err(ResErr::ServerErr(
        //             controller::auth::route::register::ServerErr::EmailTaken,
        //         )) => {
        //             register_email_err.set("email is taken".to_string());
        //         }
        //         Err(ResErr::ServerErr(
        //             controller::auth::route::register::ServerErr::UsernameInvalid(err),
        //         )) => {
        //             register_username_err.set(err);
        //         }
        //         Err(ResErr::ServerErr(
        //             controller::auth::route::register::ServerErr::PasswordInvalid(err),
        //         )) => {
        //             register_password_err.set(err);
        //         }
        //         Err(ResErr::ServerErr(
        //             controller::auth::route::register::ServerErr::UsernameTaken,
        //         )) => {
        //             register_username_err.set("username is taken".to_string());
        //         }
        //         Err(err) => {
        //             register_general_err.set(err.to_string());
        //         }
        //     }
        // });

        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] min-h-[100dvh]">
                <Nav/>
                // <div class=move || format!("grid  text-white {}", if api_register.is_pending() || api_register.is_complete() || api_invite.is_complete() || api_invite.is_pending() || get_query_token().is_some() || get_query_email().is_some() {"items-center"} else {"justify-stretch"})>
                <div class=move || format!("grid  text-white {}", if api.is_pending_tracked() {"items-center"} else {"justify-stretch"})>
                    <div class=move||format!("mx-auto text-[1.5rem] {}", if api.is_pending_tracked() {""} else {"hidden"})>
                        <h1>"LOADING..."</h1>
                    </div>
                    <div class=move||format!("mx-auto flex flex-col gap-2 text-center {}", if query_kind_is_check_email() && !api.is_pending_tracked() {""} else {"hidden"})>
                        <h1 class="text-[1.5rem] my-[4rem]">"VERIFY EMAIL"</h1>
                        <p class="max-w-[30rem]">"Verification email was sent to \""{get_query_email_or_err}"\" click the confirmtion link in the email."</p>
                        // <a href="/login" class="underline">"Go to Login"</a>
                    </div>
                    // <form method="POST" action="" on:submit=on_invite class=move || format!("flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full {}", if api_invite.is_pending() || api_invite.is_complete() || get_query_token().is_some() || get_query_email().is_some() {"hidden"} else {""})>
                    <form method="POST" action="" on:submit=on_invite class=move || format!("flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full {}", if get_query_kind().is_none() && !api.is_pending_tracked() {""} else {"hidden"})>
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
                    <form method="POST" action="" on:submit=on_register class=move || format!("flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full {}", if query_kind_is_reg() && !api.is_pending_tracked() {""} else {"hidden"})>
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
                                <input value=move|| register_email_decoded.get() readonly placeholder="loading..." id="email" node_ref=register_email type="text" class="border-b-2 border-white w-full mt-1 " />
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
    use leptos::html;
    use leptos::{html::Input, prelude::*};

    use crate::api::{Api, ApiWeb, ServerLoginErr, ServerRes};
    use crate::view::app::components::nav::Nav;
    use crate::view::app::{Acc, GlobalState};
    use crate::view::toolbox::prelude::*;
    use tracing::{error, trace};
    use web_sys::SubmitEvent;

    // use crate::{
    //     controller::{self, valid::auth::proccess_email},
    //     view::app::GlobalState,
    // };

    #[component]
    pub fn Page() -> impl IntoView {
        let global_state = expect_context::<GlobalState>();
        let main_ref: NodeRef<html::Main> = NodeRef::new();
        let input_email: NodeRef<Input> = NodeRef::new();
        let input_password: NodeRef<Input> = NodeRef::new();
        let general_err = RwSignal::new(String::new());
        let email_err = RwSignal::new(String::new());
        let navigate = leptos_router::hooks::use_navigate();
        let api = ApiWeb::new();

        // let api_login = controller::auth::route::login::client.ground();
        let on_login = move |e: SubmitEvent| {
            e.prevent_default();
            let (Some(email), Some(password)) = (input_email.get(), input_password.get()) else {
                return;
            };

            // let email = proccess_email(email.value());
            let email = email.value();
            let password = password.value();
            // let password = proccess_password(password.value(), None); NEVER PUT PASSWORD VERIFICATION ON LOGIN; if password verification rules ever change the old accounts wont be able to login.

            // email_err.set(email.clone().err().unwrap_or_default());
            // password_err.set(password.clone().err().unwrap_or_default());
            general_err.set(String::new());

            // let Ok(email) = email else {
            //     return;
            // };

            trace!("lohin dispatched");
            api.login(email, password)
                .send_web(move |result| async move {
                    match result {
                        Ok(ServerRes::Ok) => {
                            global_state.update_auth();
                        }
                        Ok(res) => {
                            error!("expected Ok, received {res:?}");
                        }
                        // Err(ServerLoginErr::) => {
                        //     let r = general_err.try_set(err.to_string());
                        //     if r.is_some() {
                        //         error!("global state acc was disposed somehow");
                        //     }
                        // }
                        Err(err) => {
                            let r = general_err.try_set(err.to_string());
                            if r.is_some() {
                                error!("global state acc was disposed somehow");
                            }
                        }
                    }
                });
            // api_login.dispatch(controller::auth::route::login::Input { email, password });
        };
        // let login_completed = {let login = login.clone(); move || login.is_complete()};
        // let login_pending = {let login = login.clone(); move || login.is_pending()};
        //
        // Effect::new(move || {
        //     let Some(result) = api_login.value_tracked() else {
        //         trace!("does anything work?");
        //         return;
        //     };
        //     trace!("received {result:#?}");
        //     match result {
        //         Ok(res) => {
        //             global_state.acc.set(Some(Acc {
        //                 username: res.username,
        //             }));
        //
        //             navigate("/", Default::default());
        //         }
        //         Err(ResErr::ClientErr(err)) => {
        //             general_err.set(format!("Error sending request \"{err}\"."));
        //         }
        //         Err(ResErr::ServerErr(controller::auth::route::login::ServerErr::Incorrect)) => {
        //             general_err.set("Email or Password is incorrect.".to_string());
        //         }
        //         Err(ResErr::ServerErr(controller::auth::route::login::ServerErr::ServerErr))
        //         | Err(ResErr::ServerErr(
        //             controller::auth::route::login::ServerErr::CreateCookieErr,
        //         )) => {
        //             general_err.set("Server error.".to_string());
        //         }
        //         Err(err) => {
        //             general_err.set(err.to_string());
        //         }
        //     }
        // });
        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] min-h-[100dvh]">
                <Nav/>
                <div class=move || format!("grid  text-white {}", if api.is_pending_tracked() {"items-center"} else {"justify-stretch"})>
                    <div class=move||format!("mx-auto text-[1.5rem] {}", if api.is_pending_tracked() {""} else {"hidden"})>
                        <h1>"LOADING..."</h1>
                    </div>
                    <form method="POST" action="" on:submit=on_login class=move || format!("flex flex-col px-[4rem] max-w-[30rem] mx-auto w-full {}", if api.is_pending_tracked() || api.is_succ_tracked()  {"hidden"} else {""})>
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
