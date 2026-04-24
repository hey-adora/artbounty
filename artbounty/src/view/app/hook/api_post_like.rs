use leptos::prelude::*;

use crate::{
    api::{Api, Server404Err, ServerErr},
    path::link_img,
};
use tracing::{error, info, trace, warn};

#[derive(Clone, Copy)]
pub struct ApiPostLike<API: Api> {
    // ui
    // pub items: RwSignal<Vec<Img>, LocalStorage>,
    // pub imgs_links: RwSignal<Vec<(String, f64)>, LocalStorage>,
    // pub title: RwSignal<String, LocalStorage>,
    // pub author: RwSignal<String, LocalStorage>,
    // pub description: RwSignal<String, LocalStorage>,
    // pub favorites: RwSignal<u64, LocalStorage>,
    // pub not_found: RwSignal<bool, LocalStorage>,

    pub api: API,
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

impl<API: Api> ApiPostLike<API> {
    pub async fn check() {
        // api.check_post_like(post_id)
        //     .send_web(async move |result| match result {
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
        //     });
    }
}
