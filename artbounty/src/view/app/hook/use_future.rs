use std::marker::PhantomData;

use crate::{
    api::{Api, ApiWeb},
    path::{PATH_LOGIN, PATH_UPLOAD, link_settings, link_user},
    view::{
        app::{GlobalState, components::gallery::Img},
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
use web_sys::{HtmlElement, IntersectionObserverInit};

// #[derive(Clone, Copy)]
// pub struct FutureFn2<T1, T: Fn(T1) + Sync + Send + 'static> {
//     pub is_busy: RwSignal<bool>,
//     pub fut: StoredValue<T>,
// }
//
// #[derive(Clone, Copy)]
// pub struct FutureFn2<T: Fn() + Sync + Send + 'static> {
//     pub is_busy: RwSignal<bool>,
//     pub fut: StoredValue<T>,
// }

// #[derive( Copy)]
pub struct FutureFn<Input, Fut, F>
where
    // Input:
    F: FnRun<Fut, Input> + Sync + Send + 'static,
    Fut: Future<Output = ()>+ Sync + Send + 'static,
{
    pub is_busy: RwSignal<bool>,
    pub callback: StoredValue<F, LocalStorage>,
    phantom: PhantomData<(Input, Fut)>,
    // pub fut: StoredValue<F>,
    // pub callback: StoredValue<(Box<Fn(Box<dyn Any>) + 'static>, LocalStorage)>,
}

impl<Input, Fut, F> Clone for FutureFn<Input, Fut, F>
where
    F: FnRun<Fut, Input> + Sync + Send+ 'static,
    Fut: Future<Output = ()> + Sync + Send + 'static,
{
    fn clone(&self) -> Self {
        Self {
            is_busy: self.is_busy.clone(),
            callback: self.callback.clone(),
            phantom: self.phantom,
        }
    }
}

impl<Input, Fut, F> Copy for FutureFn<Input, Fut, F>
where
    F: FnRun<Fut, Input> + Sync + Send+ 'static,
    Fut: Future<Output = ()> + Sync + Send + 'static,
{
}

pub enum SendHelp<T> {
    T0(Box<dyn Fn() -> () + Send + Sync + 'static>),
    T1(Box<dyn Fn(T) + Send + Sync + 'static>),
}

// impl <T, Fut, F: FnRun<Fut, T> + Clone >FutureFn<T, Fut, F>
// where
//     F: Fn() + Sync + Send + 'static,
//     Fut: Future<Output = ()> + 'static,

impl<Input, Fut, F> FutureFn<Input, Fut, F>
where
    F: FnRun<Fut, Input> + Sync + Send + 'static,
    Fut: Future<Output = ()> + Sync + Send + 'static,
{
    pub fn new(f: F) -> Self {
        let busy = RwSignal::new(false);

        FutureFn {
            callback: StoredValue::new_local(f),
            is_busy: busy,
            phantom: PhantomData,
        }
    }
}

impl<T1: 'static, Fut, F> FutureFn<(T1,), Fut, F>
where
    F: FnRun<Fut, (T1,)> + Sync + Send + 'static,
    Fut: Future<Output = ()> + Sync + Send + 'static,
{
    pub fn run(&self, t1: T1) {
        let busy = self.is_busy.clone();
        let callback = self.callback.clone();

        if busy.get_untracked() {
            return;
        }

        busy.set(true);
        spawn_local(async move {
            let fut = callback.read_value().run((t1,));
            fut.await;
            busy.set(false);
        });
    }
}

impl<Fut, F> FutureFn<(), Fut, F>
where
    F: FnRun<Fut, ()> + Sync + Send + 'static,
    Fut: Future<Output = ()> + Sync + Send + 'static,
{
    // pub fn new(f: F) -> Self {
    //     let busy = RwSignal::new(false);
    //
    //     FutureFn {
    //         callback: StoredValue::new_local(f),
    //         is_busy: busy,
    //         phantom: PhantomData,
    //     }
    // }

    pub fn run(&self) {
        let busy = self.is_busy.clone();
        let callback = self.callback.clone();

        if busy.get_untracked() {
            return;
        }

        busy.set(true);
        spawn_local(async move {
            let fut = callback.read_value().run(());
            fut.await;
            busy.set(false);
        });
    }
}

// impl <F, Fut, T1> FutureFn<F, Fut>
// where
//     F: Fn(T1) -> Fut + Clone + Sync + Send + 'static,
//     Fut: Future<Output = ()> + 'static,
//
// {
//     pub fn new(f: F) -> Self
//     {
//         let busy = RwSignal::new(false);
//         let run = move |t1: T1| {
//             if busy.get_untracked() {
//                 return;
//             }
//             let mut f = f.clone();
//             busy.set(true);
//             spawn_local(async move {
//                 let fut = f(t1);
//                 fut.await;
//                 busy.set(false);
//             });
//             //
//         };
//
//         FutureFn {
//             fut: StoredValue::new(Box::new(run)),
//             is_busy: busy,
//             phantom: PhantomData,
//         }
//     }
//
//     pub fn run(&self) {
//         self.fut.to_fn()();
//     }
// }

// pub fn use_future<Fut, F>(f: F) -> UseFuture
// where
//     Fut: Future<Output = ()> + 'static,
//     F: FnMut() -> Fut + Clone + Sync + Send + 'static,
// {
// }
// #[derive(Clone, Copy)]
// pub struct FutureFn<T, Fut, F: FnRun<Fut, T>>
// where
//     F: Fn() + Sync + Send + 'static,
//     Fut: Future<Output = ()> + 'static,
// {
//     pub is_busy: RwSignal<bool>,
//     pub fut: StoredValue<F>,
//     pub fut2: StoredValue<Box<dyn FnRun<Future<Output = ()>, ()>>>,
//     phantom: PhantomData<(T, Fut)>,
// }
//
// impl <T, Fut, F: FnRun<Fut, T> + Clone >FutureFn<T, Fut, F>
// where
//     F: Fn() + Sync + Send + 'static,
//     Fut: Future<Output = ()> + 'static,
//
//
// {
//     pub fn new(f: F) -> Self
//     // where
//     //     F: FnMut() -> Fut + Clone + Sync + Send + 'static,
//     {
//         let busy = RwSignal::new(false);
//         let run = move || {
//             if busy.get_untracked() {
//                 return;
//             }
//             let mut f = f.clone();
//             busy.set(true);
//             spawn_local(async move {
//                 let fut = f();
//                 fut.await;
//                 busy.set(false);
//             });
//             //
//         };
//
//         FutureFn {
//             fut: StoredValue::new(run),
//             is_busy: busy,
//             phantom: PhantomData,
//         }
//     }
// }
//
// // pub fn use_future<Fut, F>(f: F) -> UseFuture
// // where
// //     Fut: Future<Output = ()> + 'static,
// //     F: FnMut() -> Fut + Clone + Sync + Send + 'static,
// // {
// // }
