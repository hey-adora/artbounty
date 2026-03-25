use crate::view::{app::hook::use_infinite_scroll_basic::InfiniteBasic, toolbox::mutation_observer::new_raw};
use leptos::prelude::*;
use tracing::{error, trace};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, MutationObserver, MutationRecord};

pub fn use_post_comments_baisc() {
    let infinite_basic = InfiniteBasic::new(move |v| async move {
        //


        

    });

}
