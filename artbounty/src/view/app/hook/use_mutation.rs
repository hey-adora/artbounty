use crate::view::toolbox::mutation_observer::new_raw;
use leptos::prelude::*;
use tracing::{error, trace};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, MutationObserver, MutationRecord};

#[derive(Clone, Copy)]
pub struct Mutation<F>
where
    F: FnMut(Vec<MutationRecord>, web_sys::MutationObserver) + Clone + 'static,
{
    pub observer: StoredValue<Option<MutationObserver>, LocalStorage>,
    pub callback: StoredValue<F, LocalStorage>,
}

impl<F> Mutation<F>
where
    F: FnMut(Vec<MutationRecord>, web_sys::MutationObserver) + Clone + 'static,
{
    pub fn new(callback: F) -> Self {
        let observer = StoredValue::new_local(None::<web_sys::MutationObserver>);
        let callback = StoredValue::new_local(callback);
        let mutation = Self { observer, callback };

        on_cleanup(move || {
            let Some(raw_observer) = mutation.observer.get_value() else {
                return;
            };

            raw_observer.disconnect();
        });

        mutation
    }

    pub fn get(&self) -> MutationObserver {
        let observer = self.observer;
        let callback = self.callback;
        observer.get_value().unwrap_or_else(|| {
            trace!("mutation observer created");
            let inner_observer = new_raw(callback.get_value());
            observer.set_value(Some(inner_observer.clone()));
            inner_observer
        })
    }

    pub fn observe<E, O>(&self, target: E, options: O)
    where
        E: JsCast + Clone + 'static + Into<Element>,
        O: Into<web_sys::MutationObserverInit> + Clone + 'static,
    {
        let observer = self.get();
        let target = Into::<Element>::into(target);
        let options = options.clone().into();
        let _ = observer
            .observe_with_options(&target, &options)
            .inspect_err(|err| error!("error observing: {err:?}"));

    }

    pub fn observe_only<E, O>(&self, target: E, options: O)
    where
        E: JsCast + Clone + 'static + Into<Element>,
        O: Into<web_sys::MutationObserverInit> + Clone + 'static,
    {
        let observer = self.get();
        let target = Into::<Element>::into(target);
        let options = options.clone().into();
        observer.disconnect();
        let _ = observer
            .observe_with_options(&target, &options)
            .inspect_err(|_| error!("error observing: {target:?}"));
    }

    pub fn disconnect(&self) {
        let observer = self.get();
        observer.disconnect();
    }
}
