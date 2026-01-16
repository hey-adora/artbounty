use leptos::prelude::{RwSignal, StoredValue};
use web_sys::MouseEvent;

use crate::api::ApiWeb;
use crate::get_timestamp;
use crate::view::app::GlobalState;
use crate::view::toolbox::prelude::*;

#[derive(Clone, Copy)]
pub struct PostLike {
    // pub err_general: RwQuery<String>,
    // pub email: RwQuery<String>,
    // pub form_stage: RwQuery<ChangePasswordFormStage>,
    // pub btn_stage: StoredValue<Box<dyn Fn() -> ChangePasswordBtnStage + Sync + Send + 'static>>,
    pub on_like: StoredValue<Box<dyn Fn(MouseEvent) + Sync + Send + 'static>>,
    // pub token: RwQuery<String>,
}

pub fn use_post_like() -> PostLike {
    let api = ApiWeb::new();
    let on_like = move |_: MouseEvent| {
        // api.li;
    };

    PostLike {
        on_like: StoredValue::new(Box::new(on_like)),
    }
}
