use leptos::html::{self, ElementType};
use leptos::prelude::*;
use leptos::tachys::reactive_graph::bind::GetValue;
use tracing::{error, trace};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, MouseEvent, SubmitEvent};

use crate::api::{Api, ApiWeb, ServerRes};
use crate::get_timestamp;
use crate::view::app::GlobalState;
use crate::view::app::hook::use_infinite_scroll::{
    InfiniteMerge, InfiniteStage, use_infinite_scroll,
};
use crate::view::toolbox::prelude::*;

pub trait SizedIntoView: IntoView + Sized {}

#[derive(Clone, Copy)]
pub struct PostComment {
    // pub err_general: RwQuery<String>,
    // pub email: RwQuery<String>,
    // pub form_stage: RwQuery<ChangePasswordFormStage>,
    // pub btn_stage: StoredValue<Box<dyn Fn() -> ChangePasswordBtnStage + Sync + Send + 'static>>,
    // pub stage: RwSignal<PostLikeStage>,
    pub comments: StoredValue<Box<dyn Fn() -> AnyView + Sync + Send + 'static>>,
    pub on_comment: StoredValue<Box<dyn Fn(SubmitEvent) + Sync + Send + 'static>>,
    // pub token: RwQuery<String>,
}

pub fn use_post_comment<ContainerElm>(
    comment_container_ref: NodeRef<ContainerElm>,
    text_area_ref: NodeRef<html::Textarea>,
    // input_new_email: NodeRef<html::Input>,
    post_id: Memo<Option<String>>,
) -> PostComment
where
    ContainerElm: ElementType,
    ContainerElm::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    // TextAreaElm: ElementType,
    // TextAreaElm::Output: JsCast + Clone + 'static + Into<HtmlElement>,
{
    let api = ApiWeb::new();

    let infinite_fn = move |stage: InfiniteStage| async move {
        // vec![ view! { <div class="" >"wtf"</div> } ]

        //
        // let index_val = index.get_value();
        let post_id = match stage {
            InfiniteStage::Init => post_id.get(),
            _ => post_id.get_untracked(),
        };
        let Some(post_id) = post_id else {
            return InfiniteMerge::None;
        };
        let views = match stage {
            InfiniteStage::Manual => {
                if let Some(text_input) = text_area_ref.get_untracked() {
                    // let text_input: HtmlElement = text_input.into();
                    let text = text_input.value();
                    let result = api.add_post_comment(post_id, text).send_native().await;

                    match result {
                        Ok(ServerRes::Comment(comment)) => {
                            // if condition {
                            //     stage.set(PostLikeStage::Liked);
                            // } else {
                            //     stage.set(PostLikeStage::Unliked);
                            // }

                            text_input.set_value("");

                            let comments = vec![comment]
                            .into_iter()
                            .map(move |comment| view! { <div class="border border-base0E px-2 py-1" >{comment.text}</div> })
                            .collect_view();


                            InfiniteMerge::Top(comments)
                        }
                        Ok(err) => {
                            error!("unexpected server response: {err:?}");

                            InfiniteMerge::None
                        }
                        Err(err) => {
                            error!("use_post_like: {err}");
                            InfiniteMerge::None
                        }
                    }
                } else {
                    InfiniteMerge::None
                }
            }
            InfiniteStage::Init => {
                let result = api.get_post_comment(post_id).send_native().await;

                match result {
                    Ok(ServerRes::Comments(comments)) => {
                        // if condition {
                        //     stage.set(PostLikeStage::Liked);
                        // } else {
                        //     stage.set(PostLikeStage::Unliked);
                        // }
                        let comments = comments
                            .into_iter()
                            .map(move |comment| view! { <div class="border border-base0E px-2 py-1" >{comment.text}</div> })
                            .collect_view();

                        InfiniteMerge::Btm(comments)
                    }
                    Ok(err) => {
                        error!("unexpected server response: {err:?}");

                        InfiniteMerge::None
                    }
                    Err(err) => {
                        error!("use_post_like: {err}");
                        InfiniteMerge::None
                    }
                }
            }
            InfiniteStage::Top => {
                //
                InfiniteMerge::None
            }
            InfiniteStage::Btm => {
                //
                InfiniteMerge::None
            }
        };
        // let views_len = views.len();
        // index.update_value(|v| {
        //     *v += views_len;
        // });

        views
    };

    let infinte = use_infinite_scroll(comment_container_ref, infinite_fn);

    let on_comment = move |stage: SubmitEvent| {
        stage.prevent_default();

        trace!("executing on_post");

        let Some(post_id) = post_id.get_untracked() else {
            return;
        };

        trace!("post_id {post_id}");

        infinte.trigger.run();


    };

    // Effect::new(move || {
    //     let Some(post_id) = post_id.get() else {
    //         return;
    //     };
    //     api.get_post_comment(post_id).send_web(async move |result| {
    //         match result {
    //             Ok(ServerRes::Comments(comments)) => {
    //                 // if condition {
    //                 //     stage.set(PostLikeStage::Liked);
    //                 // } else {
    //                 //     stage.set(PostLikeStage::Unliked);
    //                 // }
    //             }
    //             Ok(err) => {
    //                 error!("use_post_like: expected ServerRes::Condition, received: {err:?}");
    //                 // stage.set(PostLikeStage::Unliked);
    //             }
    //             Err(err) => {
    //                 error!("use_post_like: {err}");
    //                 // stage.set(PostLikeStage::Unliked);
    //             }
    //         }
    //
    //         //
    //     });
    // });

    PostComment {
        comments: infinte.view,
        on_comment: StoredValue::new(Box::new(on_comment)),
    }
}
