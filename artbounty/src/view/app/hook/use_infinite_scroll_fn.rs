use crate::{
    api::{Api, ApiWeb},
    path::{PATH_LOGIN, PATH_UPLOAD, link_settings, link_user},
    view::{
        app::{
            GlobalState,
            components::gallery::Img,
            hook::{use_intersection::Intersection, use_mutation::Mutation},
        },
        toolbox::prelude::*,
    },
};
use leptos::{
    attr::{Attribute, AttributeKey},
    html::{self, ElementType},
    prelude::*,
    svg::View,
    tachys::{
        html::node_ref::{NodeRefAttr, NodeRefContainer, node_ref},
        view::any_view::{AnyViewState, AnyViewWithAttrs},
    },
    task::spawn_local,
};
use tracing::{error, trace};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    Element, HtmlElement, IntersectionObserver, IntersectionObserverInit, MutationObserver,
};

pub struct InfiniteItem {
    pub total_items_count: usize,
    pub triggered_index: usize,
    pub elm: HtmlElement,
}

#[derive(Clone, Copy)]
pub struct InfiniteScrollFn {
    pub on: StoredValue<Box<dyn Fn(Element) + Sync + Send + 'static>>,
}

impl InfiniteScrollFn {
    pub fn new<FnGetData>(f: FnGetData) -> Self
    where
        FnGetData: Fn(Option<InfiniteItem>) -> () + Clone + 'static,
    {
        let activated_btm = StoredValue::new(false);
        let intersection = Intersection::new(move |entry, b| {
            let Some(entry) = entry.first() else {
                return;
            };

            let is_intersecting = entry.is_intersecting();

            if !is_intersecting {
                activated_btm.set_value(true);
                return;
            }

            if !activated_btm.get_value() {
                return;
            }

            activated_btm.set_value(false);
            trace!("yo wtf is going on");

            f(None);
        });

        let mutation = Mutation::new(move |entry, b| {
            trace!("infinite scroll fn running MUTATION 0");
            let Some(last) = entry
                .first()
                .and_then(|v| v.target())
                .map(|v| Into::<JsValue>::into(v))
                .map(|v| Into::<Element>::into(v))
                .and_then(|v| v.last_element_child())
            else {
                trace!("running mutation bounced");
                return;
            };
            trace!("infinite scroll fn running MUTATION 1");
            intersection.observe_only(last);
        });

        let on = move |target: Element| {
            let html = target.outer_html();
            trace!("infinite scroll fn running ON {html}");
            mutation.observe_only(
                target,
                MutationObserverOptions::new().set_child_list(),
                // .subtree()
                // .character_data()
            );
        };

        InfiniteScrollFn {
            on: StoredValue::new(Box::new(on)),
        }
    }

    pub fn observe_only<Elm>(&self, target: Elm)
    where
        Elm: JsCast + Clone + 'static + Into<Element>,
    {
        let target = Into::<Element>::into(target);
        (self.on.to_fn())(target);
    }
}
