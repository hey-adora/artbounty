use crate::{
    api::{Api, ApiWeb, Order, ServerRes, TimeRange, shared::post_comment::UserPostComment},
    view::{
        app::hook::{
            use_future::FutureFn,
            use_infinite_scroll_basic::InfiniteBasic,
            use_infinite_scroll_fn::{InfiniteItem, InfiniteScrollFn},
            use_post_comments_manual::CommentsManual,
        },
        toolbox::prelude::*,
    },
};
use leptos::{
    html::{ElementType, Textarea},
    prelude::*,
};
use tracing::{error, trace};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, HtmlTextAreaElement, MutationObserver, MutationRecord};

#[derive(Copy, Clone)]
pub struct CommentsBaisc {
    pub err_post: RwSignal<String, LocalStorage>,
    pub items: RwSignal<Vec<UserPostComment>, LocalStorage>,
    pub observer: StoredValue<
        Box<dyn Fn((HtmlTextAreaElement, Element, String, String, usize)) + 'static>,
        LocalStorage,
    >,
    pub post: StoredValue<Box<dyn Fn() + 'static>, LocalStorage>,
}

impl CommentsBaisc {
    pub fn new() -> Self {
        // let api = ApiWeb::new();

        let comments_manual = CommentsManual::new(None, false);

        // let async_callback = async move |a: &mut Vec<UserPostComment>, b: Option<InfiniteItem>| {
        //     comments_manual.fetch_btm();
        // };
        let infinite_fn = InfiniteScrollFn::new(move |a| {
            comments_manual.fetch_btm();
        });

        let observer = move |(post_elm, container_elm, post_id, comment_key, count)| {
            comments_manual.observe_only(Some(post_elm), post_id, comment_key, count, false);
            infinite_fn.observe_only(container_elm);
            comments_manual.fetch_btm();
        };

        let post = move || {
            comments_manual.post();
        };

        Self {
            err_post: comments_manual.err_post,
            items: comments_manual.items,
            observer: StoredValue::new_local(Box::new(observer)),
            post: StoredValue::new_local(Box::new(post)),
        }
    }

    pub fn post(&self) {
        self.post.run();
    }

    pub fn observe_only(
        &self,
        post_input: HtmlTextAreaElement,
        comment_container: Element,
        post_id: String,
        comment_key: String,
        count: usize,
    ) {
        self.observer
            .run((post_input, comment_container, post_id, comment_key, count));
    }
}
