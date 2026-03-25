use crate::view::toolbox::intersection_observer::new_raw;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Element, HtmlElement, IntersectionObserver, IntersectionObserverEntry, MutationObserver, MutationRecord
};

#[derive(Clone, Copy)]
pub struct Intersection<F>
where
    F: FnMut(Vec<IntersectionObserverEntry>, IntersectionObserver) + Clone + Send + Sync + 'static,
{
    pub observer: StoredValue<Option<IntersectionObserver>, LocalStorage>,
    pub callback: StoredValue<F, LocalStorage>,
}

impl<F> Intersection<F>
where
    F: FnMut(Vec<IntersectionObserverEntry>, IntersectionObserver) + Clone + Send + Sync + 'static,
{
    pub fn new(callback: F) -> Self {
        let observer = StoredValue::new_local(None::<IntersectionObserver>);
        let callback = StoredValue::new_local(callback);
        let intersection = Self { observer, callback };

        on_cleanup(move || {
            let Some(raw_observer) = intersection.observer.get_value() else {
                return;
            };

            raw_observer.disconnect();
        });

        intersection
    }

    pub fn get(&self) -> IntersectionObserver {
        let observer = self.observer;
        let callback = self.callback;
        observer.get_value().unwrap_or_else(|| {
            let inner_observer = new_raw(callback.get_value());
            observer.set_value(Some(inner_observer.clone()));
            inner_observer
        })
    }

    pub fn observe<E>(&self, target: E)
    where
        E: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        let observer = self.get();
        let target = Into::<HtmlElement>::into(target);
        observer.observe(&target);
    }

    pub fn observe_only<E>(&self, target: E)
    where
        E: JsCast + Clone + 'static + Into<Element>,
    {
        let observer = self.get();
        let target = Into::<Element>::into(target);
        observer.disconnect();
        observer.observe(&target);
    }

    pub fn disconnect(&self) {
        let observer = self.get();
        observer.disconnect();
    }
}
