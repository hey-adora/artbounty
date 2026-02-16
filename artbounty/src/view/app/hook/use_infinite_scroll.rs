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
    SubmitEvent, js_sys::Array,
};

#[derive(
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    // strum::EnumString,
    strum::Display,
    // strum::EnumIter,
    strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum InfiniteStage<ItemData: Clone> {
    Init,
    Top(ItemData),
    Btm(ItemData),
    Manual,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum InfiniteMerge<ItemData, ItemView>
where
    ItemView: IntoView + 'static,
    ItemData: Clone,
{
    Top {
        data: Vec<ItemData>,
        views: Vec<ItemView>,
    },
    Btm {
        data: Vec<ItemData>,
        views: Vec<ItemView>,
    },
    None,
}

#[derive(Clone, Copy)]
pub struct InfiniteScroll {
    pub view: StoredValue<Box<dyn Fn() -> AnyView + Sync + Send + 'static>>,
    pub trigger: StoredValue<Box<dyn Fn() + Sync + Send + 'static>>,
    // pub view: Memo<SendWrapper<AnyView>>,
    // pub btn_stage: StoredValue<Box<dyn (Fn() -> impl IntoView) + Sync + Send + 'static>>,
    // pub email: RwQuery<String>,
    // pub form_stage: RwQuery<ChangePasswordFormStage>,
    // pub btn_stage: StoredValue<Box<dyn Fn() -> ChangePasswordBtnStage + Sync + Send + 'static>>,
    // pub on_change: StoredValue<Box<dyn Fn(SubmitEvent) + Sync + Send + 'static>>,
    // pub token: RwQuery<String>,
}

pub fn use_infinite_scroll<Elm, F, Fut, ItemView, ItemData>(
    infinite_scroll_ref: NodeRef<Elm>,
    callback: F,
) -> InfiniteScroll
where
    Elm: ElementType,
    Elm::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    ItemData: Clone + 'static,
    F: Fn(InfiniteStage<ItemData>) -> Fut + Clone + Sync + Send + 'static,
    Fut: Future<Output = InfiniteMerge<ItemData, ItemView>> + 'static,
    ItemView: IntoView + 'static,
    // Attr: Attribute,
    ItemView::Output<NodeRefAttr<html::Div, NodeRef<html::Div>>>: Clone,
{
    let item_views = RwSignal::new_local(Vec::new());
    let item_data = RwSignal::new_local(Vec::<ItemData>::new());
    let item_refs = RwSignal::new(Vec::<NodeRef<html::Div>>::new());
    let observer_intersection_top = RwSignal::new(None::<SendWrapper<IntersectionObserver>>);
    let observer_intersection_bottom = RwSignal::new(None::<SendWrapper<IntersectionObserver>>);
    let observer_mutation = RwSignal::new(None::<SendWrapper<MutationObserver>>);
    // let btm_nodes = RwSignal::new(Vec::<NodeRef<html::Div>>::new());
    let delayed_scroll = StoredValue::<Option<(bool, f64, f64, usize)>>::new(None);
    let busy = StoredValue::new(false);

    // let add = move |stage: InfiniteStage| {
    //
    //     //
    // };

    let get_items = move |stage: InfiniteStage<ItemData>| {
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
                let (data, items, data_len, items_len, is_top) = match items {
                    InfiniteMerge::Top { data, views } => {
                        let items_len = views.len();
                        let data_len = data.len();
                        (data, views, items_len, data_len, true)
                    }
                    InfiniteMerge::Btm { data, views } => {
                        let items_len = views.len();
                        let data_len = data.len();
                        (data, views, data_len, items_len, false)
                    }
                    InfiniteMerge::None => {
                        return;
                    }
                };

                if items_len != data_len {
                    error!("data and view length must be equal");
                    return;

                }

                let mut new_nodes = Vec::new();
                let mut new_views = Vec::new();

                for (i, item) in items.into_iter().enumerate() {
                    let n = NodeRef::<html::Div>::new();
                    let item = item.into_view();
                    let a = item.add_any_attr(node_ref(n));

                    new_views.push(a);
                    new_nodes.push(n);
                }
                let new_nodes_len = new_nodes.len();

                let width = infinite_scroll_elm.client_width() as f64;
                let height = infinite_scroll_elm.client_height() as f64;
                let scroll_height = infinite_scroll_elm.scroll_height() as f64;
                let scroll_top = infinite_scroll_elm.scroll_top() as f64;

                let expected_scroll_height = height * 2.0;

                trace!("expected scroll {expected_scroll_height} scroll_top {scroll_top}");

                let removed = item_refs.try_update(|current_nodes| {
                    trace!("updating all_nodes");
                    let mut scroll_height_save = 0.0_f64;
                    // let mut scroll_height_after = scroll_height;
                    let current_nodes_len = current_nodes.len();
                    // let mut removed = 0_usize;

                    let mut index = if is_top {
                        // current_nodes_len.saturating_sub(1)
                        0
                    } else {
                        // 0
                        current_nodes_len.saturating_sub(1)
                    };

                    //     match stage {
                    //     InfiniteStage::Btm | InfiniteStage::Init => 0,
                    //     InfiniteStage::Top => current_nodes_len.saturating_sub(1),
                    // };

                    while let Some(node) = current_nodes.get(index) {
                        if scroll_height_save >= expected_scroll_height {
                            trace!("exiting saving loop");
                            break;
                        }

                        if let Some(node) = node.get_untracked() {
                            // if scroll_height_save >= expected_scroll_height {
                            //     break;
                            // }
                            let width = node.client_width() as f64;
                            let height = node.client_height() as f64;
                            trace!("saving {height}");

                            scroll_height_save += height;
                            // removed += 1;
                        } else {
                            break;
                        }

                        if is_top {

                                // index = index.saturating_sub(1);
                                index += 1;
                        } else {

                                // index += 1;
                                index = index.saturating_sub(1);
                        }


                        // match stage {
                        //     InfiniteStage::Btm | InfiniteStage::Init => {
                        //     }
                        //     InfiniteStage::Top => {
                        //     }
                        // }
                    }

                    if is_top {

                            trace!("index({index}) < current_nodes_len({current_nodes_len})");
                            if index < current_nodes_len {
                                trace!("set scroll scroll_height({scroll_height}) - scroll_height_save({scroll_height_save}) = {}", scroll_height - scroll_height_save);
                                // delayed_scroll.set_value(scroll_height - scroll_height_after);

                                delayed_scroll.set_value(Some((true, height, scroll_height_save, items_len)));
                                trace!("draining nodes {index}..");
                                current_nodes.drain(index..);
                            }
                            *current_nodes = [new_nodes, current_nodes.clone()].concat();
                    } else {
                            trace!("index({index}) > 0");
                            if index > 0 {
                                // delayed_scroll.set_value(scroll_height_after - scroll_height);
                               delayed_scroll.set_value(Some((false, height, scroll_height_save, items_len)));
                                trace!("draining nodes 0..{index}");
                                current_nodes.drain(0..index);
                            }
                            current_nodes.extend(new_nodes);

                    }

                    



                    // match stage {
                    //     InfiniteStage::Btm | InfiniteStage::Init => {
                    //     }
                    //     InfiniteStage::Top => {
                    //     }
                    // }

                    // trace!("extending notes with {} new nodes", new_nodes.len());

                    // let a = [v.clone(), nodes].concat();
                    // v.extend(nodes);

                    index
                });

                item_views.try_update(|current_views| {
                    if let Some(removed) = removed {
                        if is_top {
                            trace!("draining nodes {removed}..");
                            current_views.drain(removed..);
                        } else {
                            trace!("draining nodes 0..{removed}");
                            current_views.drain(0..removed);
                        }
                    }

                    if is_top {
                        trace!("extending views [new_view({}), current_views({})]", new_views.len(), current_views.len());
                        *current_views = [new_views, current_views.clone()].concat();

                    } else {
                        trace!("extending views with {} new views", new_views.len());
                        current_views.extend(new_views);
                    }

                    // if let Some(removed) = removed {
                    //     match stage {
                    //         InfiniteStage::Btm | InfiniteStage::Init => {
                    //         }
                    //         InfiniteStage::Top => {
                    //         }
                    //     }
                    // }
                    // if let Some(removed) = removed
                    //     && removed > 0
                    // {
                    //     trace!("draining items 0..{removed}");
                    //     v.drain(0..removed);
                    // }
                });

                item_data.try_update(|current_data| {
                    if let Some(removed) = removed {
                        if is_top {
                            trace!("draining data {removed}..");
                            current_data.drain(removed..);
                        } else {
                            trace!("draining data 0..{removed}");
                            current_data.drain(0..removed);
                        }
                    }

                    if is_top {
                        trace!("extending data [new_data({}), current_data({})]", data.len(), current_data.len());
                        *current_data = [data, current_data.clone()].concat();
                    } else {
                        trace!("extending views with {} new data", data.len());
                        current_data.extend(data);
                    }

                });


                let item_views_len = item_views.with_untracked(|v| v.len());
                let item_refs_len = item_refs.with_untracked(|v| v.len());
                let item_data_len = item_data.with_untracked(|v| v.len());

                if item_views_len != item_refs_len || item_refs_len != item_data_len {
                    error!("items buffers missmatch item_views_len({item_views_len}) item_refs_len({item_refs_len}) item_data_len({item_data_len})");
                }

            };




            fut.await;
            busy.set_value(false);
        });
    };

    let trigger = {
        let get_items = get_items.clone();
        move || {
            // e.prevent_default();

            get_items(InfiniteStage::Manual);

            //
        }
    };

    infinite_scroll_ref.add_resize_observer(move |entry, _observer| {
        trace!("RESIZINGGGGGG");
        let width = entry.content_rect().width() as u32;
    });

    on_cleanup(move || {
        if let Some(observer) = observer_mutation.get_untracked() {
            observer.disconnect();
        };
        if let Some(observer) = observer_intersection_bottom.get_untracked() {
            observer.disconnect();
        };
    });

    Effect::new(move || {
        let (Some(observer_intersection_btm), Some(observer_intersection_top)) = (
            observer_intersection_bottom.get(),
            observer_intersection_top.get(),
        ) else {
            return;
        };

        let first = item_refs.with(move |v| v.first().cloned());
        let last = item_refs.with(move |v| v.last().cloned());
        let (Some(first), Some(last)) = (first.and_then(|v| v.get()), last.and_then(|v| v.get()))
        else {
            return;
        };
        trace!(
            "intersection attached! \n{:?}\n{:?}",
            first.text_content(),
            last.text_content()
        );
        observer_intersection_top.disconnect();
        observer_intersection_top.observe(&first);

        observer_intersection_btm.disconnect();
        observer_intersection_btm.observe(&last);
    });

    // create the intersection observer
    let activated_top = StoredValue::new(false);
    let activated_btm = StoredValue::new(false);
    Effect::new({
        let get_items = get_items.clone();
        move || {
            let get_items = get_items.clone();
            let observer = intersection_observer::new_raw({
                let get_items = get_items.clone();

                move |entry, _observer| {
                    let Some(entry) = entry.first() else {
                        return;
                    };

                    // entry.e;
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

                    let Some(data) = item_data.with_untracked(|v| v.last().cloned()) else {
                        error!("missing data for item last");
                        return;
                    };
                    get_items(InfiniteStage::Btm(data));
                }
            });
            observer_intersection_bottom.set(Some(SendWrapper::new(observer)));

            let observer = intersection_observer::new_raw({
                let get_items = get_items.clone();

                move |entry, _observer| {
                    let Some(entry) = entry.first() else {
                        return;
                    };

                    // entry.e;
                    let is_intersecting = entry.is_intersecting();

                    if !is_intersecting {
                        activated_top.set_value(true);
                        return;
                    }

                    if !activated_top.get_value() {
                        return;
                    }

                    activated_top.set_value(false);

                    let Some(data) = item_data.with_untracked(|v| v.first().cloned()) else {
                        error!("missing data for item 0");
                        return;
                    };
                    trace!("yo wtf is going on");
                    get_items(InfiniteStage::Top(data));
                }
            });
            observer_intersection_top.set(Some(SendWrapper::new(observer)));

            let observer = mutation_observer::new_raw(move |a, b| {
                let Some((is_top, scroll_height_before, scroll_height_save, count)) =
                    delayed_scroll.get_value()
                else {
                    return;
                };
                if count == 0 {
                    return;
                }
                // if scroll == 0.0 {
                //     return;
                // }
                let Some(infinite_scroll_elm) = infinite_scroll_ref.get_untracked() else {
                    trace!("gallery NOT found");
                    return;
                };
                let infinite_scroll_elm: HtmlElement = infinite_scroll_elm.into();

                let width = infinite_scroll_elm.client_width() as f64;
                let height = infinite_scroll_elm.client_height() as f64;
                let scroll_height_current = infinite_scroll_elm.scroll_height() as f64;
                let scroll_top = infinite_scroll_elm.scroll_top() as f64;

                trace!(
                    "scroll_height_current{scroll_height_current} scroll_height_before{scroll_height_before} scroll_height_save{scroll_height_save}"
                );

                let elms = infinite_scroll_elm.children();
                let elm_first = elms.get_with_index(0 as u32);
                let elm_n = elms.get_with_index(count as u32);

                let (Some(elm_n), Some(elm_first)) = (elm_n, elm_first) else {
                    return;
                };

                let scroll = if is_top {
                    let rect_first = elm_first.get_bounding_client_rect();
                    let rect_n = elm_n.get_bounding_client_rect();

                    let y_first = rect_first.y();
                    let y_n = rect_n.y();

                    let y_diff = y_n - y_first;
                    trace!("y_n({y_n}) - y_first({y_first}) = y_diff({})", y_diff);

                    y_diff
                } else {
                    0.0
                };

                // let scroll = if is_top {
                //     scroll_height_save - scroll_height_current
                // } else {
                //     scroll_height_current - scroll_height_save
                // };

                // let scroll = match stage {
                //     InfiniteStage::Btm | InfiniteStage::Init => {
                //     }
                //     InfiniteStage::Top => {
                //         // let removed = scroll_height_before - scroll_height_after;
                //         // 50.0
                //     }
                // };

                trace!("scrolling by {scroll}");
                infinite_scroll_elm.scroll_by_with_x_and_y(0.0, scroll);
                delayed_scroll.set_value(None);
            });
            observer_mutation.set(Some(SendWrapper::new(observer)));
            //
        }
    });

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

        get_items(InfiniteStage::Init);
    });

    InfiniteScroll {
        view: StoredValue::new(Box::new(move || {
            let items = item_views.get();

            let view = view! {
                { items }
            }
            .into_view()
            .into_any();

            view
        })),
        trigger: StoredValue::new(Box::new(trigger)),
    }
}

type ViewWithAttr<Item> =
    leptos::prelude::View<<Item as AddAnyAttr>::Output<NodeRefAttr<html::Div, NodeRef<html::Div>>>>;

struct ItemNode<Item>
where
    Item: IntoView + Clone + Sync + Send + 'static,
{
    node_ref: NodeRef<html::Div>,
    view: ViewWithAttr<Item>,
}

pub fn items_to_nodes_and_views<Item>(
    new_items: Vec<Item>,
) -> (Vec<NodeRef<html::Div>>, Vec<ViewWithAttr<Item>>)
where
    Item: IntoView + Clone + Sync + Send + 'static,
{
    // let items_len = new_items.len();
    let mut new_nodes = Vec::new();
    let mut new_views = Vec::new();

    for (i, item) in new_items.into_iter().enumerate() {
        let item = item.into_view();
        let node = NodeRef::<html::Div>::new();
        let view = item.add_any_attr(node_ref(node));

        new_views.push(view);
        new_nodes.push(node);
    }

    (new_nodes, new_views)
}

// #[cfg(test)]
// mod infinite_scroll_test {
//     use wasm_bindgen_test::*;
//
//     #[wasm_bindgen_test]
//     fn pass() {
//         assert_eq!(1, 1);
//     }
// }

pub fn remove_items_btm<Item>(
    nodes: Vec<NodeRef<html::Div>>,
    views: Vec<ViewWithAttr<Item>>,
) -> (Vec<NodeRef<html::Div>>, Vec<ViewWithAttr<Item>>)
where
    Item: IntoView + Clone + Sync + Send + 'static,
{
    (nodes, views)
}

pub fn add_new_items<Item>(new_items: Vec<Item>)
where
    Item: IntoView + Clone + Sync + Send + 'static,
{
    let (nodes, views) = items_to_nodes_and_views(new_items);

    // let items_len = new_items.len();
    // let mut new_nodes = Vec::new();
    // let mut new_views = Vec::new();
    //
    // for (i, item) in new_items.into_iter().enumerate() {
    //     let item = item.into_view();
    //     let node = NodeRef::<html::Div>::new();
    //     let view = item.add_any_attr(node_ref(node));
    //
    //     new_views.push(view);
    //     new_nodes.push(node);
    // }
    // let new_nodes_len = new_nodes.len();

    // let width = infinite_scroll_elm.client_width() as f64;
    // let height = infinite_scroll_elm.client_height() as f64;
    // let scroll_height = infinite_scroll_elm.scroll_height() as f64;
    // let scroll_top = infinite_scroll_elm.scroll_top() as f64;
    //
    // let expected_scroll_height = height * 2.0;
    //
    // trace!("expected scroll {expected_scroll_height} scroll_top {scroll_top}");
    //
    // let removed = all_nodes.try_update(|current_nodes| {
    //     trace!("updating all_nodes");
    //     let mut scroll_height_after = scroll_height;
    //     let current_nodes_len = current_nodes.len();
    //     // let mut removed = 0_usize;
    //
    //     let mut index = match stage {
    //         InfiniteStage::Btm | InfiniteStage::Init => 0,
    //         InfiniteStage::Top => current_nodes_len.saturating_sub(1),
    //     };
    //
    //     while let Some(node) = current_nodes.get(index) {
    //         if scroll_height_after <= expected_scroll_height {
    //             trace!("exiting node remove loop");
    //             break;
    //         }
    //
    //         if let Some(node) = node.get_untracked() {
    //             if scroll_height_after <= expected_scroll_height {
    //                 break;
    //             }
    //             let width = node.client_width() as f64;
    //             let height = node.client_height() as f64;
    //             trace!("removing {height}");
    //
    //             scroll_height_after -= height;
    //             // removed += 1;
    //         } else {
    //             break;
    //         }
    //
    //         match stage {
    //             InfiniteStage::Btm | InfiniteStage::Init => {
    //                 index += 1;
    //             }
    //             InfiniteStage::Top => {
    //                 index = index.saturating_sub(1);
    //             }
    //         }
    //     }
    //
    //     match stage {
    //         InfiniteStage::Btm | InfiniteStage::Init => {
    //             if index > 0 {
    //                 delayed_scroll.set_value(scroll_height_after - scroll_height);
    //
    //                 trace!("draining nodes 0..{index}");
    //                 current_nodes.drain(0..index);
    //             }
    //             current_nodes.extend(new_nodes);
    //         }
    //         InfiniteStage::Top => {
    //             if index < current_nodes_len {
    //                 delayed_scroll.set_value(scroll_height - scroll_height_after);
    //
    //                 trace!("draining nodes {index}..");
    //                 current_nodes.drain(index..);
    //             }
    //             *current_nodes = [new_nodes, current_nodes.clone()].concat();
    //         }
    //     }
    //
    //     index
    // });
    //
    // if let Some(removed) = removed {
    //     match stage {
    //         InfiniteStage::Btm | InfiniteStage::Init => {
    //             trace!("draining nodes 0..{removed}");
    //             v.drain(0..removed);
    //         }
    //         InfiniteStage::Top => {
    //             trace!("draining nodes {removed}..");
    //             v.drain(removed..);
    //         }
    //     }
    // }
    //
    // trace!("extending views with {} new views", new_views.len());
    // v.extend(new_views);
}
