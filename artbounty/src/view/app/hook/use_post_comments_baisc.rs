use crate::{
    api::{Api, ApiWeb, Order, ServerRes, TimeRange, shared::post_comment::UserPostComment},
    view::{
        app::hook::{
            use_future::FutureFn,
            use_infinite_scroll_basic::InfiniteBasic,
            use_infinite_scroll_fn::{InfiniteItem, InfiniteScrollFn},
            use_post_comments_manual::{CommentKind, CommentKind2, CommentsApi, CommentsApi2},
            use_spawner::Spawner,
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
pub struct CommentsBaisc<API: Api> {
    pub reply_editor_show: RwSignal<bool, LocalStorage>,
    pub comments_manual: CommentsApi2<API>,
    pub err_post: RwSignal<String, LocalStorage>,
    pub items: RwSignal<Vec<UserPostComment>, LocalStorage>,
    pub spawner: Spawner,
    pub infinite_fn: InfiniteScrollFn,
    // pub observer: StoredValue<
    //     Box<dyn Fn((HtmlTextAreaElement, Element, String, String, usize)) + 'static>,
    //     LocalStorage,
    // >,
    pub post: StoredValue<Box<dyn Fn() + 'static>, LocalStorage>,
}

impl<API: Api + Clone + Copy + 'static> CommentsBaisc<API> {
    pub fn new(api: API, spawner: Spawner) -> Self {
        // let api = ApiWeb::new();

        let comments_manual = CommentsApi2::new(api, 5, CommentKind2::Root);
        // let spawner = Spawner::new();

        // let async_callback = async move |a: &mut Vec<UserPostComment>, b: Option<InfiniteItem>| {
        //     comments_manual.fetch_btm();
        // };
        let infinite_fn = InfiniteScrollFn::new(move |_a| {
            spawner.spawn(comments_manual.fetch());
        });

        // let observer = move |(post_elm, container_elm, post_id, comment_key, count)| {
        //     comments_manual.post_key.set_value(post_id);
        //     // comments_manual.observe_only(Some(post_elm), post_id, count, false);
        //     infinite_fn.observe_only(container_elm);
        //     comments_manual.fetch();
        // };

        let post = move || {
            // comments_manual.post();
        };

        Self {
            comments_manual,
            reply_editor_show: comments_manual.show_editor,
            err_post: comments_manual.err_post,
            items: comments_manual.items,
            infinite_fn,
            spawner,
            // observer: StoredValue::new_local(Box::new(observer)),
            post: StoredValue::new_local(Box::new(post)),
        }
    }

    pub fn post(&self) {
        self.post.run();
    }

    pub async fn observe_only(
        self,
        // post_input: HtmlTextAreaElement,
        comment_container: Element,
        post_id: String,
        // comment_key: String,
        // count: usize,
    ) {
        self.comments_manual.observe_only(post_id);
        // self.spawner.spawn(self.comments_manual.fetch());
        self.comments_manual.fetch().await;
        self.infinite_fn.observe_only(comment_container);
        // self.observer
        //     .run((post_input, comment_container, post_id, comment_key, count));
    }
}
