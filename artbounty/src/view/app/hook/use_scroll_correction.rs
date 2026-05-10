use leptos::ev;
use leptos::{ev::EventDescriptor, html::ElementType, prelude::*};
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{Element, Event, HtmlElement, MutationRecord};

use crate::{
    api::{Api, Server404Err, ServerErr},
    path::link_img,
    view::{app::hook::use_mutation::Mutation, toolbox::prelude::*},
};
use tracing::{error, info, trace, trace_span, warn};

#[derive(Clone, Copy)]
pub struct ScrollCorrection
// where
//     F: FnMut(Vec<MutationRecord>, web_sys::MutationObserver) + Clone + 'static,
// where
//     Elm: ElementType,
//     Elm::Output: JsCast + Clone + 'static + Into<HtmlElement>,
{
    // pub mutation_observer: Mutation<Box<dyn FnMut() + 'static>>,
    pub observe: StoredValue<Box<dyn Fn(Element) + 'static>, LocalStorage>,
    pub target: StoredValue<Option<Element>, LocalStorage>,
    pub anchor_first: StoredValue<Option<ScrollState>, LocalStorage>,
    pub anchor_last: StoredValue<Option<ScrollState>, LocalStorage>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ScrollState {
    pub id: String,
    pub client_y: f64,
}

impl ScrollCorrection {
    pub fn new() -> Self {
        let anchor_first = StoredValue::new_local(None::<ScrollState>);
        let anchor_last = StoredValue::new_local(None::<ScrollState>);
        let elm_target = StoredValue::new_local(None::<Element>);

        let mutation_observer = Mutation::new(move |entry, b| {
            let _guard = trace_span!("scroll correction").entered();

            let Some(elm) = entry
                .first()
                .and_then(|v| v.target())
                .and_then(|v| TryInto::<JsValue>::try_into(v).ok())
                .and_then(|v| TryInto::<Element>::try_into(v).ok())
            else {
                warn!("failed to get target");
                return;
            };

            trace!("scroll correction mutated");

            let dom = document();
            // let new_scroll_top = elm.scroll_top() as f64;

            {
                let anchor_first = anchor_first.get_value().and_then(|old_scroll| {
                    dom.get_element_by_id(&old_scroll.id)
                        .inspect(|v| {
                            trace!("first anchor found {old_scroll:?}");
                        })
                        .map(|v| (v, old_scroll))
                });
                let anchor_last = anchor_last.get_value().and_then(|old_scroll| {
                    dom.get_element_by_id(&old_scroll.id)
                        .inspect(|v| {
                            trace!("last anchor found {old_scroll:?}");
                        })
                        .map(|v| (v, old_scroll))
                });
                let anchor = if anchor_first.is_some() {
                    anchor_first
                } else if anchor_last.is_some() {
                    anchor_last
                } else {
                    None
                };

                // let anchor = anchor_first
                //     .get_value()
                //     .map(|old_scroll| {
                //         trace!("trying to find first anchor {}", old_scroll.id);
                //         dom.get_element_by_id(&old_scroll.id)
                //             .inspect(|v| {
                //                 trace!("first anchor found {v:?}");
                //             })
                //             .map(|v| (v, old_scroll))
                //     })
                //     .unwrap_or_else(|| {
                //         anchor_last.get_value().and_then(|old_scroll| {
                //             trace!("trying to find last anchor {}", old_scroll.id);
                //             dom.get_element_by_id(&old_scroll.id)
                //                 .inspect(|v| {
                //                     trace!("last anchor found {v:?}");
                //                 })
                //                 .map(|v| (v, old_scroll))
                //         })
                //     });

                let diff = anchor
                    .map(|(v_new, old_scroll)| {
                        let rect = v_new.get_bounding_client_rect();
                        let y_new = rect.y();
                        let y_diff = y_new - old_scroll.client_y;

                        trace!(
                            "y_diff = new({}) - old({}) = {}",
                            y_new, old_scroll.client_y, y_diff
                        );

                        y_diff
                    })
                    .inspect(|v| {
                        info!("ANCHOR WAS FOUND {v}");
                    })
                    .unwrap_or_else(|| {
                        warn!("NO ANCHORS FOUND");
                        0.0
                    });

                trace!("scrolled byyyyyyyyyy {diff}");
                if diff != 0.0 {
                    elm.scroll_by_with_x_and_y(0.0, diff);
                }

                // let diff = prev_middle
                //     .as_ref()
                //     .inspect(|v| {
                //         trace!("first anchor found {v:?}");
                //     })
                //     .and_then(|old_scroll| {
                //         dom.get_element_by_id(&old_scroll.id)
                //             .map(|v| (v, old_scroll))
                //     })
                //     .map(|(v_new, old_scroll)| {
                //         let rect = v_new.get_bounding_client_rect();
                //         let y_new = rect.y();
                //         // let top_diff = new_scroll_top - old_scroll.scroll_top;
                //         // trace!(
                //         //     "top_diff = new({}) - old({}) = {}",
                //         //     new_scroll_top, old_scroll.scroll_top, top_diff
                //         // );
                //         let y_diff = y_new - old_scroll.client_y;
                //         trace!(
                //             "y_diff = new({}) - old({}) = {}",
                //             y_new, old_scroll.client_y, y_diff
                //         );
                //         // let diff = y_diff - top_diff;
                //         // trace!("diff = new({}) - old({}) = {}", y_diff, top_diff, y_diff);
                //
                //         y_diff
                //     })
                //     .unwrap_or_default();
            }

            {
                let new_first = elm
                    .first_element_child()
                    .map(|v| {
                        let id = v.id();
                        let rect = v.get_bounding_client_rect();
                        let y = rect.y();
                        ScrollState { id, client_y: y }
                    })
                    .inspect(|v| {
                        trace!("first anchor was set {v:?}");
                    });
                let new_last = elm
                    .last_element_child()
                    .map(|v| {
                        let id = v.id();
                        let rect = v.get_bounding_client_rect();
                        let y = rect.y();
                        ScrollState { id, client_y: y }
                    })
                    .inspect(|v| {
                        trace!("last anchor was set {v:?}");
                    });

                anchor_first.set_value(new_first);
                anchor_last.set_value(new_last);
                // let elm_count = elm.child_element_count();
                // let middle_n = elm_count / 2;
                // let new_middle = elm
                //     .children()
                //     .get_with_index(middle_n)
                //     .map(|v| {
                //         let id = v.id();
                //         let rect = v.get_bounding_client_rect();
                //         let y = rect.y();
                //         ScrollState {
                //             id,
                //             client_y: y,
                //             // scroll_top: new_scroll_top,
                //         }
                //     })
                //     .inspect(|v| {
                //         trace!("middle anchor was set {v:?}");
                //     });
                // anchor.set_value(new_middle);
            }

            // let prev_first = first_elm.get_value();
            // let prev_last = last_elm.get_value();

            // let prev_first_exists = prev_first
            //     .as_ref()
            //     .inspect(|(id, y)| {
            //         trace!("first anchor found {id} {y}");
            //     })
            //     .and_then(|(id, y_old)| dom.get_element_by_id(id).map(|v| (v, *y_old)))
            //     .map(|(v_new, y_old)| {
            //         let rect = v_new.get_bounding_client_rect();
            //         let y_new = rect.y();
            //         trace!("first diff {y_new} - {y_old} = {}", y_new - y_old);
            //         y_new - y_old
            //     });
            //
            // let prev_last_exists = prev_last
            //     .as_ref()
            //     .inspect(|(id, y)| {
            //         trace!("last anchor found {id} {y}");
            //     })
            //     .and_then(|(id, y_old)| dom.get_element_by_id(id).map(|v| (v, y_old)))
            //     .map(|(new_elm, y_old)| {
            //         let rect = new_elm.get_bounding_client_rect();
            //         let y_new = rect.y();
            //         trace!("last diff {y_new} - {y_old} = {}", y_new - y_old);
            //         y_new - y_old
            //     });
            //
            // let diff = if let Some(diff) = prev_first_exists {
            //     diff
            // } else if let Some(diff) = prev_last_exists {
            //     diff
            // } else {
            //     0.0
            // };

            // let same_first = prev_first
            //     .as_ref()
            //     .map(|(id_prev, _)| new_first.as_ref().map(|(id_new, _)| id_prev == id_new))
            //     .flatten()
            //     .unwrap_or_default();
            // let same_last = prev_last
            //     .as_ref()
            //     .map(|(elm_prev, _)| new_last.as_ref().map(|(elm_new, _)| elm_prev == elm_new))
            //     .flatten()
            //     .unwrap_or_default();

            // let diff_first = prev_first
            //     .as_ref()
            //     .map(|(_, prev_y)| prev_first_exists.as_ref().map(|(_, new_y)| prev_y - new_y))
            //     .flatten()
            //     .unwrap_or_default();
            // let diff_last = prev_last
            //     .as_ref()
            //     .map(|(_, prev_y)| new_last.as_ref().map(|(_, new_y)| prev_y - new_y))
            //     .flatten()
            //     .unwrap_or_default();

            // let diff = if prev_first_exists {
            //     diff_first
            // } else if prev_last_exists {
            //     diff_last
            // } else {
            //     0.0
            // };
            // let same_last = prev_last == last;

            // let rect_first = elm_first.get_bounding_client_rect();
            // let rect_n = elm_n.get_bounding_client_rect();

            // scrolled top or btm? and diff
            // let (diff, is_btm) = match (prev_first_exists, same_first, prev_last_exists, same_last) {
            //     (true, false, _, _) => (diff_last, false),
            //     _ => (0.0, true),
            // };

            // let new_first = elm
            //     .first_element_child()
            //     .map(|v| {
            //         let id = v.id();
            //         let rect = v.get_bounding_client_rect();
            //         let y = rect.y();
            //         (id, y)
            //     })
            //     .inspect(|(id, y)| {
            //         trace!("first anchor was set {id} {y}");
            //     });
            // let new_last = elm
            //     .last_element_child()
            //     .map(|v| {
            //         let id = v.id();
            //         let rect = v.get_bounding_client_rect();
            //         let y = rect.y();
            //         (id, y)
            //     })
            //     .inspect(|(id, y)| {
            //         trace!("first anchor was set {id} {y}");
            //     });

            // first_elm.set_value(new_first);
            // last_elm.set_value(new_last);
        });

        let observe = move |target: Element| {
            trace!("scroll correction OBSERVE");
            elm_target.set_value(Some(target.clone()));
            mutation_observer.observe_only(
                target.clone(),
                MutationObserverOptions::new()
                    .set_child_list()
                    .set_attributes()
                    .set_child_list()
                    .character_data(),
            );

            // let closure = Closure::<dyn FnMut(_)>::new({
            //     let target = target.clone();
            //     move |v: Event| {
            //         // let new_scroll_top = target.scroll_top() as f64;
            //         let dom = document();
            //         anchor.update_value(|v| {
            //             if let Some(v) = v {
            //                 let Some(new_elm) = document().get_element_by_id(&v.id) else {
            //                     return;
            //                 };
            //                 let rect = new_elm.get_bounding_client_rect();
            //                 let new_y = rect.y();
            //                 trace!(
            //                     "scroll correction updated {} new_y from {} to {}",
            //                     v.id, v.client_y, new_y
            //                 );
            //                 v.client_y = new_y;
            //             }
            //         });
            //         // if let Some(e) = middle_elm.get_value() {
            //         //     let new_data = new_scroll_top;
            //         //
            //         //     //
            //         // }
            //         //
            //     }
            // })
            // .into_js_value();
            // let result = target.add_event_listener_with_callback(
            //     &ev::scroll.name(),
            //     closure.as_ref().unchecked_ref(),
            // );
            // if result.is_err() {
            //     error!("scroll correction, failed to add scroll event listener");
            // }
        };

        // elm.add_mutation_observer();

        Self {
            observe: StoredValue::new_local(Box::new(observe)),
            // old_y: old_y_store,
            target: elm_target,
            anchor_first,
            anchor_last,
        }
    }

    pub fn observe_only(&self, target: impl Into<Element>) {
        let target: Element = target.into();
        self.observe.run(target);
    }

    pub fn update(&self) {
        trace!("scroll correction running update",);
        let fn_update = |v: &mut Option<ScrollState>| {
            if let Some(v) = v {
                let Some(new_elm) = document().get_element_by_id(&v.id) else {
                    return;
                };
                let rect = new_elm.get_bounding_client_rect();
                let new_y = rect.y();
                trace!(
                    "scroll correction updated {} new_y from {} to {}",
                    v.id, v.client_y, new_y
                );
                v.client_y = new_y;
            }
        };
        self.anchor_first.update_value(fn_update);
        self.anchor_last.update_value(fn_update);

        // pub old_y: StoredValue<f64, LocalStorage>,
        // pub elm: NodeRef<Elm>,
        // pub scroll_top: f64,
        // where
        //     F: FnMut(Vec<MutationRecord>, web_sys::MutationObserver) + Clone + 'static,
        // where
        //     Elm: ElementType,
        //     Elm::Output: JsCast + Clone + 'static + Into<HtmlElement>,
        // let old_y_store = StoredValue::new_local(0.0);
        // let first_elm = StoredValue::new_local(None::<(String, f64)>);
        // let last_elm = StoredValue::new_local(None::<(String, f64)>);
        // let old_y = self.old_y;
        // let old_y = old_y.get_value();
        // old_y.set_value(new_y);
        // let target: Element = target.into();
        // self.observe.run(target);
    }
}
