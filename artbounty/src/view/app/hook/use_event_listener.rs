use std::fmt::Debug;
use leptos::{ev::EventDescriptor, prelude::*};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Element, HtmlElement, MutationObserver, MutationRecord};

#[derive(Clone, Copy)]
pub struct EventListener<T, F>
where
    T: EventDescriptor + Debug + 'static,
    F: FnMut(<T as EventDescriptor>::EventType) + Clone + 'static,
{
    pub event: StoredValue<T, LocalStorage>,
    pub callback: StoredValue<F, LocalStorage>,
}

impl<T, F> EventListener<T, F>
where
    T: EventDescriptor + Debug + 'static,
    F: FnMut(<T as EventDescriptor>::EventType) + Clone + 'static,
{
    pub fn new(event: T, f: F) -> Self {
        Self {
            event: StoredValue::new_local(event),
            callback: StoredValue::new_local(f),
        }
    }

    pub fn add<E>(&self, element: E)
    where
        E: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        let node: HtmlElement = element.into();
        let event_name = self.event.get_value().name();
        let closure = Closure::<dyn FnMut(_)>::new(self.callback.get_value()).into_js_value();

        node.add_event_listener_with_callback(&event_name, closure.as_ref().unchecked_ref())
            .unwrap();
    }
}
