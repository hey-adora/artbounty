use crate::{
    api::{Api, ApiWeb},
    path::{PATH_LOGIN, PATH_UPLOAD, link_settings, link_user},
    view::{
        app::{
            GlobalState,
            components::gallery::Img,
            hook::{
                use_future::FutureFn,
                use_infinite_scroll_fn::{InfiniteItem, InfiniteScrollFn},
            },
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

impl<ItemData> Clone for InfiniteBasic<ItemData> {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            observer: self.observer.clone(),
        }
    }
}

impl<ItemData> Copy for InfiniteBasic<ItemData> {}

impl<ItemData> InfiniteBasic<ItemData>
where
    ItemData: Clone + std::fmt::Debug + 'static,
{
    pub fn new<F>(callback: F) -> InfiniteBasic<ItemData>
    where
        ItemData: Sync + Send + 'static,
        F: AsyncFn(&mut Vec<ItemData>, Option<InfiniteItem>) + Clone + 'static,
    {
        let scroll_items = RwSignal::new_local(Vec::<ItemData>::new());

        let fut = FutureFn::new(move |a| {
            trace!("infinite basic callback");
            let callback = callback.clone();
            async move {
                let mut v = scroll_items.write();
                callback(&mut *v, a).await;
            }
        });

        let infinite_fn = InfiniteScrollFn::new(move |a| {
            fut.run(a);
        });

        let observe = move |target: Element| {
            trace!("infinite basic observe");
            infinite_fn.observe_only(target);
        };

        InfiniteBasic {
            items: scroll_items,
            observer: StoredValue::new(Box::new(observe)),
        }
    }

    pub fn observe_only<Elm>(&self, elm: Elm)
    where
        Elm: JsCast + Clone + 'static + Into<Element>,
    {
        let elm: Element = elm.into();
        self.observer.run(elm);
    }
}
