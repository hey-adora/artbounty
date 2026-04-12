use crate::{
    api::{
        Api, ApiWeb, Order, ServerErr, ServerRes, TimeRange, shared::post_comment::UserPostComment,
    },
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
use tracing::{error, trace, warn};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, HtmlTextAreaElement, MutationObserver, MutationRecord};

#[derive(Default, Clone, strum::Display, strum::EnumIs)]
pub enum CommentKind2 {
    #[default]
    Root,
    // Comment {
    //     parent: RwSignal<Vec<UserPostComment>, LocalStorage>,
    //     comment: UserPostComment,
    // },
    Reply {
        parent_items: RwSignal<Vec<UserPostComment>, LocalStorage>,
        parent_replies_count: RwSignal<usize, LocalStorage>,
        comment: UserPostComment,
    },
    Flat {
        parent_items: RwSignal<Vec<UserPostComment>, LocalStorage>,
        parent_replies_count: RwSignal<usize, LocalStorage>,
        comment: UserPostComment,
    },
    None {
        parent_items: RwSignal<Vec<UserPostComment>, LocalStorage>,
        parent_replies_count: RwSignal<usize, LocalStorage>,
        comment: UserPostComment,
    }, // Comment {
       //     parent: CommentsApi,
       //     comment: UserPostComment,
       // },
       // Reply {
       //     parent: CommentsApi,
       //     comment: UserPostComment,
       // },
       // Flat {
       //     parent: CommentsApi,
       //     comment: UserPostComment,
       // },
}

#[derive(Clone, Copy)]
pub struct CommentsApi2<API: Api> {
    // ui
    pub items: RwSignal<Vec<UserPostComment>, LocalStorage>,
    pub finished: RwSignal<bool, LocalStorage>,
    pub replies_count: RwSignal<usize, LocalStorage>,
    pub show_editor: RwSignal<bool, LocalStorage>,
    // pub is_last: RwSignal<bool, LocalStorage>,
    // pub is_last: StoredValue<Box<dyn Fn() -> bool + 'static>, LocalStorage>,
    pub err_post: RwSignal<String, LocalStorage>,
    pub err_fetch: RwSignal<String, LocalStorage>,
    pub err_delete: RwSignal<String, LocalStorage>,

    // params
    pub post_key: StoredValue<String, LocalStorage>,
    // pub input_elm: StoredValue::;
    pub kind: StoredValue<CommentKind2, LocalStorage>,
    pub fetch_count: usize,
    api: API,
}

impl<API> CommentsApi2<API>
where
    API: Api,
{
    pub fn new(api: API, fetch_count: usize, kind: CommentKind2) -> Self {
        let replies_count = match &kind {
            CommentKind2::Root => 0,
            CommentKind2::Flat { comment, .. }
            | CommentKind2::None { comment, .. }
            | CommentKind2::Reply { comment, .. } => comment.replies_count,
        };
        Self {
            // ui
            items: RwSignal::new_local(Vec::new()),
            finished: RwSignal::new_local(false),
            replies_count: RwSignal::new_local(replies_count),
            // is_last: RwSignal::new_local(false),
            show_editor: RwSignal::new_local(false),
            err_post: RwSignal::new_local(String::new()),
            err_fetch: RwSignal::new_local(String::new()),
            err_delete: RwSignal::new_local(String::new()),
            // params
            post_key: StoredValue::new_local(String::new()),
            kind: StoredValue::new_local(kind),
            fetch_count,
            api,
        }
    }

    fn handle_fetch_result(&self, result: Result<ServerRes, ServerErr>) -> Vec<UserPostComment> {
        // let fetch_count = self.fetch_count;
        // let finished = self.finished;

        match result {
            Ok(ServerRes::Comments(comments)) => {
                let fetch_count = self.fetch_count;
                let finished = self.finished;
                let len = comments.len();
                let is_finished = finished.get_untracked();

                if len == fetch_count && is_finished {
                    finished.set(false);
                } else if !is_finished && len < fetch_count {
                    finished.set(true);
                }

                return comments;
                // trace!(
                //     "comments manual (len){len} < (fetch_count){fetch_count} = {}",
                //     len < fetch_count
                // );
                //
                // if len == fetch_count {
                //     finished.set(false);
                // } else if !finished.get_untracked() && len < fetch_count {
                //     finished.set(true);
                // }
                //
                // if len > 0 {
                //     let replies_count = self.replies_count;
                //     self.items.update(|v| {
                //         trace!("comments manual before {v:#?}");
                //         v.extend(comments);
                //         let len = v.len();
                //
                //         trace!("replies count {} {}", replies_count.get_untracked(), len);
                //         // panic!("stop");
                //         if replies_count.get_untracked() < len {
                //             replies_count.set(len);
                //
                //         }
                //         trace!("comments manual after {v:#?}");
                //     });
                // }
            }
            Ok(err) => {
                let err = format!("post comments basic: unexpected res: {err:?}");
                error!(err);
                self.err_fetch.set(err);
            }
            Err(err) => {
                let err = format!("post comments basic: {err}");
                error!(err);
                self.err_fetch.set(err);
            }
        };
        Vec::new()
    }

    async fn fetch_replies(&self, comment_key: String, flatten: bool) -> Vec<UserPostComment> {
        let post_key = self.post_key.get_value();
        if post_key.is_empty() {
            warn!("post key not found");
            return Vec::new();
        }
        let fetch_count = self.fetch_count;
        let finished = self.finished;
        let last_item = self.items.with_untracked(|v| v.last().cloned());
        let order = Order::OneTwoThree;
        let time_range = if let Some(last_item) = last_item {
            TimeRange::More(last_item.created_at)
        } else {
            let time = time_now_ns();
            TimeRange::LessOrEqual(time)
        };

        let result = self
            .api
            .get_post_comment(
                post_key,
                Some(comment_key),
                fetch_count,
                time_range,
                order,
                flatten,
            )
            .send_native()
            .await;

        self.handle_fetch_result(result)
    }

    async fn fetch_comments(&self) -> Vec<UserPostComment> {
        let post_key = self.post_key.get_value();
        if post_key.is_empty() {
            warn!("post key not found");
            return Vec::new();
        }
        let fetch_count = self.fetch_count;
        let finished = self.finished;
        let last_item = self.items.with_untracked(|v| v.last().cloned());
        let order = Order::ThreeTwoOne;
        let time_range = if let Some(last_item) = last_item {
            TimeRange::Less(last_item.created_at)
        } else {
            let time = time_now_ns();
            TimeRange::LessOrEqual(time)
        };

        let result = self
            .api
            .get_post_comment(post_key, None, fetch_count, time_range, order, false)
            .send_native()
            .await;

        self.handle_fetch_result(result)
    }

    pub async fn fetch(self) {
        self.err_fetch.update(|v| v.clear());
        let kind = self.kind.get_value();
        match kind {
            CommentKind2::Root => {
                let comments = self.fetch_comments().await;
                if comments.is_empty() {
                    return;
                }

                self.items.update(|v| {
                    trace!("comments manual before {v:#?}");
                    v.extend(comments);
                    trace!("comments manual after {v:#?}");
                });
            }
            CommentKind2::Reply { comment, .. } => {
                let comments = self.fetch_replies(comment.key.clone(), false).await;
                if comments.is_empty() {
                    return;
                }

                let replies_count = self.replies_count;
                self.items.update(|v| {
                    trace!("comments manual before {v:#?}");
                    v.extend(comments);
                    trace!("comments manual after {v:#?}");

                    let len = v.len();
                    trace!("replies count {} {}", replies_count.get_untracked(), len);
                    if replies_count.get_untracked() < len {
                        replies_count.set(len);
                    }
                });
            }
            CommentKind2::Flat { comment, .. } => {
                let comments = self.fetch_replies(comment.key.clone(), true).await;
                if comments.is_empty() {
                    return;
                }

                let replies_count = self.replies_count;

                self.items.update(|v| {
                    trace!("comments manual before {v:#?}");
                    v.extend(comments);
                    trace!("comments manual after {v:#?}");

                    let len = v.len();
                    trace!("replies count {} {}", replies_count.get_untracked(), len);
                    if replies_count.get_untracked() < len {
                        replies_count.set(len);
                    }
                });
            }
            CommentKind2::None { comment, .. } => {
                warn!("not implemented");
                //
            }
        }
    }

    fn handle_post_result(&self, result: Result<ServerRes, ServerErr>) -> Option<UserPostComment> {
        match result {
            Ok(ServerRes::Comment(comment)) => {
                self.show_editor.set(false);
                return Some(comment);
                // self.items.update(|v| {
                //     v.insert(0, comment);
                // });
                // // comments_local.update(|v| v.insert(0, comment));
                // self.show_editor.set(false);
            }
            Ok(err) => {
                let err = format!("post comments basic: unexpected res: {err:?}");
                error!(err);
                self.err_post.set(err);
            }
            Err(err) => {
                let err = format!("post comments basic: {err}");
                error!(err);
                self.err_post.set(err);
            }
        };
        None
    }

    async fn post_comment(&self, text: impl Into<String>) -> Option<UserPostComment> {
        let post_key = self.post_key.get_value();

        if post_key.is_empty() {
            error!("trying to post reply without setting post key");
            return None;
        }

        let result = self
            .api
            .add_post_comment(post_key, None, text)
            .send_native()
            .await;

        self.handle_post_result(result)
    }

    async fn post_reply(
        &self,
        text: impl Into<String>,
        comment_key: impl Into<String>,
    ) -> Option<UserPostComment> {
        let post_key = self.post_key.get_value();

        if post_key.is_empty() {
            error!("trying to post reply without setting post key");
            return None;
        }

        let result = self
            .api
            .add_post_comment(post_key, Some(comment_key.into()), text)
            .send_native()
            .await;

        self.handle_post_result(result)
    }

    pub async fn delete(self) {
        self.err_delete.update(|v| v.clear());
        match self.kind.get_value() {
            CommentKind2::Root => {
                return;
            }
            CommentKind2::Flat {
                parent_items: parent,
                comment,
                parent_replies_count,
            }
            | CommentKind2::Reply {
                parent_items: parent,
                comment,
                parent_replies_count,
            }
            | CommentKind2::None {
                parent_items: parent,
                comment,
                parent_replies_count,
            } => {
                let result = self
                    .api
                    .delete_post_comment(comment.key.clone())
                    .send_native()
                    .await;

                match result {
                    Ok(ServerRes::Ok) => {
                        // if let Some(parent) = parent {
                        let len_before = parent.with_untracked(|v| v.len());
                        parent.update(|v| {
                            *v = v
                                .clone()
                                .into_iter()
                                .filter(|v| {
                                    !(v.key == comment.key
                                        || v.parent_key.iter().any(|v| *v == comment.key))
                                })
                                .collect::<Vec<UserPostComment>>();
                            // let Some(pos) = v.iter().position(|v| v.key == comment.key) else {
                            //     return;
                            // };
                            // v.remove(pos);
                        });
                        let len_after = parent.with_untracked(|v| v.len());
                        let removed = len_before.saturating_sub(len_after);

                        // let is_not_none = self.kind.with_value(|v| !v.is_none());
                        // if is_not_none {
                        // }
                        parent_replies_count.update(|v: &mut usize| {
                            *v = v.saturating_sub(removed);
                        });
                    }
                    Ok(err) => {
                        let err = format!("post comments basic: unexpected res: {err:?}");
                        error!(err);
                        self.err_delete.set(err);
                    }
                    Err(err) => {
                        let err = format!("post comments basic: {err}");
                        error!(err);
                        self.err_delete.set(err);
                    }
                };
            }
        }
    }

    pub async fn post(self, text: impl Into<String>) {
        // let post_key = self.post_key.get_value();
        self.err_post.update(|v| v.clear());
        let kind = self.kind.get_value();

        match kind {
            CommentKind2::Root => {
                let Some(comment) = self.post_comment(text).await else {
                    error!("failed to post");
                    return;
                };
                self.items.update(|v| {
                    if v.is_empty() {
                        v.push(comment);
                        return;
                    }
                    v.insert(0, comment);
                });
                // self.replies_count.update(|v| *v += 1);
            }
            // CommentKind2::Comment { parent, comment }
            CommentKind2::Reply {
                parent_items: parent,
                comment,
                ..
            }
            | CommentKind2::Flat {
                parent_items: parent,
                comment,
                ..
            } => {
                let Some(comment) = self.post_reply(text, comment.key).await else {
                    error!("failed to post");
                    return;
                };

                self.items.update(|v| {
                    v.push(comment);
                });
                self.replies_count.update(|v| *v += 1);
            }
            CommentKind2::None {
                parent_items: parent,
                comment,
                parent_replies_count,
            } => {
                let Some(comment) = self.post_reply(text, comment.key).await else {
                    error!("failed to post");
                    return;
                };

                parent.update(|v| {
                    v.push(comment);
                });
                parent_replies_count.update(|v| *v += 1);
            }
        }
    }

    pub fn is_last(&self) -> bool {
        let kind = self.kind.get_value();
        match kind {
            CommentKind2::Root => false,
            // CommentKind2::Comment { parent, comment }
            CommentKind2::Reply {
                parent_items: parent,
                comment,
                ..
            }
            | CommentKind2::Flat {
                parent_items: parent,
                comment,
                ..
            }
            | CommentKind2::None {
                parent_items: parent,
                comment,
                ..
            } => parent
                .with(|v| v.last().map(|v| v.key.clone()))
                .map(|v| v == comment.key)
                .unwrap_or_default(),
        }
    }

    pub fn observe_only(&self, post_key: impl Into<String>) {
        self.post_key.set_value(post_key.into());
    }
}

#[derive(Clone, Copy)]
pub struct CommentsApi {
    pub reply_editor_show: RwSignal<bool, LocalStorage>,
    pub reply_count: RwSignal<usize, LocalStorage>,
    pub finished: RwSignal<bool, LocalStorage>,
    pub grand_parent_items: Option<RwSignal<Vec<UserPostComment>, LocalStorage>>,
    pub items: RwSignal<Vec<UserPostComment>, LocalStorage>,
    pub err_post: RwSignal<String, LocalStorage>,
    pub observer: StoredValue<
        Box<dyn Fn((Option<HtmlTextAreaElement>, String, usize, bool)) + 'static>,
        LocalStorage,
    >,
    pub is_last: StoredValue<Box<dyn Fn() -> bool + 'static>, LocalStorage>,
    pub fetch_btm: StoredValue<Box<dyn Fn() + 'static>, LocalStorage>,
    pub post: StoredValue<Box<dyn Fn() + 'static>, LocalStorage>,
    pub delete: StoredValue<Box<dyn Fn(String) + 'static>, LocalStorage>,
    //
}

#[derive(Default, Clone, strum::Display, strum::EnumIs)]
pub enum CommentKind {
    #[default]
    Root,
    Comment {
        parent: CommentsApi,
        comment: UserPostComment,
    },
    Reply {
        parent: CommentsApi,
        comment: UserPostComment,
    },
    Flat {
        parent: CommentsApi,
        comment: UserPostComment,
    },
}

// pub enum CommentParent {
//
// }

impl CommentsApi {
    pub fn new(
        // comment_key: String,
        // replies_count: usize,
        // comment: Op
        // parent: Option<CommentsManual>,
        kind: CommentKind,
        // use_parent: bool,
        // reverse: bool,
    ) -> Self {
        let api = ApiWeb::new();

        let finished = RwSignal::new_local(true);
        let err_post = RwSignal::new_local(String::new());
        let post_key = StoredValue::new_local(String::new());
        // let comment_key = StoredValue::new_local(String::new());
        let fetch_count = StoredValue::new_local(50_usize);
        let flatten = StoredValue::new_local(false);
        let reply_editor_show = RwSignal::new_local(false);
        let reply_count = RwSignal::new_local(match &kind {
            CommentKind::Flat {
                parent: parent_api,
                comment: parent_comment,
            }
            | CommentKind::Reply {
                parent: parent_api,
                comment: parent_comment,
            }
            | CommentKind::Comment {
                parent: parent_api,
                comment: parent_comment,
            } => parent_comment.replies_count,
            CommentKind::Root => 0,
        });
        let parent_items = match &kind {
            CommentKind::Flat {
                parent: parent_api,
                comment: parent_comment,
            }
            | CommentKind::Reply {
                parent: parent_api,
                comment: parent_comment,
            }
            | CommentKind::Comment {
                parent: parent_api,
                comment: parent_comment,
            } => Some(parent_api.items),
            CommentKind::Root => None,
        };
        let input_elm = StoredValue::new_local(None::<HtmlTextAreaElement>);
        let comments_local = RwSignal::new_local(Vec::<UserPostComment>::new());
        // let items = if use_parent {
        //     parent
        //         .map(|v| v.items)
        //         .unwrap_or_else(|| RwSignal::new_local(Vec::<UserPostComment>::new()))
        // } else {
        //     RwSignal::new_local(Vec::<UserPostComment>::new())
        // };

        // on_cleanup(move || {
        //     trace!("cleaning up shit");
        //     items.update(move |v| {
        //         v.clear();
        //     });
        // });
        let handle_comments_result = move |result, fetch_count| {
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
                    if len > 0 {
                        return Some(comments);
                    }
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
            None
        };

        let handle_replies_result = move |result| {
            match result {
                Ok(ServerRes::Comment(comment)) => {
                    return Some(comment);
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
            None
        };

        let fetch_comments = async move || {
            let post_id = post_key.get_value();
            if post_id.is_empty() {
                return None;
            }
            let fetch_count = fetch_count.get_value();
            let last_item = comments_local.with_untracked(|v| v.last().cloned());
            let order = Order::ThreeTwoOne;
            let time_range = if let Some(last_item) = last_item {
                TimeRange::Less(last_item.created_at)
            } else {
                let time = time_now_ns();
                TimeRange::LessOrEqual(time)
            };

            let result = api
                .get_post_comment(post_id, None, fetch_count, time_range, order, false)
                .send_native()
                .await;

            handle_comments_result(result, fetch_count)
        };

        let fetch_replies = async move |flatten: bool| {
            let post_id = post_key.get_value();
            if post_id.is_empty() {
                return None;
            }
            let fetch_count = fetch_count.get_value();
            let order = Order::OneTwoThree;
            let last_item = comments_local.with_untracked(|v| v.last().cloned());
            let time_range = if let Some(last_item) = last_item {
                TimeRange::More(last_item.created_at)
            } else {
                let time = time_now_ns();
                TimeRange::LessOrEqual(time)
            };

            let result = api
                .get_post_comment(post_id, None, fetch_count, time_range, order, flatten)
                .send_native()
                .await;

            handle_comments_result(result, fetch_count)
        };

        let fetch_btm = FutureFn::new({
            // let comment_key = comment_key.clone();
            let kind = kind.clone();
            move || {
                let kind = kind.clone();
                async move {
                    match &kind {
                        CommentKind::Reply { parent, comment } => {
                            let Some(comments) = fetch_replies(false).await else {
                                return;
                            };

                            parent.items.update(|v| {
                                trace!("comments manual extended reply {v:#?}");
                                v.extend(comments);
                            });
                        }
                        CommentKind::Flat { parent, comment } => {
                            let Some(comments) = fetch_replies(true).await else {
                                return;
                            };

                            parent.items.update(|v| {
                                trace!("comments manual extended flat {v:#?}");
                                v.extend(comments);
                            });
                        }
                        CommentKind::Comment { parent, comment } => {
                            let Some(comments) = fetch_comments().await else {
                                return;
                            };

                            parent.items.update(|v| {
                                trace!("comments manual extended comment {v:#?}");
                                v.extend(comments);
                            });
                        }
                        CommentKind::Root => {
                            let Some(comments) = fetch_comments().await else {
                                return;
                            };

                            comments_local.update(|v| {
                                trace!("comments manual extended root {v:#?}");
                                v.extend(comments);
                            });
                        }
                    }
                }
            }
            // let fetch_flat = async move || {
            //     let post_id = post_key.get_value();
            //     if post_id.is_empty() {
            //         return None;
            //     }
            //     let fetch_count = fetch_count.get_value();
            //     let order = Order::OneTwoThree;
            //     let last_item = comments_local.with_untracked(|v| v.last().cloned());
            //     let time_range = if let Some(last_item) = last_item {
            //         TimeRange::More(last_item.created_at)
            //     } else {
            //         let time = time_now_ns();
            //         TimeRange::LessOrEqual(time)
            //     };
            //
            //     let result = api
            //         .get_post_comment(post_id, None, fetch_count, time_range, order, true)
            //         .send_native()
            //         .await;
            //
            //     handle_result(result, fetch_count)
            // };

            // move || {
            //     let comment_key = comment_key.clone();
            //     async move {
            //         if use_parent {
            //             warn!("comments manual - trying to fetch replies on no_replies element");
            //             return;
            //         }
            //         let (post_id, comment_key) = {
            //             let post_id = post_key.get_value();
            //             if post_id.is_empty() {
            //                 return;
            //             }
            //             trace!("comments manual 4 {comment_key}");
            //
            //             // TODO what is this nonsense
            //             let comment_key = if comment_key.is_empty() {
            //                 None
            //             } else {
            //                 Some(comment_key)
            //             };
            //
            //             (post_id, comment_key)
            //         };
            //         let fetch_count = fetch_count.get_value();
            //         let time = time_now_ns();
            //         let flatten = flatten.get_value();
            //         let order = if reverse {
            //             Order::OneTwoThree
            //         } else {
            //             Order::ThreeTwoOne
            //         };
            //
            //         // let items = &mut *items.write();
            //
            //         trace!("comments manual 3 {comment_key:?}");
            //         let items = if use_parent {
            //             parent.map(|v| v.items).unwrap_or(comments_local)
            //         } else {
            //             comments_local
            //         };
            //         let items_empty = items.with_untracked(|v| v.is_empty());
            //         let last_item = items.with_untracked(|v| v.last().cloned());
            //
            //         let result = if items_empty {
            //             api.get_post_comment(
            //                 post_id,
            //                 comment_key,
            //                 fetch_count,
            //                 TimeRange::LessOrEqual(time),
            //                 order,
            //                 flatten,
            //             )
            //             .send_native()
            //             .await
            //         } else if let Some(item) = last_item {
            //             api.get_post_comment(
            //                 post_id,
            //                 comment_key,
            //                 fetch_count,
            //                 TimeRange::Less(item.created_at),
            //                 order,
            //                 flatten,
            //             )
            //             .send_native()
            //             .await
            //         } else {
            //             return;
            //         };
            //
            //         match result {
            //             Ok(ServerRes::Comments(comments)) => {
            //                 let len = comments.len();
            //                 trace!(
            //                     "comments manual (len){len} < (fetch_count){fetch_count} = {}",
            //                     len < fetch_count
            //                 );
            //                 if len == fetch_count {
            //                     finished.set(false);
            //                 } else if !finished.get_untracked() && len < fetch_count {
            //                     finished.set(true);
            //                 }
            //                 if len > 0 {
            //                     items.update(|v| {
            //                         trace!("comments manual extended {v:#?}");
            //                         v.extend(comments);
            //                     });
            //                 }
            //                 // items.extend(comments);
            //                 // reply_editor_show.set(false);
            //             }
            //             Ok(err) => {
            //                 let err = format!("post comments basic: unexpected res: {err:?}");
            //                 error!(err);
            //             }
            //             Err(err) => {
            //                 let err = format!("post comments basic: {err}");
            //                 error!(err);
            //             }
            //         };
            //
            //         ()
            //     }
            // }
        });

        let post_comment_fn = async move || {
            err_post.update(|err| {
                err.clear();
            });
            let post_key = post_key.get_value();
            if post_key.is_empty() {
                trace!("comments manual no post key");
                return None;
            }
            let time = time_now_ns();
            let Some(post_elm) = input_elm.get_value() else {
                trace!("comments manual no input elm");
                return None;
            };
            let text = post_elm.value();

            trace!("comments manual 2 {post_key:?}");
            let result = api
                .add_post_comment(post_key, None, text)
                .send_native()
                .await;

            handle_replies_result(result)
            // match result {
            //     Ok(ServerRes::Comment(comment)) => {
            //         post_elm.set_value("");
            //         comments_local.update(|v| v.insert(0, comment));
            //         reply_editor_show.set(false);
            //         reply_count.update(|v| *v += 1);
            //     }
            //     Ok(err) => {
            //         let err = format!("post comments basic: unexpected res: {err:?}");
            //         error!(err);
            //         err_post.set(err);
            //     }
            //     Err(err) => {
            //         let err = format!("post comments basic: {err}");
            //         error!(err);
            //         err_post.set(err);
            //     }
            // };
        };

        let post_reply_fn = async move |comment_key: String| {
            err_post.update(|err| {
                err.clear();
            });
            let post_key = post_key.get_value();
            if post_key.is_empty() {
                trace!("no post key");
                return None;
            }
            let time = time_now_ns();
            let Some(post_elm) = input_elm.get_value() else {
                return None;
            };
            let text = post_elm.value();

            trace!("comments manual 2 {post_key:?}");
            let result = api
                .add_post_comment(post_key, Some(comment_key), text)
                .send_native()
                .await;

            handle_replies_result(result)
            // match result {
            //     Ok(ServerRes::Comment(comment)) => {
            //         post_elm.set_value("");
            //         comments_local.update(|v| v.insert(0, comment));
            //         reply_editor_show.set(false);
            //         reply_count.update(|v| *v += 1);
            //     }
            //     Ok(err) => {
            //         let err = format!("post comments basic: unexpected res: {err:?}");
            //         error!(err);
            //         err_post.set(err);
            //     }
            //     Err(err) => {
            //         let err = format!("post comments basic: {err}");
            //         error!(err);
            //         err_post.set(err);
            //     }
            // };
        };

        let post_fn = FutureFn::new({
            let kind = kind.clone();
            move || {
                let kind = kind.clone();
                async move {
                    let kind = kind.clone();
                    match kind {
                        CommentKind::Reply {
                            parent: parent_api,
                            comment: parent_comment,
                        } => {
                            let Some(comment) = post_reply_fn(parent_comment.key).await else {
                                return;
                            };
                            parent_api.items.update(move |v| {
                                v.push(comment);
                            });

                            //
                        }
                        CommentKind::Flat {
                            parent: parent_api,
                            comment: parent_comment,
                        } => {
                            let Some(comment) = post_reply_fn(parent_comment.key).await else {
                                return;
                            };
                            parent_api.items.update(move |v| {
                                v.push(comment);
                            });
                            //
                        }
                        CommentKind::Comment {
                            parent: parent_api,
                            comment: parent_comment,
                        } => {
                            let Some(comment) = post_comment_fn().await else {
                                return;
                            };

                            parent_api.items.update(move |v| {
                                v.insert(0, comment);
                            });
                        }
                        CommentKind::Root => {
                            let Some(comment) = post_comment_fn().await else {
                                return;
                            };

                            comments_local.update(move |v| {
                                v.insert(0, comment);
                            });
                        }
                    };
                    // post_reply_fn();
                    //
                }
            }
        });

        // let post_run = FutureFn::new({
        //     let kind = kind.clone();
        //     // let comment_key = comment_key.clone();
        //     move || {
        //         let kind = kind.clone();
        //         // let comment_key = comment_key.clone();
        //         async move {
        //             // trace!("running post_run");
        //             // err_post.update(|err| {
        //             //     err.clear();
        //             // });
        //             //
        //             // let (post_key, comment_key, time, text, post_elm) = {
        //             //     let post_key = post_key.get_value();
        //             //     if post_key.is_empty() {
        //             //         trace!("no post key");
        //             //         return;
        //             //     }
        //             //
        //             //     trace!("comments manual {comment_key}");
        //             //     let comment_key = if comment_key.is_empty() {
        //             //         None
        //             //     } else {
        //             //         Some(comment_key)
        //             //     };
        //             //     let time = time_now_ns();
        //             //
        //             //     let Some(post_elm) = input_elm.get_value() else {
        //             //         return;
        //             //     };
        //             //     let text = post_elm.value();
        //             //     (post_key, comment_key, time, text, post_elm)
        //             // };
        //             //
        //             // trace!("comments manual 2 {comment_key:?}");
        //             // let result = api
        //             //     .add_post_comment(post_key, comment_key, text)
        //             //     .send_native()
        //             //     .await;
        //             //
        //             // match result {
        //             //     Ok(ServerRes::Comment(comment)) => {
        //             //         post_elm.set_value("");
        //             //         // let items = &mut *items.write();
        //             //
        //             //         if let Some(parent) = parent
        //             //             && use_parent
        //             //         {
        //             //             parent.items.update(|v| {
        //             //                 if reverse {
        //             //                     trace!("comments manual push 0 {v:#?}");
        //             //                     v.push(comment);
        //             //                 } else {
        //             //                     trace!("comments manual push 1 {v:#?}");
        //             //                     v.insert(0, comment);
        //             //                 }
        //             //             });
        //             //         } else {
        //             //             comments_local.update(|v| {
        //             //                 trace!("comments manual push 2 {v:#?}");
        //             //                 v.push(comment);
        //             //             });
        //             //         }
        //             //         reply_editor_show.set(false);
        //             //         reply_count.update(|v| *v += 1);
        //             //     }
        //             //     Ok(err) => {
        //             //         let err = format!("post comments basic: unexpected res: {err:?}");
        //             //         error!(err);
        //             //         err_post.set(err);
        //             //     }
        //             //     Err(err) => {
        //             //         let err = format!("post comments basic: {err}");
        //             //         error!(err);
        //             //         err_post.set(err);
        //             //     }
        //             // };
        //         }
        //     }
        // });

        let delete_comment_fn = {
            let kind = kind.clone();
            async move || {
                // match &kind {
                // C

                //}
                // let result = api
                //     .delete_post_comment(comment_key.clone())
                //     .send_native()
                //     .await;
            }
        };

        let delete_fn = FutureFn::new(move |comment_key: String| async move {
            trace!("comments manual 2 {comment_key:?}");
            let result = api
                .delete_post_comment(comment_key.clone())
                .send_native()
                .await;

            match result {
                Ok(ServerRes::Ok) => {
                    // if let Some(parent) = parent {
                    //     parent.items.update(|v| {
                    //         let Some(pos) = v.iter().position(|v| v.key == comment_key) else {
                    //             return;
                    //         };
                    //         v.remove(pos);
                    //     });
                    //     parent.reply_count.update(|v: &mut usize| {
                    //         *v = v.saturating_sub(1);
                    //     });
                    // }
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

        let is_last = {
            let kind = kind.clone();
            move || -> bool {
                let parent_last = match &kind {
                    CommentKind::Flat {
                        parent: parent_api,
                        comment: parent_comment,
                    }
                    | CommentKind::Reply {
                        parent: parent_api,
                        comment: parent_comment,
                    }
                    | CommentKind::Comment {
                        parent: parent_api,
                        comment: parent_comment,
                    } => parent_api.items.with(|v| v.last().map(|v| v.key.clone())),
                    CommentKind::Root => {
                        return false;
                    }
                };

                let current_last = comments_local.with(|v| v.last().map(|v| v.key.clone()));

                parent_last == current_last
                // match () {
                //     (Some(parent_last), Some(current_last)) = pa
                //
                //     _ => false
                // }
                //
                //
                //
                // last.map(|v| v.);
                // if comment_key.is_empty() {
                //     return false;
                // }
                //
                // parent
                //     .and_then(|v| v.items.with(|v| v.last().map(|v| v.key == comment_key)))
                //     .unwrap_or_default()
            }
        };

        let observe = move |(post_input, new_post_id, new_count, new_flatten): (
            Option<HtmlTextAreaElement>,
            String,
            usize,
            bool,
        )| {
            post_key.set_value(new_post_id);
            // comment_key.set_value(new_comment_key);
            fetch_count.set_value(new_count);
            input_elm.set_value(post_input);
            flatten.set_value(new_flatten);
        };

        let fetch_btm = move || {
            fetch_btm.run();
        };

        let post = move || {
            post_fn.run();
        };

        let delete_fn = move |comment_key: String| {
            delete_fn.run(comment_key);
        };

        Self {
            reply_editor_show,
            reply_count,
            err_post,
            grand_parent_items: parent_items,
            items: comments_local,
            finished,
            is_last: StoredValue::new_local(Box::new(is_last)),
            delete: StoredValue::new_local(Box::new(delete_fn)),
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
        count: usize,
        flatten: bool,
    ) {
        // trace!("comments manual observe {comment_key}");
        self.observer.run((post_input, post_id, count, flatten));
    }
}

// #[cfg(test)]
#[cfg(test)]
pub mod tests {
    use crate::{
        api::{shared::post_comment::UserPostComment, tests::ApiTestApp},
        view::{
            app::hook::use_post_comments_manual::{
                CommentKind, CommentKind2, CommentsApi, CommentsApi2,
            },
            logger,
            toolbox::prelude::*,
        },
    };
    use hydration_context::HydrateSharedContext;
    use leptos::prelude::*;
    use std::sync::Arc;
    use surrealdb::types::ToSql;
    use tokio::process::Command;
    use tracing::{debug, trace};

    use crate::init_test_log;
    //
    #[tokio::test]
    pub async fn hook_comments_api_post() {
        println!("hello");
        init_test_log();
        let owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
        let mut app = ApiTestApp::new(10).await;

        let mut time = 0_u128;
        let mut t = move || {
            time += 1;
            time
        };

        let auth_token = app
            .register(t(), "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        app.api.pre_load_token = auth_token.clone();

        let post = app.add_post(t(), &auth_token).await.unwrap();

        let hook_root = CommentsApi2::new(&app.api, 2, CommentKind2::Root);
        hook_root.observe_only(post.id.clone());

        (app.set_time(t()).await, hook_root.post("c0").await);
        (app.set_time(t()).await, hook_root.post("c1").await);
        (app.set_time(t()).await, hook_root.post("c2").await);
        (app.set_time(t()).await, hook_root.post("c3").await);

        let c0 = hook_root.items.with_untracked(|v| v[3].clone());
        let hook_reply = CommentsApi2::new(
            &app.api,
            2,
            CommentKind2::Reply {
                parent_items: hook_root.items,
                parent_replies_count: hook_root.replies_count,
                comment: c0.clone(),
            },
        );
        hook_reply.observe_only(post.id.clone());

        (app.set_time(t()).await, hook_reply.post("c0_r0x1").await);
        (app.set_time(t()).await, hook_reply.post("c0_r1x1").await);
        (app.set_time(t()).await, hook_reply.post("c0_r2x1").await);
        (app.set_time(t()).await, hook_reply.post("c0_r3x1").await);

        let c0_r0x1 = hook_reply.items.with_untracked(|v| v[0].clone());
        let hook_flat = CommentsApi2::new(
            &app.api,
            2,
            CommentKind2::Flat {
                parent_items: hook_reply.items,
                parent_replies_count: hook_reply.replies_count,
                comment: c0_r0x1.clone(),
            },
        );
        hook_flat.observe_only(post.id.clone());

        (app.set_time(t()).await, hook_flat.post("c0_r0x2").await);
        (app.set_time(t()).await, hook_flat.post("c0_r1x2").await);
        (app.set_time(t()).await, hook_flat.post("c0_r2x2").await);
        (app.set_time(t()).await, hook_flat.post("c0_r3x2").await);

        let c0_r0x2 = hook_flat.items.with_untracked(|v| v[3].clone());
        let hook_none = CommentsApi2::new(
            &app.api,
            2,
            CommentKind2::None {
                parent_items: hook_flat.items,
                parent_replies_count: hook_flat.replies_count,
                comment: c0_r0x2.clone(),
            },
        );
        hook_none.observe_only(post.id.clone());

        (app.set_time(t()).await, hook_none.post("c0_r0x3").await);
        (app.set_time(t()).await, hook_none.post("c0_r1x3").await);
        (app.set_time(t()).await, hook_none.post("c0_r2x3").await);
        (app.set_time(t()).await, hook_none.post("c0_r3x3").await);

        let items_root = hook_root.items.get_untracked();

        assert_eq!(items_root.len(), 4);
        assert_eq!(items_root[0].text, "c3");
        assert_eq!(items_root[3].text, "c0");

        let items_reply = hook_reply.items.get_untracked();

        assert_eq!(items_reply.len(), 4);
        assert_eq!(items_reply[0].text, "c0_r0x1");
        assert_eq!(items_reply[3].text, "c0_r3x1");

        let items_flat = hook_flat.items.get_untracked();

        assert_eq!(items_flat.len(), 8);
        assert_eq!(items_flat[0].text, "c0_r0x2");
        assert_eq!(items_flat[7].text, "c0_r3x3");

        let items_none = hook_none.items.get_untracked();

        assert_eq!(items_none.len(), 0);

        // get

        let all_comments = app.state.db.get_post_comments_all().await.unwrap();
        let mut output = String::new();
        for comment in all_comments {
            let line = format!(
                "{} - {} - {} - {:?}\n",
                comment.id.key.to_sql(),
                comment.text,
                comment.created_at,
                comment.parent
            );
            output.push_str(&line);
        }
        trace!("all comments \n{output}");
        // trace!("all comments {all_comments:#?}");
        // panic!("wtf");

        let hook_root = CommentsApi2::new(&app.api, 4, CommentKind2::Root);
        hook_root.observe_only(post.id.clone());
        hook_root.fetch().await;
        let items_root = hook_root.items.get_untracked();

        assert_eq!(items_root.len(), 4);
        assert_eq!(items_root[0].text, "c3");
        assert_eq!(items_root[3].text, "c0");

        let c0 = items_root[3].clone();
        let hook_reply = CommentsApi2::new(
            &app.api,
            4,
            CommentKind2::Reply {
                parent_items: hook_root.items,
                parent_replies_count: hook_root.replies_count,
                comment: c0.clone(),
            },
        );
        hook_reply.observe_only(post.id.clone());
        hook_reply.fetch().await;
        let items_reply = hook_reply.items.get_untracked();

        assert_eq!(items_reply.len(), 4);
        assert_eq!(items_reply[0].text, "c0_r0x1");
        assert_eq!(items_reply[3].text, "c0_r3x1");

        let c0_r0x1 = items_reply[0].clone();
        let hook_flat = CommentsApi2::new(
            &app.api,
            4,
            CommentKind2::Flat {
                parent_items: hook_reply.items,
                parent_replies_count: hook_reply.replies_count,
                comment: c0_r0x1.clone(),
            },
        );
        hook_flat.observe_only(post.id.clone());
        hook_flat.fetch().await;
        let items_flat = hook_flat.items.get_untracked();

        assert_eq!(items_flat.len(), 4);
        assert_eq!(items_flat[0].text, "c0_r0x2");
        assert_eq!(items_flat[3].text, "c0_r3x2");

        hook_flat.fetch().await;
        let items_flat = hook_flat.items.get_untracked();

        assert_eq!(items_flat.len(), 8);
        assert_eq!(items_flat[0].text, "c0_r0x2");
        assert_eq!(items_flat[7].text, "c0_r3x3");
    }

    #[tokio::test]
    pub async fn hook_comments_api_delete() {
        println!("hello");
        init_test_log();
        let owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
        let mut app = ApiTestApp::new(10).await;

        let mut time = 0_u128;
        let mut t = move || {
            time += 1;
            time
        };

        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        app.api.pre_load_token = auth_token.clone();

        let post = app.add_post(t(), &auth_token).await.unwrap();

        let hook_root = CommentsApi2::new(&app.api, 2, CommentKind2::Root);
        hook_root.observe_only(post.id.clone());

        (app.set_time(t()).await, hook_root.post("c0").await);
        (app.set_time(t()).await, hook_root.post("c1").await);

        hook_root.delete().await;
        let items_root = hook_root.items.get();
        assert_eq!(items_root.len(), 2);

        let c0 = hook_root.items.with_untracked(|v| v[1].clone());

        let hook_reply = CommentsApi2::new(
            &app.api,
            2,
            CommentKind2::Reply {
                parent_items: hook_root.items,
                parent_replies_count: hook_root.replies_count,
                comment: c0,
            },
        );
        hook_reply.observe_only(post.id.clone());

        (app.set_time(t()).await, hook_reply.post("c0_r0x1").await);
        (app.set_time(t()).await, hook_reply.post("c0_r1x1").await);

        let items_reply = hook_reply.items.get();
        assert_eq!(items_reply.len(), 2);

        let c0_r0x1 = hook_reply.items.with_untracked(|v| v[0].clone());

        let hook_flat = CommentsApi2::new(
            &app.api,
            2,
            CommentKind2::Flat {
                parent_items: hook_reply.items,
                parent_replies_count: hook_reply.replies_count,
                comment: c0_r0x1,
            },
        );
        hook_flat.observe_only(post.id.clone());

        (app.set_time(t()).await, hook_flat.post("c0_r0x2").await);
        (app.set_time(t()).await, hook_flat.post("c0_r1x2").await);

        let items_flat = hook_flat.items.get();
        assert_eq!(items_flat.len(), 2);

        let c0_r0x2 = hook_flat.items.with_untracked(|v| v[0].clone());

        let hook_none = CommentsApi2::new(
            &app.api,
            2,
            CommentKind2::None {
                parent_items: hook_flat.items,
                parent_replies_count: hook_flat.replies_count,
                comment: c0_r0x2,
            },
        );
        hook_none.observe_only(post.id.clone());

        (app.set_time(t()).await, hook_none.post("c0_r0x3").await);
        (app.set_time(t()).await, hook_none.post("c0_r1x3").await);

        let items_flat = hook_flat.items.get();
        assert_eq!(items_flat.len(), 4);

        // let c0_r0x3 = hook_flat.items.with_untracked(|v| v[3].clone());
        // let hook_none2 = CommentsApi2::new(
        //     &app.api,
        //     2,
        //     CommentKind2::None {
        //         parent_items: hook_flat.items,
        //         parent_replies_count: hook_flat.replies_count,
        //         comment: c0_r0x3,
        //     },
        // );
        // hook_none2.observe_only(post.id.clone());
        //
        // (app.set_time(t()).await, hook_none2.post("c0_r0x4").await);
        //
        // let items_flat = hook_flat.items.get();
        // assert_eq!(items_flat.len(), 5);

        {
            let replies_count = hook_flat.replies_count.get_untracked();
            assert_eq!(replies_count, 4);

            hook_none.delete().await;

            let items_flat = hook_flat.items.get_untracked();
            let replies_count = hook_flat.replies_count.get_untracked();

            assert_eq!(replies_count, 1);
            assert_eq!(items_flat.len(), 1);
            assert_eq!(items_flat[0].text, "c0_r1x2");

            // assert_eq!(items_flat[2].text, "c0_r1x3");
        }

        {
            let replies_count = hook_reply.replies_count.get_untracked();
            assert_eq!(replies_count, 2);

            hook_flat.delete().await;

            let items_reply = hook_reply.items.get();
            let replies_count = hook_reply.replies_count.get_untracked();

            assert_eq!(replies_count, 1);
            assert_eq!(items_reply.len(), 1);
            assert_eq!(items_reply[0].text, "c0_r1x1");
        }

        {
            let replies_count = hook_root.replies_count.get_untracked();
            assert_eq!(replies_count, 0);

            hook_reply.delete().await;

            let items_root = hook_root.items.get();
            let replies_count = hook_root.replies_count.get_untracked();

            assert_eq!(replies_count, 0);
            assert_eq!(items_root.len(), 1);
            assert_eq!(items_root[0].text, "c1");
        }
    }

    #[test]
    pub fn hook_comments_api_get() {
        let mut rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .enable_io()
            .build()
            .unwrap();
        rt.block_on(async {
            println!("hello");
            init_test_log();
            let owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
            let mut app = ApiTestApp::new(10).await;

            let auth_token = app
                .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
                .await
                .unwrap();

            app.api.pre_load_token = auth_token.clone();

            let post = app.add_post(0, &auth_token).await.unwrap();

            let hook_root = CommentsApi2::new(&app.api, 2, CommentKind2::Root);
            hook_root.post_key.set_value(post.id.clone());

            // (app.set_time(0).await, hook_root.post("comment0").await);
            // let comment0 = hook_root.items.with_untracked(|v| v[0].clone());

            let comment0 = app
                .add_post_comment(0, &auth_token, post.id.clone(), None, "wowza".to_string())
                .await
                .unwrap();

            let mut time = 0_u128;
            let mut get_time = move || {
                time += 1;
                time
            };

            let comment0_reply0 = app
                .add_post_comment(
                    get_time(),
                    &auth_token,
                    post.id.clone(),
                    Some(comment0.key.clone()),
                    "comment0_reply0".to_string(),
                )
                .await
                .unwrap();

            let comment0_reply0_reply0 = app
                .add_post_comment(
                    get_time(),
                    &auth_token,
                    post.id.clone(),
                    Some(comment0_reply0.key.clone()),
                    "comment0_reply0_reply0".to_string(),
                )
                .await
                .unwrap();

            let comment0_reply0_times_3 = app
                .add_post_comment(
                    get_time(),
                    &auth_token,
                    post.id.clone(),
                    Some(comment0_reply0_reply0.key.clone()),
                    "comment0_reply0_times_3".to_string(),
                )
                .await
                .unwrap();

            let comment0_reply0_times_4 = app
                .add_post_comment(
                    get_time(),
                    &auth_token,
                    post.id.clone(),
                    Some(comment0_reply0_times_3.key.clone()),
                    "comment0_reply0_times_4".to_string(),
                )
                .await
                .unwrap();

            let comment0_reply0_times_5 = app
                .add_post_comment(
                    get_time(),
                    &auth_token,
                    post.id.clone(),
                    Some(comment0_reply0_times_4.key.clone()),
                    "comment0_reply0_times_5".to_string(),
                )
                .await
                .unwrap();

            let comment0_reply0_times_6 = app
                .add_post_comment(
                    get_time(),
                    &auth_token,
                    post.id.clone(),
                    Some(comment0_reply0_times_5.key.clone()),
                    "comment0_reply0_times_6".to_string(),
                )
                .await
                .unwrap();

            let comment0_reply0_reply1 = app
                .add_post_comment(
                    get_time(),
                    &auth_token,
                    post.id.clone(),
                    Some(comment0_reply0.key.clone()),
                    "comment0_reply0_reply1".to_string(),
                )
                .await
                .unwrap();

            let comment0_reply0_reply2 = app
                .add_post_comment(
                    get_time(),
                    &auth_token,
                    post.id.clone(),
                    Some(comment0_reply0.key.clone()),
                    "comment0_reply0_reply2".to_string(),
                )
                .await
                .unwrap();

            let comment0_reply0_reply3 = app
                .add_post_comment(
                    get_time(),
                    &auth_token,
                    post.id.clone(),
                    Some(comment0_reply0.key.clone()),
                    "comment0_reply0_reply3".to_string(),
                )
                .await
                .unwrap();

            let comment0_reply1 = app
                .add_post_comment(
                    get_time(),
                    &auth_token,
                    post.id.clone(),
                    Some(comment0.key.clone()),
                    "comment0_reply1".to_string(),
                )
                .await
                .unwrap();

            let comment0_reply2 = app
                .add_post_comment(
                    get_time(),
                    &auth_token,
                    post.id.clone(),
                    Some(comment0.key.clone()),
                    "comment0_reply2".to_string(),
                )
                .await
                .unwrap();

            let comment0_reply3 = app
                .add_post_comment(
                    get_time(),
                    &auth_token,
                    post.id.clone(),
                    Some(comment0.key.clone()),
                    "comment0_reply3".to_string(),
                )
                .await
                .unwrap();

            let comment1 = app
                .add_post_comment(
                    get_time(),
                    &auth_token,
                    post.id.clone(),
                    None,
                    "comment1".to_string(),
                )
                .await
                .unwrap();

            let comment2 = app
                .add_post_comment(
                    get_time(),
                    &auth_token,
                    post.id.clone(),
                    None,
                    "comment2".to_string(),
                )
                .await
                .unwrap();

            let comment3 = app
                .add_post_comment(
                    get_time(),
                    &auth_token,
                    post.id.clone(),
                    None,
                    "comment3".to_string(),
                )
                .await
                .unwrap();

            let replies_count = hook_root.replies_count.get_untracked();
            assert_eq!(replies_count, 0);

            hook_root.fetch().await;

            let post_comments = hook_root.items.get_untracked();
            let replies_count = hook_root.replies_count.get_untracked();

            assert_eq!(replies_count, 0);
            assert_eq!(post_comments.len(), 2);
            assert_eq!(post_comments[0], comment3);
            assert_eq!(post_comments[1], comment2);

            hook_root.fetch().await;

            let post_comments = hook_root.items.get_untracked();
            assert_eq!(post_comments.len(), 4);
            assert_eq!(post_comments[0], comment3);
            assert_eq!(post_comments[1], comment2);
            assert_eq!(post_comments[2], comment1);
            assert_eq!(post_comments[3].key, comment0.key);

            let comment4 = app
                .add_post_comment(
                    4,
                    &auth_token,
                    post.id.clone(),
                    None,
                    "comment4".to_string(),
                )
                .await
                .unwrap();

            hook_root.fetch().await;

            let post_comments = hook_root.items.get_untracked();
            assert_eq!(post_comments.len(), 4);
            assert_eq!(post_comments[0], comment3);
            assert_eq!(post_comments[1], comment2);
            assert_eq!(post_comments[2], comment1);
            assert_eq!(post_comments[3].key, comment0.key);

            let hook_comment = CommentsApi2::new(
                &app.api,
                2,
                CommentKind2::Reply {
                    parent_items: hook_root.items,
                    parent_replies_count: hook_root.replies_count,
                    comment: comment0.clone(),
                },
            );
            hook_comment.post_key.set_value(post.id.clone());

            hook_comment.fetch().await;
            let comment0_replies = hook_comment.items.get_untracked();

            assert_eq!(comment0_replies.len(), 2);
            assert_eq!(comment0_replies[0].key, comment0_reply0.key);
            assert_eq!(comment0_replies[0].replies_count, 4);
            assert_eq!(comment0_replies[1], comment0_reply1);

            hook_comment.fetch().await;
            let comment0_replies = hook_comment.items.get_untracked();

            assert_eq!(comment0_replies.len(), 4);
            assert_eq!(comment0_replies[0].key, comment0_reply0.key);
            assert_eq!(comment0_replies[0].replies_count, 4);
            assert_eq!(comment0_replies[1], comment0_reply1);
            assert_eq!(comment0_replies[2], comment0_reply2);
            assert_eq!(comment0_replies[3], comment0_reply3);

            let hook_reply = CommentsApi2::new(
                &app.api,
                2,
                CommentKind2::Reply {
                    parent_items: hook_comment.items,
                    parent_replies_count: hook_comment.replies_count,
                    comment: comment0_reply0.clone(),
                },
            );
            hook_reply.post_key.set_value(post.id.clone());

            // trace!("yo yo yo yo did u run or no");
            hook_reply.fetch().await;
            let comment0_reply0_replies = hook_reply.items.get_untracked();
            // trace!("WHAT THE F*CK: {comment0_reply0_replies:#?}");
            // hook_reply.items.update(|v| {
            //     trace!("WHAT THE F*CK 2: {v:#?}");
            // });

            assert_eq!(comment0_reply0_replies.len(), 2);
            assert_eq!(comment0_reply0_replies[0].key, comment0_reply0_reply0.key);
            assert_eq!(comment0_reply0_replies[0].replies_count, 1);
            assert_eq!(comment0_reply0_replies[1], comment0_reply0_reply1);

            hook_reply.fetch().await;
            let comment0_reply0_replies = hook_reply.items.get_untracked();

            assert_eq!(comment0_reply0_replies.len(), 4);
            assert_eq!(comment0_reply0_replies[0].key, comment0_reply0_reply0.key);
            assert_eq!(comment0_reply0_replies[0].replies_count, 1);
            assert_eq!(comment0_reply0_replies[1], comment0_reply0_reply1);
            assert_eq!(comment0_reply0_replies[2], comment0_reply0_reply2);
            assert_eq!(comment0_reply0_replies[3], comment0_reply0_reply3);

            let hook_flat = CommentsApi2::new(
                &app.api,
                2,
                CommentKind2::Flat {
                    parent_items: hook_reply.items,
                    parent_replies_count: hook_reply.replies_count,
                    comment: comment0_reply0_reply0.clone(),
                },
            );
            hook_flat.post_key.set_value(post.id.clone());

            trace!("comment0_reply0_reply0 {comment0_reply0_reply0:#?}");

            let replies_count = hook_flat.replies_count.get_untracked();
            assert_eq!(replies_count, 0);

            hook_flat.fetch().await;

            let comment0_reply0_reply0_replies = hook_flat.items.get_untracked();
            let replies_count = hook_flat.replies_count.get_untracked();
            trace!("comment0_reply0_reply0_replies {comment0_reply0_reply0_replies:#?}");

            assert_eq!(replies_count, 2);
            assert_eq!(comment0_reply0_reply0_replies.len(), 2);
            assert_eq!(
                comment0_reply0_reply0_replies[0].key,
                comment0_reply0_times_3.key
            );
            assert_eq!(
                comment0_reply0_reply0_replies[1].key,
                comment0_reply0_times_4.key
            );

            hook_flat.fetch().await;
            let comment0_reply0_reply0_replies = hook_flat.items.get_untracked();
            let replies_count = hook_flat.replies_count.get_untracked();

            assert_eq!(replies_count, 4);
            assert_eq!(comment0_reply0_reply0_replies.len(), 4);
            assert_eq!(
                comment0_reply0_reply0_replies[0].key,
                comment0_reply0_times_3.key
            );
            assert_eq!(
                comment0_reply0_reply0_replies[1].key,
                comment0_reply0_times_4.key
            );
            assert_eq!(
                comment0_reply0_reply0_replies[2].key,
                comment0_reply0_times_5.key
            );
            assert_eq!(
                comment0_reply0_reply0_replies[3].key,
                comment0_reply0_times_6.key
            );
            assert_eq!(hook_flat.finished.get_untracked(), false);

            hook_flat.fetch().await;
            let comment0_reply0_reply0_replies = hook_flat.items.get_untracked();

            assert_eq!(comment0_reply0_reply0_replies.len(), 4);
            assert_eq!(hook_flat.finished.get_untracked(), true);

            // let (mut browser, mut handler) =
            // Browser::launch(BrowserConfig::builder().with_head().build().unwrap())
            //     .await
            //     .unwrap();
        });
    }

    // #[tokio::test]
    pub fn comments_hook() {
        console_error_panic_hook::set_once();
        logger::simple_web_logger_init();
        tracing::debug!("yo wtf");
        debug!("hello");

        stringify!({
            let list_root = Vec::<UserPostComment>::new();
            let list_normal = Vec::<UserPostComment>::new();
            let list_reply = Vec::<UserPostComment>::new();
            let list_reply2 = Vec::<UserPostComment>::new();
            let list_flat = Vec::<UserPostComment>::new();
            let list_flat1 = Vec::<UserPostComm>::new();
        });

        //init_test_log();
        // leptos::mount::hydrate_body(App);
        // _ = Executor::init_wasm_bindgen();
        // let owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
        //
        // let api = CommentsApi::new(CommentKind::Root);
        // api.fetch_btm();
        //
        // let list_root = Vec::<UserPostComment>::new();
        // let list_normal = Vec::<UserPostComment>::new();
        // let list_reply = Vec::<UserPostComment>::new();
        // let list_reply2 = Vec::<UserPostComment>::new();
        // let list_flat = Vec::<UserPostComment>::new();

        // let result = Command::new("cargo")
        //     .args([
        //         "build",
        //         "--package=artbounty",
        //         "--lib",
        //         "--target=wasm32-unknown-unknown",
        //         "--features=hydrate",
        //         "--profile",
        //         "wasm-debug",
        //     ])
        //     .output()
        //     .await
        //     .expect("failed to execute process");
        // debug!("{result:#?}");

        // let a = Command::new();

        // let a = RwSignal::new(0);

        // a.set(69);

        // println!("yoyyo");

        // trace!("hello {}", a.get_untracked());
    }

    // TODO add error test
}
