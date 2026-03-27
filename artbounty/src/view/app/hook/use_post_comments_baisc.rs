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

#[derive(Copy, Clone)]
pub struct CommentsBaisc<ContainerElm>
where
    ContainerElm: ElementType,
    ContainerElm::Output: JsCast + Clone + 'static + Into<Element>,
{
    pub err_post: RwSignal<String, LocalStorage>,
    pub items: RwSignal<Vec<UserPostComment>, LocalStorage>,
    pub observer: StoredValue<
        Box<
            dyn Fn(
                    (
                        HtmlTextAreaElement,
                        NodeRef<ContainerElm>,
                        String,
                        String,
                        usize,
                    ),
                ) + 'static,
        >,
        LocalStorage,
    >,
    pub post: StoredValue<Box<dyn Fn() + 'static>, LocalStorage>,
}

impl<ContainerElm> CommentsBaisc<ContainerElm>
where
    ContainerElm: ElementType,
    ContainerElm::Output: JsCast + Clone + 'static + Into<Element>,
{
    pub fn new() -> Self {
        let api = ApiWeb::new();

        let err_post = RwSignal::new_local(String::new());
        let post_key = StoredValue::new_local(String::new());
        let comment_key = StoredValue::new_local(String::new());
        let fetch_count = StoredValue::new_local(50_usize);
        let input_elm = StoredValue::new_local(None::<HtmlTextAreaElement>);

        let async_callback = async move |a: &mut Vec<UserPostComment>, b: Option<InfiniteItem>| {
            trace!("comments basic 0");
            let post_id = post_key.get_value();
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
                Ok(ServerRes::Comment(comment)) => {}
                Ok(ServerRes::Comments(comments)) => {
                    trace!("comments basic extended");
                    a.extend(comments);
                }
                Ok(err) => {
                    let err = format!("post comments basic: unexpected res: {err:?}");
                    error!(err);
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
            move |(post_input, container_elm, new_post_id, new_comment_key, new_count): (
                HtmlTextAreaElement,
                NodeRef<ContainerElm>,
                String,
                String,
                usize,
            )| async move {
                post_key.set_value(new_post_id);
                comment_key.set_value(new_comment_key);
                fetch_count.set_value(new_count);
                input_elm.set_value(Some(post_input));

                let mut v = Vec::new();
                async_callback(&mut v, None).await;

                infinite_basic.items.set(v);

                let Some(e) = container_elm.get_untracked() else {
                    error!("comments basic E not found");
                    return;
                };
                infinite_basic.observe_only(e);

                //
            },
        );

        let post_run = FutureFn::new(move || async move {
            err_post.update(|err| {
                err.clear();
            });

            let (post_key, comment_key, time, text, post_elm) = {
                let post_key = post_key.get_value();
                if post_key.is_empty() {
                    return;
                }

                let comment_key = comment_key.get_value();
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

            let result = api
                .add_post_comment(post_key, comment_key, text)
                .send_native()
                .await;

            match result {
                Ok(ServerRes::Comment(comment)) => {
                    post_elm.set_value("");
                    infinite_basic.items.update(|v| {
                        v.insert(0, comment);
                    });
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

        let observer = move |v| {
            init_run.run(v);
        };

        let post = move || {
            post_run.run();
        };

        Self {
            err_post,
            items: infinite_basic.items,
            observer: StoredValue::new_local(Box::new(observer)),
            post: StoredValue::new_local(Box::new(post)),
        }
    }

    pub fn post(&self, post_input: NodeRef<Textarea>, comment_key: String) {
        self.post.run();
    }

    pub fn observe_only(
        &self,
        post_input: HtmlTextAreaElement,
        comment_container: NodeRef<ContainerElm>,
        post_id: String,
        comment_key: String,
        count: usize,
    ) {
        self.observer
            .run((post_input, comment_container, post_id, comment_key, count));
    }
}
