use crate::view::{app::hook::use_mutation::Mutation, toolbox::prelude::*};
use leptos::{html::ElementType, prelude::*};
use tracing::trace;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

pub fn use_text_counter<E>(target: NodeRef<E>, count: RwSignal<usize, LocalStorage>)
where
    E: ElementType,
    E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
{
    let mutation = Mutation::new(move |a, b| {
        let Some(target) = target.get_untracked().map(|v| Into::<HtmlElement>::into(v)) else {
            return;
        };

        let length = target.text_content().map(|v| v.len()).unwrap_or_default();
        let id = target.id();

        let label = format!("use_text_counter_{id}");
        // let debug_data = anchor
        //     .as_ref()
        //     .map(|v| serde_json::to_string(&v.1).unwrap_or_else(|e| e.to_string()))
        //     .unwrap_or_else(|| String::from("null"));
        debug_data_push(&label, length.to_string());
        trace!("{label} {length}");

        count.set(length);

        // .map(|v| v.len() as i32)
        // .map(|v| v.text_content().map(|v|v.len() as i32).unwrap_or(-2))
        // // .map(|v| v.len() as i32)
        // .unwrap_or(-1);

        // let description_len = a
        //     .first()
        //     .and_then(|v| v.target())
        //     .map(|v| JsValue::from(v))
        //     .and_then(|v| TryInto::<HtmlElement>::try_into(v).ok())
        //     .and_then(|v| v.text_content())
        //     .map(|v| v.len())
        //     .unwrap_or_default();
    });
    Effect::new(move || {
        let Some(target) = target.get().map(|v| Into::<HtmlElement>::into(v)) else {
            return;
        };
        mutation.observe_only(
            target,
            MutationObserverOptions::new()
                .character_data()
                .set_child_list()
                .subtree(),
        );
    });
}
