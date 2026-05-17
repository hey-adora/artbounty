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
pub struct ScrollCorrection {
    // pub observe: StoredValue<Box<dyn Fn(Element) + 'static>, LocalStorage>,
    pub target_container: StoreSignal<Option<Element>>,
    pub anchor_first: StoreSignal<Option<ScrollState>>,
    pub anchor_last: StoreSignal<Option<ScrollState>>,
}

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct ScrollState {
    pub id: String,
    pub client_y: f64,
}

impl ScrollCorrection {
    pub fn new() -> Self {
        let anchor_first =
            StoreSignal::new_with_formmater(false, "anchor_first", None::<ScrollState>, |v| {
                serde_json::to_string(v).unwrap_or_else(|e| e.to_string())
            });
        let anchor_last =
            StoreSignal::new_with_formmater(false, "anchor_last", None::<ScrollState>, |v| {
                serde_json::to_string(v).unwrap_or_else(|e| e.to_string())
            });
        let elm_target = StoreSignal::new_with_formmater(
            false,
            "scroll_correction_container",
            None::<Element>,
            |v| {
                v.as_ref()
                    .map(|v| v.id())
                    .unwrap_or_else(|| "null".to_string())
            },
        );

        Self {
            target_container: elm_target,
            anchor_first,
            anchor_last,
        }
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
        self.anchor_first.update_untracked(fn_update);
        self.anchor_last.update_untracked(fn_update);
    }

    pub fn run(&self, container_target: impl AsRef<Element>) {
        let anchor_first = self.anchor_first;
        let anchor_last = self.anchor_last;
        let container_target = container_target.as_ref();
        let dom = document();

        {
            let anchor_first = anchor_first.get_untracked().and_then(|old_scroll| {
                dom.get_element_by_id(&old_scroll.id)
                    .inspect(|v| {
                        trace!("first anchor found {old_scroll:?}");
                    })
                    .map(|v| (v, old_scroll))
            });
            let anchor_last = anchor_last.get_untracked().and_then(|old_scroll| {
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

            let debug_data = anchor
                .as_ref()
                .map(|v| serde_json::to_string(&v.1).unwrap_or_else(|e| e.to_string()))
                .unwrap_or_else(|| String::from("null"));
            debug_data_push("anchor_selected", debug_data);

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
                });

            trace!("scrolled byyyyyyyyyy {diff:?}");
            debug_data_push(
                "scroll_correction",
                diff.map(|v| v.to_string())
                    .unwrap_or_else(|| "null".to_string()),
            );
            if let Some(diff) = diff {
                container_target.scroll_by_with_x_and_y(0.0, diff);
            }
        }

        {
            let new_first = container_target
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
            let new_last = container_target
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

            anchor_first.set_untracked(new_first);
            anchor_last.set_untracked(new_last);
        }
    }

    pub fn reset(&self) {

        debug_data_push("scroll_correction_reset", "null");
        trace!("SCROLL CORRECTION BEFORE {:#?}", self.anchor_first.get_untracked());
        self.anchor_first.set_untracked(None);
        self.anchor_last.set_untracked(None);
        trace!("SCROLL CORRECTION AFTER {:#?}", self.anchor_first.get_untracked());
    }
}
