use crate::api::shared::post_comment::UserPostComment;
use crate::api::{Api, ApiWeb, Server404Err, ServerErr};
use crate::path::{PATH_LOGIN, link_home, link_img, link_user};
use crate::view::app::GlobalState;
use crate::view::app::components::nav::Nav;
use crate::view::app::hook::use_infinite_scroll_fn::InfiniteScrollFn;
use crate::view::app::hook::use_infinite_scroll_virtual::{
    InfiniteStage, use_infinite_scroll_virtual,
};
use crate::view::app::hook::use_post_comment::use_post_comment;
use crate::view::app::hook::use_post_comments_baisc::CommentsBaisc;
use crate::view::app::hook::use_post_like::{self, PostLikeStage, use_post_like};
use crate::view::toolbox::prelude::*;
use leptos::{Params, task::spawn_local};
use leptos::{html, prelude::*};
use leptos_router::hooks::{use_location, use_params};
use leptos_router::params::Params;
use tracing::{debug, error, trace};
use web_sys::SubmitEvent;

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

    let comment_container_ref = NodeRef::<html::Div>::new();
    let comment_input_ref = NodeRef::<html::Textarea>::new();
    let comment_basic = CommentsBaisc::new();
    let post_comment = move |e: SubmitEvent| {
        e.prevent_default();
        comment_basic.post.run();
    };
    Effect::new(move || {
        trace!("comments basic start");
        let (Some(post_id), Some(comment_input)) = (param_post.get(), comment_input_ref.get())
        else {
            return;
        };

        trace!("comments basic observe");
        comment_basic.observe_only(
            comment_input,
            comment_container_ref,
            post_id,
            String::new(),
            50,
        );
    });

    // let post_comments = use_post_comment(
    //     false,
    //     10,
    //     comment_container_ref,
    //     comment_input_ref,
    //     param_post,
    //     None::<String>,
    // );
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
        <main node_ref=main_ref class="font-hi grid grid-rows-[auto_1fr] h-screen text-base05">
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

                        <div node_ref=comment_container_ref class=" flex flex-col gap-2 relative 0h-[20rem] 0overflow-y-scroll">
                            <For
                                each=move || comment_basic.items.get()
                                key=|state| state.key.clone()
                                let(data)
                            >
                                {
                                    view!{
                                        <PostCommentElm comment=data param_post max_depth=0 parent_depth=0 />
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
pub fn PostCommentElm(
    comment: UserPostComment,
    param_post: Memo<Option<String>>,
    max_depth: usize,
    parent_depth: usize,
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
    let comment_input_ref = NodeRef::<html::Textarea>::new();
    let reply_shown = RwSignal::new(false);

    // let post_comments = use_post_comment(
    //     false,
    //     10,
    //     comment_container_ref,
    //     comment_input_ref,
    //     param_post,
    //     Some(comment.key.clone()),
    // );

    // let show_reply = move |key: String| {
    //     reply_shown.update(|v| {
    //         if *v == key {
    //             v.clear();
    //         } else {
    //             *v = key;
    //         }
    //     });
    // };

    // let post_comment_views = move || {
    //     let time_now = global_state.get_time_ns();
    //     // let comment = comment.clone();
    //
    //     let result = post_comments
    //         .data
    //         .get()
    //         .into_iter()
    //         .map(move |comment| {
    //             // let comment_key = comment.key.clone();
    //             view! {
    //                 // <div>{comment.text}</div>
    //
    //
    //                 <PostCommentElm comment param_post max_depth=max_depth parent_depth=current_depth />
    //             }
    //             .into_any()
    //         })
    //         .collect_view();
    //
    //     result
    // };

    view! {

        <div class="flex gap-4 px-2 py-1 " >
            <p class="text-[1rem] rounded-full h-[3rem] w-[3rem] shrink-0 bg-base05"></p>
            <div class="flex flex-col w-full ">
                <div class="flex gap-2 place-items-center ">
                    <div class="text-[1.2rem]"> {comment.user.username} </div>
                    <div class="text-[1rem] text-base03"> {ns_to_str(global_state.get_time_ns().saturating_sub(comment.created_at))}" ago"</div>
                    <SVGTrash class="size-6 ml-auto"/>
                </div>

                <div class=" mb-2 text-[1.1rem] break-all"> {comment.text} </div>
                <div>
                    <button on:click=move |_| reply_shown.update(|v| *v = !*v ) type="submit" class=move || format!("mb-4 ml-auto rounded-full font-semibold text-[1rem] font-medium px-[0.8rem] py-[0.2rem] w-[5rem]  {}", if reply_shown.get() { "text-base05 bg-base01 hover:bg-base02" } else { "text-base01 bg-base0D hover:bg-base05" })>
                        <Show when=move || reply_shown.get() fallback=|| "Reply">
                            <SVGArrowDown class="size-6 mx-auto"/>
                        </Show>
                    </button>
                </div>
                <Show when=move || reply_shown.get()>
                    // <form class=move || format!("mb-4 flex bg-base01 rounded-xl flex-col gap-2 py-2 px-4 w-full {}", if global_state.is_logged_in().unwrap_or_default() || !global_state.acc_pending() { "" } else { "hidden" }) on:submit=post_comments.on_comment.to_fn() >
                    <form class=move || format!("mb-4 flex bg-base01 rounded-xl flex-col gap-2 py-2 px-4 w-full {}", if global_state.is_logged_in().unwrap_or_default() || !global_state.acc_pending() { "" } else { "hidden" })  >
                        <textarea placeholder="Comment" node_ref=comment_input_ref class="focus:outline-none! appearance-none border-none resize text-[1.1rem] w-full" id="story" name="story" rows="5"  ></textarea>
                        // <ul class="text-base08 list-disc ml-[1rem]">
                        //     {move || post_comments.err_post.get().map(|v| v.trim().split("\n").filter(|v| v.len() > 1).map(|v| v.to_string()).map(move |v: String| view! { <li>{v}</li> }).collect_view()) }
                        // </ul>
                        <div class="flex justify-between place-items-center">
                            <p>"0/2000"</p>
                            <input  type="submit" value="Reply" class="ml-auto rounded-full font-medium text-[1rem] font-bold px-[0.8rem] py-[0.2rem] hover:bg-base05 bg-base0D text-base01 text-center w-[5rem]"/>
                        </div>
                    </form>
                    <div node_ref=comment_container_ref class="0h-[20rem] 0overflow-y-scroll">
                        // <For
                        //     each=move || post_comments.data.get()
                        //     key=|state| state.key.clone()
                        //     let(data)
                        // >
                        //     {
                        //         view!{
                        //             <PostCommentElm comment=data param_post max_depth=max_depth parent_depth=current_depth />
                        //         }.into_any()
                        //     }
                        // </For>
                        // { post_comment_views }
                    </div>
                </Show>
            </div>
        </div>
    }
}
