use crate::{
    api::{Api, ApiWeb},
    path::{PATH_LOGIN, PATH_UPLOAD, link_settings, link_user},
    view::{
        app::{
            GlobalState,
            components::gallery::Img,
            hook::{use_future::FutureFn, use_infinite_scroll_fn::{InfiniteItem, InfiniteScrollFn}},
        },
        toolbox::{leptos_helpers::FnRunT1, prelude::*},
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

pub struct InfiniteBasic<ItemData> {
    pub items: RwSignal<Vec<ItemData>, LocalStorage>,
    pub observer: StoredValue<Box<dyn Fn(Element) + Sync + Send + 'static>>,
}

impl<ItemData> InfiniteBasic<ItemData>
where
    ItemData: Clone + std::fmt::Debug + 'static,
{
    pub fn new<Fut, F>(callback: F) -> InfiniteBasic<ItemData>
    where
        ItemData: Sync + Send + 'static,
        Fut: Future<Output = Vec<ItemData>> + Sync + Send + 'static,
        F: Fn(&mut Vec<ItemData>, Option<InfiniteItem>) -> Fut + Clone + Sync + Send + 'static,
    {
        let scroll_items = RwSignal::new_local(Vec::<ItemData>::new());

        let fut = FutureFn::new(move |a| {
            let callback = callback.clone();
            async move {
                // scroll_items.update(|v| {
                // });

                let mut v = scroll_items.write();
                callback(&mut *v, a).await;

                // v.tra
                // let last = scroll_items.with_untracked(|v| v.last().cloned());
                // let new_items = callback(last).await;
                // if new_items.is_empty() {
                //     return;
                // }
                // scroll_items.update(|v| {
                //     v.extend(new_items);
                // });
            }
        });

        let infinite_fn = InfiniteScrollFn::new(move |a| {
            fut.run(a);
        });

        let observe = move |target: Element| {
            infinite_fn.observe_only(target);
        };

        InfiniteBasic {
            items: scroll_items,
            observer: StoredValue::new(Box::new(observe)),
        }
    }

    pub fn observe<Elm>(&self, elm: Elm)
    where
        Elm: JsCast + Clone + 'static + Into<Element>,
    {
        let elm: Element = elm.into();
        self.observer.run(elm);
    }
}
// pub fn use_infinite_scroll_basic<Elm, Fut, ItemData, FnGetData>(
//     container_ref: NodeRef<Elm>,
//     callback: FnGetData,
// ) where
//     Elm: ElementType,
//     Elm::Output: JsCast + Clone + 'static + Into<HtmlElement>,
//     ItemData: Clone + std::fmt::Debug + 'static,
//     Fut: Future<Output = Vec<ItemData>> + 'static,
//     FnGetData: Fn(Option<ItemData>) -> Fut + Clone + Sync + Send + 'static,
// {
// }
