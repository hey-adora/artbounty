use std::time::Duration;

use crate::api::shared::post_comment::UserPostComment;
use crate::api::{Api, ApiWeb, Server404Err, ServerErr};
use crate::path::{PATH_LOGIN, link_home, link_img, link_user};
use crate::valid::MAX_POST_DESCRIPTION_LENGTH;
use crate::valid::MAX_POST_TAGS_LENGTH;
use crate::valid::MAX_POST_TITLE_LENGTH;
use crate::view::app::GlobalState;
use crate::view::app::components::auto_textarea::AutoTextArea;
use crate::view::app::components::btn_primary::BtnPrimary;
use crate::view::app::components::btn_secondary::BtnSecondary;
use crate::view::app::components::errors::Errors;
use crate::view::app::components::nav::Nav;
use crate::view::app::components::svg_star::Star;
use crate::view::app::hook::api_post::PostApi;
use crate::view::app::hook::api_post_comments::{
    CommentKind, CommentKind2, CommentsApi, CommentsApi2,
};
use crate::view::app::hook::api_post_file_upload::FileUpload;
use crate::view::app::hook::use_event_listener::EventListener;
use crate::view::app::hook::use_future::FutureFn;
use crate::view::app::hook::use_infinite_scroll_fn::InfiniteScrollFn;
use crate::view::app::hook::use_infinite_scroll_virtual::{
    InfiniteStage, use_infinite_scroll_virtual,
};
use crate::view::app::hook::use_mutation::Mutation;
use crate::view::app::hook::use_post_comment::use_post_comment;
use crate::view::app::hook::use_post_comments_baisc::CommentsBaisc;
use crate::view::app::hook::use_post_like::{self, PostLikeStage, use_post_like};
use crate::view::app::hook::use_spawner::Spawner;
use crate::view::app::hook::use_text_length_counter::use_text_counter;
use crate::view::toolbox::prelude::{set_timeout, *};
use leptos::{Params, task::spawn_local};
use leptos::{ev, html, prelude::*};
use leptos_router::hooks::{use_location, use_params};
use leptos_router::params::Params;
use tracing::{debug, error, trace, warn};
use wasm_bindgen::JsValue;
use web_sys::{
    Event, EventTarget, HtmlDivElement, HtmlElement, HtmlPreElement, MouseEvent, ScrollBehavior,
    ScrollIntoViewOptions, ScrollLogicalPosition, SubmitEvent,
};
use web_sys::{HtmlInputElement, HtmlTextAreaElement};

#[derive(Params, PartialEq, Clone)]
pub struct PostParams {
    pub username: Option<String>,
    pub post: Option<String>,
}

#[component]
pub fn Page() -> impl IntoView {
    // TODO add max limit for description and tags and other stuff
    let main_ref = NodeRef::new();
    let api_post = ApiWeb::new();
    let api_comments = ApiWeb::new();
    let global_state = expect_context::<GlobalState>();

    let param = use_params::<PostParams>();
    let param_username = move || param.read().as_ref().ok().and_then(|v| v.username.clone());
    let param_post = Memo::new(move |_| param.read().as_ref().ok().and_then(|v| v.post.clone()));

    let location = use_location();

    let spawner_post = Spawner::new();
    let post_api = PostApi::new(api_post);
    let edit_tags_input = NodeRef::<html::Textarea>::new();
    let description_input_editor = NodeRef::<html::Textarea>::new();
    let title_input_editor = NodeRef::<html::Textarea>::new();

    // use_text_counter(description_input_editor, post_api.live_description_length);
    // use_text_counter(edit_tags_input, post_api.live_tags_length);

    let spawner_comments = Spawner::new();
    let comment_container_ref = NodeRef::<html::Div>::new();
    let comment_input_ref = NodeRef::<html::Textarea>::new();
    let comment_basic = CommentsBaisc::new(api_comments, spawner_post);
    let post_comment = move || {
        let Some(input_elm) = comment_input_ref.get() else {
            return;
        };
        spawner_comments.spawn(async move {
            let text = input_elm.value();
            comment_basic.comments_manual.post(text).await;
            let post_err = comment_basic.err_post.get_untracked();
            if !post_err.is_empty() {
                return;
            }
            input_elm.set_value("");
        });
        // comment_basic.post.run();
    };
    Effect::new(move || {
        let Some(post_id) = param_post.get() else {
            return;
        };

        spawner_post.spawn(post_api.get(post_id));
    });
    Effect::new(move || {
        trace!("comments basic start");
        let (Some(post_id), Some(comment_container_ref)) =
            (param_post.get(), comment_container_ref.get())
        else {
            return;
        };

        trace!("comments basic observe");
        spawner_comments.spawn(comment_basic.observe_only(comment_container_ref.into(), post_id));
    });
    // Effect::new(move || {
    //     // let (Some(description_ref), Some(edit_description_ref)) = (
    //     //     ) else {
    //     //     return;
    //     // };
    //     description_mutation.disconnect();

    //     // if let Some(elm) = description_input.get() {
    //     //     description_mutation.observe(
    //     //         elm,
    //     //         MutationObserverOptions::new()
    //     //             .character_data()
    //     //             .set_child_list()
    //     //             .subtree(),
    //     //     );
    //     // }

    //     if let Some(elm) = description_input_editor.get() {
    //         description_mutation.observe(
    //             elm,
    //             MutationObserverOptions::new()
    //                 .character_data()
    //                 .set_child_list()
    //                 .subtree(),
    //         );
    //     }

    //     // trace!("comments basic observe");
    //     // spawner_comments.spawn(comment_basic.observe_only(comment_container_ref.into(), post_id));
    // });

    let post_like = use_post_like(param_post);
    let post_like_fn = move || {
        post_like.on_like.with_value(|f| {
            (f)();
        });
    };
    let is_post_liked_fn = move || {
        post_like
            .stage
            .with_value(|f| (f)() == PostLikeStage::Liked)
    };
    let post_like_btn_style = move || {
        format!(
            "border-2 text-[1.3rem] font-bold px-4 py-1 {}",
            match post_like.stage.run() {
                PostLikeStage::Liked => "border-base01 bg-base05 text-base01",
                PostLikeStage::Unliked =>
                    "border-base05 bg-base01 text-base05 hover:bg-base05 hover:text-base01",
                PostLikeStage::Loading => "border-base05 bg-base01 text-base05",
            }
        )
    };
    let post_like_btn_text = move || match post_like.stage.run() {
        PostLikeStage::Liked => "Un-Favorite",
        PostLikeStage::Unliked => "Favorite",
        PostLikeStage::Loading => "Loading",
    };

    let selected_img = move || -> AnyView {
        let hash = location.hash.get();
        let imgs_links = post_api.imgs_links.get();
        let selected_n = if hash.len() > 3 {
            usize::from_str_radix(&hash[3..], 10).unwrap_or_default()
        } else {
            0
        };
        let Some((selected_url, selected_ratio)) = imgs_links.get(selected_n).cloned() else {
            return view! {
                <p>
                    "No Image"
                </p>
                // <img id=move || format!("id0") class="max-h-full" src=selected_url />
            }
            .into_any();
        };

        view! {
            <img id=move || format!("id{selected_n}") class="max-h-full" src=selected_url />
        }
        .into_any()
    };

    let imgs = move || -> Vec<AnyView> {
        let imgs = post_api.imgs_links.get();
        if imgs.is_empty() {
            let view = view! {
                    <div style:aspect-ratio="1.0" class=" w-full h-[50%] grid place-items-center bg-base02">
                        <p id="id0" >"No Images"</p>
                        // <img id=move || format!("id{i}") class="" src=url />
                    </div>

            }
            .into_any();
            return vec![view];
        }
        imgs
                .into_iter()
                .enumerate()
                .map(|(i, (url, ratio))| view! {
                    <div style:aspect-ratio=ratio.to_string() class="w-full grid place-items-center bg-base02">
                        <img id=move || format!("id{i}") class="" src=url />
                    </div>
                }.into_any())
                .collect_view()
    };

    let edit_description_mode_toggle = move || {
        let description_len = post_api.description.with_untracked(|v| v.len());
        post_api.live_description_length.set(description_len);
        post_api.update_description_mode.update(|v| *v = !*v);
    };
    let edit_description_save = move || {
        let (Some(post_key), Some(new_description)) = (
            param_post.get(),
            description_input_editor
                .get_untracked()
                .map(|v: HtmlTextAreaElement| v.value()),
        ) else {
            return;
        };
        spawner_post.spawn(post_api.update_description(post_key, new_description));
    };
    // let edit_description_cancel = move || {
    //     // post_api.description.update(|_v| ());
    //     edit_description_mode_toggle();
    // };

    // let edit_description_keydown = move |e: Event| {
    //     let v = e
    //         .target()
    //         .map(|v: EventTarget| Into::<HtmlPreElement>::into(JsValue::from(v)) )
    //         .map(|v| v.id())
    //         ;
    //
    //     // let Some(new_description) = (
    //     //     edit_description_input
    //     //         .get_untracked()
    //     //         .and_then(|v: HtmlPreElement| v.text_content()),
    //     // ) else {
    //     //     return;
    //     // };
    //     trace!("wtf description changed {v:?}");
    // };

    // let description = move || {
    //     let mut description = post_api.description.get();
    //     if description.is_empty() {
    //         description.push_str("No description.");
    //     }
    //     description
    // };
    let title = move || {
        let mut title = post_api.title.get();

        if title.is_empty() {
            title.push_str("No Title.");
        }

        title
    };
    let title_is_empty = move || post_api.title.with(|v| v.is_empty());
    let edit_title_mode_toggle = move || {
        trace!("toggling title edit mode");
        post_api.update_title_mode.update(|v| *v = !*v);
    };

    let edit_title_save = move || {
        let (Some(post_key), Some(title)) = (
            param_post.get(),
            title_input_editor
                .get_untracked()
                .map(|v: HtmlTextAreaElement| v.value()),
        ) else {
            return;
        };
        trace!("{title}");
        spawner_post.spawn(post_api.update_title(post_key, title));
    };

    let description = move || {
        post_api.description.get()
        // let mut a = view! {};
        // post_api
        //     .description
        //     .get()
        //     .split('\n')
        //     .into_iter()
        //     .map(|v| {
        //         view! {
        //             <div class="">
        //                 {v}
        //             </div>
        //         }
        //     })
        //     .collect_view()
    };

    let tags = move || {
        post_api
            .tags
            .get()
            .split_whitespace()
            .into_iter()
            .map(|v| {
                view! {
                    <div class="bg-base02 rounded-full text-[1rem] px-3 py-1">
                        {v}
                    </div>

                }
            })
            .collect_view()
    };

    let edit_tags = move || {
        post_api.update_tags_mode.update(|v| *v = !*v);
    };

    let edit_tags_save = move || {
        let (Some(post_key), Some(tags)) = (
            param_post.get(),
            edit_tags_input
                .get_untracked()
                .map(|v: HtmlTextAreaElement| v.value()),
        ) else {
            return;
        };
        spawner_post.spawn(post_api.update_tags(post_key, tags));
    };

    let previews = move || {
        let mut imgs = post_api.imgs_links.get();

        // imgs.push((String::new(), 0.0);

        let mut views = imgs
                .into_iter()
                .enumerate()
                .map(|(i, (url, ratio))| {
                    let id = format!("#id{i}");
                    let id2 = id.clone();

                    view! { <a
                        href=id2
                        class=move ||  {
                            let hash = location.hash.get();
                            trace!("hash: {hash}");
                            format!("h-[5rem] w-[5rem] bg-base05 bg-cover bg-center {}", if id == hash || (hash.is_empty() && i == 0) {"border-2 border-base08"} else {""})
                        }
                        style:background-image=move || format!("url(\"{url}\")") ></a> }
                }.into_any()
                )
                .collect_view();

        let preview_add = {
            // let i = views.len();
            // let id = format!("#id{i}");
            // let id2 = id.clone();

            view! { <button
                    id="previw_add"
                    // href=id2
                    class=move ||  {
                        let hash = location.hash.get();
                        trace!("hash: {hash}");
                        format!("text-[2rem] grid place-items-center h-[5rem] w-[5rem] rounded-xl bg-base05/10 bg-cover bg-center border-2 border-base05")
                    }
                    // style:background-image=move || format!("url(\"{url}\")")
                    >"+"</button>
            }
        };

        views.push(preview_add.into_any());

        views
    };

    let delete_post = move |_| {
        let Some(post_id) = param_post.get() else {
            return;
        };
        spawner_post.spawn(post_api.delete(post_id));
    };

    let uploader = FileUpload::new();
    let upload_image = NodeRef::<html::Input>::new();
    let on_upload = move |_| {
        let (Some(files),): (Option<Vec<web_sys::File>>,) = (
            (upload_image.get_untracked())
                .and_then(|f: HtmlInputElement| f.files())
                .map(|f| f.get_files()),
            // upload_title.get_untracked() as Option<HtmlInputElement>,
            // upload_description.get_untracked() as Option<HtmlTextAreaElement>,
            // upload_tags.get_untracked() as Option<HtmlTextAreaElement>,
        ) else {
            return;
        };

        uploader.upload(&files[..]);

        trace!("files selected: {}", files.len());
    };

    view! {
        <main node_ref=main_ref class="relative font-hi grid grid-rows-[auto_1fr] h-screen text-base05">
            <Nav/>

            <Show when=move|| post_api.post_state.get().is_not_found() >
                <div class=move || format!("grid place-items-center text-[1.5rem] ")>
                    "Not Found"
                </div>
            </Show>

            <Show when=move|| post_api.post_state.get().is_deleted() >
                <div class=move || format!("grid place-items-center text-[1.5rem] ")>
                    "deleted"
                </div>
            </Show>

            <Show when=move|| {
                let state = post_api.post_state.get();
                state.is_normal() || state.is_loading()
            } >
                <div class=move || format!("flex flex-col lg:grid grid-cols-[2fr_1fr] grid-cols-[2fr_1fr] lg:max-h-[calc(100vh-3rem)] gap-2  md:gap-6 flex")>
                    <div class="col-span-2 flex justify-between px-4 md:px-6 ">
                        <div></div>
                        <div>
                            <button on:click=delete_post>
                                <SVGTrash class="size-[1.1rem] text-base08 "/>
                            </button>
                        </div>
                    </div>
                    <div class="lg:hidden h-[50vh] flex justify-center place-items-center bg-base02" >
                        { selected_img }
                    </div>
                    <div class="hidden lg:flex flex-col gap-2 lg:overflow-y-scroll" >
                        { imgs }
                    </div>
                    <div class="flex flex-col gap-2 md:gap-6 px-4 md:px-6  lg:overflow-y-scroll">
                        <div class="flex justify-start gap-2 flex flex-wrap">
                            { previews }
                        </div>

                        <div>
                            <input on:change=on_upload type="file" id="image" name="image" node_ref=upload_image multiple />
                            // <input on:change=on_file_change type="file" id="image" name="image" node_ref=upload_image multiple />
                        </div>


                        <div class="flex flex-col gap-2">
                            <Show when=move || global_state.is_logged_in().unwrap_or_default() >
                                <div class="flex gap-2 place-items-center">
                                    <Errors
                                        error=move||post_api.err_title.get()
                                    />
                                    <Show when=move|| post_api.update_title_mode.get()>
                                        <LengthCounter
                                            class=move || "ml-auto"
                                            counter_current=move||post_api.live_title_length.get()
                                            counter_max=move||MAX_POST_TITLE_LENGTH
                                        />
                                    </Show>
                                    <EditSaveCancel
                                        id=move || "title"
                                        class_edit=move || "ml-auto"
                                        when=move || post_api.update_title_mode.get()
                                        on_save=move || edit_title_save()
                                        on_cancel=move || edit_title_mode_toggle()
                                        on_edit=move || edit_title_mode_toggle()
                                    />
                                </div>
                            </Show>
                            <div class="flex justify-between">
                                <Show when=move || !post_api.update_title_mode.get()>
                                    <h1 class=move || format!("text-[1.5rem] text-ellipsis {}", if title_is_empty() { "text-base03" } else { "text-base0F" })>{ title }</h1>
                                </Show>
                                <Show when=move || post_api.update_title_mode.get()>
                                    <AutoTextArea
                                        id=move||"post_description_editable"
                                        placeholder=move||"title"
                                        node_ref=title_input_editor
                                        on_input=move|v:HtmlTextAreaElement| post_api.live_title_length.set(v.value().len())
                                        min_height=50.0
                                        class=move||"w-full bg-base01 text-[1.5rem] text-base05 px-4 py-2 rounded-xl"
                                    >
                                        {
                                            move || if title_is_empty() {
                                                "".to_string()
                                            } else {
                                                title()
                                            }
                                        }
                                    </AutoTextArea>
                                </Show>

                                // <BtnSecondary id=move|| "btn_edit_title" on_click=move|_|edit_title_mode_toggle() >
                                //     "Edit"
                                // </BtnSecondary>

                                // <button on:click=post_like.on_like.to_fn() disabled=move || post_like.stage.run() == PostLikeStage::Loading class=post_like_btn_style >{ post_like_btn_text }</button>
                            </div>
                            <div class="flex justify-between place-items-start">
                                <div class="flex gap-2">
                                    <p class="text-[1rem] rounded-full h-[3rem] w-[3rem] bg-base05"></p>
                                    <div class="flex flex-col gap-1">
                                        <div class="flex gap-1">
                                            <p class="text-[1rem] text-base03">"by"</p>
                                            <a href=move || post_api.author_link.get() class="text-[1rem] font-bold text-base0B">{ move || post_api.author.get() }</a>
                                        </div>
                                        <p class="text-[1rem]">"9999 followers"</p>
                                    </div>
                                </div>
                                // <div>{move || post_api.favorites.get() }" favorites"</div>
                                <BtnSecondary class=move || format!("flex gap-2 place-items-center ") id=move || "btn_favorite" on_click=move|_|post_like_fn()>
                                    <span class="mt-[0.1rem]">"Favorite"</span>
                                    <Star class=move||"shrink-0 w-[1.5rem] pb-[0.1rem]" fill=move||is_post_liked_fn() />
                                </BtnSecondary>
                            </div>
                        </div>
                        // <div>
                        //     <AutoTextArea/>
                        // </div>
                        <div class="flex flex-col gap-2 md:gap-4 justify-between mt-4">
                            <div class="flex justify-between">
                                <h1 class="text-[1.3rem] text-base0F">"Description"</h1>
                                <div class="flex gap-2 items-center">

                                    <Show when=move || global_state.is_logged_in().unwrap_or_default() >
                                        <Show when=move|| post_api.update_description_mode.get()>
                                            <LengthCounter
                                                counter_current=move||post_api.live_description_length.get()
                                                counter_max=move||MAX_POST_DESCRIPTION_LENGTH
                                            />
                                        </Show>
                                        <EditSaveCancel
                                            id=move || "description"
                                            when=move || post_api.update_description_mode.get()
                                            on_save=move || edit_description_save()
                                            on_cancel=move || edit_description_mode_toggle()
                                            on_edit=move || edit_description_mode_toggle()
                                        />
                                    </Show>
                                </div>
                            </div>

                            <Show when=move || post_api.err_description.with(|v| !v.is_empty()) >
                                <ul id="description_errors" class="ml-[1rem] text-base08 list-disc">
                                    {move || post_api.err_description.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                </ul>
                            </Show>

                            <Show when=move || post_api.update_description_mode.get() fallback=move || view!{
                                <pre
                                    id="post_description"
                                    class=move || format!("whitespace-break-spaces break-all text-ellipsis overflow-hidden padding max-w-[calc(100vw-1rem)] rounded {}",
                                        if post_api.description.with(|v| v.is_empty()) { "text-base03" }  else { "" }

                                        )>
                                    { description }
                                </pre>
                            }>

                            <AutoTextArea
                                id=move||"post_description_editable"
                                node_ref=description_input_editor
                                on_input=move|v:HtmlTextAreaElement| post_api.live_description_length.set(v.value().len())
                                class=move||"bg-base01 text-base05 px-4 py-2 rounded"
                            >
                                { description }
                            </AutoTextArea>
                                // <pre
                                //     id="post_description_editable"
                                //     // on:change=edit_description_keydown
                                //     node_ref=description_input_editor
                                //     contenteditable=true
                                //     class="whitespace-break-spaces break-all text-ellipsis overflow-hidden padding max-w-[calc(100vw-1rem)] bg-base01 text-base05 px-4 py-2 rounded">
                                //     { move || post_api.description.get() }
                                // </pre>
                            </Show>
                        </div>
                        <div class="flex flex-col gap-2 md:gap-4 justify-between mt-4">
                            <div class="flex justify-between">
                                <h1 class="text-[1.3rem] text-base0F">"Tags"</h1>
                                <div class="flex gap-2 items-center">

                                    <Show when=move || global_state.is_logged_in().unwrap_or_default() >
                                        <Show when=move|| post_api.update_tags_mode.get()>
                                            <LengthCounter
                                                counter_current=move||post_api.live_tags_length.get()
                                                counter_max=move||MAX_POST_TAGS_LENGTH
                                            />
                                        </Show>
                                        <EditSaveCancel
                                            id=move || "tags"
                                            when=move || post_api.update_tags_mode.get()
                                            on_save=move || edit_tags_save()
                                            on_cancel=move || edit_tags()
                                            on_edit=move || edit_tags()
                                        />
                                    </Show>

                                </div>

                            </div>
                            <Show when=move || post_api.err_tags.with(|v| !v.is_empty()) >
                                <ul class="ml-[1rem] text-base08 list-disc">
                                    {move || post_api.err_tags.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                </ul>
                            </Show>
                            <div class=move || format!("text-ellipsis flex flex-wrap gap-1 overflow-hidden padding max-w-[calc(100vw-1rem)] {} ",
                                    if post_api.tags.with(|v| v.is_empty()) {"text-base03"} else {"text-base05"}
                                )>

                                <Show when={move || post_api.update_tags_mode.get() } fallback={move || view!{
                                    <Show when={move || post_api.tags.with(|v| !v.is_empty()) } fallback={move || "No tags." }>
                                        { tags }
                                    </Show>
                                } }>
                                    <AutoTextArea
                                        id=move||"post_tags_editable"
                                        node_ref=edit_tags_input
                                        on_input=move|v:HtmlTextAreaElement| post_api.live_tags_length.set(v.value().len())
                                        class=move||"text-[1.1rem] break-all focus:outline-none! appearance-none border-none resize w-full rounded bg-base01 px-4 py-2"
                                        min_height=100.0
                                    >
                                         {move || post_api.tags.get()}
                                    </AutoTextArea>
                                    // <div contenteditable=true
                                    //      node_ref=edit_tags_input
                                    //      class={move || format!("  ")}>
                                    //      {move || post_api.tags.get()}
                                    // </div>
                                </Show>
                             </div>
                        </div>
                        <div  class="flex flex-col gap-2 md:gap-4 justify-between mt-4 pb-1">
                            <h1 class="text-[1.3rem] text-base0F ">"Comments"</h1>
                            <div class=move || format!( "bg-base01 rounded-xl grid place-items-center py-5 px-2 {}", if  global_state.acc_pending() { "" } else { "hidden" })>
                                <div class="flex flex-col gap-2">
                                    <div class="text-base03">"loading..."</div>
                                </div>
                            </div>
                            <div class=move || format!( "bg-base01 rounded-xl grid place-items-center py-5 px-2 {}", if global_state.is_logged_in().unwrap_or_default() || global_state.acc_pending() { "hidden" } else { "" })>
                                <div class="flex flex-col gap-2">
                                    <div class="text-base03">"You must login to comment"</div>
                                    <a class="mx-auto rounded-full font-semibold text-[1rem] font-medium px-[0.8rem] py-[0.2rem] hover:bg-base05 bg-base0D text-base01" href=PATH_LOGIN >"Login"</a>
                                </div>
                            </div>
                            // <form class=move || format!("flex bg-base01 rounded-xl flex-col gap-2 py-2 px-4 {}", if global_state.is_logged_in().unwrap_or_default()  { "" } else { "hidden" }) on:submit=post_comments.on_comment.to_fn() >
                            <div class=move || format!("flex bg-base01 rounded-xl flex-col gap-1 py-2 px-4 {}", if global_state.is_logged_in().unwrap_or_default()  { "" } else { "hidden" }) >
                                <textarea placeholder="Comment" node_ref=comment_input_ref class="focus:outline-none! appearance-none border-none resize text-[1.1rem]" id="story" name="story" rows="3" cols="5" ></textarea>
                                <ul class="text-base08 list-disc ml-[1rem]">
                                    {move || comment_basic.err_post.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                </ul>
                                <div class="flex justify-between place-items-center">
                                    <p class="text-[1rem]">"0/2000"</p>
                                    <BtnPrimary id=move|_:()|String::new() on_click=move |_| post_comment() class=move || "ml-auto">
                                        "Post"
                                    </BtnPrimary>
                                    // <BtnPrimary on_click=move |_| post_comment()>
                                    //     "Post" <Star class="size-5 mb-[0.1rem]"/>
                                    // </BtnPrimary>
                                </div>
                            </div>

                            <div class="flex flex-col gap-2">
                                <div node_ref=comment_container_ref class=" flex flex-col gap-2 relative 0h-[20rem] 0overflow-y-scroll">
                                    <For
                                        each=move || comment_basic.items.get()
                                        key=|state| state.key.clone()
                                        let(data)
                                    >
                                        {
                                            view!{
                                                <PostCommentElm
                                                    parent_key=String::new()
                                                    parent_items=comment_basic.items
                                                    parent_reply_count=comment_basic.replies_count
                                                    comment=data
                                                    param_post
                                                    max_depth=2
                                                    parent_depth=0 />
                                            }.into_any()
                                        }
                                    </For>
                                </div>
                                <Show when=move || comment_basic.comments_manual.err_fetch.with(|v| !v.is_empty()) >
                                    <ul class="ml-[1rem] text-base08 list-disc">
                                        {move || comment_basic.comments_manual.err_fetch.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                    </ul>
                                </Show>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>


            // TODO probably change 1fr to fixed size or auto or minmax bs
        </main>
    }
}

#[component]
pub fn LengthCounter(
    #[prop(optional, into)] class: Option<Callback<(), String>>,
    #[prop(optional, into)] counter_current: Option<Callback<(), usize>>,
    #[prop(optional, into)] counter_max: Option<Callback<(), usize>>,
) -> impl IntoView {
    let counter_current_fn = move || counter_current.map(|v| v.run(())).unwrap_or_default();
    let counter_max_fn = move || counter_max.map(|v| v.run(())).unwrap_or_default();
    let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();

    view! {
        <div class=move || format!("{} {}", if counter_current_fn() >= counter_max_fn() {"text-base08"} else {""}, class_fn())>
            <span id="description_length">{counter_current_fn}</span>"/"{counter_max_fn}
        </div>
    }
}

#[component]
pub fn EditSaveCancel(
    #[prop(optional, into)] when: Option<Callback<(), bool>>,
    #[prop(optional, into)] disable_save_when: Option<Callback<(), bool>>,
    #[prop(optional, into)] class_save: Option<Callback<(), String>>,
    #[prop(optional, into)] class_cancel: Option<Callback<(), String>>,
    #[prop(optional, into)] class_edit: Option<Callback<(), String>>,
    #[prop(optional, into)] id: Option<Callback<(), String>>,
    #[prop(optional, into)] on_save: Option<Callback<()>>,
    #[prop(optional, into)] on_cancel: Option<Callback<()>>,
    #[prop(optional, into)] on_edit: Option<Callback<()>>,
    // #[prop(optional, into)] class: Option<Callback<(), String>>,
    // #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
    // children: Children,
) -> impl IntoView {
    // let global_state = expect_context::<GlobalState>();
    let when_fn = move || when.map(|v| v.run(())).unwrap_or_default();
    let disable_save_when_fn = move || disable_save_when.map(|v| v.run(())).unwrap_or_default();
    let id_fn = move || id.map(|v| v.run(())).unwrap_or_default();
    let class_save = move || class_save.map(|v| v.run(())).unwrap_or_default();
    let class_cancel = move || class_cancel.map(|v| v.run(())).unwrap_or_default();
    let class_edit = move || class_edit.map(|v| v.run(())).unwrap_or_default();
    let on_save_fn = move |e| {
        if let Some(f) = on_save {
            f.run(());
        }
    };
    let on_cancel_fn = move |_| {
        if let Some(f) = on_cancel {
            f.run(());
        }
    };
    let on_edit_fn = move |_| {
        if let Some(f) = on_edit {
            f.run(());
        }
    };
    // let id_fn = move || id.map(|v| v.run(())).unwrap_or_default();
    // let class_fn = move || class.map(|v| v.run(())).unwrap_or_default();

    view! {
        <Show when=when_fn >
            <BtnPrimary class=move || format!("w-[5rem] {}", class_save()) id=move || format!("btn_save_{}", id_fn()) on_click=on_save_fn>
                "Save"
            </BtnPrimary>
            <BtnSecondary class=move || format!("w-[5rem] {}", class_cancel()) id=move || format!("btn_cancel_{}", id_fn()) on_click=on_cancel_fn>
                "Cancel"
            </BtnSecondary>
        </Show>
        <Show when=move || !when_fn() >
            <BtnSecondary class=move || format!("w-[5rem] {}", class_edit()) id=move || format!("btn_edit_{}", id_fn()) on_click=on_edit_fn>
                "Edit"
            </BtnSecondary>
        </Show>
    }
}

#[component]
pub fn SVGTrash(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class=class>
          <path stroke-linecap="round" stroke-linejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
        </svg>
    }
}

#[component]
pub fn SVGArrowDown(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class=class>
          <path stroke-linecap="round" stroke-linejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" />
        </svg>
    }
}

#[component]
pub fn SVGTriangle(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <svg width="12" height="11" viewBox="0 0 12 11" fill="none" xmlns="http://www.w3.org/2000/svg" class=class>
            <path d="M6.63067 9.75C6.24577 10.4167 5.28352 10.4167 4.89862 9.75L0.135483 1.5C-0.249417 0.833333 0.231708 -2.83122e-07 1.00151 -2.83122e-07L10.5278 -2.83122e-07C11.2976 -2.83122e-07 11.7787 0.833333 11.3938 1.5L6.63067 9.75Z" fill="currentColor"/>
        </svg>
    }
}

#[component]
pub fn PostCommentElm(
    parent_key: String,
    parent_items: RwSignal<Vec<UserPostComment>, LocalStorage>,
    parent_reply_count: RwSignal<usize, LocalStorage>,
    comment: UserPostComment,
    param_post: Memo<Option<String>>,
    max_depth: usize,
    parent_depth: usize,
) -> impl IntoView {
    let current_depth = parent_depth + 1;
    let global_state = expect_context::<GlobalState>();
    let comment_container_ref = NodeRef::<html::Div>::new();
    let comment_edit_ref = NodeRef::new();
    let comment_input_ref = NodeRef::<html::Textarea>::new();
    let flatten = current_depth >= max_depth;
    let reply_render_comments = current_depth <= max_depth;
    // let reply_btn_shown = RwSignal::new(false);
    let replies_shown = RwSignal::new(false);
    // let edit_enabled = RwSignal::new(false);
    let api = ApiWeb::new();
    let comment_key = comment.key.clone();
    let is_owned_fn = {
        let key = comment.user.key.clone();
        move || {
            global_state
                .get_acc_id_tracked()
                .map(|v| v == key)
                .unwrap_or_default()
        }
    };

    let comment_edit_event = EventListener::new(ev::change, |a| {
        trace!("omg is it working edit magic");

        //
    });
    Effect::new(move || {
        let Some(elm) = comment_edit_ref.get() else {
            return;
        };
        comment_edit_event.add(elm);
    });

    let spawner = Spawner::new();
    let kind = if current_depth < max_depth {
        CommentKind2::Reply {
            parent_key: parent_key.clone(),
            parent_items: parent_items,
            parent_replies_count: parent_reply_count,
            comment: comment.clone(),
        }
    } else if current_depth == max_depth {
        CommentKind2::Flat {
            parent_key: parent_key.clone(),
            parent_items: parent_items,
            parent_replies_count: parent_reply_count,
            comment: comment.clone(),
        }
    } else {
        CommentKind2::None {
            parent_key: parent_key.clone(),
            parent_items: parent_items,
            parent_replies_count: parent_reply_count,
            comment: comment.clone(),
        }
    };

    let comments_manual = CommentsApi2::new(api, 10, kind.clone());
    let post_comment = move |_| {
        let Some(input_elm) = comment_input_ref.get() else {
            return;
        };
        spawner.spawn(async move {
            let text = input_elm.value();
            comments_manual.post(text).await;
            let post_err = comments_manual.err_post.get_untracked();
            if !post_err.is_empty() {
                return;
            }
            input_elm.set_value("");
        });
    };
    let delete_comment = move |_| {
        spawner.spawn(comments_manual.delete());
    };
    let fetch_comments = move || {
        spawner.spawn(comments_manual.fetch());
    };
    let toggle_btn = move |_| {
        comments_manual.show_editor.update(|v| *v = !*v);
        let show = comments_manual.show_editor.get_untracked();
        if !show {
            return;
        }
        replies_shown.set(true);
        spawner.spawn(comments_manual.fetch());
    };
    let toggle_replies = move |_| {
        trace!("KILL ME YOU FUCK");
        replies_shown.update(|v| *v = !*v);
        let show = replies_shown.get_untracked();
        trace!("KILL ME YOU FUCK 2 {show}");
        if !show {
            return;
        }
        trace!("KILL ME YOU FUCK 3 {show}");
        spawner.spawn(comments_manual.fetch());
    };

    Effect::new(move || {
        trace!("comments manual start");
        let (Some(post_id),) = (
            param_post.get(),
            // comment_container_ref.get(),
        ) else {
            return;
        };

        trace!("comments manual observe depth({current_depth})");
        comments_manual.observe_only(post_id);
    });

    let show_replies_fn = move || {
        (reply_render_comments && replies_shown.get())
            || (reply_render_comments && comments_manual.show_editor.get())
    };
    let show_line = move || {
        current_depth > 0 && comments_manual.items.with(|v| v.len() > 0) && show_replies_fn()
    };

    let is_bubble = 'f: {
        if !kind.is_none() {
            break 'f false;
        }
        let Some(last) = comment.parent_key.last() else {
            break 'f false;
        };
        *last != parent_key
    };

    let bubble = 'f: {
        if !is_bubble {
            break 'f None;
        }
        // let last = comment.parent_key.last().cloned();
        let Some(last) = comment.parent_key.last() else {
            break 'f None;
        };
        parent_items.with(|v| v.iter().find(|v| v.key == *last).cloned())
    };

    let on_bubble_click = {
        let bubble = bubble.clone();
        let comment_key = comment_key.clone();
        move || {
            let Some(elm) = bubble.and_then(|v| document().get_element_by_id(&v.key)) else {
                warn!("cant find element for bubble click {}", comment_key);
                return;
            };
            let options = ScrollIntoViewOptions::new();
            options.set_behavior(ScrollBehavior::Auto);
            options.set_block(ScrollLogicalPosition::Center);
            options.set_inline(ScrollLogicalPosition::Center);
            elm.scroll_into_view_with_scroll_into_view_options(&options);
            let anim = "animate-[glow_1s_linear]";
            let classes = elm.class_list();
            let _ = classes.add_1(anim);
            // elm.set_class_name(anim);
            let result = set_timeout(
                move || {
                    let _ = classes.remove_1(anim);
                },
                Duration::from_secs(1),
            );
            if let Err(err) = result {
                error!("{err}");
            }
        }
    };
    let on_bubble_click_fn = move |_| {
        (on_bubble_click.clone())();
    };

    let click_edit = move |_| {
        // edit_enabled.update(|v| *v = !*v);
        if comments_manual.edit_mode.get_untracked() {
            let Some(text) = comment_edit_ref
                .get_untracked()
                .and_then(|v: HtmlDivElement| v.text_content())
            else {
                return;
            };
            spawner.spawn(comments_manual.update_comment(text));
        }

        comments_manual.edit_mode.set(true);
    };

    let click_cancel = move |_| {
        let Some(elm) = comment_edit_ref.get_untracked() as Option<HtmlDivElement> else {
            return;
        };

        let txt = comments_manual.text.get_untracked();

        elm.set_text_content(Some(&txt));

        comments_manual.err_update.update(|v| v.clear());
        comments_manual.edit_mode.set(false);
    };

    view! {
        <div class=" flex flex-col "  >
            <div id=comment.key.clone() class=" rounded 0bg-base03 flex flex-col">
                <Show when=move || is_bubble>
                    <button on:click=on_bubble_click_fn.clone() class="cursor-pointer flex gap-2 items-center">
                        <div class="flex place-items-end h-[1.5rem] w-[3.2rem] shrink-0">
                            <div class=" mb-[0.5rem] w-[1.7rem] h-[0.5rem] border-base05 border-l-[0.2rem] border-t-[0.2rem] rounded-tl-[2rem] ml-auto box-border shrink-0"></div>
                        </div>
                        <p class="ml-2 text-[1rem] rounded-full h-[1rem] w-[1rem] shrink-0 bg-base05"></p>
                        <div>
                            {bubble.clone().map(|v| v.text).unwrap_or_else(|| "failed to load msg".to_string())}
                        </div>
                    </button>
                </Show>
                <div class="grid grid-cols-[auto_1fr] grid-rows-[100%] ">
                    <div class=" mb-[0.5rem] w-[3.2rem] h-full grid grid-rows-[auto_100%] items-start place-items-center shrink-0">
                        <p class="text-[1rem] rounded-full h-[3.2rem] w-[3.2rem] shrink-0 bg-base05"></p>
                        <Show when={move || { !comments_manual.is_last() } || show_replies_fn() }>
                            <div class="rounded w-[0.2rem] mt-[0.5rem] h-[calc(100%-3.2rem-0.5rem)] bg-base05 shrink-0"></div>
                        </Show>
                    </div>
                    <div  class="pl-4  flex flex-col w-full group">
                        <div class="flex gap-2 place-items-center ">
                            <div class="text-[1.2rem]"> {comment.user.username} </div>
                            <div class="text-[1rem] text-base03"> {move || ns_to_str(global_state.get_time_ns().saturating_sub(comment.created_at))}" ago"</div>

                            <Show when={move || is_owned_fn() || comments_manual.edit_mode.get()} >
                                <div class=move || format!(" gap-2 ml-auto place-items-center {}", if comments_manual.edit_mode.get() {"flex"} else {"group-hover:flex hidden"} )>
                                    <button on:click=click_edit class=move || format!("text-center   rounded-full font-semibold text-[0.8rem] font-medium px-[0.8rem] w-[4rem]  {}", if comments_manual.edit_mode.get() { " hover:bg-base05 bg-base0D text-base01" } else { " text-base05 bg-base01 hover:bg-base05 hover:text-base01" })>
                                        <Show when={move || comments_manual.edit_mode.get() } fallback={move || "Edit" }>
                                            "Save"
                                        </Show>
                                    </button>
                                    <Show when=move || comments_manual.edit_mode.get() >
                                        <button on:click=click_cancel class=move || format!("text-center  rounded-full font-semibold text-[0.8rem] font-medium px-[0.8rem] w-[4rem] text-base05 bg-base01 hover:bg-base05 hover:text-base01")>
                                            "Cancel"
                                        </button>
                                    </Show>
                                    <Show when=move || !comments_manual.edit_mode.get() >
                                        <button on:click=delete_comment class="">
                                            <SVGTrash class="size-[1.1rem] text-base08 "/>
                                        </button>
                                    </Show>
                                </div>
                            </Show>
                        </div>


                        <div contenteditable={move || comments_manual.edit_mode.get()}
                             node_ref=comment_edit_ref
                             class={move || format!(" text-[1.1rem] break-all focus:outline-none! appearance-none border-none resize w-full rounded {}", if comments_manual.edit_mode.get() { "bg-base01 px-4 py-2" } else { "" })} >
                            {
                                move || comments_manual.text.get()
                            }
                        </div>
                        <Show when=move || comments_manual.err_update.with(|v| !v.is_empty()) >
                            <ul class="ml-[1rem] text-base08 list-disc">
                                {move || comments_manual.err_update.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                            </ul>
                        </Show>
                        <Show when=move || comments_manual.err_delete.with(|v| !v.is_empty()) >
                            <ul class="ml-[1rem] text-base08 list-disc">
                                {move || comments_manual.err_delete.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                            </ul>
                        </Show>
                        // <div class=" mb-2 text-[1.1rem] break-all"> {comment.text} </div>
                        <div class=" h-[1.6rem] flex gap-2 place-items-center">
                            <Show when=move || reply_render_comments >
                                // <button on:click=toggle_replies type="submit" class=move || format!("group  gap-1 flex place-items-center rounded-full font-semibold text-[0.8rem] font-medium px-[0.8rem] py-[0.2rem]  {}", if replies_shown.get() { "text-base05 bg-base01 hover:bg-base03" } else { "text-base05 bg-base01 hover:bg-base05 hover:text-base01" })>
                                <Show when=move || {comments_manual.replies_count.get() > 0} fallback=move || view!{
                                    <p class=move || format!("group text-base03 gap-1 flex place-items-center rounded-full font-semibold text-[0.8rem] font-medium ")>
                                        <div class="0group-hover:bg-base01 size-3 bg-base03 aspect-square rounded mx-auto"/>
                                        "no replies"
                                    </p>

                                }>
                                    <button on:click=toggle_replies class=move || format!("group  gap-1 flex place-items-center rounded-full font-semibold text-[0.8rem] font-medium ")>
                                        <Show when=move || replies_shown.get() fallback={|| view!{
                                            <div class="0group-hover:bg-base01 size-3 bg-base05 aspect-square rounded mx-auto"/>
                                        }}>
                                            <SVGTriangle class="size-3 mx-auto"/>
                                        </Show>
                                        <Show when=move || !comments_manual.kind.with_value(|v| v.is_flat()) fallback={|| "replies"}>
                                            {move || comments_manual.replies_count.get() }
                                            " replies"
                                        </Show>
                                    </button>
                                </Show>
                            </Show>
                            <Show when=move || global_state.is_logged_in().unwrap_or_default()>
                                <button on:click=toggle_btn type="submit" class=move || format!("  rounded-full font-semibold text-[0.8rem] font-medium px-[0.8rem] w-[4rem]  {}", if comments_manual.show_editor.get() { "text-base05 bg-base01 hover:bg-base03" } else { "text-base05 bg-base01 hover:bg-base05 hover:text-base01" })>
                                    <Show when=move || comments_manual.show_editor.get() fallback=|| "Reply">
                                        <SVGArrowDown class="size-4 mx-auto"/>
                                    </Show>
                                </button>
                            </Show>
                        </div>
                        <Show when=move || comments_manual.show_editor.get()>
                            <div class=move || format!("flex bg-base01 rounded-xl flex-col gap-2 py-2 px-4 w-full {}", if global_state.is_logged_in().unwrap_or_default() || !global_state.acc_pending() { "" } else { "hidden" })  >
                                <textarea placeholder="Comment" node_ref=comment_input_ref class="focus:outline-none! appearance-none border-none resize text-[1.1rem] w-full" rows="2" wrap="hard"  ></textarea>
                                // <ul class="text-base08 list-disc ml-[1rem]">
                                //     {move || post_comments.err_post.get().map(|v| v.trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view()) }
                                // </ul>on:submit=post_comment
                                <ul class="text-base08 list-disc ml-[1rem]">
                                    {move || comments_manual.err_post.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                </ul>
                                <div class="flex justify-between place-items-center">
                                    <p>"0/2000"</p>
                                    <button on:click=post_comment class="ml-auto rounded-full font-medium text-[0.8rem] font-bold px-[0.8rem] py-[0.2rem] hover:bg-base0D bg-base03 text-base05 text-center w-[5rem]">
                                        "Reply"
                                    </button>
                                </div>
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
            <div class=move || format!("grid grid-rows-[100%] grid-cols-[auto_1fr] w-full {}", if reply_render_comments && show_replies_fn() {"pt-2"} else {""})>
                <Show when=show_line>
                    <div class="relative ml-[1.5rem] w-[1rem] h-full flex justify-sart shrink-0">
                        <div class="w-[1rem] h-[1.61rem] border-base05 border-l-[0.2rem] border-b-[0.2rem] rounded-bl-[2rem] ml-auto box-border shrink-0"></div>
                        <Show when=move || { !comments_manual.is_last() }>
                            <div class="absolute w-[0.2rem] h-full bg-base05 shrink-0"></div>
                        </Show>
                    </div>
                </Show>
                // <form class=move || format!("mb-4 flex bg-base01 rounded-xl flex-col gap-2 py-2 px-4 w-full {}", if global_state.is_logged_in().unwrap_or_default() || !global_state.acc_pending() { "" } else { "hidden" }) on:submit=post_comments.on_comment.to_fn() >

                <div class="flex flex-col w-full">
                    // <Show when=move || reply_render_comments && (replies_shown.get() || comments_manual.reply_editor_show.get())>
                    <Show when=move || reply_render_comments>
                        <div node_ref=comment_container_ref class=move || format!("flex flex-col gap-2 0h-[20rem] 0overflow-y-scroll {} ", if show_replies_fn() {""} else {"hidden"} )>
                            {
                                let comment_key = comment_key.clone();

                                view! {
                                    <For
                                        each=move || comments_manual.items.get()
                                        key=|state| state.key.clone()
                                        let(data)
                                    >
                                        {
                                            // let key = data.key.clone();
                                            // let is_last = comments_manual.items.with(|v| v.last().map(|v| v.key == key).unwrap_or_default());
                                            let comment_key = comment_key.clone();
                                            view!{
                                                <PostCommentElm
                                                    parent_key=comment_key.clone()
                                                    parent_items=comments_manual.items
                                                    parent_reply_count=comments_manual.replies_count
                                                    comment=data
                                                    param_post max_depth=max_depth parent_depth=current_depth />
                                            }.into_any()
                                        }
                                    </For>
                                }
                            }
                            <button on:click=move |_| { fetch_comments(); } class=move || format!("px-4 py-2 bg-base01 rounded-xl text-center text-base05 font-[1.2rem] w-full {}", if comments_manual.finished.get() {"hidden"} else {""})>
                                "load more"
                            </button>
                            <Show when=move || comments_manual.err_fetch.with(|v| !v.is_empty()) >
                                <ul class="text-base08 list-disc">
                                    {move || comments_manual.err_fetch.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                                </ul>
                            </Show>
                        </div>
                        // { post_comment_views }
                    </Show>
                </div>

            </div>
        </div>
    }
}
