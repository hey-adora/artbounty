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
    // pub observer: StoredValue<Option<MutationObserver>, LocalStorage>,
    // pub callback: StoredValue<F, LocalStorage>,
    pub on: StoredValue<Box<dyn Fn(Element) + Sync + Send + 'static>>,
}

impl InfiniteScrollFn {
    pub fn new<FnGetData>(f: FnGetData) -> Self
    where
        // Elm: ElementType,
        // Elm::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        FnGetData: Fn(Option<InfiniteItem>) -> () + Clone + Sync + Send + 'static,
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
            // .and_then(|v| v.last_child())
            // .inspect(|v| trace!("infinite scroll fn {v:?}"))
            // .and_then(|v| {
            //     if v.is_instance_of::<Element>() {
            //         Some(v)
            //     } else {
            //         None
            //     }
            // })
            // .and_then(|v| v.v.elements().first().cloned())
            // .and_then(|v| v.last_element_child())
            else {
                trace!("running mutation bounced");
                return;
            };
            trace!("infinite scroll fn running MUTATION 1");
            intersection.observe_only(last);
        });

        let on = move |target: Element| {
            trace!("infinite scroll fn running ON");
            mutation.observe_only(target, MutationObserverOptions::new().set_child_list());
        };

        InfiniteScrollFn {
            on: StoredValue::new(Box::new(on)),
        }
        // Effect::new(move || {
        //     // NO
        //     container_ref.get_untracked()
        //     mutation.observe(target);
        // });

        // on_cleanup(move || {
        //     if let Some(observer) = observer_mutation.get_value() {
        //         observer.disconnect();
        //     };
        //     if let Some(observer) = observer_intersection_bottom.get_value() {
        //         observer.disconnect();
        //     };
        // });
        //
        // Effect::new(move || {
        //     let intersection_observer_options = IntersectionObserverInit::new();
        //     intersection_observer_options.set_threshold(&JsValue::from_f64(0.0));
        //
        //     let new_interception_observer_btm = intersection_observer::new_with_options_raw(
        //         {
        //             let f = f.clone();
        //             move |entry, _observer| {
        //                 let Some(entry) = entry.first() else {
        //                     return;
        //                 };
        //
        //                 let is_intersecting = entry.is_intersecting();
        //
        //                 if !is_intersecting {
        //                     activated_btm.set_value(true);
        //                     return;
        //                 }
        //
        //                 if !activated_btm.get_value() {
        //                     return;
        //                 }
        //
        //                 activated_btm.set_value(false);
        //                 trace!("yo wtf is going on");
        //
        //                 f(None);
        //             }
        //         },
        //         &intersection_observer_options,
        //     );
        //     observer_intersection_bottom.set_value(Some(new_interception_observer_btm.clone()));
        //
        //     let new_mutation_observer = mutation_observer::new_raw(move |a, b| {
        //         let Some(infinite_scroll_elm) = container_ref
        //             .get_untracked()
        //             .map(|v| Into::<HtmlElement>::into(v))
        //             .and_then(|v| v.last_element_child())
        //         else {
        //             trace!("running mutation bounced");
        //             return;
        //         };
        //         new_interception_observer_btm.disconnect();
        //         new_interception_observer_btm.observe(&infinite_scroll_elm);
        //     });
        //
        //     observer_mutation.set_value(Some(new_mutation_observer));
        //     //
        // });
    }

    pub fn observe_only<Elm>(&self, target: Elm)
    where
        Elm: JsCast + Clone + 'static + Into<Element>,
    {
        let target = Into::<Element>::into(target);
        (self.on.to_fn())(target);
    }
}
