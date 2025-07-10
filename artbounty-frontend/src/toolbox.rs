pub mod prelude {
    pub use super::api::Grounder;
    pub use super::dropzone::{self, AddDropZone};
    pub use super::event_listener::{self, AddEventListener};
    pub use super::file::{self, GetFileStream, GetFiles, GetStreamChunk, PushChunkToVec};
    pub use super::intersection_observer::{self, AddIntersectionObserver, IntersectionOptions};
    pub use super::interval::{self};
    pub use super::random::{random_u8, random_u32, random_u32_ranged, random_u64};
    pub use super::resize_observer::{self, AddResizeObserver, GetContentBoxSize};
}

pub mod api {
    use std::marker::PhantomData;

    use leptos::{
        prelude::{ArcRwSignal, Read, Set},
        task::spawn_local,
    };

    pub trait Grounder<Func, FuncFuture, DTO, ApiValue, ApiErr>
    where
        Func: Fn(DTO) -> FuncFuture,
        FuncFuture: Future<Output = Result<ApiValue, ApiErr>> + 'static,
        ApiValue: Clone + 'static,
        ApiErr: Clone + 'static,
    {
        fn ground(self) -> Api<Func, FuncFuture, DTO, ApiValue, ApiErr>;
    }

    pub trait Caller<T> {
        fn call(self) -> T;
    }

    impl<T, F: Fn() -> T> Caller<T> for F {
        fn call(self) -> T {
            (self)()
        }
    }

    // pub trait Grounder {
    //     fn ground<Func, FuncFuture, ApiValue, ApiErr>() -> Api<Func, FuncFuture, ApiValue, ApiErr>
    //     where
    //         Func: Fn() -> FuncFuture,
    //         FuncFuture: Future<Output = Result<ApiValue, ApiErr>> + 'static,
    //         ApiValue: Clone + 'static,
    //         ApiErr: Clone + 'static;
    // }

    impl<Func, FuncFuture, DTO, ApiValue, ApiErr> Grounder<Func, FuncFuture, DTO, ApiValue, ApiErr>
        for Func
    where
        Func: Fn(DTO) -> FuncFuture,
        FuncFuture: Future<Output = Result<ApiValue, ApiErr>> + 'static,
        ApiValue: Clone + 'static,
        ApiErr: Clone + 'static,
    {
        fn ground(self) -> Api<Func, FuncFuture, DTO, ApiValue, ApiErr> {
            ground(self)
        }
    }

    #[derive(Clone)]
    pub struct Api<Func, FuncFuture, DTO, ApiValue, ApiErr>
    where
        Func: Fn(DTO) -> FuncFuture,
        FuncFuture: Future<Output = Result<ApiValue, ApiErr>> + 'static,
        ApiValue: Clone + 'static,
        ApiErr: Clone + 'static,
    {
        pub fut: Func,
        pub value: ArcRwSignal<Option<Result<ApiValue, ApiErr>>>,
        pub _phantom: PhantomData<DTO>,
    }

    impl<Func, FuncFuture, DTO, ApiValue, ApiErr> Api<Func, FuncFuture, DTO, ApiValue, ApiErr>
    where
        Func: Fn(DTO) -> FuncFuture,
        FuncFuture: Future<Output = Result<ApiValue, ApiErr>> + 'static,
        ApiValue: Clone + 'static,
        ApiErr: Clone + 'static,
    {
        pub fn dispatch(&self, dto: DTO) {
            let fut = (self.fut)(dto);
            let value = self.value.clone();
            spawn_local(async move {
                let result = fut.await;
                value.set(Some(result));
            });
        }

        pub fn value(&self) -> Option<Result<ApiValue, ApiErr>> {
            let v = self.value.read_only().read().clone();
            v
        }
    }

    pub fn ground<Func, FuncFuture, DTO, ApiValue, ApiErr>(
        fut: Func,
    ) -> Api<Func, FuncFuture, DTO, ApiValue, ApiErr>
    where
        Func: Fn(DTO) -> FuncFuture,
        FuncFuture: Future<Output = Result<ApiValue, ApiErr>> + 'static,
        ApiValue: Clone + 'static,
        ApiErr: Clone + 'static,
    {
        let api = Api::<Func, FuncFuture, DTO, ApiValue, ApiErr> {
            fut,
            value: ArcRwSignal::new(None),
            _phantom: PhantomData,
        };
        api
        // let result = ArcRwSignal::new();
        // s
    }
}

pub mod random {
    use web_sys::js_sys::Math::random;

    pub fn random_u8() -> u8 {
        (random().to_bits() % 255) as u8
    }

    pub fn random_u64() -> u64 {
        random().to_bits()
    }

    pub fn random_u32() -> u32 {
        random_u64() as u32
    }

    pub fn random_u32_ranged(min: u32, max: u32) -> u32 {
        (random_u32() + min) % max
    }
}

pub mod uuid {
    use std::str::FromStr;

    use tracing::error;
    use uuid::Uuid;
    
    use web_sys::Element;

    fn debug(log: &str, elm: &Element) {
        // let node = elm.node_type();
        // trace!(node);
        // unsafe {
        //     let p = &raw const elm as *const [usize; 64];
        //     let r = *p;
        //     // let p = ptr::read(r[0]);
        //     let p = r[0] as *const [usize; 64];
        //     let r = *p;
        //     trace!("{log} ID: {r:x?}");
        // }
    }

    pub fn get_id(target: &Element, field_name: &str) -> Option<Uuid> {
        debug("GET", target);

        let Some(id) = target.get_attribute(field_name) else {
            error!(
                "{} was not set {:?}",
                field_name,
                target.to_string().as_string()
            );
            return None;
        };
        let id = match Uuid::from_str(&id) {
            Ok(id) => id,
            Err(err) => {
                error!(
                    "{} is invalid {:?}",
                    field_name,
                    target.to_string().as_string()
                );
                return None;
            }
        };

        Some(id)
    }

    pub fn set_id(target: &Element, field_name: &str, id: Uuid) {
        debug("SET", target);

        target.set_attribute(field_name, &id.to_string()).unwrap();
    }
}

pub mod interval {
    use std::time::Duration;

    use leptos::prelude::{Effect, GetValue, SetValue, StoredValue, on_cleanup};
    use thiserror::Error;
    use tracing::{error, trace};
    use wasm_bindgen::{JsCast, prelude::Closure};
    use web_sys::window;

    #[derive(Debug, Clone, Copy)]
    pub struct IntervalHandle(StoredValue<Option<i32>>);

    #[derive(Debug, Error, Clone)]
    pub enum ErrorIntervalClear {
        #[error("failed to get Window object")]
        GettingWindow,
    }

    #[derive(Debug, Error, Clone)]
    pub enum ErrorSetInterval {
        #[error("failed to get Window object")]
        GettingWindow,

        #[error("failed to set interval \"{0}\"")]
        SettingInterval(String),
    }

    impl IntervalHandle {
        pub fn new() -> Self {
            Self(StoredValue::new(None))
        }

        pub fn clear(self) -> Result<bool, ErrorIntervalClear> {
            let Some(handle) = self.0.get_value() else {
                return Ok(false);
            };
            window()
                .ok_or(ErrorIntervalClear::GettingWindow)?
                .clear_interval_with_handle(handle);
            Ok(true)
        }

        pub fn set(&self, handle: i32) {
            self.0.set_value(Some(handle));
        }

        pub fn unset(&self) {
            self.0.set_value(None);
        }
    }

    #[track_caller]
    pub fn new<F>(callback: F, duration: Duration) -> Result<IntervalHandle, ErrorSetInterval>
    where
        F: Fn() + Clone + 'static,
    {
        let handle = IntervalHandle::new();
        let caller_location = std::panic::Location::caller();

        Effect::new(move || {
            let window = window().ok_or(ErrorSetInterval::GettingWindow);
            let window = match window {
                Ok(v) => v,
                Err(err) => {
                    error!("failed to set interval at {} : {}", caller_location, err);
                    return;
                }
            };
            let closure = Closure::<dyn Fn()>::new(callback.clone()).into_js_value();
            let ms = duration.as_millis() as i32;
            let handle_id = window
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    ms,
                )
                .map_err(|e| {
                    ErrorSetInterval::SettingInterval(
                        e.as_string()
                            .unwrap_or_else(|| String::from("uwknown error")),
                    )
                });
            let handle_id = match handle_id {
                Ok(v) => v,
                Err(err) => {
                    error!("failed to set interval at {} : {}", caller_location, err);
                    return;
                }
            };

            handle.set(handle_id);
        });

        on_cleanup(move || {
            let result = handle.clear();
            let result = match result {
                Ok(v) => v,
                Err(err) => {
                    error!("failed to clear interval at {} : {}", caller_location, err);
                    return;
                }
            };
            if result {
                trace!("interval cleared");
            } else {
                trace!("no interval set");
            }
        });

        Ok(handle)
    }
}

pub mod intersection_observer {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::hash::{DefaultHasher, Hash, Hasher};
    
    use std::sync::LazyLock;

    use leptos::html::ElementType;
    use leptos::prelude::*;
    // use leptos::html;
    // use leptos::{html::ElementType, prelude::*};
    use ordered_float::OrderedFloat;
    use send_wrapper::SendWrapper;
    use sha2::Digest;
    use tracing::{trace, trace_span, warn};
    use uuid::Uuid;
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{
        HtmlElement, IntersectionObserver, IntersectionObserverEntry,
        IntersectionObserverInit, js_sys::Array,
    };

    use super::uuid::{get_id, set_id};

    const ID_FIELD_NAME: &str = "data-leptos_toolbox_intersection_observer_id";
    const ID_FIELD_ROOT_NAME: &str = "data-leptos_toolbox_intersection_observer_root_id";
    // static OBSERVERS: LazyLock<RwLock<HashMap<u64, SendWrapper<IntersectionObserver>>>> =
    //     LazyLock::new(|| RwLock::new(HashMap::new()));
    // static CALLBACKS: LazyLock<
    //     RwLock<
    //         HashMap<
    //             Uuid,
    //             Box<
    //                 dyn FnMut(IntersectionObserverEntry, IntersectionObserver)
    //                     + Send
    //                     + Sync
    //                     + 'static,
    //             >,
    //         >,
    //     >,
    // > = LazyLock::new(|| RwLock::new(HashMap::new()));
    thread_local! {
        // #[thread_local]
        // #[thread_local]
        static OBSERVERS: LazyLock<RefCell<HashMap<u64, SendWrapper<IntersectionObserver>>>> =
            LazyLock::new(|| RefCell::new(HashMap::new()));
        // static OBSERVERS: LazyLock<RefCell<HashMap<u64, SendWrapper<IntersectionObserver>>>> =
        //     MutStat;
        static CALLBACKS: LazyLock<
            RefCell<
                HashMap<
                    Uuid,
                    Box<
                        dyn FnMut(IntersectionObserverEntry, IntersectionObserver)
                            + Send
                            + Sync
                            + 'static,
                    >,
                >,
            >,
        > = LazyLock::new(|| RefCell::new(HashMap::new()));
    // static HASHMAP: LazyLock<HashMap<u32, &str>> = LazyLock::new(|| {
    //     let mut m = HashMap::new();
    //     m.insert(0, "foo");
    //     m.insert(1, "bar");
    //     m.insert(2, "baz");
    //     m
    // });
    }

    pub trait AddIntersectionObserver {
        fn add_intersection_observer_with_options<F, R>(
            &self,
            callback: F,
            options: IntersectionOptions<R>,
        ) where
            R: ElementType,
            R::Output: JsCast + Clone + 'static + Into<HtmlElement>,
            F: FnMut(IntersectionObserverEntry, IntersectionObserver)
                + Send
                + Sync
                + Clone
                + 'static;
    }

    impl<E> AddIntersectionObserver for NodeRef<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        fn add_intersection_observer_with_options<F, R>(
            &self,
            callback: F,
            options: IntersectionOptions<R>,
        ) where
            R: ElementType,
            R::Output: JsCast + Clone + 'static + Into<HtmlElement>,
            F: FnMut(IntersectionObserverEntry, IntersectionObserver)
                + Send
                + Sync
                + Clone
                + 'static,
        {
            new(self.clone(), callback, options);
        }
    }

    // #[derive(Default, Clone)]
    // pub struct GlobalState {
    //     pub observer: StoredValue<HashMap<u64, SendWrapper<IntersectionObserver>>>,
    //     pub callbacks: StoredValue<
    //         HashMap<
    //             Uuid,
    //             Box<
    //                 dyn FnMut(IntersectionObserverEntry, IntersectionObserver)
    //                     + Send
    //                     + Sync
    //                     + 'static,
    //             >,
    //         >,
    //     >,
    // }

    #[derive(Clone)]
    pub struct IntersectionOptions<E = leptos::html::Div>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        root: Option<NodeRef<E>>,
        root_margin: Option<String>,
        threshold: Option<OrderedFloat<f64>>,
    }

    impl<E> Default for IntersectionOptions<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        fn default() -> Self {
            Self {
                root: None,
                root_margin: None,
                threshold: None,
            }
        }
    }

    impl<E> IntersectionOptions<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        pub fn set_root(mut self, root: NodeRef<E>) -> Self {
            self.root = Some(root);
            self
        }

        pub fn set_root_margin(mut self, root_margin: String) -> Self {
            self.root_margin = Some(root_margin);
            self
        }

        pub fn set_threshold(mut self, threshold: f64) -> Self {
            self.threshold = Some(OrderedFloat(threshold));
            self
        }
    }

    impl<E> Hash for IntersectionOptions<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.root
                .as_ref()
                .and_then(|v| {
                    let root: HtmlElement = v.get().unwrap().into();
                    root.get_attribute(ID_FIELD_ROOT_NAME)
                })
                .hash(state);
            self.root_margin.hash(state);
            self.threshold.hash(state);
        }
    }

    pub fn new<E, R, F>(target: NodeRef<E>, callback: F, options: IntersectionOptions<R>)
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        R: ElementType,
        R::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        F: FnMut(IntersectionObserverEntry, IntersectionObserver) + Clone + Send + Sync + 'static,
    {
        // let ctx = match use_context::<GlobalState>() {
        //     Some(v) => v,
        //     None => {
        //         provide_context(GlobalState::default());
        //         expect_context::<GlobalState>()
        //     }
        // };
        let id = Uuid::new_v4();
        let options_hash = StoredValue::new(None::<u64>);

        Effect::new(move || {
            let span = trace_span!("intersection observer", "{}", &id).entered();

            let Some(target) = target.get() else {
                return;
            };

            let root: Option<HtmlElement> = match &options.root {
                Some(v) => match v.get() {
                    Some(v) => {
                        let root: HtmlElement = v.into();

                        Some(root)
                    }
                    None => {
                        trace!("root is not ready");
                        return;
                    }
                },
                None => None,
            };

            trace!("root options parsed");

            let mut hasher = DefaultHasher::new();
            options.hash(&mut hasher);
            let hash = hasher.finish();
            options_hash.set_value(Some(hash));
            trace!("hash of options: {}", hash);

            let target: HtmlElement = target.into();

            set_id(&target, ID_FIELD_NAME, id);
            trace!("id set");

            {
                // HASHMAP.get(k);
                CALLBACKS.with(|callbacks| {
                    let mut callbacks = callbacks.borrow_mut();
                    callbacks.insert(id, Box::new(callback.clone()));
                    trace!("created callback");
                });
            }
            // LazyLock::force(&CALLBACKS).borrow_mut()(|v| {
            //     v.insert(id, Box::new(callback.clone()));
            //     trace!("created callback");
            // });
            trace!("callback set");

            {
                // let mut observers = LazyLock::force(&OBSERVERS).borrow_mut();
                OBSERVERS.with(|observers| {
                    let mut observers = observers.borrow_mut();
                    trace!("getting observer...");
                    match observers.get_mut(&hash) {
                        Some(observer) => {
                            observer.observe(&target);
                            trace!("observer already exists");
                        }
                        None => {
                            trace!("no observer found");

                            let observer_settings = IntersectionObserverInit::new();

                            if let Some(root) = root {
                                trace!("root option set");
                                observer_settings.set_root(Some(&root));
                            }

                            if let Some(margin) = &options.root_margin {
                                trace!("margin option set");
                                observer_settings.set_root_margin(margin);
                            }

                            if let Some(threshold) = options.threshold {
                                trace!("threshold option set");
                                observer_settings.set_threshold(&JsValue::from_f64(*threshold));
                            }

                            trace!("creating raw observer");
                            let observer = new_with_options_raw(
                                move |entries, observer| {
                                    CALLBACKS.with(|v| {
                                        let mut callbacks = v.borrow_mut();

                                        for entry in entries {
                                            let target = entry.target();
                                            let Some(id) = get_id(&target, ID_FIELD_NAME) else {
                                                continue;
                                            };

                                            let Some(callback) = callbacks.get_mut(&id) else {
                                                continue;
                                            };
                                            callback(entry, observer.clone());
                                        }
                                    });
                                    // ctx.callbacks.update_value(|callbacks| {
                                    // });
                                },
                                &observer_settings,
                            );

                            trace!("inserting raw observer...");
                            let observer =
                                observers.entry(hash).or_insert(SendWrapper::new(observer));
                            observer.observe(&target);
                            trace!("observer created");
                        }
                    };
                });
            }
            // ctx.observer.update_value(|observers| {
            // });

            span.exit();
        });

        on_cleanup(move || {
            let span = trace_span!("intersection observer").entered();

            let Some(target) = target.get_untracked() else {
                return;
            };

            let Some(options_hash) = options_hash.get_value() else {
                return;
            };

            let target: HtmlElement = target.into();

            let Some(id) = get_id(&target, ID_FIELD_NAME) else {
                return;
            };

            {
                OBSERVERS.with(|observers| {
                    let observers = observers.borrow_mut();
                    match observers.get(&options_hash) {
                        Some(observer) => {
                            observer.unobserve(&target);
                        }
                        None => {
                            warn!("observer not found with hash {} for {}", options_hash, id);
                        }
                    }
                });
            }
            // ctx.observer
            //     .with_value(|observers| match observers.get(&options_hash) {
            //         Some(observer) => {
            //             observer.unobserve(&target);
            //         }
            //         None => {
            //             warn!("observer not found with hash {} for {}", options_hash, id);
            //         }
            //     });

            {
                CALLBACKS.with(|callbacks| {
                    let mut callbacks = callbacks.borrow_mut();
                    callbacks.remove(&id);
                    trace!("removed {}", &id);
                });
            }
            // ctx.callbacks.update_value(|callbacks| {
            //     callbacks.remove(&id);
            //     trace!("removed {}", &id);
            // });

            span.exit();
        });
    }

    pub fn new_closure(
        mut callback: impl FnMut(Vec<IntersectionObserverEntry>, IntersectionObserver) + 'static,
    ) -> JsValue {
        Closure::<dyn FnMut(Array, IntersectionObserver)>::new(
            move |entries: Array, observer: IntersectionObserver| {
                let entries: Vec<IntersectionObserverEntry> = entries
                    .to_vec()
                    .into_iter()
                    .map(|v| v.unchecked_into::<IntersectionObserverEntry>())
                    .collect();
                callback(entries, observer);
            },
        )
        .into_js_value()
    }

    pub fn new_raw<F>(callback: F) -> IntersectionObserver
    where
        F: FnMut(Vec<IntersectionObserverEntry>, IntersectionObserver) + Clone + 'static,
    {
        IntersectionObserver::new(new_closure(callback).as_ref().unchecked_ref()).unwrap()
    }

    pub fn new_with_options_raw<F>(
        callback: F,
        options: &IntersectionObserverInit,
    ) -> IntersectionObserver
    where
        F: FnMut(Vec<IntersectionObserverEntry>, IntersectionObserver) + Clone + 'static,
    {
        IntersectionObserver::new_with_options(
            new_closure(callback).as_ref().unchecked_ref(),
            options,
        )
        .unwrap()
    }
}

pub mod resize_observer {
    use std::collections::HashMap;

    use leptos::{
        html::ElementType,
        prelude::{
            Effect, Get, GetUntracked, GetValue, NodeRef, SetValue, StoredValue, UpdateValue, expect_context, on_cleanup, provide_context,
            use_context,
        },
    };
    use send_wrapper::SendWrapper;
    use tracing::{trace, trace_span};
    use uuid::Uuid;
    use wasm_bindgen::prelude::*;
    use web_sys::{
        self, HtmlElement, ResizeObserver, ResizeObserverEntry, ResizeObserverSize,
        js_sys::Array,
    };

    use super::uuid::{get_id, set_id};

    const ID_FIELD_NAME: &str = "data-leptos_toolbox_resize_observer_id";

    pub trait AddResizeObserver {
        fn add_resize_observer<F>(&self, callback: F)
        where
            F: FnMut(ResizeObserverEntry, ResizeObserver) + Send + Sync + Clone + 'static;
    }

    pub trait GetContentBoxSize {
        fn get_content_box_size(&self) -> Vec<ResizeObserverSize>;
    }

    impl GetContentBoxSize for ResizeObserverEntry {
        fn get_content_box_size(&self) -> Vec<ResizeObserverSize> {
            self.content_box_size()
                .to_vec()
                .into_iter()
                .map(|v| v.unchecked_into::<ResizeObserverSize>())
                .collect()
        }
    }

    impl<E> AddResizeObserver for NodeRef<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        fn add_resize_observer<F>(&self, callback: F)
        where
            F: FnMut(ResizeObserverEntry, ResizeObserver) + Send + Sync + Clone + 'static,
        {
            new(self.clone(), callback);
        }
    }

    #[derive(Default, Clone)]
    pub struct GlobalState {
        pub observer: StoredValue<Option<SendWrapper<ResizeObserver>>>,
        pub callbacks: StoredValue<
            HashMap<
                Uuid,
                Box<dyn FnMut(ResizeObserverEntry, ResizeObserver) + Send + Sync + 'static>,
            >,
        >,
    }

    // impl Default for GlobalState {
    //     fn default() -> Self {
    //         provide_context(GlobalState {
    //             callbacks: StoredValue::new(HashMap::new()),
    //             observer: StoredValue::new(None),
    //         });
    //         let ctx = expect_context::<GlobalState>();

    //         let observer = new_raw(move |entries, observer| {
    //             ctx.callbacks.update_value(|callbacks| {
    //                 for entry in entries {
    //                     let target = entry.target();
    //                     let Some(id) = get_id(&target, ID_FIELD_NAME) else {
    //                         continue;
    //                     };

    //                     let Some(callback) = callbacks.get_mut(&id) else {
    //                         continue;
    //                     };
    //                     callback(entry, observer.clone());
    //                 }
    //             });
    //         });
    //         GlobalState {
    //             callbacks: StoredValue::new(HashMap::new()),
    //             observer: StoredValue::new(Some(SendWrapper::new(observer))),
    //         }
    //     }
    // }

    // pub fn init_global_state() {
    //     provide_context(GlobalState::default());

    //     Effect::new(move || {
    //         let ctx = expect_context::<GlobalState>();

    //         let observer = new_raw(move |entries, observer| {
    //             ctx.callbacks.update_value(|callbacks| {
    //                 for entry in entries {
    //                     let target = entry.target();
    //                     let Some(id) = get_id(&target, ID_FIELD_NAME) else {
    //                         continue;
    //                     };

    //                     let Some(callback) = callbacks.get_mut(&id) else {
    //                         continue;
    //                     };
    //                     callback(entry, observer.clone());
    //                 }
    //             });
    //         });

    //         ctx.observer.set(Some(SendWrapper::new(observer)));
    //     });
    // }

    pub fn new<E, F>(target: NodeRef<E>, callback: F)
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        F: FnMut(ResizeObserverEntry, web_sys::ResizeObserver) + Clone + Send + Sync + 'static,
    {
        let ctx = match use_context::<GlobalState>() {
            Some(v) => v,
            None => {
                provide_context(GlobalState::default());
                expect_context::<GlobalState>()
            }
        };
        // let ctx = expect_context::<GlobalState>();
        let id = Uuid::new_v4();

        Effect::new(move || {
            let span = trace_span!("resize observer").entered();

            let observer = match ctx.observer.get_value() {
                Some(observer) => observer,
                None => {
                    let observer = new_raw(move |entries, observer| {
                        ctx.callbacks.update_value(|callbacks| {
                            for entry in entries {
                                let target = entry.target();
                                let Some(id) = get_id(&target, ID_FIELD_NAME) else {
                                    continue;
                                };

                                let Some(callback) = callbacks.get_mut(&id) else {
                                    continue;
                                };
                                callback(entry, observer.clone());
                            }
                        });
                    });
                    ctx.observer.set_value(Some(SendWrapper::new(observer)));
                    ctx.observer.get_value().unwrap()
                }
            };

            let Some(target) = target.get() else {
                return;
            };

            let target: HtmlElement = target.into();

            set_id(&target, ID_FIELD_NAME, id);

            ctx.callbacks.update_value(|v| {
                v.insert(id, Box::new(callback.clone()));
                trace!("created {}", &id);
            });

            observer.observe(&target);

            span.exit();
        });

        on_cleanup(move || {
            let span = trace_span!("resize observer").entered();

            let (Some(target), Some(observer)) = (target.get_untracked(), ctx.observer.get_value())
            else {
                return;
            };

            let target: HtmlElement = target.into();

            let Some(id) = get_id(&target, ID_FIELD_NAME) else {
                return;
            };

            observer.unobserve(&target);

            ctx.callbacks.update_value(|callbacks| {
                callbacks.remove(&id);
                trace!("removed {}", &id);
            });

            span.exit();
        });
    }

    pub fn new_raw<F>(mut callback: F) -> ResizeObserver
    where
        F: FnMut(Vec<web_sys::ResizeObserverEntry>, web_sys::ResizeObserver) + Clone + 'static,
    {
        let resize_observer_closure = Closure::<dyn FnMut(Array, ResizeObserver)>::new(
            move |entries: Array, observer: ResizeObserver| {
                let entries: Vec<ResizeObserverEntry> = entries
                    .to_vec()
                    .into_iter()
                    .map(|v| v.unchecked_into::<ResizeObserverEntry>())
                    .collect();
                callback(entries, observer);
            },
        )
        .into_js_value();
        ResizeObserver::new(resize_observer_closure.as_ref().unchecked_ref()).unwrap()
    }
}

pub mod event_listener {
    use std::fmt::Debug;

    use leptos::{ev::EventDescriptor, html::ElementType, prelude::*};
    use tracing::{trace, trace_span};
    use wasm_bindgen::prelude::*;
    use web_sys::HtmlElement;

    pub trait AddEventListener {
        fn add_event_listener<T, F>(&self, event: T, callback: F)
        where
            T: EventDescriptor + Debug + 'static,
            F: FnMut(<T as EventDescriptor>::EventType) + Clone + 'static;
    }

    impl<E> AddEventListener for NodeRef<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        fn add_event_listener<T, F>(&self, event: T, callback: F)
        where
            T: EventDescriptor + Debug + 'static,
            F: FnMut(<T as EventDescriptor>::EventType) + Clone + 'static,
        {
            new(self.clone(), event, callback);
        }
    }

    pub fn new<E, T, F>(target: NodeRef<E>, event: T, f: F)
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        T: EventDescriptor + Debug + 'static,
        F: FnMut(<T as EventDescriptor>::EventType) + Clone + 'static,
    {
        Effect::new(move || {
            let span = trace_span!("event_listener").entered();
            let Some(node) = target.get() else {
                trace!("target not found");
                return;
            };

            let node: HtmlElement = node.into();

            let closure = Closure::<dyn FnMut(_)>::new(f.clone()).into_js_value();

            node.add_event_listener_with_callback(&event.name(), closure.as_ref().unchecked_ref())
                .unwrap();

            span.exit();
        });
    }
}

pub mod file {
    use thiserror::Error;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{
        DragEvent, File, ReadableStreamDefaultReader,
        js_sys::{Object, Reflect, Uint8Array},
    };

    #[derive(Error, Debug, Clone)]
    pub enum ErrorGetFileStream {
        #[error("failed to cast as \"ReadableStreamDefaultReader\" \"{0}\"")]
        Cast(String),
    }

    #[derive(Error, Debug, Clone)]
    pub enum ErrorGetStreamChunk {
        #[error("failed to get chunk \"{0}\"")]
        GetChunk(String),

        #[error("failed to cast chunk to object \"{0}\"")]
        CastToObject(String),

        #[error("failed to cast chunk to Uint8Array \"{0}\"")]
        CastToArray(String),

        #[error("failed to read 'done' field from chunk object \"{0}\"")]
        ReadingFieldDone(String),

        #[error("failed to read 'value' field from chunk object \"{0}\"")]
        ReadingFieldValue(String),
    }

    pub trait PushChunkToVec {
        fn push_to_vec(&self, buffer: &mut Vec<u8>);
    }

    pub trait GetFiles {
        fn get_files(&self) -> Vec<File>;
    }

    pub trait GetFileStream {
        fn get_file_stream(&self) -> Result<ReadableStreamDefaultReader, ErrorGetFileStream>;
    }

    pub trait GetStreamChunk {
        async fn get_stream_chunk(&self) -> Result<Option<Uint8Array>, ErrorGetStreamChunk>;
    }

    impl PushChunkToVec for Uint8Array {
        fn push_to_vec(&self, buffer: &mut Vec<u8>) {
            let chunk = self;
            let data_len = buffer.len();
            buffer.resize(data_len + chunk.length() as usize, 0);
            chunk.copy_to(&mut buffer[data_len..]);
        }
    }

    impl GetStreamChunk for ReadableStreamDefaultReader {
        async fn get_stream_chunk(&self) -> Result<Option<Uint8Array>, ErrorGetStreamChunk> {
            get_stream_chunk(self).await
        }
    }

    impl GetFileStream for File {
        fn get_file_stream(&self) -> Result<ReadableStreamDefaultReader, ErrorGetFileStream> {
            get_file_stream(self)
        }
    }

    impl GetFiles for DragEvent {
        fn get_files(&self) -> Vec<File> {
            get_files(self)
        }
    }

    pub fn get_files(drag_event: &DragEvent) -> Vec<File> {
        let Some(files) = drag_event.data_transfer().and_then(|v| v.files()) else {
            return Vec::new();
        };
        let files = (0..files.length())
            .filter_map(|i| files.get(i))
            .collect::<Vec<File>>();
        files
    }

    pub fn get_file_stream(file: &File) -> Result<ReadableStreamDefaultReader, ErrorGetFileStream> {
        let stream = file.stream();
        let reader = stream
            .get_reader()
            .dyn_into::<ReadableStreamDefaultReader>()
            .map_err(|e| {
                ErrorGetFileStream::Cast(
                    e.as_string()
                        .unwrap_or_else(|| String::from("uwknown error")),
                )
            })?;
        Ok(reader)
    }

    pub async fn get_stream_chunk(
        reader: &ReadableStreamDefaultReader,
    ) -> Result<Option<Uint8Array>, ErrorGetStreamChunk> {
        let promise = reader.read();
        let chunk = JsFuture::from(promise)
            .await
            .map_err(|e| {
                ErrorGetStreamChunk::GetChunk(
                    e.as_string()
                        .unwrap_or_else(|| String::from("uwknown error")),
                )
            })?
            .dyn_into::<Object>()
            .map_err(|e| {
                ErrorGetStreamChunk::CastToObject(
                    e.as_string()
                        .unwrap_or_else(|| String::from("uwknown error")),
                )
            })?;
        let done = Reflect::get(&chunk, &"done".into()).map_err(|e| {
            ErrorGetStreamChunk::ReadingFieldDone(
                e.as_string()
                    .unwrap_or_else(|| String::from("uwknown error")),
            )
        })?;
        if done.is_truthy() {
            return Ok(None);
        }
        let chunk = Reflect::get(&chunk, &"value".into())
            .map_err(|e| {
                ErrorGetStreamChunk::ReadingFieldValue(
                    e.as_string()
                        .unwrap_or_else(|| String::from("uwknown error")),
                )
            })?
            .dyn_into::<Uint8Array>()
            .map_err(|e| {
                ErrorGetStreamChunk::CastToArray(
                    e.as_string()
                        .unwrap_or_else(|| String::from("uwknown error")),
                )
            })?;

        Ok(Some(chunk))
    }
}

pub mod dropzone {

    use std::{
        cell::RefCell,
        fmt::Display,
        future::Future,
        rc::Rc,
    };

    use leptos::{ev, html::ElementType, prelude::*, task::spawn_local};
    use tracing::error;
    use wasm_bindgen::prelude::*;
    
    use web_sys::{
        DragEvent, HtmlElement,
    };

    use super::event_listener;
    

    pub enum Event {
        Start,
        Enter,
        Over,
        Drop,
        Leave,
    }

    impl Display for Event {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let name = match self {
                Event::Start => "start",
                Event::Enter => "enter",
                Event::Over => "over",
                Event::Drop => "drop",
                Event::Leave => "leave",
            };
            write!(f, "{}", name)
        }
    }

    pub trait AddDropZone {
        fn on_file_drop<F, R>(&self, callback: F)
        where
            R: Future<Output = anyhow::Result<()>> + 'static,
            F: FnMut(Event, DragEvent) -> R + 'static;
    }

    impl<E> AddDropZone for NodeRef<E>
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    {
        #[track_caller]
        fn on_file_drop<F, R>(&self, callback: F)
        where
            R: Future<Output = anyhow::Result<()>> + 'static,
            F: FnMut(Event, DragEvent) -> R + 'static,
        {
            new(self.clone(), callback);
        }
    }

    #[track_caller]
    pub fn new<E, F, R>(target: NodeRef<E>, callback: F)
    where
        E: ElementType,
        E::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        R: Future<Output = anyhow::Result<()>> + 'static,
        F: FnMut(Event, DragEvent) -> R + 'static,
    {
        let callback_location = *std::panic::Location::caller();
        let callback = Rc::new(RefCell::new(callback));

        event_listener::new(target, ev::dragstart, {
            let callback = callback.clone();
            move |e| {
                let callback = callback.clone();
                let fut = async move {
                    let mut callback = callback.borrow_mut();
                    let result = callback(Event::Start, e).await;

                    if let Err(err) = result {
                        error!("dropzone error at: {}: {}", callback_location, err);
                    }
                };
                spawn_local(fut);
            }
        });

        event_listener::new(target, ev::dragleave, {
            let callback = callback.clone();

            move |e| {
                let callback = callback.clone();
                let fut = async move {
                    let mut callback = callback.borrow_mut();
                    let result = callback(Event::Leave, e).await;
                    if let Err(err) = result {
                        error!("dropzone error at: {}: {}", callback_location, err);
                    }
                };
                spawn_local(fut);
            }
        });

        event_listener::new(target, ev::dragenter, {
            let callback = callback.clone();

            move |e| {
                let callback = callback.clone();
                let fut = async move {
                    let mut callback = callback.borrow_mut();
                    let result = callback(Event::Enter, e).await;
                    if let Err(err) = result {
                        error!("dropzone error at: {}: {}", callback_location, err);
                    }
                };
                spawn_local(fut);
            }
        });

        event_listener::new(target, ev::dragover, {
            let callback = callback.clone();

            move |e| {
                e.prevent_default();

                let callback = callback.clone();
                let fut = async move {
                    let mut callback = callback.borrow_mut();
                    let result = callback(Event::Over, e).await;
                    if let Err(err) = result {
                        error!("dropzone error at: {}: {}", callback_location, err);
                    }
                };
                spawn_local(fut);
            }
        });

        event_listener::new(target, ev::drop, {
            let callback = callback.clone();

            move |e| {
                e.prevent_default();
                e.stop_propagation();

                let callback = callback.clone();
                let fut = async move {
                    let mut callback = callback.borrow_mut();
                    let result = callback(Event::Drop, e).await;
                    if let Err(err) = result {
                        error!("dropzone error at: {}: {}", callback_location, err);
                    }
                };
                spawn_local(fut);
            }
        });
    }
}
