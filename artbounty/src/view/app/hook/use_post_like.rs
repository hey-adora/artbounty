use leptos::prelude::*;
use tracing::error;
use web_sys::MouseEvent;

use crate::api::{Api, ApiWeb, ServerRes};
use crate::get_timestamp;
use crate::view::app::GlobalState;
use crate::view::toolbox::prelude::*;

#[derive(Clone, Copy)]
pub struct PostLike {
    // pub err_general: RwQuery<String>,
    // pub email: RwQuery<String>,
    // pub form_stage: RwQuery<ChangePasswordFormStage>,
    // pub btn_stage: StoredValue<Box<dyn Fn() -> ChangePasswordBtnStage + Sync + Send + 'static>>,
    // pub stage: RwSignal<PostLikeStage>,
    pub stage: StoredValue<Box<dyn Fn() -> PostLikeStage + Sync + Send + 'static>>,
    pub on_like: StoredValue<Box<dyn Fn(MouseEvent) + Sync + Send + 'static>>,
    // pub token: RwQuery<String>,
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    PartialOrd,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum PostLikeStage {
    #[default]
    Loading,
    Liked,
    Unliked,
}

pub fn use_post_like(
    // post_id: impl Fn() -> Option<String> + Clone + Sync + Send + 'static,
    post_id: Memo<Option<String>>,
) -> PostLike {
    // let post_id = post_id.into();
    let api = ApiWeb::new();
    let stage = RwSignal::new(PostLikeStage::Loading);
    let stage_view = move || {
        if api.is_pending_tracked() {
            PostLikeStage::Loading
        } else {
            stage.get()
        }
    };

    // let r = Resource::new(post_id, move |post_id| async move {
    //     let result = api.check_post_like(post_id.clone()).send_native().await;
    //
    //     match result {
    //         Ok(ServerRes::Condition(condition)) => {
    //             if condition {
    //                 stage.set(PostLikeStage::Liked);
    //             } else {
    //                 stage.set(PostLikeStage::Unliked);
    //             }
    //         }
    //         Ok(err) => {
    //             error!("use_post_like: expected ServerRes::Condition, received: {err:?}");
    //             stage.set(PostLikeStage::Unliked);
    //         }
    //         Err(err) => {
    //             error!("use_post_like: {err}");
    //             stage.set(PostLikeStage::Unliked);
    //         }
    //     }
    // });
    // let s = Memo::new(move |v| {
    //     let post_id = post_id();
    //     match post_id {
    //         Some(post_id) => {
    //             //
    //         }
    //         None => {}
    //     }
    // });
    // let s2 = LocalResource

    Effect::new({
        let post_id = post_id.clone();
        move || {
            let Some(post_id) = post_id.get() else {
                return;
            };
            api.check_post_like(post_id)
                .send_web(async move |result| match result {
                    Ok(ServerRes::Condition(condition)) => {
                        if condition {
                            stage.set(PostLikeStage::Liked);
                        } else {
                            stage.set(PostLikeStage::Unliked);
                        }
                    }
                    Ok(err) => {
                        error!("use_post_like: expected ServerRes::Condition, received: {err:?}");
                        stage.set(PostLikeStage::Unliked);
                    }
                    Err(err) => {
                        error!("use_post_like: {err}");
                        stage.set(PostLikeStage::Unliked);
                    }
                });
        }
    });

    let on_like = move |_: MouseEvent| {
        let Some(post_id) = post_id.get() else {
            return;
        };
        match stage.get_untracked() {
            PostLikeStage::Loading => {
                //
            }
            PostLikeStage::Liked => {
                api.delete_post_like(post_id).send_web(async move |result| {
                    match result {
                        Ok(ServerRes::Ok) => {
                            stage.set(PostLikeStage::Unliked);
                        }
                        Ok(res) => {
                            error!("error, expected OK, received: {res:?}");
                            stage.set(PostLikeStage::Unliked);
                        }
                        Err(err) => {
                            error!("use_post_like: {err}");
                            stage.set(PostLikeStage::Unliked);
                        }
                    };
                });
            }
            PostLikeStage::Unliked => {
                api.add_post_like(post_id.clone())
                    .send_web(async move |result| {
                        match result {
                            Ok(ServerRes::Ok) => {
                                stage.set(PostLikeStage::Liked);
                            }
                            Ok(res) => {
                                error!("error, expected OK, received: {res:?}");
                                stage.set(PostLikeStage::Unliked);
                            }
                            Err(err) => {
                                error!("use_post_like: {err}");
                                stage.set(PostLikeStage::Unliked);
                            }
                        };
                    });
            }
        }
    };

    PostLike {
        stage: StoredValue::new(Box::new(stage_view)),
        on_like: StoredValue::new(Box::new(on_like)),
    }
}
