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
use leptos::{
    html::{ElementType, Textarea},
    prelude::*,
};
use tracing::{error, trace};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, HtmlTextAreaElement, MutationObserver, MutationRecord};

#[derive(Clone, Copy)]
pub struct CommentsManual {
    pub reply_editor_show: RwSignal<bool, LocalStorage>,
    pub reply_count: RwSignal<usize, LocalStorage>,
    pub finished: RwSignal<bool, LocalStorage>,
    pub items: RwSignal<Vec<UserPostComment>, LocalStorage>,
    pub err_post: RwSignal<String, LocalStorage>,
    pub observer: StoredValue<
        Box<dyn Fn((Option<HtmlTextAreaElement>, String, String, usize, bool)) + 'static>,
        LocalStorage,
    >,
    pub fetch_btm: StoredValue<Box<dyn Fn() + 'static>, LocalStorage>,
    pub post: StoredValue<Box<dyn Fn() + 'static>, LocalStorage>,
    //
}

impl CommentsManual {
    pub fn new(
        parent_items: Option<RwSignal<Vec<UserPostComment>, LocalStorage>>,
        reverse: bool,
    ) -> Self {
        let api = ApiWeb::new();

        let finished = RwSignal::new_local(true);
        let err_post = RwSignal::new_local(String::new());
        let post_key = StoredValue::new_local(String::new());
        let comment_key = StoredValue::new_local(String::new());
        let fetch_count = StoredValue::new_local(50_usize);
        let flatten = StoredValue::new_local(false);
        let reply_editor_show = RwSignal::new_local(false);
        let reply_count = RwSignal::new_local(0);
        let input_elm = StoredValue::new_local(None::<HtmlTextAreaElement>);
        let items =
            parent_items.unwrap_or_else(|| RwSignal::new_local(Vec::<UserPostComment>::new()));

        // on_cleanup(move || {
        //     trace!("cleaning up shit");
        //     items.update(move |v| {
        //         v.clear();
        //     });
        // });

        let fetch_btm = FutureFn::new(move || async move {
            let (post_id, comment_key) = {
                let post_id = post_key.get_value();
                if post_id.is_empty() {
                    return;
                }
                let comment_key = comment_key.get_value();
                trace!("comments manual 4 {comment_key}");

                // TODO what is this nonsense
                let comment_key = if comment_key.is_empty() {
                    None
                } else {
                    Some(comment_key)
                };

                (post_id, comment_key)
            };
            let fetch_count = fetch_count.get_value();
            let time = time_now_ns();
            let flatten = flatten.get_value();
            let order = if reverse {
                Order::OneTwoThree
            } else {
                Order::ThreeTwoOne
            };

            let items = &mut *items.write();

            trace!("comments manual 3 {comment_key:?}");

            let result = if items.is_empty() {
                api.get_post_comment(
                    post_id,
                    comment_key,
                    fetch_count,
                    TimeRange::LessOrEqual(time),
                    order,
                    flatten,
                )
                .send_native()
                .await
            } else if let Some(item) = items.last() {
                api.get_post_comment(
                    post_id,
                    comment_key,
                    fetch_count,
                    TimeRange::Less(item.created_at),
                    order,
                    flatten,
                )
                .send_native()
                .await
            } else {
                return;
            };

            match result {
                Ok(ServerRes::Comments(comments)) => {
                    let len = comments.len();
                    trace!(
                        "comments manual (len){len} < (fetch_count){fetch_count} = {}",
                        len < fetch_count
                    );
                    if len == fetch_count {
                        finished.set(false);
                    } else if !finished.get_untracked() && len < fetch_count {
                        finished.set(true);
                    }
                    trace!("comments basic extended");
                    items.extend(comments);
                    // reply_editor_show.set(false);
                }
                Ok(err) => {
                    let err = format!("post comments basic: unexpected res: {err:?}");
                    error!(err);
                }
                Err(err) => {
                    let err = format!("post comments basic: {err}");
                    error!(err);
                }
            };

            ()
        });

        let post_run = FutureFn::new(move || async move {
            trace!("running post_run");
            err_post.update(|err| {
                err.clear();
            });

            let (post_key, comment_key, time, text, post_elm) = {
                let post_key = post_key.get_value();
                if post_key.is_empty() {
                    trace!("no post key");
                    return;
                }

                let comment_key = comment_key.get_value();
                trace!("comments manual {comment_key}");
                let comment_key = if comment_key.is_empty() {
                    None
                } else {
                    Some(comment_key)
                };
                let time = time_now_ns();

                let Some(post_elm) = input_elm.get_value() else {
                    return;
                };
                let text = post_elm.value();
                (post_key, comment_key, time, text, post_elm)
            };

            trace!("comments manual 2 {comment_key:?}");
            let result = api
                .add_post_comment(post_key, comment_key, text)
                .send_native()
                .await;

            match result {
                Ok(ServerRes::Comment(comment)) => {
                    post_elm.set_value("");
                    let items = &mut *items.write();
                    if reverse {
                        items.push(comment);
                    } else {
                        items.insert(0, comment);
                    }
                    reply_editor_show.set(false);
                    reply_count.update(|v| *v += 1 );
                }
                Ok(err) => {
                    let err = format!("post comments basic: unexpected res: {err:?}");
                    error!(err);
                    err_post.set(err);
                }
                Err(err) => {
                    let err = format!("post comments basic: {err}");
                    error!(err);
                    err_post.set(err);
                }
            };
        });

        let observe =
            move |(post_input, new_post_id, new_comment_key, new_count, new_flatten): (
                Option<HtmlTextAreaElement>,
                String,
                String,
                usize,
                bool,
            )| {
                post_key.set_value(new_post_id);
                comment_key.set_value(new_comment_key);
                fetch_count.set_value(new_count);
                input_elm.set_value(post_input);
                flatten.set_value(new_flatten);
            };

        let fetch_btm = move || {
            fetch_btm.run();
        };

        let post = move || {
            post_run.run();
        };

        Self {
            reply_editor_show,
            reply_count,
            err_post,
            items,
            finished,
            observer: StoredValue::new_local(Box::new(observe)),
            fetch_btm: StoredValue::new_local(Box::new(fetch_btm)),
            post: StoredValue::new_local(Box::new(post)),
        }
    }

    pub fn post(&self) {
        self.post.run();
    }

    pub fn fetch_btm(&self) {
        self.fetch_btm.run();
    }

    pub fn observe_only(
        &self,
        post_input: Option<HtmlTextAreaElement>,
        post_id: String,
        comment_key: String,
        count: usize,
        flatten: bool,
    ) {
        trace!("comments manual observe {comment_key}");
        self.observer
            .run((post_input, post_id, comment_key, count, flatten));
    }
}
