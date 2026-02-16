use leptos::html::{self, ElementType};
use leptos::prelude::*;
use tracing::{error, trace};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, SubmitEvent};

use crate::api::{Api, ApiWeb, ServerRes, TimeRange};
use crate::get_timestamp;
use crate::view::app::hook::use_infinite_scroll::{
    InfiniteMerge, InfiniteStage, use_infinite_scroll,
};
use crate::view::toolbox::prelude::*;

pub trait SizedIntoView: IntoView + Sized {}

#[derive(Clone, Copy)]
pub struct PostComment {
    pub comments: StoredValue<Box<dyn Fn() -> AnyView + Sync + Send + 'static>>,
    pub on_comment: StoredValue<Box<dyn Fn(SubmitEvent) + Sync + Send + 'static>>,
}

pub fn use_post_comment<ContainerElm>(
    fetch_count: usize,
    comment_container_ref: NodeRef<ContainerElm>,
    text_area_ref: NodeRef<html::Textarea>,
    post_id: Memo<Option<String>>,
) -> PostComment
where
    ContainerElm: ElementType,
    ContainerElm::Output: JsCast + Clone + 'static + Into<HtmlElement>,
{
    let api = ApiWeb::new();

    let infinite_fn = move |stage: InfiniteStage<u128>| async move {
        let post_id = match stage {
            InfiniteStage::Init => post_id.get(),
            _ => post_id.get_untracked(),
        };
        let Some(post_id) = post_id else {
            return InfiniteMerge::None;
        };
        let (is_top, result) = match stage {
            InfiniteStage::Manual => {
                let Some(text_input) = text_area_ref.get_untracked() else {
                    return InfiniteMerge::None;
                };
                let text = text_input.value();
                (
                    true,
                    api.add_post_comment(post_id, text).send_native().await,
                )
            }
            InfiniteStage::Init => {
                let time = get_timestamp();
                (
                    true,
                    api.get_post_comment(post_id, fetch_count, TimeRange::BeforeOrEqual(time))
                        .send_native()
                        .await,
                )
            }
            InfiniteStage::Top(data) => (
                true,
                api.get_post_comment(post_id, fetch_count, TimeRange::Before(data))
                    .send_native()
                    .await,
            ),
            InfiniteStage::Btm(data) => (
                false,
                api.get_post_comment(post_id, fetch_count, TimeRange::After(data))
                    .send_native()
                    .await,
            ),
        };

        let views = match result {
            Ok(ServerRes::Comment(comment)) => {
                let Some(text_input) = text_area_ref.get_untracked() else {
                    return InfiniteMerge::None;
                };
                text_input.set_value("");

                let comments = vec![comment];
                let data = comments.iter().map(|v| v.created_at).collect::<Vec<u128>>();

                let comments = comments
                            .into_iter()
                            .map(move |comment| view! { <div class="border border-base0E px-2 py-1" >{comment.text}</div> })
                            .collect_view();

                InfiniteMerge::Top {
                    data,
                    views: comments,
                }
            }
            Ok(ServerRes::Comments(comments)) => {
                let data = comments.iter().map(|v| v.created_at).collect::<Vec<u128>>();

                let comments = comments
                            .into_iter()
                            .map(move |comment| view! { <div class="border border-base0E px-2 py-1" >{comment.text}</div> })
                            .collect_view();

                if is_top {
                    InfiniteMerge::Top {
                        data,
                        views: comments,
                    }
                } else {
                    InfiniteMerge::Btm {
                        data,
                        views: comments,
                    }
                }
            }
            Ok(err) => {
                error!("unexpected server response: {err:?}");

                InfiniteMerge::None
            }
            Err(err) => {
                error!("use_post_like: {err}");
                InfiniteMerge::None
            }
        };

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

    PostComment {
        comments: infinte.view,
        on_comment: StoredValue::new(Box::new(on_comment)),
    }
}
