use crate::{
    api::{Api, ApiWeb, Order, ServerRes, TimeRange, shared::post_comment::UserPostComment},
    view::{
        app::hook::{
            use_future::FutureFn, use_infinite_scroll_basic::InfiniteBasic,
            use_infinite_scroll_fn::InfiniteItem,
        },
        toolbox::prelude::*,
    },
};
use leptos::prelude::*;
use tracing::{error, trace};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, MutationObserver, MutationRecord};

#[derive(Copy, Clone)]
pub struct CommentsBaisc {
    pub items: RwSignal<Vec<UserPostComment>, LocalStorage>,
    pub observer:
        StoredValue<Box<dyn Fn((Element, String, String, usize)) + 'static>, LocalStorage>,
}

impl CommentsBaisc {
    pub fn new() -> Self {
        let api = ApiWeb::new();

        let post_id = StoredValue::new_local(String::new());
        let comment_key = StoredValue::new_local(String::new());
        let fetch_count = StoredValue::new_local(50_usize);

        let async_callback = async move |a: &mut Vec<UserPostComment>, b: Option<InfiniteItem>| {
            trace!("comments basic 0");
            let post_id = post_id.get_value();
            if post_id.is_empty() {
                return;
            }
            let comment_key = comment_key.get_value();
            // TODO what is this nonsense
            let comment_key = if comment_key.is_empty() {
                None
            } else {
                Some(comment_key)
            };
            let fetch_count = fetch_count.get_value();
            let time = time_now_ns();
            trace!("comments basic 1");

            let result = if a.is_empty() {
                api.get_post_comment(
                    post_id,
                    comment_key,
                    fetch_count,
                    TimeRange::LessOrEqual(time),
                    Order::ThreeTwoOne,
                )
                .send_native()
                .await
            } else if let Some(item) = a.last() {
                api.get_post_comment(
                    post_id,
                    comment_key,
                    fetch_count,
                    TimeRange::Less(item.created_at),
                    Order::ThreeTwoOne,
                )
                .send_native()
                .await
            } else {
                return;
            };

            match result {
                Ok(ServerRes::Comment(comment)) => {
                    // let Some(text_input) = text_area_ref.get_untracked() else {
                    //     return InfiniteMerge::None;
                    // };
                    // text_input.set_value("");
                    //
                    // let comments = vec![comment];
                    //
                    // InfiniteMerge::Top { data: comments }
                }
                Ok(ServerRes::Comments(comments)) => {
                    trace!("comments basic extended");
                    a.extend(comments);
                    // if is_top {
                    //     InfiniteMerge::Top {
                    //         data: comments.into_iter().rev().collect(),
                    //     }
                    // } else {
                    //     InfiniteMerge::Btm { data: comments }
                    // }
                }
                Ok(err) => {
                    let err = format!("post comments basic: unexpected res: {err:?}");
                    error!(err);
                    // err_post.set(err);
                    //
                    // InfiniteMerge::None
                }
                Err(err) => {
                    let err = format!("post comments basic: {err}");
                    error!(err);
                    // err_post.set(err);
                    // InfiniteMerge::None
                }
            };
        };
        let infinite_basic = InfiniteBasic::new(async_callback.clone());
        let init_run = FutureFn::new(
            move |(e, new_post_id, new_comment_key, new_count): (
                Element,
                String,
                String,
                usize,
            )| async move {
                post_id.set_value(new_post_id);
                comment_key.set_value(new_comment_key);
                fetch_count.set_value(new_count);

                let mut v = Vec::new();
                async_callback(&mut v, None).await;

                infinite_basic.items.set(v);
                infinite_basic.observe_only(e);

                //
            },
        );

        let observer = move |v| {
            init_run.run(v);
        };

        Self {
            items: infinite_basic.items,
            observer: StoredValue::new_local(Box::new(observer)),
        }
    }

    pub fn observe_only<Elm>(&self, elm: Elm, post_id: String, comment_key: String, count: usize)
    where
        Elm: JsCast + Clone + 'static + Into<Element>,
    {
        let elm: Element = elm.into();
        self.observer.run((elm, post_id, comment_key, count));
    }
}
