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
    pub stage: RwSignal<PostLikeStage>,
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

pub fn use_post_like() -> PostLike {
    let api = ApiWeb::new();

    let stage = RwSignal::new(PostLikeStage::Loading);

    // let s = Memo::new(move |_| );
    // let s2 = LocalResource

    Effect::new(move || {
        api.check_post_like(post_id)
    });

    let on_like = move |post_id: String| {
        api.add_post_like(post_id).send_web(async move |result| {
            let err = match result {
                Ok(ServerRes::Ok) => Ok(()),
                Ok(res) => Err(format!("error, expected OK, received: {res:?}")),
                Err(err) => Err(err.to_string()),
            };

            if let Err(err) = err {
                error!(err);
                stage.set(PostLikeStage::Unliked);
            } else {
                stage.set(PostLikeStage::Liked);
            }
        });
    };

    PostLike {
        stage,
        on_like: StoredValue::new(Box::new(on_like)),
    }
}
