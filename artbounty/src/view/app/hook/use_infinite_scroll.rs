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
    Element, HtmlElement, IntersectionObserver, IntersectionObserverInit, MutationObserver,
    MutationObserverInit,
};

#[derive(Debug, Clone, PartialEq, PartialOrd, strum::Display, strum::EnumIs)]
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
    // strum::EnumIter,
    strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum InfiniteMerge<ItemData>
where
    // ItemView: IntoView + 'static,
    ItemData: Clone,
{
    Top {
        data: Vec<ItemData>,
        // views: Vec<ItemView>,
    },
    Btm {
        data: Vec<ItemData>,
        // views: Vec<ItemView>,
    },
    None,
}

#[derive(Clone, Copy)]
pub struct InfiniteScroll<T: Clone + 'static> {
    pub data: RwSignal<Vec<T>, LocalStorage>,
    // pub view: StoredValue<Box<dyn Fn() -> AnyView + Sync + Send + 'static>>,
    pub trigger: StoredValue<Box<dyn Fn() + Sync + Send + 'static>>,
}

#[derive(Clone, Copy, Debug)]
pub struct DelayedScroll {
    pub is_top: bool,
    pub scroll_height_before: f64,
    pub scroll_top_before: f64,
    pub elm_last_y: f64,
    pub elm_last_height: f64,
    pub elm_first_y: f64,
    pub elm_first_height: f64,
    pub saved_height_with_gaps: f64,
    pub saved_height_without_gaps: f64,
    pub removed_height_with_gaps: f64,
    pub removed_height_without_gaps: f64,
    pub cursor: usize,
    pub new_elm_count: usize,
}

pub fn use_infinite_scroll<Elm, Fut, ItemData, FnGetData>(
    infinite_scroll_ref: NodeRef<Elm>,
    callback: FnGetData,
) -> InfiniteScroll<ItemData>
where
    Elm: ElementType,
    Elm::Output: JsCast + Clone + 'static + Into<HtmlElement>,
    ItemData: Clone + std::fmt::Debug + 'static,
    Fut: Future<Output = InfiniteMerge<ItemData>> + 'static,
    FnGetData: Fn(InfiniteStage<ItemData>) -> Fut + Clone + Sync + Send + 'static,
    // ItemView: IntoView + 'static,
    // FnGenView: Fn(ItemData) -> ItemView + Clone + Sync + Send + 'static,
    // ItemView::Output<NodeRefAttr<html::Div, NodeRef<html::Div>>>: Clone,
{
    // let item_views = RwSignal::new_local(Vec::new());
    // let item_refs = RwSignal::new(Vec::<NodeRef<html::Div>>::new());
    let item_data = RwSignal::new_local(Vec::<ItemData>::new());
    let observer_intersection_top = RwSignal::new(None::<SendWrapper<IntersectionObserver>>);
    let observer_intersection_bottom = RwSignal::new(None::<SendWrapper<IntersectionObserver>>);
    let observer_mutation = RwSignal::new(None::<SendWrapper<MutationObserver>>);
    let delayed_scroll = StoredValue::<Option<DelayedScroll>>::new(None);
    let busy = StoredValue::new(false);

    let get_items = move |stage: InfiniteStage<ItemData>| {
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
                let (new_data, new_data_len, is_top) = match items {
                    InfiniteMerge::Top { data } => {
                        let data_len = data.len();
                        (data, data_len, true)
                    }
                    InfiniteMerge::Btm { data } => {
                        let data_len = data.len();
                        (data, data_len, false)
                    }
                    InfiniteMerge::None => {
                        return;
                    }
                };

                let current_data = item_data.get_untracked();
                let height = infinite_scroll_elm.client_height() as f64;
                let scroll_height = infinite_scroll_elm.scroll_height() as f64;
                let scroll_top = infinite_scroll_elm.scroll_top() as f64;
                let save_height = if is_top {
                    scroll_top + height
                } else {
                    scroll_height - scroll_top
                };
                let save_height = if save_height < (height * 3.0) {
                    height * 3.0
                } else {
                    save_height
                };
                trace!("save_height: {save_height}");
                let elms = infinite_scroll_elm.children();
                let elms_len = infinite_scroll_elm.child_element_count() as usize;
                let max_remove = new_data_len;
                let elm_first = elms.get_with_index(0);
                let elm_last = elms.get_with_index(elms_len.saturating_sub(1) as u32);

                let Some((cursor, saved_height)) = crop_view(
                    |i| {
                        elms.get_with_index(i as u32)
                            .map(|v| v.client_height() as f64)
                    },
                    elms_len,
                    max_remove,
                    is_top,
                    save_height,
                    1.0,
                    scroll_height,
                ) else {
                    trace!("no view was cropped");
                    let current_data = merge_data(current_data, new_data, is_top);
                    trace!("settings data: {current_data:#?}");
                    item_data.set(current_data);

                    let first_rect = elm_first.map(|v| v.get_bounding_client_rect());
                    let last_rect = elm_last.map(|v| v.get_bounding_client_rect());

                    delayed_scroll.set_value(Some(DelayedScroll {
                        is_top,
                        scroll_height_before: scroll_height,
                        scroll_top_before: scroll_top,
                        elm_last_y: last_rect.clone().map(|v| v.y()).unwrap_or_default(),
                        elm_last_height: last_rect.map(|v| v.height()).unwrap_or_default(),
                        elm_first_y: first_rect.clone().map(|v| v.y()).unwrap_or_default(),
                        elm_first_height: first_rect.map(|v| v.height()).unwrap_or_default(),
                        saved_height_without_gaps: 0.0,
                        saved_height_with_gaps: 0.0,
                        removed_height_with_gaps: 0.0,
                        removed_height_without_gaps: 0.0,
                        cursor: new_data_len.saturating_sub(1),
                        new_elm_count: new_data_len,
                    }));
                    // delayed_scroll.set_value(Some((is_top, scroll_height, 0.0, new_data_len)));
                    return;
                };

                trace!("cursor {cursor}");

                let (last_y, last_height, first_y, first_height, saved_height_with_gaps, removed_height_with_gaps) = elms
                    .get_with_index(cursor as u32)
                    .and_then(|elm_n| {
                        elms.get_with_index(elms_len.saturating_sub(1) as u32)
                            .map(|elm_last| (elm_n, elm_last))
                    })
                    .and_then(|(elm_n, elm_last)| {
                        elms.get_with_index(0)
                            .map(|elm_first| (elm_first, elm_n, elm_last))
                    })
                    .map(|(elm_first, elm_n, elm_last)| {
                        let rect_first = elm_first.get_bounding_client_rect();
                        let rect_n = elm_n.get_bounding_client_rect();
                        let rect_last = elm_last.get_bounding_client_rect();

                        let first_y = rect_first.y();
                        let first_height = rect_first.height();
                        let n_y = rect_n.y();
                        let n_height = rect_n.height();
                        let last_y = rect_last.y();
                        let last_height = rect_last.height();

                        let v = elm_last.text_content();

                        trace!("1 first_y({first_y}) first_height({first_height}) n_y({n_y}) n_height({n_height}) last_y({last_y}) last_height({last_height}) {v:?}");

                        if is_top {
                            let saved_height_with_gaps = n_y - first_y;
                            let removed_height_with_gaps = (last_y + last_height) - n_y;
                            // (last_y + last_height) - (n_y + n_height);
                            (last_y, last_height, first_y, first_height, saved_height_with_gaps, removed_height_with_gaps)
                        } else {
                            // let n_y = rect_n.y();
                            // let n_height = rect_n.height();
                            // let first_y = rect_first.y();
                            // let first_height = rect_first.height();

                            let saved_height_with_gaps = (last_y + last_height) - (n_y + n_height);
                            let removed_height_with_gaps = if first_y < 0.0 {
                                n_y + n_height + first_y.abs()

                            } else {

                                n_y - first_y
                            };
                            // let removed_height_with_gaps = n_y - first_y;
                            // let removed_height_with_gaps = (n_y + n_height) - first_y;
                            (last_y, last_height, first_y, first_height, saved_height_with_gaps, removed_height_with_gaps)
                        }
                    })
                    .unwrap_or_default();

                let removed_height_without_gaps = calc_removed(
                    |i| {
                        elms.get_with_index(i as u32)
                            .map(|v| v.client_height() as f64)
                    },
                    cursor,
                    is_top,
                );

                let current_data = crop_data(current_data, is_top, cursor);
                let current_data = merge_data(current_data, new_data, is_top);
                trace!("settings data: {current_data:#?}");
                item_data.set(current_data);
                delayed_scroll.set_value(Some(DelayedScroll {
                    is_top,
                    scroll_height_before: scroll_height,
                    scroll_top_before: scroll_top,
                    elm_last_y: last_y,
                    elm_last_height: last_height,
                    elm_first_y: first_y,
                    elm_first_height: first_height,
                    saved_height_without_gaps: saved_height,
                    saved_height_with_gaps,
                    removed_height_with_gaps,
                    removed_height_without_gaps,
                    cursor,
                    new_elm_count: new_data_len,
                }));
                // delayed_scroll.set_value(Some((is_top, scroll_height, saved_height, new_data_len)));
            };

            fut.await;
            busy.set_value(false);
        });
    };

    let trigger = {
        let get_items = get_items.clone();
        move || {
            get_items(InfiniteStage::Manual);
        }
    };

    // infinite_scroll_ref.add_resize_observer(move |entry, _observer| {
    //     trace!("RESIZINGGGGGG");
    //     let width = entry.content_rect().width() as u32;
    // });

    on_cleanup(move || {
        if let Some(observer) = observer_mutation.get_untracked() {
            observer.disconnect();
        };
        if let Some(observer) = observer_intersection_bottom.get_untracked() {
            observer.disconnect();
        };
        if let Some(observer) = observer_intersection_top.get_untracked() {
            observer.disconnect();
        };
    });

    // Effect::new(move || {
    //     let (Some(observer_intersection_btm), Some(observer_intersection_top)) = (
    //         observer_intersection_bottom.get(),
    //         observer_intersection_top.get(),
    //     ) else {
    //         return;
    //     };
    //
    //     let elms = infinite_scroll_elm.children();
    //     let elms_len = infinite_scroll_elm.child_element_count() as usize;
    //     // let first = item_refs.with(move |v| v.first().cloned());
    //     // let last = item_refs.with(move |v| v.last().cloned());
    //     // let (Some(first), Some(last)) = (first.and_then(|v| v.get()), last.and_then(|v| v.get()))
    //     // else {
    //     //     return;
    //     // };
    //     // trace!(
    //     //     "intersection attached! \n{:?}\n{:?}",
    //     //     first.text_content(),
    //     //     last.text_content()
    //     // );
    //     observer_intersection_top.disconnect();
    //     // observer_intersection_top.observe(&first);
    //
    //     observer_intersection_btm.disconnect();
    //     // observer_intersection_btm.observe(&last);
    // });

    // create observers
    // Effect::new({
    //     let get_items = get_items.clone();
    //     move || {}
    // });

    // init
    let activated_top = StoredValue::new(false);
    let activated_btm = StoredValue::new(false);
    Effect::new(move || {
        // let (Some(infinite_scroll_elm), Some(observer_mutation)) = (
        //     infinite_scroll_ref.get().map(|v| v.into()) as Option<HtmlElement>,
        //     observer_mutation.get(),
        // ) else {
        //     trace!("gallery NOT found");
        //     return;
        // };
        let Some(infinite_scroll_elm): Option<HtmlElement> =
            infinite_scroll_ref.get().map(|v| v.into())
        else {
            trace!("gallery NOT found");
            return;
        };

        let intersection_observer_options = IntersectionObserverInit::new();
        intersection_observer_options.set_threshold(&JsValue::from_f64(0.0));

        let get_items = get_items.clone();
        let new_interception_observer_btm = intersection_observer::new_with_options_raw(
            {
                let get_items = get_items.clone();

                move |entry, _observer| {
                    if delayed_scroll.with_value(|v| v.is_some()) {
                        return;
                    }

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

                    let Some(data) = item_data.with_untracked(|v| v.last().cloned()) else {
                        error!("missing data for item last");
                        return;
                    };
                    trace!("data picked {data:?}");
                    get_items(InfiniteStage::Btm(data));
                }
            },
            &intersection_observer_options,
        );
        observer_intersection_bottom.set(Some(SendWrapper::new(
            new_interception_observer_btm.clone(),
        )));

        let new_interception_observer_top = intersection_observer::new_with_options_raw(
            {
                let get_items = get_items.clone();

                move |entry, _observer| {
                    if delayed_scroll.with_value(|v| v.is_some()) {
                        return;
                    }

                    let Some(entry) = entry.first() else {
                        return;
                    };

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
                    trace!("data picked {data:?}");
                    get_items(InfiniteStage::Top(data));
                }
            },
            &intersection_observer_options,
        );
        observer_intersection_top.set(Some(SendWrapper::new(
            new_interception_observer_top.clone(),
        )));

        let new_mutation_observer = mutation_observer::new_raw(move |a, b| {
            trace!("running mutation");
            let (Some(infinite_scroll_elm),) = (infinite_scroll_ref.get_untracked(),) else {
                return;
            };

            let infinite_scroll_elm: HtmlElement = infinite_scroll_elm.into();
            let elms = infinite_scroll_elm.children();
            let elm_len = infinite_scroll_elm.child_element_count();
            let elm_first = elms.get_with_index(0 as u32);
            let elm_last = elms.get_with_index(elm_len.saturating_sub(1));

            let (Some(elm_first), Some(elm_last)) = (elm_first, elm_last) else {
                return;
            };

            trace!("mutation updating observers");

            new_interception_observer_top.disconnect();
            new_interception_observer_top.observe(&elm_first);

            new_interception_observer_btm.disconnect();
            new_interception_observer_btm.observe(&elm_last);

            let (Some(delayed_scroll_dto),) = (delayed_scroll.get_value(),) else {
                return;
            };

            // if delayed_scroll_dto.cursor == 0 {
            //     return;
            // }

            //
            let scroll_height_current = infinite_scroll_elm.scroll_height() as f64;
            trace!(
                "mutation running scroll fix {delayed_scroll_dto:#?} scroll_height_current({scroll_height_current})"
            );
            //
            //
            // trace!(
            //     "scroll_height_current{scroll_height_current} scroll_height_before{} scroll_height_save{scroll_height_save}", delayed_scroll_dto.scroll_height_before
            // );
            //
            let scroll = if delayed_scroll_dto.is_top {
                let elm_n = elms.get_with_index(delayed_scroll_dto.new_elm_count as u32);
                let (Some(elm_n),) = (elm_n,) else {
                    return;
                };

                let rect_first = elm_first.get_bounding_client_rect();
                let rect_n = elm_n.get_bounding_client_rect();

                let y_first = rect_first.y();
                let y_n = rect_n.y();

                let y_diff = y_n - y_first;
                trace!("y_n({y_n}) - y_first({y_first}) = y_diff({})", y_diff);

                y_diff
            } else if delayed_scroll_dto.saved_height_without_gaps > 0.0 {
                // let elm_n = elms.get_with_index(delayed_scroll_dto.new_elm_count as u32);
                // let (Some(elm_n),) = (elm_n,) else {
                //     return;
                // };
                //
                // let rect_first = elm_first.get_bounding_client_rect();
                // let rect_n = elm_n.get_bounding_client_rect();
                //
                // let y_first = rect_first.y();
                // let y_n = rect_n.y();
                //
                // let y_diff = y_n - y_first;
                // -y_diff
                let elm_n = elms.get_with_index(
                    elm_len.saturating_sub(delayed_scroll_dto.new_elm_count as u32 + 1) as u32,
                );
                let (Some(elm_n),) = (elm_n,) else {
                    return;
                };

                let rect_first = elm_first.get_bounding_client_rect();
                let rect_n = elm_n.get_bounding_client_rect();
                let rect_last = elm_last.get_bounding_client_rect();
                let n_y = rect_n.y();
                // let n_top = rect_n.top();
                let n_height = rect_n.height();
                let last_y = rect_last.y();
                let last_height = rect_last.height();
                let v = elm_n.text_content();

                trace!(
                    "2 n_y({n_y}) n_height({n_height}) last_y({last_y}) last_height({last_height}) {v:?}"
                );

                // let diff = (last_y + last_height) - n_y;
                // -diff
                // let y_first = rect_first.y();
                // let y_n = rect_n.y();
                //
                // let y_diff = y_first - y_n;
                // trace!("y_first({y_first}) - y_n({y_n}) = y_diff({})", y_diff);
                // -(diff - delayed_scroll_dto.removed_height_without_gaps)
                // -delayed_scroll_dto.removed_height_with_gaps

                let y_diff = n_y - delayed_scroll_dto.elm_last_y;
                // let y_diff = delayed_scroll_dto.scroll_height_before - scroll_height_current;
                // trace!(
                //     "{} {} {}",
                //     scroll_height_current,
                //     delayed_scroll_dto.scroll_height_before,
                //     delayed_scroll_dto.removed_height_with_gaps
                // );
                // let y_diff = -(y_diff.abs() - delayed_scroll_dto.removed_height_with_gaps).abs();

                // let y_diff = delayed_scroll_dto.saved_height_with_gaps - delayed_scroll_dto.scroll_height_before;
                // let y_diff = delayed_scroll_dto.removed_height_with_gaps - delayed_scroll_dto.saved_height_with_gaps;
                // let y_diff = delayed_scroll_dto.saved_height - delayed_scroll_dto.scroll_top_before;
                // // let y_diff = scroll_height_current - scroll_height_before;
                // trace!(
                //     // "scroll_height_current({scroll_height_current}) - scroll_height_before({scroll_height_before}) = y_diff({})",
                //     "scroll_height_before({scroll_height_before}) - scroll_height_current({scroll_height_current}) = y_diff({})",
                //     y_diff
                // );
                // infinite_scroll_elm.set_scroll_top((delayed_scroll_dto.scroll_top_before - y_diff) as i32);
                // delayed_scroll.set_value(None);
                // return;
                y_diff
            } else {
                0.0
            };
            //
            trace!("scrolling by {scroll}");
            infinite_scroll_elm.scroll_by_with_x_and_y(0.0, scroll);
            delayed_scroll.set_value(None);
        });
        observer_mutation.set(Some(SendWrapper::new(new_mutation_observer.clone())));
        //

        let options = MutationObserverInit::new();
        options.set_child_list(true);
        options.set_character_data(true);
        options.set_subtree(true);

        let _ = new_mutation_observer
            .observe_with_options(&infinite_scroll_elm, &options)
            .inspect_err(|_| {
                error!("failed to observe");
            });

        trace!("init items");
        get_items(InfiniteStage::Init);
    });

    InfiniteScroll {
        // view: StoredValue::new(Box::new(move || {
        //     // let items = item_views.get();
        //     //
        //     // let view = view! {
        //     //     { items }
        //     // }
        //     // .into_view()
        //     // .into_any();
        //
        //
        //     view
        //     // view
        // })),
        data: item_data,
        trigger: StoredValue::new(Box::new(trigger)),
    }
}

fn crop_view(
    get_elm_height: impl Fn(usize) -> Option<f64>,
    elms_len: usize,
    max_remove: usize,
    is_top: bool,
    height: f64,
    height_mult: f64,
    scroll_height: f64,
) -> Option<(usize, f64)> {
    let expected_scroll_height = height * height_mult;

    trace!("expected scroll {expected_scroll_height} scroll_top {scroll_height}");
    let mut scroll_height_save = 0.0_f64;

    let mut index = if is_top {
        0_usize
    } else {
        elms_len.saturating_sub(1)
    };

    trace!("init index {index}");

    while let Some(height) = get_elm_height(index) {
        if scroll_height_save >= expected_scroll_height
        // || (is_top && index > max_remove)
        // || (!is_top && elms_len.saturating_sub(index) > max_remove)
        {
            trace!("exiting saving loop");
            break;
        }
        trace!("saving {height}");

        scroll_height_save += height;

        if is_top {
            index += 1;
        } else if index > 0 {
            index -= 1;
        } else {
            break;
        }
    }

    if is_top && index < elms_len {
        Some((index, scroll_height_save))
    } else if index > 0 {
        Some((index, scroll_height_save))
    } else {
        None
    }
}

fn calc_removed(
    get_elm_height: impl Fn(usize) -> Option<f64>,
    mut cursor: usize,
    is_top: bool,
) -> f64 {
    let mut removed_height = 0.0_f64;

    // if is_top {
    //     cursor += 1;
    // } else {
    //     cursor = cursor.saturating_sub(1);
    // }

    while let Some(height) = get_elm_height(cursor) {
        removed_height += height;

        if is_top {
            cursor += 1;
        } else if cursor > 0 {
            cursor = cursor - 1;
        } else {
            break;
        }
    }

    removed_height
}

fn crop_data<T>(current_data: impl Into<Vec<T>>, is_top: bool, cursor_position: usize) -> Vec<T> {
    let mut current_data = current_data.into();
    if is_top {
        trace!("draining data {cursor_position}..");
        current_data.drain(cursor_position..);
        current_data
    } else {
        trace!("draining data 0..{cursor_position}");
        current_data.drain(0..=cursor_position);
        current_data
    }
}

fn merge_data<T>(
    current_data: impl Into<Vec<T>>,
    new_data: impl Into<Vec<T>>,
    is_top: bool,
) -> Vec<T> {
    let mut current_data = current_data.into();
    let mut new_data = new_data.into();
    if is_top {
        new_data.extend(current_data);
        new_data
    } else {
        current_data.extend(new_data);
        current_data
    }
}

// fn create_delayed_scroll() -> (bool, f64, f64, usize) {
//     trace!("index({index}) < current_nodes_len({elms_len})");
//     trace!("index({index}) > 0");
//     if is_top {
//         trace!(
//             "set scroll scroll_height({scroll_height}) - scroll_height_save({scroll_height_save}) = {}",
//             scroll_height - scroll_height_save
//         );
//
//         (true, height, scroll_height_save, elms_len as usize)
//     } else {
//         (false, height, scroll_height_save, elms_len as usize)
//     }
// }

#[cfg(test)]
mod use_infinite_scroll_tests {
    use crate::{
        init_test_log,
        view::app::hook::use_infinite_scroll::{calc_removed, crop_data, crop_view, merge_data},
    };

    #[test]
    fn calc_removed_test() {
        init_test_log();
        let heights = [10.0_f64, 20.0, 30.0, 40.0, 50.0];
        let get_height = |i: usize| heights.get(i).cloned();

        let result = calc_removed(get_height, 2, true);
        assert_eq!(result, 30.0 + 40.0 + 50.0);

        let result = calc_removed(get_height, 2, false);
        assert_eq!(result, 10.0 + 20.0 + 30.0);
    }

    #[test]
    fn crop_view_test() {
        init_test_log();
        let heights = [10.0_f64, 10.0, 10.0, 10.0, 10.0];
        let heights_len = heights.len();
        let scroll_height = heights.iter().fold(0.0, |a, b| a + *b);
        let get_height = |i: usize| heights.get(i).cloned();

        let result = crop_view(get_height, heights_len, 2, true, 10.0, 2.0, scroll_height);
        assert_eq!(result, Some((2, 20.0)));

        let result = crop_view(get_height, heights_len, 2, false, 10.0, 2.0, scroll_height);
        assert_eq!(result, Some((2, 20.0)));

        // let result = crop_view(get_height, heights_len, 1, true, 10.0, 2.0, scroll_height);
        // assert_eq!(result, Some((2, 20.0)));
        //
        // let result = crop_view(get_height, heights_len, 1, false, 10.0, 2.0, scroll_height);
        // assert_eq!(result, Some((2, 20.0)));

        // let result = crop_view(get_height, heights_len, 2, true, 10.0, 2.0, scroll_height);
        // assert_eq!(result, Some((2, 20.0)));
        //
        // let result = crop_view(get_height, heights_len, 2, false, 10.0, 2.0, scroll_height);
        // assert_eq!(result, Some((2, 20.0)));
    }

    #[test]
    fn crop_data_test() {
        init_test_log();
        let current_data = [4, 5, 6, 7, 8];
        let new_data = [1, 2, 3];

        let result = crop_data(current_data, true, 2);
        let result = merge_data(result, new_data, true);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);

        let new_data = [9, 10, 11];
        let result = crop_data(current_data, false, 2);
        let result = merge_data(result, new_data, false);
        assert_eq!(result, vec![7, 8, 9, 10, 11]);
    }
}

// trace!("index({index}) < current_nodes_len({elms_len})");
// trace!("index({index}) > 0");
// let delayed_scroll = if is_top && index < elms_len {
//     trace!(
//         "set scroll scroll_height({scroll_height}) - scroll_height_save({scroll_height_save}) = {}",
//         scroll_height - scroll_height_save
//     );
//
//     Some((true, height, scroll_height_save, elms_len as usize))
// } else if index > 0 {
//     Some((false, height, scroll_height_save, elms_len as usize))
// } else {
//     None
// };

// if is_top {
//     trace!(
//         "extending data [new_data({}), current_data({})]",
//         data.len(),
//         current_data.len()
//     );
//     *current_data = [data, current_data.clone()].concat();
// } else {
//     trace!("extending views with {} new data", data.len());
//     current_data.extend(data);
// }

// let item_views_len = item_views.with_untracked(|v| v.len());
// let item_refs_len = item_refs.with_untracked(|v| v.len());
// let item_data_len = item_data.with_untracked(|v| v.len());

// if item_views_len != item_refs_len || item_refs_len != item_data_len {
//     error!(
//         "items buffers missmatch item_views_len({item_views_len}) item_refs_len({item_refs_len}) item_data_len({item_data_len})"
//     );
// }
// item_data.with_untracked(|v| {
//     debug!("item_data: {v:#?}");
// });
