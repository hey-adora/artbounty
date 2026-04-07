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
use tracing::{error, trace, warn};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, HtmlTextAreaElement, MutationObserver, MutationRecord};

#[derive(Clone, Copy)]
pub struct CommentsApi2<API: Api> {
    pub items: RwSignal<Vec<UserPostComment>, LocalStorage>,
    pub finished: RwSignal<bool, LocalStorage>,
    pub post_key: StoredValue<String, LocalStorage>,
    pub fetch_count: usize,
    api: API,
}

impl<API> CommentsApi2<API>
where
    API: Api,
{
    pub fn new(api: API, fetch_count: usize) -> Self {
        Self {
            items: RwSignal::new_local(Vec::new()),
            finished: RwSignal::new_local(false),
            post_key: StoredValue::new_local(String::new()),
            fetch_count,
            api,
        }
    }

    pub async fn fetch_btm(&self) {
        let post_key = self.post_key.get_value();
        if post_key.is_empty() {
            return;
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
                    self.items.update(|v| {
                        v.extend(comments);
                    });
                    // return Some(comments);
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
            app::hook::use_post_comments_manual::{CommentKind, CommentsApi, CommentsApi2},
            logger,
            toolbox::prelude::*,
        },
    };
    use hydration_context::HydrateSharedContext;
    use leptos::prelude::*;
    use std::sync::Arc;
    use tokio::process::Command;
    use tracing::{debug, trace};

    use crate::init_test_log;
    //

    #[tokio::test]
    pub async fn kill_them_all() {
        init_test_log();
        let owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
        let mut app = ApiTestApp::new(10).await;

        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let post = app.add_post(0, &auth_token).await.unwrap();

        let comment0 = app
            .add_post_comment(0, &auth_token, post.id.clone(), None, "wowza".to_string())
            .await
            .unwrap();

        let comment1 = app
            .add_post_comment(1, &auth_token, post.id.clone(), None, "wowza2".to_string())
            .await
            .unwrap();

        let comment2 = app
            .add_post_comment(2, &auth_token, post.id.clone(), None, "wowza2".to_string())
            .await
            .unwrap();

        let comment3 = app
            .add_post_comment(3, &auth_token, post.id.clone(), None, "wowza2".to_string())
            .await
            .unwrap();

        app.api.pre_load_token = auth_token;

        let hook = CommentsApi2::new(&app.api, 2);
        hook.post_key.set_value(post.id.clone());

        hook.fetch_btm().await;

        let items = hook.items.get();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], comment3);
        assert_eq!(items[1], comment2);

        hook.fetch_btm().await;

        let items = hook.items.get();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0], comment3);
        assert_eq!(items[1], comment2);
        assert_eq!(items[2], comment1);
        assert_eq!(items[3], comment0);

        // let (mut browser, mut handler) =
        // Browser::launch(BrowserConfig::builder().with_head().build().unwrap())
        //     .await
        //     .unwrap();
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
}
