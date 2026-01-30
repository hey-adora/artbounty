use crate::{
    api::{Api, ApiWeb},
    path::{PATH_LOGIN, PATH_UPLOAD, link_settings, link_user},
    view::{
        app::{GlobalState, components::gallery::Img},
        toolbox::prelude::*,
    },
};
// use axum::extract::FromRef;
use futures::SinkExt;
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
use send_wrapper::SendWrapper;
use tracing::{debug, error, trace};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    Element, HtmlElement, IntersectionObserver, MutationObserver, MutationObserverInit,
    js_sys::Array,
};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum InfiniteStage {
    Init,
    Top,
    Btm,
}

#[derive(Clone, Copy)]
pub struct InfiniteScroll<Item: IntoView + Sync + Send + 'static> {
    pub items: RwSignal<Item>,
    // pub btn_stage: StoredValue<Box<dyn (Fn() -> impl IntoView) + Sync + Send + 'static>>,
    // pub email: RwQuery<String>,
    // pub form_stage: RwQuery<ChangePasswordFormStage>,
    // pub btn_stage: StoredValue<Box<dyn Fn() -> ChangePasswordBtnStage + Sync + Send + 'static>>,
    // pub on_change: StoredValue<Box<dyn Fn(SubmitEvent) + Sync + Send + 'static>>,
    // pub token: RwQuery<String>,
}

pub fn use_infinite_scroll<Elm, F, Fut, Item>(
    infinite_scroll_ref: NodeRef<Elm>,
    callback: F,
) -> impl IntoView
where
    Elm: ElementType,
    Elm::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    F: Fn(InfiniteStage) -> Fut + Clone + Sync + Send + 'static,
    Fut: Future<Output = Vec<Item>> + Sync + Send + 'static,
    Item: IntoView + Clone + Sync + Send + 'static,
    // Attr: Attribute,
    Item::Output<NodeRefAttr<html::Div, NodeRef<html::Div>>>: Clone,
{
    // let a = (async move |_:()|  { Ok(()) } );
    // let a = (async move |_:()|  { () } );
    // let api = a.ground();
    // api.dispatch(());
    // let infinite_scroll_ref = NodeRef::<html::Div>::new();
    // action.dispatch(());
    let all_items = RwSignal::new_local(Vec::new());
    let observer_intersection_bottom = RwSignal::new(None::<SendWrapper<IntersectionObserver>>);
    let observer_mutation = RwSignal::new(None::<SendWrapper<MutationObserver>>);
    let all_nodes = RwSignal::new(Vec::<NodeRef<html::Div>>::new());
    // let btm_nodes = RwSignal::new(Vec::<NodeRef<html::Div>>::new());
    let delayed_scroll = StoredValue::new(0.0 as f64);
    let busy = StoredValue::new(false);

    let get_items = move |stage: InfiniteStage| {
        // let stage = *stage;
        if busy.get_value() {
            return;
        }
        let callback = callback.clone();
        busy.set_value(true);
        spawn_local(async move {
            let fut = async move {
                let Some(infinite_scroll_elm): Option<HtmlElement> =
                    infinite_scroll_ref.get_untracked().map(|v| v.into())
                else {
                    trace!("gallery NOT found");
                    return;
                };
                let items = callback(stage).await;
                let items_len = items.len();
                let mut nodes = Vec::new();
                let mut views = Vec::new();

                for (i, item) in items.into_iter().enumerate() {
                    let n = NodeRef::<html::Div>::new();
                    let item = item.into_view();
                    let a = item.add_any_attr(node_ref(n));

                    views.push(a);
                    nodes.push(n);
                }

                let width = infinite_scroll_elm.client_width() as f64;
                let height = infinite_scroll_elm.client_height() as f64;
                let scroll_height = infinite_scroll_elm.scroll_height() as f64;
                let scroll_top = infinite_scroll_elm.scroll_top() as f64;

                let expected_scroll_height = height * 2.0;

                trace!("expected scroll {expected_scroll_height} scroll_top {scroll_top}");

                let removed = all_nodes.try_update(|v| {
                    trace!("updating all_nodes");
                    let mut scroll_height_after = scroll_height;
                    let mut removed = 0_usize;
                    for (i, node) in v.iter().enumerate() {
                        if scroll_height_after <= expected_scroll_height {
                            trace!("exiting node remove loop");
                            break;
                        }

                        if let Some(node) = node.get_untracked() {
                            if scroll_height_after <= expected_scroll_height {
                                break;
                            }
                            let width = node.client_width() as f64;
                            let height = node.client_height() as f64;
                            trace!("removing {height}");

                            scroll_height_after -= height;
                            removed += 1;
                        } else {
                            break;
                        }
                    }

                    if removed > 0 {
                        delayed_scroll.set_value(scroll_height_after - scroll_height);

                        trace!("draining nodes 0..{removed}");
                        v.drain(0..removed);
                    }

                    trace!("extending notes with {} new nodes", nodes.len());

                    v.extend(nodes);

                    removed
                });

                all_items.try_update(|v| {
                    if let Some(removed) = removed
                        && removed > 0
                    {
                        trace!("draining items 0..{removed}");
                        v.drain(0..removed);
                    }

                    trace!("extending views with {} new views", views.len());
                    v.extend(views);
                    // for view in v {
                    //     view.into();
                    // }
                });
            };

            fut.await;
            busy.set_value(false);
        });
    };
    // let get_items = Action::new(move |stage: &InfiniteStage| {});

    infinite_scroll_ref.add_resize_observer(move |entry, _observer| {
        trace!("RESIZINGGGGGG");
        let width = entry.content_rect().width() as u32;

        // let prev_imgs = gallery.get_untracked();
        // trace!("stage r1: width:{width} {prev_imgs:#?} ");
        // let resized_imgs = resize_v2(prev_imgs, width, row_height);
        // trace!("stage r2 {resized_imgs:#?}");
        // gallery.set(resized_imgs);
    });

    // infinite_scroll_ref.add_mutation_observer(
    //     move |entries, observer| {
    //         trace!("IT HAS MUTATED");
    //         let Some(infinite_scroll_elm) = infinite_scroll_ref.get_untracked() else {
    //             trace!("gallery NOT found");
    //             return;
    //         };
    //         let infinite_scroll_elm: HtmlElement = infinite_scroll_elm.into();
    //
    //         // let delayed_scroll_value = delayed_scroll.get_untracked();
    //         // if delayed_scroll_value == 0 {
    //         //     return;
    //         // }
    //         infinite_scroll_elm.scroll_by_with_x_and_y(0.0, -50.0 as f64);
    //         // delayed_scroll.set(0);
    //     },
    //     MutationObserverOptions::new().set_child_list(true),
    // );

    // let get_items = move |target: HtmlElement, stage: InfiniteStage| {
    //     // debug!("oneeeeeeeeeee");
    //     let items = callback(stage);
    //     let items_len = items.len();
    //     let mut nodes = Vec::new();
    //     let mut views = Vec::new();
    //
    //     for (i, item) in items.into_iter().enumerate() {
    //         let n = NodeRef::<html::Div>::new();
    //         let item = item.into_view();
    //         let a = item.add_any_attr(node_ref(n));
    //
    //         views.push(a);
    //         nodes.push(n);
    //     }
    //
    //     let width = target.client_width() as f64;
    //     let height = target.client_height() as f64;
    //
    //     let removed = all_items.try_update(|v| {
    //         v.extend(views);
    //     });
    //     all_nodes.update(|v| {
    //         v.extend(nodes);
    //     });
    // };

    on_cleanup(move || {
        if let Some(observer) = observer_mutation.get_untracked() {
            observer.disconnect();
        };
        if let Some(observer) = observer_intersection_bottom.get_untracked() {
            observer.disconnect();
        };
    });

    Effect::new(move || {
        // debug!("zeroooooooooo");
        let (Some(observer_intersection),) = (observer_intersection_bottom.get(),) else {
            return;
        };

        // all_nodes.update(|v| {
        //     for node in v {
        //         if let Some(node) = node.get_untracked() {
        //             let class_list = node.class_list();
        //             let class = ["absolute"]
        //                 .into_iter()
        //                 .map(|x| JsValue::from_str(x))
        //                 .collect::<Array>();
        //             // let class = JsValue::from_ref(["absolute"]);
        //             // let class = Array::from(["absolute"]);
        //             let _ = class_list.add(&class).inspect_err(|_| {
        //                 tracing::error!("failed to set class for {node:?}");
        //             });
        //         } else {
        //             return;
        //         }
        //     }
        // });

        let last = all_nodes.with(move |v| v.last().cloned());
        // debug!("nooooooooooo");
        let Some(last) = last.and_then(|v| v.get()) else {
            return;
        };
        trace!("intersection attached!");
        observer_intersection.disconnect();
        observer_intersection.observe(&last);
        // debug!("wooooooooow");
    });

    // create the intersection observer
    let activated = StoredValue::new(false);
    Effect::new({
        let get_items = get_items.clone();
        move || {
            let get_items = get_items.clone();
            let observer = intersection_observer::new_raw(move |entry, _observer| {
                // let Some(infinite_scroll_elm): Option<HtmlElement> =
                //     infinite_scroll_ref.get().map(|v| v.into())
                // else {
                //     trace!("gallery NOT found");
                //     return;
                // };
                let Some(entry) = entry.first() else {
                    return;
                };

                // entry.e;
                let is_intersecting = entry.is_intersecting();

                if !is_intersecting {
                    activated.set_value(true);
                    return;
                }

                if !activated.get_value() {
                    return;
                }

                activated.set_value(false);
                // let items = callback.clone()(InfiniteStage::Init);
                trace!("yo wtf is going on");
                get_items(InfiniteStage::Btm);
                // all_items.update(|v| {
                //     v.extend(items);
                // });
            });
            observer_intersection_bottom.set(Some(SendWrapper::new(observer)));

            let observer = mutation_observer::new_raw(move |a, b| {
                let scroll = delayed_scroll.get_value();
                if scroll == 0.0 {
                    return;
                }
                let Some(infinite_scroll_elm) = infinite_scroll_ref.get_untracked() else {
                    trace!("gallery NOT found");
                    return;
                };

                let infinite_scroll_elm: HtmlElement = infinite_scroll_elm.into();
                trace!("wowza");
                infinite_scroll_elm.scroll_by_with_x_and_y(0.0, scroll);
                delayed_scroll.set_value(0.0);
            });
            observer_mutation.set(Some(SendWrapper::new(observer)));
            //
        }
    });

    // Effect::new(move || {
    //     let Some(observer) = all_observers.get() else {
    //         return;
    //     };
    //     let nodes = all_nodes.get();
    //     let nodes_len = nodes.len();
    //     let Some(last_node) = nodes.last() else {
    //         return;
    //     };
    //
    //     btm_nodes.update(|v| {
    //         v.push(last_node.clone());
    //     });
    //     // for (i, node) in nodes.into_iter().enumerate() {
    //     //     if let Some(node) = node.get()
    //     //         && i + 1 == nodes_len
    //     //     {
    //     //         observer.observe(&node);
    //     //         // trace!("who? me?");
    //     //     } else {
    //     //         // trace!("thats right baby >:3");
    //     //     }
    //     // }
    // });

    Effect::new(move || {
        let (Some(infinite_scroll_elm), Some(observer_mutation)) = (
            infinite_scroll_ref.get().map(|v| v.into()) as Option<HtmlElement>,
            observer_mutation.get(),
        ) else {
            trace!("gallery NOT found");
            return;
        };
        let options = MutationObserverInit::new();
        options.set_child_list(true);
        options.set_character_data(true);
        options.set_subtree(true);

        let _ = observer_mutation
            .observe_with_options(&infinite_scroll_elm, &options)
            .inspect_err(|_| {
                error!("failed to observe");
            });

        //         // let width = infinite_scroll_elm.client_width() as f64;
        // let height = infinite_scroll_elm.client_height() as f64;

        get_items(InfiniteStage::Init);
        // let items = callback(InfiniteStage::Init);
        // let items_len = items.len();
        // let mut nodes = Vec::new();
        // let mut views = Vec::new();
        // // let mut views = Vec::<AnyView>::new();
        //
        // for (i, item) in items.into_iter().enumerate() {
        //     let n = NodeRef::<html::Div>::new();
        //     let item = item.into_view();
        //     let a = item.add_any_attr(node_ref(n));
        //
        //     views.push(a);
        //     nodes.push(n);
        // }
        //
        // all_items.set(views);
        // all_nodes.set(nodes);
    });
    // let o = views.collect_view();
    // let a = callback().into_iter().fold(Vec::<NodeRef<html::Div>>::new(), |mut a, v| {
    //     let n = NodeRef::<html::Div>::new();
    //     nodes.update(|v| {
    //         v.push(n);
    //     });
    //     let e = v.into_view();
    //     let a = e.add_any_attr(node_ref(n));
    //     let a = a.into_any();
    //     a
    // });
    // a.collect_view();

    // let nodes = nodes.get();
    // let observe = observe.get();
    //
    // let Some(observe) = observe else {
    //     return;
    // };
    //
    // for node in nodes {
    //     let v = node.get();
    //     let Some(node) = v else {
    //         return;
    //     };
    //     observe.observe(&node);
    // }

    // if let Some(n) = n.get_untracked()
    //     && i + 1 == items_len
    // {
    //
    //     observer.observe(&n);
    //     trace!("who? me?");
    // } else {
    //
    //     trace!("thats right baby >:3");
    // }
    // let view = match i {
    //
    // }
    // let view = item.into_any();

    // let items = move || {
    //     let a = callback().into_iter().map(|v| {
    //         // let e = v.elements().into_iter().map(|v| {
    //         //
    //         //     // v.sc;
    //         //
    //         //     v
    //         // });
    //         // e.collect_view()
    //         //
    //
    //         let n = NodeRef::<html::Div>::new();
    //
    //         nodes.update(|v| {
    //             v.push(n);
    //         });
    //
    //         let e = v.into_view();
    //
    //         // let f  = e.elements();
    //         let a = e.add_any_attr(node_ref(n));
    //         let a = a.into_any();
    //
    //         // view! {
    //         //     < {..} node_ref=n > {e} </>
    //         // }
    //         a
    //     });
    //     a.collect_view()
    // };

    let get_all_items = move || {
        // all_items.track();
        all_items.get()
        // let items = all_items.with(|v| {
        //     v.close();
        // });
        // items
    };

    view! {
        // <div>"duck"</div>
        { get_all_items }
        // <TriggerWrap node_ref=r > "A" </TriggerWrap>
    }
    // InfiniteScroll { items }
}

pub fn foo(el: Element) {
    // let mut highlighted = false;
    //
    // let handle = el.clone().on(click, move |_| {
    //     highlighted = !highlighted;
    //
    //     if highlighted {
    //         el.style(("background-color", "yellow"));
    //     } else {
    //         el.style(("background-color", "transparent"));
    //     }
    // });
    // on_cleanup(move || drop(handle));
}

// #[component]
// pub fn TriggerWrap(mut children: ChildrenFragmentMut) -> impl IntoView {
//     let r = NodeRef::new();
//
//     let children = move || {
//         //
//         let a = children();
//
//         let b = a.nodes.into_iter().map(|v| {
//             // v.bind(AttributeKey, signal);
//             // v.e
//             // let e = v.e;
//             // let e: HtmlElement = v.try_into().unwrap();
//             // let g = r.load(v);
//             v
//         });
//     };
//
//     view! {
//         { children }
//         // <Fragment node_ref=r nodes=children />
//     }
// }
