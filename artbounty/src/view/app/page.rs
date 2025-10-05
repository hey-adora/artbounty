pub mod post {
    use crate::api::{Api, ApiWeb, ServerErr, ServerGetErr};
    use crate::path::{link_home, link_img, link_user};
    use crate::view::app::components::nav::Nav;
    use crate::view::toolbox::prelude::*;
    use leptos::prelude::*;
    use leptos::{Params, task::spawn_local};
    use leptos_router::hooks::{use_location, use_params};
    use leptos_router::params::Params;
    use tracing::{error, trace};

    #[derive(Params, PartialEq, Clone)]
    pub struct PostParams {
        pub username: Option<String>,
        pub post: Option<String>,
    }

    #[component]
    pub fn Page() -> impl IntoView {
        let main_ref = NodeRef::new();
        let api = ApiWeb::new();
        let param = use_params::<PostParams>();
        let param_username = move || param.read().as_ref().ok().and_then(|v| v.username.clone());
        let param_post = move || param.read().as_ref().ok().and_then(|v| v.post.clone());
        let imgs_links = RwSignal::new(Vec::<(String, f64)>::new());
        let title = RwSignal::new(String::new());
        let author = RwSignal::new(String::new());
        let description = RwSignal::new(String::from("loading..."));
        let favorites = RwSignal::new(0_u64);
        let not_found = RwSignal::new(false);
        let location = use_location();

        let fn_link = move || {
            let author = author.get();
            if author.is_empty() {
                link_home()
            } else {
                link_user(author)
            }
        };
        let fn_title = move || {
            let title = title.get();
            if title.is_empty() {
                "loading...".to_string()
            } else {
                title
            }
        };
        let fn_author = move || {
            let author = author.get();
            if author.is_empty() {
                "loading...".to_string()
            } else {
                author
            }
        };
        let fn_description_is_empty = move || description.with(|v| v.is_empty());
        let fn_description = move || {
            let description = description.get();

            if description.is_empty() {
                return "No description.".to_string();
            }

            description
        };
        let fn_favorites = move || favorites.get();

        Effect::new(move || {
            let (Some(username), Some(post_id)) = (param_username(), param_post()) else {
                return;
            };

            api.get_post(post_id).send_web(move |result| async move {
                match result {
                    Ok(crate::api::ServerRes::Post(post)) => {
                        title.set(post.title);
                        author.set(post.user.username);
                        description.set(post.description);
                        favorites.set(post.favorites);
                        imgs_links.set(
                            post.file
                                .into_iter()
                                .map(|file| {
                                    (
                                        link_img(file.hash, file.extension),
                                        file.width as f64 / file.height as f64,
                                    )
                                })
                                .collect(),
                        );

                        // let mut links = Vec::new():
                        // for post_file in post.file {
                        //     trace!("rec: {post:#?}");
                        //
                        // }
                    }
                    Ok(res) => {
                        error!("wrong res, expected Post, got {:?}", res);
                    }
                    Err(ServerErr::ServerGetErr(ServerGetErr::NotFound)) => {
                        not_found.set(true);
                    }
                    Err(err) => {
                        error!("unexpected err {:#?}", { err });
                    }
                }
            });
        });

        let selected_img = move || {
            let hash = location.hash.get();
            let imgs_links = imgs_links.get();
            let selected_n = if hash.len() > 3 {
                usize::from_str_radix(&hash[3..], 10).unwrap_or_default()
            } else {
                0
            };
            let (selected_url, selected_ratio) =
                imgs_links.get(selected_n).cloned().unwrap_or_else(|| {
                    imgs_links
                        .first()
                        .cloned()
                        .unwrap_or(("/404.webp".to_string(), 1920.0 / 1080.0))
                });

            view! {
                // <div style:aspect-ratio=selected_ratio.to_string() class="w-full grid place-items-center bg-base02">
                // </div>
                <img id=move || format!("id{selected_n}") class="max-h-full" src=selected_url />
            }
        };

        let imgs = move || {
            imgs_links
                .get()
                .into_iter()
                .enumerate()
                .map(|(i, (url, ratio))| view! {
                    <div style:aspect-ratio=ratio.to_string() class="w-full grid place-items-center bg-base02">
                        <img id=move || format!("id{i}") class="" src=url />
                    </div>
                })
                .collect_view()
        };

        let previews = move || {
            imgs_links
                .get()
                .into_iter()
                .enumerate()
                .map(|(i, (url, ratio))| {
                    let id = format!("#id{i}");
                    let id2 = id.clone();
                    // let location = location.clone();

                    view! { <a
                        href=id2
                        class=move ||  {
                            let hash = location.hash.get();
                            trace!("hash: {hash}");
                            format!("h-[5rem] w-[5rem] bg-base05 bg-cover bg-center {}", if id == hash || (hash.is_empty() && i == 0) {"border-2 border-base08"} else {""})
                        }
                        style:background-image=move || format!("url(\"{url}\")") ></a> }
                })
                .collect_view()
        };

        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] h-screen text-base05">
                <Nav/>

                <div class=move || format!("place-items-center text-[1.5rem] {}", if not_found.get() {"grid"} else {"hidden"})>
                    "Not Found"
                </div>

                <div class=move || format!("flex flex-col lg:grid grid-cols-[2fr_1fr] grid-cols-[2fr_1fr] lg:max-h-[calc(100vh-3rem)] gap-2 px-2 md:gap-6 md:px-6 {}", if not_found.get() {"hidden"} else {"flex"})>
                    <div class="lg:hidden h-[50vh] flex justify-center place-items-center bg-base02" >
                        { selected_img }
                    </div>
                    <div class="hidden lg:flex flex-col gap-2 lg:overflow-y-scroll" >
                        { imgs }
                    </div>
                    <div class="flex flex-col gap-2 md:gap-6 lg:overflow-y-scroll">
                        <div class="flex justify-start gap-2 flex flex-wrap">
                            { previews }
                            // <div class="h-[5rem] bg-base05"></div>
                            // <div class="h-[5rem] bg-base05"></div>
                            // <div class="h-[5rem] bg-base05"></div>
                            // <div class="h-[5rem] bg-base05"></div>
                            // <div class="h-[5rem] bg-base05"></div>
                        </div>

                        <div class="flex flex-col gap-2">
                            <div class="flex justify-between">
                                <h1 class="text-[1.5rem] text-ellipsis text-base0F">{ fn_title }</h1>
                                <div>"X"</div>
                            </div>
                            <div class="flex justify-between">
                                <div class="flex gap-2">
                                    <p class="text-[1rem] rounded-full h-[3rem] w-[3rem] bg-base05"></p>
                                    <div class="flex flex-col gap-1">
                                        <div class="flex gap-1">
                                            <p class="text-[1rem] text-base03">"by"</p>
                                            <a href=fn_link class="text-[1rem] font-bold text-base0B">{ fn_author }</a>
                                        </div>
                                        <p class="text-[1rem]">"9999 followers"</p>
                                    </div>
                                </div>
                                <div>{fn_favorites}" favorites"</div>
                            </div>
                        </div>
                        <div class="flex flex-col gap-2 md:gap-4 justify-between mt-4">
                            <h1 class="text-[1.2rem] text-base0F">"Description"</h1>
                            <div class=move || format!("text-ellipsis overflow-hidden padding max-w-[calc(100vw-1rem)] {}", if fn_description_is_empty() {"text-base03"} else {"text-base05"} )>{fn_description}</div>
                        </div>
                    </div>
                </div>
            </main>
        }
    }
}
pub mod settings {
    use std::rc::Rc;

    use crate::api::{
        Api, ApiWeb, EmailConfirmTokenKind, ServerAddPostErr, ServerErr, ServerReqImg, ServerRes,
    };
    use crate::path::{
        link_post, link_reg, link_settings, link_settings_form_email, link_settings_form_username,
    };
    use crate::valid::auth::{proccess_post_description, proccess_post_title, proccess_username};
    use crate::view::app::GlobalState;
    use crate::view::app::components::nav::Nav;
    use crate::view::toolbox::prelude::*;
    use leptos::prelude::*;
    use leptos::{Params, task::spawn_local};
    use leptos_router::NavigateOptions;
    use leptos_router::{hooks::use_params, params::Params};

    use leptos_router::hooks::use_query;
    use tracing::{error, trace};
    use web_sys::{HtmlInputElement, HtmlTextAreaElement, MouseEvent, SubmitEvent};

    #[derive(
        Clone,
        Debug,
        PartialEq,
        PartialOrd,
        serde::Serialize,
        serde::Deserialize,
        strum::EnumString,
        strum::Display,
    )]
    #[strum(serialize_all = "lowercase")]
    pub enum SelectedForm {
        None,
        ChangeUsername,
        ChangeEmail,
        UsernameChanged,
        ChangePassword,
    }

    #[derive(
        Clone,
        Debug,
        PartialEq,
        PartialOrd,
        serde::Serialize,
        serde::Deserialize,
        strum::EnumString,
        strum::EnumIter,
        strum::Display,
    )]
    #[strum(serialize_all = "lowercase")]
    pub enum EmailChangeStage {
        SendConfirm,
        ClickConfirm,
        EnterNewEmail,
        ConfirmNewEmail,
        Finish,
    }

    #[derive(
        Clone,
        Debug,
        PartialEq,
        PartialOrd,
        serde::Serialize,
        serde::Deserialize,
        strum::EnumString,
        strum::EnumIter,
        strum::Display,
    )]
    #[strum(serialize_all = "lowercase")]
    pub enum UsernameChangeStage {
        SendConfirm,
        ClickConfirm,
        EnterNewUsername,
        Finish,
    }

    #[derive(Params, PartialEq, Clone, Default)]
    pub struct PageParams {
        pub selected_form: Option<SelectedForm>,
        pub new_email: Option<String>,
        pub new_username: Option<String>,
        pub old_username: Option<String>,
        pub confirm_token: Option<String>,
        pub email_stage: Option<EmailChangeStage>,
        pub username_stage: Option<UsernameChangeStage>,
        // pub current_email: Option<String>,
    }

    #[component]
    pub fn Page() -> impl IntoView {
        let main_ref = NodeRef::new();
        let global_state = expect_context::<GlobalState>();
        // let selected_form = RwSignal::new(SelectedForm::None);

        let navigate = leptos_router::hooks::use_navigate();
        let query = use_query::<PageParams>();
        let get_query_selected_form = move || {
            query
                .read()
                .as_ref()
                .ok()
                .and_then(|v| v.selected_form.clone())
                .unwrap_or(SelectedForm::None)
        };
        let get_query = move || {
            query
                .read()
                .as_ref()
                .ok()
                .map(|v| v.clone())
                .unwrap_or_default()
        };
        let get_query_untracked = move || {
            query
                .read_untracked()
                .as_ref()
                .ok()
                .map(|v| v.clone())
                .unwrap_or_default()
        };
        let get_query_new_username = move || {
            query
                .read()
                .as_ref()
                .ok()
                .and_then(|v| v.new_username.clone())
        };
        let get_query_old_username = move || {
            query
                .read()
                .as_ref()
                .ok()
                .and_then(|v| v.old_username.clone())
        };

        let get_query_email_stage = move || {
            query
                .read()
                .as_ref()
                .ok()
                .and_then(|v| v.email_stage.clone())
                .unwrap_or(EmailChangeStage::SendConfirm)
        };

        // let is_current_stage = move |stage: usize| -> bool {
        //
        //     let current_stage = get_query_form_stage();
        // };

        let view_current_stage_label = move |stage: EmailChangeStage| {
            let current_stage = get_query_email_stage();
            let (text, style) = if current_stage == stage {
                ("Current", "text-base0C")
            } else if current_stage > stage {
                ("Done", "text-base0B")
            } else {
                ("Next", "text-base03")
            };

            view! {
                <span class=style>{text}</span>
            }
        };

        let api = ApiWeb::new();
        let change_username_username_general_err = RwSignal::new(String::new());
        let change_username_username = NodeRef::new();
        let change_username_username_err = RwSignal::new(String::new());
        let change_username_password = NodeRef::new();

        let change_email_send_confirm_err = RwSignal::new(String::new());

        let change_email_enter_new_email_input = NodeRef::new();
        let change_email_enter_new_email_err = RwSignal::new(String::new());
        // let change_username_password_err = RwSignal::new(String::new());

        let on_change_username = {
            let navigate = navigate.clone();
            move |e: SubmitEvent| {
                e.prevent_default();
                let navigate = navigate.clone();
                let (Some(username), Some(password)) = (
                    change_username_username.get_untracked() as Option<HtmlInputElement>,
                    change_username_password.get_untracked() as Option<HtmlInputElement>,
                ) else {
                    return;
                };

                let username = proccess_username(username.value());
                let password = password.value();

                change_username_username_err.set(username.clone().err().unwrap_or_default());
                change_username_username_general_err.set(String::new());

                let (Ok(username),) = (username,) else {
                    return;
                };
                api.change_username(password, username)
                    .send_web(move |result| {
                        let navigate = navigate.clone();
                        async move {
                            match result {
                                Ok(crate::api::ServerRes::User {
                                    username: new_username,
                                }) => {
                                    let old_username = global_state
                                        .get_username_untracked()
                                        .unwrap_or("404".to_string());
                                    // let current_email = global_state.get_email_untracked().unwrap_or("404".to_string());
                                    global_state.change_username(new_username.clone());
                                    navigate(
                                        &link_settings_form_username(
                                            UsernameChangeStage::Finish,
                                            Some(old_username),
                                            Some(new_username),
                                        ),
                                        NavigateOptions::default(),
                                    );
                                    // selected_form.try_set(SelectedForm::None);
                                }
                                Ok(err) => {
                                    error!("expected Post, received {err:?}");
                                    let _ = change_username_username_general_err
                                        .try_set("SERVER ERROR, wrong response.".to_string());
                                }
                                Err(ServerErr::ChangeUsernameErr(
                                    crate::api::ChangeUsernameErr::UsernameIsTaken(_),
                                )) => {
                                    change_username_username_err
                                        .set("Username is taken".to_string());
                                }
                                Err(err) => {
                                    let _ = change_username_username_general_err
                                        .try_set(err.to_string());
                                }
                            }
                        }
                    });
            }
        };

        let on_email_change_send_confirm = {
            let navigate = navigate.clone();
            move |e: SubmitEvent| {
                e.prevent_default();
                let navigate = navigate.clone();

                api.send_email_confirmation(EmailConfirmTokenKind::ChangeEmail)
                    .send_web(move |result| {
                        let navigate = navigate.clone();
                        async move {
                            match result {
                                Ok(ServerRes::Ok) => {
                                    navigate(
                                        &link_settings_form_email(
                                            EmailChangeStage::ClickConfirm,
                                            None,
                                            None,
                                        ),
                                        NavigateOptions::default(),
                                    );
                                }
                                Ok(err) => {
                                    error!("expected Post, received {err:?}");
                                    let _ = change_email_send_confirm_err
                                        .try_set("SERVER ERROR, wrong response.".to_string());
                                }
                                Err(err) => {
                                    let _ = change_email_send_confirm_err.try_set(err.to_string());
                                }
                            }
                        }
                    });

                //
            }
        };

        let on_email_change_enter_new_email = {
            let navigate = navigate.clone();
            move |e: SubmitEvent| {
                e.prevent_default();
                let navigate = navigate.clone();

                let confirm_token = get_query_untracked().confirm_token;
                let confirm_token_is_none = confirm_token.is_none();
                let (Some(confirm_token), Some(new_email)) = (
                    confirm_token,
                    change_email_enter_new_email_input.get() as Option<HtmlInputElement>,
                ) else {
                    if confirm_token_is_none {
                        change_email_enter_new_email_err
                            .set("missing confirm_token query field".to_string());
                    }

                    return;
                };

                let new_email = new_email.value();

                if new_email.is_empty() {
                    change_email_enter_new_email_err
                        .set("new_email field cant be empty".to_string());
                    return;
                }

                api.change_email(confirm_token, new_email.clone())
                    .send_web(move |result| {
                        let navigate = navigate.clone();
                        let new_email = new_email.clone();
                        async move {
                            match result {
                                Ok(ServerRes::Ok) => {
                                    navigate(
                                        &link_settings_form_email(
                                            EmailChangeStage::ConfirmNewEmail,
                                            Some(new_email),
                                            None,
                                        ),
                                        NavigateOptions::default(),
                                    );
                                }
                                Ok(err) => {
                                    error!("expected Ok, received {err:?}");
                                    let _ = change_email_enter_new_email_err
                                        .try_set("SERVER ERROR, wrong response.".to_string());
                                }
                                Err(err) => {
                                    let _ =
                                        change_email_enter_new_email_err.try_set(err.to_string());
                                }
                            }
                        }
                    });

                //
            }
        };

        // let on_open_change_username = move |_e: MouseEvent| selected_form.set(SelectedForm::ChangeUsername);
        // let on_close = move |e: MouseEvent| {
        //     e.prevent_default();
        //     navigate(&link_settings(), NavigateOptions::default());
        // };

        // let should_disable = move || api.is_pending_tracked();
        //top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2
        view! {
            <main node_ref=main_ref class="text-base05 grid grid-rows-[auto_1fr] h-screen relative">
                <Nav/>
                <div class="flex flex-col px-[2rem] mx-auto gap-[2rem]">
                    <h1 class="text-[1.5rem] text-base0A text-center mt-[4rem] mb-[2rem]">"Settings"</h1>
                    <h2 class="text-[1.3rem] text-base0A mt-[4rem] mb-[2rem]">"Profile"</h2>
                    <form class="flex flex-col gap-2">
                        <label for="current_username" class="text-[1.2rem] ">"Username"</label>
                        <div class="flex">
                            <input value=move || global_state.get_username_tracked() id="current_username" name="current_username" disabled type="text" class="bg-base01 text-base0B w-full pl-2 " />
                            <a href=link_settings_form_username(UsernameChangeStage::SendConfirm, None::<String>, None::<String>) class="border-2 border-base0E text-[1.3rem] font-bold px-4 py-1 hover:bg-base02 text-base0E">"Change"</a>
                            // <a href=link_settings_form(SelectedForm::ChangeUsername) class="border-2 border-base0E text-[1rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950">"Change Username"</a>
                        </div>
                    </form>
                    <form class="flex flex-col gap-2">
                        <label for="current_email" class="text-[1.2rem] ">"Email"</label>
                        <div class="flex">
                            <input value=move || global_state.get_email_tracked() id="current_email" name="current_email" disabled type="text" class="bg-base01 text-base0B w-full pl-2 " />
                            <a href=link_settings_form_email(EmailChangeStage::SendConfirm, None, None) class="border-2 border-base0E text-[1.3rem] font-bold px-4 py-1 hover:bg-base02 text-base0E">"Change"</a>
                            // <a href=link_settings_form(SelectedForm::ChangeUsername) class="border-2 border-base0E text-[1rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950">"Change Username"</a>
                        </div>
                    </form>
                    // <button on:click=on_open_change_username class="border-2 border-base05 text-[1.3rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950">"Change Username"</button>

                </div>

                // username change
                // step 1
                <div class=move || format!("absolute top-0 left-0 w-full h-full grid place-items-center bg-base00/80 {}", if get_query_selected_form() == SelectedForm::ChangeUsername { "flex" } else { "hidden" } )>
                    <form method="POST" on:submit=on_change_username action="" class="flex flex-col px-[2rem] md:px-[4rem] max-w-[30rem] mx-auto w-full border-2 border-base05 bg-base00">
                        <h2 class="text-[1.5rem]  text-center mt-[4rem]">"Change Username"</h2>
                        <div class=move||format!("text-red-600 text-center my-[2rem] {}", if change_username_username_general_err.with(|v| v.is_empty()) { "invisible" } else { "" } )>{move || { change_username_username_general_err.get() }}</div>
                        <div class="flex flex-col gap-6">
                            <div class="flex flex-col gap-0">
                                <label for="username" class="text-[1.2rem] ">"New Username"</label>
                                <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if false {"text-[0rem]"} else {"text-[1rem]"}) >
                                    <ul class="list-disc ml-[1rem]">
                                        {move || change_username_username_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </div>
                                <input placeholder="kaiju" id="username" name="username" node_ref=change_username_username type="text" class="border-b-2 border-base05 w-full mt-1 " />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="password" class="text-[1.2rem]">"Password"</label>
                                <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if false {"text-[0rem]"} else {"text-[1rem]"}) >
                                    <ul class="list-disc ml-[1rem]">
                                        // {move || upload_title_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </div>
                                <input placeholder="current password" id="password" name="password" node_ref=change_username_password type="password" class="border-b-2 border-base05 w-full mt-1 " />
                            </div>
                        </div>
                        <div class="flex flex-row gap-[1.3rem] mx-auto my-[4rem] text-center">
                            // <button on:click=on_close disabled=move || api.is_pending_tracked() class="border-2 border-base05 text-[1.3rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950">"Cancel"</button>
                            <a href=link_settings() class="border-2 border-base05 text-[1.3rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950">"Cancel"</a>
                            <input type="submit" value=move || if api.is_pending_tracked() {"Saving..."} else {"Confirm"} disabled=move || api.is_pending_tracked() class="border-2 border-base05 text-[1.3rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950"/>
                        </div>
                    </form>
                </div>

                // step 4
                <div class=move || format!("absolute top-0 left-0 w-full h-full grid place-items-center bg-base00/80 {}", if get_query_selected_form() == SelectedForm::UsernameChanged { "flex" } else { "hidden" } )>
                    <div class="flex flex-col px-[2rem] md:px-[4rem] max-w-[30rem] mx-auto w-full gap-[2rem] py-[2rem] border-0 border-base05 bg-base01">
                        <h2 class="text-[1.5rem] text-base0F text-center ">"Username Changed"</h2>
                        <ol class="text-[1.2rem] list-decimal">
                            <li>"Send confirmation email to "
                                <span class="text-base0E">{move || format!("{}.", global_state.get_email_tracked().unwrap_or("404".to_string()))}</span>
                                <span class="text-base0B">" Done"</span>
                            </li>
                            <li>"Click on confirmation link that was sent to ."<span class="text-base0B">" Done"</span></li>
                            <li>"Enter new username."<span class="text-base0B">" Done"</span></li>
                            <li>
                                <div>"Finish."<span class="text-base0B">" Done"</span></div>
                                <div class="text-[1rem] text-base09">
                                    "Username changed from "
                                    <span class="text-base0B">" "{move || get_query_old_username().unwrap_or("404".to_string())}" "</span>
                                    <span>" to "</span>
                                    <span class="text-base0B">" "{move || get_query_new_username().unwrap_or("404".to_string())}" "</span>
                                </div>
                            </li>
                        </ol>
                        // <p class="text-[1.2rem] text-base0B text-center ">{move || get_query_new_username().unwrap_or("404".to_string())}</p>
                        <div class="w-full flex justify-end">
                            <a href=link_settings() class="border-2 border-base0E text-[1.3rem] font-bold px-4 py-1 hover:bg-base02 text-base0E">"Close"</a>
                        </div>
                    </div>
                </div>

                // email change
                // step 1
                <div class=move || format!("absolute top-0 left-0 w-full h-full grid place-items-center bg-base00/80 {}", if get_query_selected_form() == SelectedForm::ChangeEmail { "flex" } else { "hidden" } )>
                    <div class="flex flex-col px-[2rem] md:px-[4rem] max-w-[30rem] mx-auto w-full gap-[2rem] py-[2rem] border-0 border-base05 bg-base01">
                        <h2 class="text-[1.5rem] text-base0F text-center ">"Email Change"</h2>
                        <ol class="text-[1.2rem] list-decimal grid gap-2">
                            <li>"Send confirmation email to "
                                <span class="text-base0E">{move || format!("{}. ", global_state.get_email_tracked().unwrap_or("404".to_string()))}</span>
                                {move || view_current_stage_label(EmailChangeStage::SendConfirm) }
                                <div class=move || format!("text-[1rem] text-base08 {}", if get_query_email_stage() >= EmailChangeStage::SendConfirm { "visible" } else {"hidden"} )>
                                    <ul class="list-disc ml-[1rem]">
                                        {move || change_email_send_confirm_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </div>
                            </li>
                            <li>"Click on confirmation link that was sent to "
                                <span class="text-base0E">{move || format!("{}. ", global_state.get_email_tracked().unwrap_or("404".to_string()))}</span>
                                {move || view_current_stage_label(EmailChangeStage::ClickConfirm) }
                            </li>
                            <li>"Enter new email. "
                                {move || view_current_stage_label(EmailChangeStage::EnterNewEmail) }
                                <div class=move || format!(" {}", if get_query_email_stage() >= EmailChangeStage::EnterNewEmail { "visible" } else {"hidden"} )>
                                    <input node_ref=change_email_enter_new_email_input placeholder="email@example.com" class="bg-base02 mt-2 pl-2" type="email" />
                                </div>
                                <div class=move || format!("text-[1rem] text-base08 {}", if get_query_email_stage() >= EmailChangeStage::EnterNewEmail { "visible" } else {"hidden"} )>
                                    <ul class="list-disc ml-[1rem]">
                                        {move || change_email_enter_new_email_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </div>
                            </li>
                            <li>
                                <div>"Finish. "
                                    {move || view_current_stage_label(EmailChangeStage::Finish) }
                                </div>
                                <div class=move || format!("text-[1rem] text-base09 {}", if get_query_email_stage() >= EmailChangeStage::Finish { "visible" } else {"hidden"} )>
                                    "Username changed from "
                                    <span class="text-base0B">" "{move || get_query_old_username().unwrap_or("404".to_string())}" "</span>
                                    <span>" to "</span>
                                    <span class="text-base0B">" "{move || get_query_new_username().unwrap_or("404".to_string())}" "</span>
                                </div>
                            </li>
                        </ol>
                        // <p class="text-[1.2rem] text-base0B text-center ">{move || get_query_new_username().unwrap_or("404".to_string())}</p>
                        <div class=move || format!("w-full flex gap-4 justify-center {}", if api.is_pending_tracked() {"visible"} else {"hidden"})>
                            "loading..."
                        </div>
                        <div class=move || format!("w-full flex gap-4 justify-end {}", if api.is_pending_tracked() {"hidden"} else {"visible"})>
                            // { move || {
                            //     match get_query_email_stage() {
                            //         EmailChangeStage::SendConfirm => view! {
                            //             <div>
                            //                 <a href=link_settings() class="border-2 border-base0E text-[1.3rem] font-bold px-4 py-1 hover:bg-base02 text-base0E">"Close"</a>
                            //                 <a href=link_settings() class="border-2 border-base0E text-[1.3rem] font-bold px-4 py-1 hover:bg-base02 text-base0E">"Send"</a>
                            //             </div>
                            //         },
                            //         _ => view! {
                            //             <div>
                            //                 <a href=link_settings() class="border-2 border-base0E text-[1.3rem] font-bold px-4 py-1 hover:bg-base02 text-base0E">"Close"</a>
                            //             </div>
                            //
                            //         }}
                            // } }
                            // <a href=link_settings() class="border-2 border-base0E text-[1.3rem] font-bold px-4 py-1 hover:bg-base02 text-base0E">"Send"</a>
                            <a href=link_settings() class="border-2 border-base0E text-[1.3rem] font-bold px-4 py-1 hover:bg-base02 text-base0E">"Close"</a>
                            <form method="POST" action="" on:submit=on_email_change_send_confirm class=move || format!(" {}", if get_query_email_stage() == EmailChangeStage::SendConfirm { "visible" } else { "hidden" })>
                                <input type="submit" value="Send" class=move || format!("border-2 border-base0E text-[1.3rem] font-bold px-4 py-1 hover:bg-base02 text-base0E")/>
                            </form>
                            <form method="POST" action="" on:submit=on_email_change_enter_new_email class=move || format!(" {}", if get_query_email_stage() == EmailChangeStage::EnterNewEmail { "visible" } else { "hidden" })>
                                <input type="submit" value="Confirm" class=move || format!("border-2 border-base0E text-[1.3rem] font-bold px-4 py-1 hover:bg-base02 text-base0E")/>
                            </form>
                        </div>
                    </div>
                </div>


            </main>
        }
    }
}
pub mod upload {
    use std::rc::Rc;

    use crate::api::{Api, ApiWeb, ServerAddPostErr, ServerErr, ServerReqImg};
    use crate::path::link_post;
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
        let navigate = leptos_router::hooks::use_navigate();
        // let api_post = controller::post::route::add::client.ground();
        let on_upload = move |e: SubmitEvent| {
            e.prevent_default();
            trace!("uploading...");
            let navigate = navigate.clone();
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
                    .send_web(move |res| {
                        let navigate = navigate.clone();
                        async move {
                            match res {
                                Ok(crate::api::ServerRes::Post(post)) => {
                                    //
                                    navigate(
                                        &link_post(post.user.username, post.id),
                                        Default::default(),
                                    );
                                }
                                Err(ServerErr::ServerAddPostErr(
                                    ServerAddPostErr::ServerImgErr(errs),
                                )) => {
                                    let msg = errs
                                        .clone()
                                        .into_iter()
                                        .map(|err| err.err.to_string())
                                        .collect::<Vec<String>>()
                                        .join("\n");
                                    let _ = upload_image_err.try_set(msg);
                                }
                                // Err(ServerErr::ServerAddPostErr(
                                //     ServerAddPostErr::ServerDirCreationFailed(err),
                                // )) => {
                                //     let _ = upload_general_err.try_set(err.to_string());
                                // }
                                Ok(err) => {
                                    error!("expected Post, received {err:?}");
                                    let _ = upload_general_err
                                        .try_set("SERVER ERROR, wrong response.".to_string());
                                }
                                Err(err) => {
                                    let _ = upload_general_err.try_set(err.to_string());
                                }
                            };
                        }
                    });
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
                                <input placeholder="Funny looking cat" id="title" name="name" node_ref=upload_title type="text" class="border-b-2 border-base05 w-full mt-1 " />
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
                                <textarea class="border-l-2 border-base05 pl-2 bg-base05" node_ref=upload_description id="description" name="description" rows="4" cols="50">""</textarea>
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="tags" class="text-[1.2rem] ">"Tags"</label>
                                <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if upload_tags_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
                                    <ul class="list-disc ml-[1rem]">
                                        {move || upload_tags_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </div>
                                <textarea class="border-l-2 border-base05 pl-2 bg-base05" node_ref=upload_tags id="tags" name="tags" rows="1" cols="50">""</textarea>
                            </div>
                        </div>
                        <div class="flex flex-col gap-[1.3rem] mx-auto my-[4rem] text-center">
                            <input type="submit" value="Post" class="border-2 border-base05 text-[1.3rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950"/>
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
    use crate::api::ServerGetErr;
    use crate::api::ServerRes;
    use crate::view::app::components::gallery::Gallery;
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
        let user_username = RwSignal::new(None::<String>);

        Effect::new(move || {
            let Some(username) = param_username() else {
                return;
            };
            api.get_user(username).send_web(move |result| async move {
                match result {
                    Ok(ServerRes::User { username }) => {
                        user_username.set(Some(username));
                    }
                    Ok(res) => {
                        user_username.set(Some(format!("expected Uesr, received {res:?}")));
                        error!("expected Uesr, received {res:?}");
                    }
                    Err(ServerErr::ServerGetErr(ServerGetErr::NotFound)) => {
                        user_username.set(Some("Not Found".to_string()));
                    }
                    Err(err) => {
                        user_username.set(Some(err.to_string()));
                        error!("get user err: {err}");
                    }
                }
            });
        });

        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_auto_1fr] h-screen">
                <Nav/>
                <div>
                    <h1>{move || user_username.get()}</h1>
                </div>
                <Gallery row_height=250 username=user_username />
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

                api.send_email_invite(email_clone).send_web(move |result| {
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
            }
        };
        let on_register = move |e: SubmitEvent| {
            e.prevent_default();
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
                                    }
                                    res => {
                                        error!("expected User, received {res:?}");
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
        };

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
        });

        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] min-h-[100dvh]">
                <Nav/>
                // <div class=move || format!("grid  text-base05 {}", if api_register.is_pending() || api_register.is_complete() || api_invite.is_complete() || api_invite.is_pending() || get_query_token().is_some() || get_query_email().is_some() {"items-center"} else {"justify-stretch"})>
                <div class=move || format!("grid  text-base05 {}", if api.is_pending_tracked() {"items-center"} else {"justify-stretch"})>
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
                            <input placeholder="alice@mail.com" id="email" node_ref=invite_email type="text" class="border-b-2 border-base05 w-full mt-1 " />
                        </div>
                        <div class="flex flex-col gap-[1.3rem] mx-auto my-[4rem] text-center">
                            <input type="submit" value="Register" class="border-2 border-base05 text-[1.3rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950"/>
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
                                <input placeholder="Alice" id="username" node_ref=register_username type="text" class="border-b-2 border-base05 w-full mt-1 " />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="email" class="text-[1.2rem] ">"Email"</label>
                                <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if register_email_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
                                    <ul class="list-disc ml-[1rem]">
                                        {move || register_email_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </div>
                                <input value=move|| register_email_decoded.get() readonly placeholder="loading..." id="email" node_ref=register_email type="text" class="border-b-2 border-base05 w-full mt-1 " />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="password" class="text-[1.2rem] ">"Password"</label>
                                <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if register_password_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
                                    <ul class="list-disc ml-[1rem]">
                                        {move || register_password_err.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </div>
                                <input id="password" node_ref=register_password type="password" class="border-b-2 border-base05 w-full mt-1 " />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="password_confirmation" class="text-[1.3rem] ">"Password Confirmation"</label>
                                <input id="password_confirmation" node_ref=register_password_confirmation type="password" class="border-b-2 border-base05 w-full mt-1 " />
                            </div>
                        </div>
                        <div class="flex flex-col gap-[1.3rem] mx-auto my-[4rem] text-center">
                            <input type="submit" value="Register" class="border-2 border-base05 text-[1.3rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950"/>
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

            trace!("login dispatched");
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
                <div class=move || format!("grid  text-base05 {}", if api.is_pending_tracked() {"items-center"} else {"justify-stretch"})>
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
                                <input placeholder="alice@mail.com" id="email" node_ref=input_email type="email" class="border-b-2 border-base05" />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label for="password" class="text-[1.2rem] ">"Password"</label>
                                // <div class=move || format!("text-red-600 transition-[font-size] duration-300 ease-in {}", if password_err.with(|err| err.is_empty()) {"text-[0rem]"} else {"text-[1rem]"}) >
                                //     <ul class="list-disc ml-[1rem]">
                                //         {move || password_err.get().trim().split("\n").filter(|v| v.len() > 1).into_iter().map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                //     </ul>
                                // </div>
                                <input id="password" node_ref=input_password type="password" class="border-b-2 border-base05" />
                            </div>
                        </div>
                        <div class="flex flex-col gap-[1.3rem] mx-auto my-[4rem] text-center">
                            <input type="submit" value="Login" class="border-2 border-base05 text-[1.3rem] font-bold px-4 py-1 hover:bg-base05 hover:text-gray-950"/>
                            <a href="/register" class="underline">"or Register"</a>
                        </div>
                    </form>
                </div>
            </main>
        }
    }
}
