use crate::api::shared::post_comment::UserPostComment;
use crate::api::{Api, ApiWeb, Server404Err, ServerErr};
use crate::path::{PATH_LOGIN, link_home, link_img, link_user};
use crate::view::app::GlobalState;
use crate::view::app::components::nav::Nav;
use crate::view::app::hook::use_event_listener::EventListener;
use crate::view::app::hook::use_future::FutureFn;
use crate::view::app::hook::use_infinite_scroll_fn::InfiniteScrollFn;
use crate::view::app::hook::use_infinite_scroll_virtual::{
    InfiniteStage, use_infinite_scroll_virtual,
};
use crate::view::app::hook::use_post_comment::use_post_comment;
use crate::view::app::hook::use_post_comments_baisc::CommentsBaisc;
use crate::view::app::hook::use_post_comments_manual::{
    CommentKind, CommentKind2, CommentsApi, CommentsApi2,
};
use crate::view::app::hook::use_post_like::{self, PostLikeStage, use_post_like};
use crate::view::app::hook::use_spawner::Spawner;
use crate::view::toolbox::prelude::*;
use leptos::{Params, task::spawn_local};
use leptos::{ev, html, prelude::*};
use leptos_router::hooks::{use_location, use_params};
use leptos_router::params::Params;
use tracing::{debug, error, trace};
use web_sys::{Event, SubmitEvent};

#[derive(Params, PartialEq, Clone)]
pub struct PostParams {
    pub username: Option<String>,
    pub post: Option<String>,
}

#[component]
pub fn Page() -> impl IntoView {
    let main_ref = NodeRef::new();
    let api = ApiWeb::new();
    let global_state = expect_context::<GlobalState>();

    let param = use_params::<PostParams>();
    let param_username = move || param.read().as_ref().ok().and_then(|v| v.username.clone());
    let param_post = Memo::new(move |_| param.read().as_ref().ok().and_then(|v| v.post.clone()));
    let imgs_links = RwSignal::new(Vec::<(String, f64)>::new());
    let title = RwSignal::new(String::new());
    let author = RwSignal::new(String::new());
    let description = RwSignal::new(String::from("loading..."));
    let favorites = RwSignal::new(0_u64);
    let not_found = RwSignal::new(false);
    let location = use_location();

    // let rw_signal_tree = RwSignalTree::<String, Vec<UserPostComment>>::new_root();

    // let infinite_fn = InfiniteScrollFn::new(move |v| {
    //     debug!("boopboopbaap");
    // });
    //
    // Effect::new(move || {
    //     let Some(elm) = comment_container_ref.get() else {
    //         return;
    //     };
    //
    //     (infinite_fn.on.to_fn())(elm.into());
    // });
    let virt_comment_input_ref = NodeRef::<html::Textarea>::new();
    let virt_comment_container_ref = NodeRef::<html::Div>::new();
    // let post_comments = use_post_comment(
    //     false,
    //     10,
    //     virt_comment_container_ref,
    //     virt_comment_input_ref,
    //     param_post,
    //     None::<String>,
    // );
    let spawner = Spawner::new();

    let comment_container_ref = NodeRef::<html::Div>::new();
    let comment_input_ref = NodeRef::<html::Textarea>::new();
    let comment_basic = CommentsBaisc::new(api, spawner);
    let post_comment = move |e: SubmitEvent| {
        e.prevent_default();
        comment_basic.post.run();
    };
    Effect::new(move || {
        trace!("comments basic start");
        let (Some(post_id), Some(comment_input), Some(comment_container_ref)) = (
            param_post.get(),
            comment_input_ref.get(),
            comment_container_ref.get(),
        ) else {
            return;
        };

        trace!("comments basic observe");
        spawner.spawn(comment_basic.observe_only(
            // comment_input,
            comment_container_ref.into(),
            post_id,
            // String::new(),
            // 10,
        ));
    });

    // let post_comment_views = move || {
    //     let time_now = global_state.get_time_ns();
    //
    //     post_comments
    //         .data
    //         .get()
    //         .into_iter()
    //         .map(move |comment| {
    //             view! {
    //                 <PostCommentElm comment param_post max_depth=0 parent_depth=0 />
    //             }
    //         })
    //         .collect_view()
    // };

    let post_like = use_post_like(param_post);
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
        let (Some(username), Some(post_id)) = (param_username(), param_post.get()) else {
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
                }
                Ok(res) => {
                    error!("wrong res, expected Post, got {:?}", res);
                }
                Err(ServerErr::NotFoundErr(Server404Err::NotFound)) => {
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
        <main node_ref=main_ref class="relative font-hi grid grid-rows-[auto_1fr] h-screen text-base05">
            <Nav/>

            <div class=move || format!("place-items-center text-[1.5rem] {}", if not_found.get() {"grid"} else {"hidden"})>
                "Not Found"
            </div>

            // <div class="z-20 fixed bg-base00 w-[100dvw] h-[100dvh] grid grid-rows-1">
            //     <div node_ref=virt_comment_container_ref class=" flex flex-col gap-2 relative overflow-y-scroll">
            //         <For
            //             each=move || post_comments.data.get()
            //             key=|state| state.key.clone()
            //             let(data)
            //         >
            //             {
            //                 view!{
            //                     <PostCommentElm comment=data parent_items=None param_post max_depth=2 parent_depth=0 />
            //                 }.into_any()
            //             }
            //         </For>
            //         // { post_comment_views }
            //     </div>
            //
            // </div>

            <div class=move || format!("flex flex-col lg:grid grid-cols-[2fr_1fr] grid-cols-[2fr_1fr] lg:max-h-[calc(100vh-3rem)] gap-2 px-4 md:gap-6 md:px-6 {}", if not_found.get() {"hidden"} else {"flex"})>
                <div class="lg:hidden h-[50vh] flex justify-center place-items-center bg-base02" >
                    { selected_img }
                </div>
                <div class="hidden lg:flex flex-col gap-2 lg:overflow-y-scroll" >
                    { imgs }
                </div>
                <div class="flex flex-col gap-2 md:gap-6 lg:overflow-y-scroll">
                    <div class="flex justify-start gap-2 flex flex-wrap">
                        { previews }
                    </div>


                    <div class="flex flex-col gap-2">
                        <div class="flex justify-between">
                            <h1 class="text-[1.5rem] text-ellipsis text-base0F">{ fn_title }</h1>
                            <button on:click=post_like.on_like.to_fn() disabled=move || post_like.stage.run() == PostLikeStage::Loading class=post_like_btn_style >{ post_like_btn_text }</button>
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
                        <h1 class="text-[1.3rem] text-base0F">"Description"</h1>
                        <div class=move || format!("text-ellipsis overflow-hidden padding max-w-[calc(100vw-1rem)] {}", if fn_description_is_empty() {"text-base03"} else {"text-base05"} )>{fn_description}</div>
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
                        <form class=move || format!("flex bg-base01 rounded-xl flex-col gap-2 py-2 px-4 {}", if global_state.is_logged_in().unwrap_or_default()  { "" } else { "hidden" })  on:submit=post_comment>
                            <textarea placeholder="Comment" node_ref=comment_input_ref class="focus:outline-none! appearance-none border-none resize text-[1.1rem]" id="story" name="story" rows="5" cols="30" ></textarea>
                            <ul class="text-base08 list-disc ml-[1rem]">
                                {move || comment_basic.err_post.get().trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view() }
                            </ul>
                            <div class="flex justify-between place-items-center">
                                <p>"0/2000"</p>
                                <input type="submit" value="Post" class="ml-auto rounded-full font-medium text-[1.2rem] font-bold px-[0.9rem] py-[0.3rem] hover:bg-base05 bg-base0D text-base01"/>
                            </div>
                        </form>

                        <div node_ref=comment_container_ref class=" flex flex-col relative 0h-[20rem] 0overflow-y-scroll">
                            <For
                                each=move || comment_basic.items.get()
                                key=|state| state.key.clone()
                                let(data)
                            >
                                {
                                    // let key = data.key.clone();
                                    // let is_last = || comment_basic.items.with(|v| v.last().map(|v| v.key == key).unwrap_or_default());
                                    view!{
                                        <PostCommentElm parent=comment_basic.items comment=data param_post max_depth=2 parent_depth=0 />
                                    }.into_any()
                                }
                            </For>
                            // { post_comment_views }
                        </div>
                    </div>
                </div>
            </div>
        </main>
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
    parent: RwSignal<Vec<UserPostComment>, LocalStorage>,
    comment: UserPostComment,
    param_post: Memo<Option<String>>,
    max_depth: usize,
    parent_depth: usize,
    // is_last: bool,
    // comment_key: Option<String>,
) -> impl IntoView {
    // let a = Some(true);
    // let b = Some(false);
    // let c = a.and_then(|a| Some(a && b?));
    // let c = a.map(|a| a && b?);
    // let c = a == Some(true) || b == Some(true);
    // let a = a == Some(true);

    let current_depth = parent_depth + 1;
    let global_state = expect_context::<GlobalState>();
    let comment_container_ref = NodeRef::<html::Div>::new();
    let comment_edit_ref = NodeRef::new();
    let comment_input_ref = NodeRef::<html::Textarea>::new();
    let flatten = current_depth >= max_depth;
    let reply_render_comments = current_depth <= max_depth;
    // let reply_btn_shown = RwSignal::new(false);
    let replies_shown = RwSignal::new(false);
    let api = ApiWeb::new();
    let replies_count = RwSignal::new(comment.replies_count);
    // let is_last = {
    //     let key = comment.key.clone();
    //     move || {
    //         parent_items.with(|v| v.map(|v| v.))
    //     }
    // };

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

    let comments_manual = CommentsApi2::new(
        api,
        10,
        if current_depth < max_depth {
            CommentKind2::Comment {
                parent,
                comment: comment.clone(),
            }
        } else if current_depth == max_depth {
            CommentKind2::Flat {
                parent,
                comment: comment.clone(),
            }
        } else {
            CommentKind2::None {
                parent,
                comment: comment.clone(),
            }
        },
    );
    let post_comment = move |_| {
        // e.prevent_default();
        trace!("comments manual posting 0");
        // comments_manual.post();
        trace!("comments manual posting");
        // reply_btn_shown.update(|v| *v = !*v)
    };
    let delete_comment = {
        let comment_key = comment.key.clone();
        move |_| {
            // comments_manual.delete.run(comment_key.clone());
        }
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
    // let fetch_comment_btm = move |_| {
    //     comments_manual.fetch_btm();
    // };

    Effect::new(move || {
        // if !reply_render_comments {
        //     return;
        // }
        trace!("comments manual start");
        let (Some(post_id),) = (
            param_post.get(),
            // comment_container_ref.get(),
        ) else {
            return;
        };

        trace!("comments manual observe depth({current_depth})");
        comments_manual.observe_only(post_id);
        // comments_manual.observe_only(comment_input_ref.get(), post_id, 10, flatten);
        // comments_manual.fetch_btm();
    });

    let show_replies_fn = move || {
        (reply_render_comments && replies_shown.get())
            || (reply_render_comments && comments_manual.show_editor.get())
    };
    let show_line = move || {
        current_depth > 0 && comments_manual.items.with(|v| v.len() > 0) && show_replies_fn()
    };

    view! {

        // <div class="flex flex-col gap-4 px-2 py-1 " style:padding-left=format!("{:.3}rem", current_depth as f32 * 0.8) >
        <div class="flex flex-col "  >
            <div class="grid grid-cols-[auto_1fr] grid-rows-[100%] gap-4">
                <div class="w-[3.2rem] h-full grid grid-rows-[auto_100%] items-start place-items-center shrink-0">
                    <p class="text-[1rem] rounded-full h-[3.2rem] w-[3.2rem] shrink-0 bg-base05"></p>
                    <Show when={move || { !comments_manual.is_last() } || show_replies_fn() }>
                        <div class="w-[0.2rem] h-[calc(100%-3.2rem)] bg-base05 shrink-0"></div>
                    </Show>
                </div>
                <div class="flex flex-col w-full group">
                    <div class="flex gap-2 place-items-start ">
                        <div class="text-[1.2rem]"> {comment.user.username} </div>
                        <div class="text-[1rem] text-base03"> {ns_to_str(global_state.get_time_ns().saturating_sub(comment.created_at))}" ago"</div>
                        <button on:click=delete_comment>
                            <SVGTrash class="size-6 text-base08 ml-auto hidden group-hover:flex"/>
                        </button>
                    </div>


                    <span contenteditable node_ref=comment_edit_ref class=" text-[1.1rem] break-all focus:outline-none! appearance-none border-none resize text-[1.1rem] w-full" >{comment.text}</span>
                    // <div class=" mb-2 text-[1.1rem] break-all"> {comment.text} </div>
                    <div class="mb-2 flex gap-2 place-items-center">
                        <Show when=move || reply_render_comments >
                            // <button on:click=toggle_replies type="submit" class=move || format!("group  gap-1 flex place-items-center rounded-full font-semibold text-[0.8rem] font-medium px-[0.8rem] py-[0.2rem]  {}", if replies_shown.get() { "text-base05 bg-base01 hover:bg-base03" } else { "text-base05 bg-base01 hover:bg-base05 hover:text-base01" })>
                            <Show when=move || {replies_count.get() > 0} fallback=move || view!{
                                <p class=move || format!("group text-base03 gap-1 flex place-items-center rounded-full font-semibold text-[0.8rem] font-medium ")>
                                    <div class="0group-hover:bg-base01 size-3 bg-base03 aspect-square rounded mx-auto"/>
                                    "no replies"
                                </p>

                            }>
                                <button on:click=toggle_replies class=move || format!("group  gap-1 flex place-items-center rounded-full font-semibold text-[0.8rem] font-medium ")>
                                    <Show when=move || replies_shown.get() fallback={|| view!{<div class="0group-hover:bg-base01 size-3 bg-base05 aspect-square rounded mx-auto"/>}}>
                                        <SVGTriangle class="size-3 mx-auto"/>
                                    </Show>
                                    {move || replies_count.get() }
                                    " replies"
                                </button>
                            </Show>
                        </Show>
                        <Show when=move || global_state.is_logged_in().unwrap_or_default()>
                            <button on:click=toggle_btn type="submit" class=move || format!("  rounded-full font-semibold text-[0.8rem] font-medium px-[0.8rem] py-[0.2rem] w-[5rem]  {}", if comments_manual.show_editor.get() { "text-base05 bg-base01 hover:bg-base03" } else { "text-base05 bg-base01 hover:bg-base05 hover:text-base01" })>
                                <Show when=move || comments_manual.show_editor.get() fallback=|| "Reply">
                                    <SVGArrowDown class="size-4 mx-auto"/>
                                </Show>
                            </button>
                        </Show>
                    </div>
                    <Show when=move || comments_manual.show_editor.get()>
                        <div class=move || format!("mb-4 flex bg-base01 rounded-xl flex-col gap-2 py-2 px-4 w-full {}", if global_state.is_logged_in().unwrap_or_default() || !global_state.acc_pending() { "" } else { "hidden" })  >
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
            <div class="grid grid-rows-[100%] grid-cols-[auto_1fr] w-full">
                <Show when=show_line>
                    <div class="relative w-[3.2rem] h-full flex justify-center shrink-0">
                        <div class="w-[1.7rem] h-[1.61rem] border-base05 border-l-[0.2rem] border-b-[0.2rem] rounded-bl-[2rem] ml-auto box-border shrink-0"></div>
                        <Show when=move || { !comments_manual.is_last() }>
                            <div class="absolute w-[0.2rem] h-full bg-base05 shrink-0"></div>
                        </Show>
                    </div>
                </Show>
                // <form class=move || format!("mb-4 flex bg-base01 rounded-xl flex-col gap-2 py-2 px-4 w-full {}", if global_state.is_logged_in().unwrap_or_default() || !global_state.acc_pending() { "" } else { "hidden" }) on:submit=post_comments.on_comment.to_fn() >

                <div class="flex flex-col w-full">
                    // <Show when=move || reply_render_comments && (replies_shown.get() || comments_manual.reply_editor_show.get())>
                    <Show when=move || reply_render_comments>
                        <div node_ref=comment_container_ref class=move || format!("0h-[20rem] 0overflow-y-scroll {} ", if show_replies_fn() {""} else {"hidden"} )>
                            <For
                                each=move || comments_manual.items.get()
                                key=|state| state.key.clone()
                                let(data)
                            >
                                {
                                    // let key = data.key.clone();
                                    // let is_last = comments_manual.items.with(|v| v.last().map(|v| v.key == key).unwrap_or_default());
                                    view!{
                                        <PostCommentElm parent=comments_manual.items comment=data param_post max_depth=max_depth parent_depth=current_depth />
                                    }.into_any()
                                }
                            </For>
                            <button on:click=move |_| { comments_manual.fetch(); } class=move || format!("px-4 py-2 bg-base01 rounded-xl text-center text-base05 font-[1.2rem] w-full {}", if comments_manual.finished.get() {"hidden"} else {""})>
                                "load more"
                            </button>
                        </div>
                        // { post_comment_views }
                    </Show>
                </div>

            </div>
        </div>
    }
}
