use leptos::html::{self, ElementType};
use leptos::prelude::*;
use tracing::{error, trace};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, SubmitEvent};

use crate::api::shared::post_comment::UserPostComment;
use crate::api::{Api, ApiWeb, Order, ServerRes, TimeRange};
use crate::view::app::hook::use_infinite_scroll_virtual::{
    InfiniteMerge, InfiniteStage, use_infinite_scroll_virtual,
};
use crate::view::toolbox::prelude::*;

pub trait SizedIntoView: IntoView + Sized {}

#[derive(Clone, Copy)]
pub struct PostComment {
    pub reply_editor_show: RwSignal<bool, LocalStorage>,
    pub err_post: RwQuery<String>,
    pub data: RwSignal<Vec<UserPostComment>, LocalStorage>,
    pub on_comment: StoredValue<Box<dyn Fn(SubmitEvent) + Sync + Send + 'static>>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum PostCommentFields {
    ErrPost,
}

pub fn use_post_comment<ContainerElm>(
    virtual_scroll: bool,
    fetch_count: usize,
    comment_container_ref: NodeRef<ContainerElm>,
    text_area_ref: NodeRef<html::Textarea>,
    post_key: Memo<Option<String>>,
    comment_key: Option<String>,
) -> PostComment
where
    ContainerElm: ElementType,
    ContainerElm::Output: JsCast + Clone + 'static + Into<HtmlElement>,
{
    let api = ApiWeb::new();

    let err_post = RwQuery::<String>::new(PostCommentFields::ErrPost.to_string());
    let reply_editor_show = RwSignal::new_local(false);

    let infinite_fn = move |stage: InfiniteStage<UserPostComment>| {
        let comment_key = comment_key.clone();
        async move {
            let post_id = match stage {
                InfiniteStage::Init => post_key.get_untracked(),
                _ => post_key.get_untracked(),
            };
            let Some(post_id) = post_id else {
                return InfiniteMerge::None;
            };
            let comment_key = comment_key.clone();
            let (is_top, result) = match stage {
                InfiniteStage::Manual => {
                    let Some(text_input) = text_area_ref.get_untracked() else {
                        return InfiniteMerge::None;
                    };
                    let text = text_input.value();
                    (
                        true,
                        api.add_post_comment(post_id, comment_key, text)
                            .send_native()
                            .await,
                    )
                }
                InfiniteStage::Init => {
                    let time = time_now_ns();
                    (
                        false,
                        api.get_post_comment(
                            post_id,
                            comment_key,
                            fetch_count,
                            TimeRange::LessOrEqual(time),
                            Order::ThreeTwoOne,
                            false,
                        )
                        .send_native()
                        .await,
                    )
                }
                InfiniteStage::Top(data) => (
                    true,
                    api.get_post_comment(
                        post_id,
                        comment_key,
                        fetch_count,
                        TimeRange::More(data.created_at),
                        Order::OneTwoThree,
                        false,
                    )
                    .send_native()
                    .await,
                ),
                InfiniteStage::Btm(data) => (
                    false,
                    api.get_post_comment(
                        post_id,
                        comment_key,
                        fetch_count,
                        TimeRange::Less(data.created_at),
                        Order::ThreeTwoOne,
                        false,
                    )
                    .send_native()
                    .await,
                ),
            };

            let datas = match result {
                Ok(ServerRes::Comment(comment)) => {
                    reply_editor_show.set(false);
                    let Some(text_input) = text_area_ref.get_untracked() else {
                        return InfiniteMerge::None;
                    };
                    text_input.set_value("");

                    let comments = vec![comment];

                    InfiniteMerge::Top { data: comments }
                }
                Ok(ServerRes::Comments(comments)) => {
                    // reply_editor_show.set(false);
                    if is_top {
                        InfiniteMerge::Top {
                            data: comments.into_iter().rev().collect(),
                        }
                    } else {
                        InfiniteMerge::Btm { data: comments }
                    }
                }
                Ok(err) => {
                    let err = format!("unexpected server response: {err:?}");
                    error!(err);
                    err_post.set(err);

                    InfiniteMerge::None
                }
                Err(err) => {
                    let err = format!("use_post_like: {err}");
                    error!(err);
                    err_post.set(err);
                    InfiniteMerge::None
                }
            };

            datas
        }
    };

    let infinte = use_infinite_scroll_virtual(comment_container_ref, infinite_fn);

    let on_comment = move |stage: SubmitEvent| {
        stage.prevent_default();

        trace!("executing on_post");

        let Some(post_id) = post_key.get_untracked() else {
            return;
        };

        trace!("post_id {post_id}");

        infinte.trigger.run();
    };

    PostComment {
        err_post,
        reply_editor_show,
        data: infinte.data,
        on_comment: StoredValue::new(Box::new(on_comment)),
    }
}
