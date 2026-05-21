pub mod nav {

    use crate::path::{link_home, link_home_search};
    use crate::{
        api::{Api, ApiWeb},
        path::{PATH_LOGIN, PATH_UPLOAD, link_settings, link_user},
        view::{app::GlobalState, toolbox::prelude::*},
    };
    use leptos::{html, prelude::*};
    use leptos_router::hooks::query_signal;
    use log::error;
    use tracing::trace;
    use web_sys::{HtmlDivElement, HtmlInputElement, KeyboardEvent, SubmitEvent};

    #[component]
    pub fn Nav() -> impl IntoView {
        let global_state = expect_context::<GlobalState>();
        let search_input = NodeRef::<html::Div>::new();
        let (get_query_tags, set_query_tags) = query_signal::<String>("tags");
        let navigate = leptos_router::hooks::use_navigate();
        // let search_ref = NodeRef::new();
        let api = ApiWeb::new();
        let logout_or_loading = move || {
            if api.is_pending_tracked() {
                "loading..."
            } else {
                "Logout"
            }
        };
        let acc_username = move || {
            global_state
                .acc
                .with(|v| v.as_ref().map(|v| v.username.clone()))
                .unwrap_or("error".to_string())
        };
        let on_logout = move |e: SubmitEvent| {
            e.prevent_default();

            global_state.logout();
        };

        // let search_fn = move || {
        //     // e.prevent_default();
        //     trace!("search_fn running");
        //     let search_text = search_input
        //         .get_untracked()
        //         .and_then(|v: HtmlDivElement| v.text_content())
        //         .unwrap_or_default();
        //
        //     if search_text.is_empty() {
        //         navigate(&link_home(), Default::default());
        //         // None
        //     } else {
        //         navigate(&link_home_search(search_text), Default::default());
        //         // Some(search_text)
        //     }
        //     // set_query_tags.set(if search_text.is_empty() {
        //     //     None
        //     // } else {
        //     //     Some(search_text)
        //     // });
        //     //
        // };

        // Effect::new(move || {
        //     get_query_tags.get_untracked()
        //     set_query_tags.set(None);
        // });

        // let callback = move || {
        //     use crate::view::{app::GlobalState, toolbox::prelude::*};
        //     // let mut wtf = KILLME.write().unwrap();
        //     // *wtf = true;
        //
        //     // let wtf = KILLME.with(|v| {
        //     //     let mut a = v.write().unwrap();
        //     //     *a = true;
        //     //     *a
        //     // });
        //
        //     // let global_state = expect_context::<GlobalState>();
        //     // let acc = global_state.acc.get_untracked();
        //     tracing::trace!("wowza {}", wtf);
        //     // tracing::trace!("wowza {acc:#?}");
        //     //
        // };

        let on_enter = move |e: KeyboardEvent| {
            let key = e.key();
            trace!("key pressed {key}");
            if key.to_lowercase() != "enter" {
                return;
            }
            e.prevent_default();

            let search_text = search_input
                .get_untracked()
                .and_then(|v: HtmlDivElement| v.text_content())
                .unwrap_or_default();

            if search_text.is_empty() {
                navigate(&link_home(), Default::default());
                // None
            } else {
                navigate(&link_home_search(search_text), Default::default());
                // Some(search_text)
            }
        };

        Effect::new(move || {
            let (Some(search_elm), val): (Option<HtmlDivElement>, Option<String>) =
                (search_input.get(), get_query_tags.get())
            else {
                return;
            };
            if let Some(v) = val {
                search_elm.set_text_content(Some(&v));
            } else {
                search_elm.set_text_content(None);
            }
            // let val = ;
        });

        // TODO set search value from url
        view! {
            <nav class="text-gray-200 flex gap-2 px-4 h-[3rem] items-center justify-between">
                <a id="banner" href="/" class="font-black text-[1.3rem]">
                    "ArtBounty"
                </a>
                // <button on:click=move |_| callback() >"wow"</button>
                <div contenteditable=true
                     id="search"
                     node_ref=search_input
                     on:keydown=on_enter
                     class={move || format!("w-full rounded text-[1rem] px-[0.8rem] py-[0.2rem] text-base05 bg-base01")}>
                     {move || get_query_tags.get()}
                </div>
                // <form class=move||format!("") on:submit=search_fn>
                //     <input id="search" value=move || get_query_tags.get() node_ref=search_ref type="text" placeholder="search tags" class="w-full rounded text-[1rem] px-[0.8rem] py-[0.2rem] text-base05 bg-base01 "/>
                // </form>
                <div class=move||format!("{}", if global_state.acc_pending() { "" } else { "hidden" })>
                    <p>"loading..."</p>
                </div>
                <div class=move||format!("{}", if global_state.is_logged_in().unwrap_or_default() || global_state.acc_pending() { "hidden" } else { "" })>
                    <a href=PATH_LOGIN>"Login"</a>
                </div>
                <div class=move||format!("flex gap-2 {}", if global_state.is_logged_in().unwrap_or_default() { "" } else { "hidden" })>
                    <a href=PATH_UPLOAD>"U"</a>
                    <a href=move|| link_user(acc_username())>{acc_username}</a>
                    <a href=move|| link_settings()>"Settings"</a>
                    <form method="POST" action="" on:submit=on_logout >
                        <input type="submit" value=logout_or_loading class="transition-all duration-300 ease-in hover:font-bold"/>
                    </form>
                </div>
            </nav>
        }
    }
}
pub mod gallery {

    use crate::api::{Api, ApiWeb, UserPost, UserPostFile};
    use crate::path::{link_img, link_post, link_post_with_history};
    // use crate::view::{KILLME, KILLME2};
    use crate::view::app::hook::api_gallery::{GalleryApi, GalleryContainerSize};
    use crate::view::app::hook::use_event_listener::EventListener;
    use crate::view::app::hook::use_intersection::Intersection;
    use crate::view::app::hook::use_intersection_switch::IntersectionSwitch;
    use crate::view::app::hook::use_scroll_correction::ScrollCorrection;
    use crate::view::app::hook::use_spawner::Spawner;
    use crate::view::toolbox::prelude::*;
    // use chrono::Utc;
    use leptos::{ev, html};
    use leptos::{html::Div, prelude::*};
    use leptos_router::hooks::{query_signal, use_params_map, use_query_map};
    use leptos_router::params::Params;
    use serde::Serialize;
    use std::default::Default;
    use std::time::Duration;
    use std::{
        fmt::{Debug, Display},
        rc::Rc,
    };
    use tracing::{debug, error, trace};
    use wasm_bindgen::JsValue;
    use web_sys::{
        Element, HtmlAnchorElement, HtmlDivElement, IntersectionObserver,
        IntersectionObserverEntry, MouseEvent,
    };

    pub fn vec_img_to_string<IMG: ResizableImage + Display>(imgs: &[IMG]) -> String {
        let mut output = String::new();

        for img in imgs {
            output += &format!("{},\n", img);
        }

        output
    }

    // #[wasm_bindgen::prelude::wasm_bindgen]
    // pub fn e2e_test1() -> bool {
    //     use crate::view::{app::GlobalState, toolbox::prelude::*};
    //     let wtf = KILLME.with(|v| {
    //         let a = *v.read().unwrap();
    //
    //         a
    //     });
    //
    //     // let wtf = KILLME.read().unwrap();
    //
    //     // let global_state = expect_context::<GlobalState>();
    //     // let acc = global_state.acc.get_untracked();
    //     trace!("wowza {}", wtf);
    //
    //     false
    // }
    //
    // #[wasm_bindgen::prelude::wasm_bindgen]
    // pub fn e2e_test2() {
    //     KILLME2.set(true);
    // }
    //
    // #[wasm_bindgen::prelude::wasm_bindgen]
    // pub fn e2e_test3() {
    //     let wtf = KILLME2.with_borrow(|v| {
    //
    //         // let v = v.get_mut();
    //         *v
    //     });
    //
    //     trace!("wowza2 {}", wtf);
    // }

    #[component]
    pub fn Gallery(
        #[prop(default = 250)] row_height: u32,
        #[prop(optional)] username: Option<RwSignal<Option<String>>>,
    ) -> impl IntoView {
        let api_top = ApiWeb::new();
        let api_btm = ApiWeb::new();
        let spawner = Spawner::new();
        let scroll_correction = ScrollCorrection::new();
        // let scroll_correction_enabled = StoredValue::new_local(true);
        let gallery_api = GalleryApi::new(api_top, api_btm, scroll_correction.clone());

        // let gallery = RwSignal::<Vec<Img>>::new(Vec::new());
        // let delayed_scroll = StoredValue::new_local(0.0);
        // let delayed_scroll = StoredValueWrap::new("delayed_scroll", 0.0);
        // let on_scroll = EventListener::new(ev::scroll, |v| {
        //     //
        // });
        let gallery_ref = NodeRef::<Div>::new();
        let gallery_initialized = StoredValue::new_local(false);

        let is_top_interector_active = StoredValue::new_local(false);
        let is_down_interector_active = StoredValue::new_local(false);
        let top_intersector_switch = IntersectionSwitch::new();
        let down_intersector_switch = IntersectionSwitch::new();
        let navigate = leptos_router::hooks::use_navigate();
        let (get_query_scroll, set_query_scroll) = query_signal::<i32>("scroll");
        let (get_query_gallery_count, set_query_gallery_count) = query_signal::<usize>("img_count");
        let (get_query_direction, set_query_direction) = query_signal::<String>("direction");
        let (get_query_time, set_query_time) = query_signal::<u128>("time");
        let set_query_time = move |v: Option<u128>| {
            debug_data_push(
                "gallery_query_time",
                v.map(|v| v.to_string())
                    .unwrap_or_else(|| "null".to_string()),
            );
            set_query_time.set(v);
        };
        let old_tags = StoredValue::new_local(String::new());
        let (get_query_tags, set_query_tags) = query_signal::<String>("tags");

        // let get_optimial_img_count = move || {
        //     let Some(gallery_elm) = gallery_ref.try_get_untracked().flatten() else {
        //         return 50;
        //     };
        //
        // };

        let set_gallery = move |width: u32, height: f64, bottom: bool, limit: usize, time: u128| {
            let Some(gallery_elm) = gallery_ref.try_get_untracked().flatten() else {
                return;
            };

            debug_data_push("set_gallery_param_limit", limit.to_string());
            // let width = gallery_elm.client_width() as u32;
            // let height = gallery_elm.client_height() as f64 * 2.0;
            // TODO img count should never be zero
            // let count = (if gallery_api.is_empty() {
            //     get_query_gallery_count.get_untracked()
            // } else {
            //     None
            // })
            // .unwrap_or_else(|| calc_fit_count(width, height, row_height));
            // let time = get_query_time
            //     .get_untracked()
            //     .unwrap_or_else(|| time_now_ns());
            let user_username = username.get_untracked().flatten().unwrap_or_default();

            // let tags = get_query_tags.get_untracked().unwrap_or_default().to_lowercase();
            let tags = get_query_tags.get_untracked().unwrap_or_default();
            trace!("wheres my super suit?");

            // scroll_correction.update();
            spawner.spawn(async move {
                let scroll = gallery_api
                    .fetch_btm_or_top(
                        bottom,
                        limit,
                        GalleryContainerSize {
                            width,
                            height,
                            row_height,
                        },
                        time,
                        tags,
                        user_username,
                    )
                    .await;

                let first_img_time = gallery_api
                    .items
                    .with_untracked(|v| v.first().map(|v| v.created_at));
                let last_img_time = gallery_api
                    .items
                    .with_untracked(|v| v.last().map(|v| v.created_at));
                let imgs_len = gallery_api.items.with_untracked(|v| v.len());

                set_query_direction.set(Some(if bottom { "down" } else { "up" }.to_string()));
                set_query_gallery_count.set(if imgs_len == 0 { None } else { Some(imgs_len) });
                trace!(
                    "GALLERY ITEMS OMG {:#?} \n {first_img_time:?} {last_img_time:?}",
                    gallery_api.items.get_untracked()
                );
                set_query_time(if bottom {
                    first_img_time
                } else {
                    last_img_time
                });
            });
        };

        let set_scroll_top = move |gallery_elm: &HtmlDivElement| {
            let scroll_top = gallery_elm.scroll_top();
            let id = gallery_elm.id();
            trace!("scroll top of {} {}", id, scroll_top);
            debug_data_push("gallery_scroll_set", scroll_top.to_string());
            set_query_scroll.set(if scroll_top == 0 {
                None
            } else {
                Some(scroll_top)
            });
        };

        gallery_ref.add_resize_observer(move |entry, _observer| {
            trace!("RESIZINGGGGGG");
            let Some(entry) = entry.first() else {
                return;
            };
            let width = entry.content_rect().width() as u32;

            let gallery = gallery_api.items;
            let prev_imgs = gallery.get_untracked();
            trace!("stage r1: width:{width} {prev_imgs:#?} ");
            let resized_imgs = resize_v2(prev_imgs, width, row_height);
            trace!("stage r2 {resized_imgs:#?}");
            gallery.set(resized_imgs);
        });

        let intersection_top = Intersection::new(move |entries, b| {
            let (Some(entry), Some(gallery_elm)) = (entries.first(), gallery_ref.get_untracked())
            else {
                return;
            };

            let id = entry.target().id();

            trace!("gallery intersection top 0 {id}");
            let is_enabled = top_intersector_switch.is_enabled(entry.is_intersecting());
            if !is_enabled {
                trace!("gallery intersection top 1 {id}");
                return;
            }
            trace!("gallery intersection top 2 {id}");

            let width = gallery_elm.client_width() as u32;
            let height = gallery_elm.client_height() as f64 * 2.0;
            let limit = calc_fit_count(width, height, row_height);
            let time = time_now_ns();
            // let time = get_query_time.get_untracked().unwrap_or_else(|| time_now_ns());
            debug_data_push("gallery_interval_top_triggered", "null");
            set_gallery(width, height, false, limit, time);
        });

        let intersection_down = Intersection::new(move |entries, b| {
            let (Some(entry), Some(gallery_elm)) = (entries.first(), gallery_ref.get_untracked())
            else {
                return;
            };
            let id = entry.target().id();

            trace!("gallery intersection btm 0 {id}");
            let is_enabled = down_intersector_switch.is_enabled(entry.is_intersecting());
            if !is_enabled {
                trace!("gallery intersection btm 2 {id}");
                return;
            }
            trace!("gallery intersection btm 3 {id}");

            let width = gallery_elm.client_width() as u32;
            let height = gallery_elm.client_height() as f64 * 2.0;
            let limit = calc_fit_count(width, height, row_height);
            let time = time_now_ns();
            // let time = get_query_time.get_untracked().unwrap_or_else(|| time_now_ns());
            debug_data_push("gallery_interval_down_triggered", "null");
            set_gallery(width, height, true, limit, time);
        });

        // let _ = interval::new(
        //     move || {
        //         let Some(gallery_elm) = gallery_ref.get_untracked() else {
        //             trace!("gallery NOT found");
        //             return;
        //         };
        //
        //         let scroll_top = gallery_elm.scroll_top() as u32;
        //         let scroll_height = gallery_elm.scroll_height() as u32;
        //         let height = gallery_elm.client_height() as u32;
        //
        //         if scroll_top < row_height {
        //             trace!("INTERVAL FETCH TOP");
        //             set_gallery(false);
        //         }
        //         if scroll_height.saturating_sub(scroll_top + height) < row_height {
        //             trace!("INTERVAL FETCH BTM");
        //             set_gallery(true);
        //         }
        //     },
        //     Duration::from_secs(2),
        // );

        let _ = interval::new(
            move || {
                let Some(gallery_elm) = gallery_ref.get_untracked() else {
                    trace!("gallery NOT found");
                    return;
                };
                if gallery_api.is_empty() {
                    return;
                }

                set_scroll_top(&gallery_elm);
            },
            Duration::from_millis(500),
        );

        let get_imgs = move || {
            let imgs = gallery_api.items.get();

            imgs.into_iter()
                .enumerate()
                .map({ move |(i, img)| view! {<GalleryImg img />} })
                .collect_view()
        };

        Effect::new(move || {
            trace!("running gallery init");

            let Some(gallery_elm) = gallery_ref.get() else {
                return;
            };

            let width = gallery_elm.client_width() as u32;
            let height = gallery_elm.client_height() as f64 * 2.0;
            let time = get_query_time
                .get_untracked()
                .unwrap_or_else(|| time_now_ns());
            let is_bottom = get_query_direction
                .get_untracked()
                .map(|v| v == "down")
                .unwrap_or(true);

            gallery_api.reset();
            let limit = (if gallery_api.is_empty() {
                get_query_gallery_count.get_untracked()
            } else {
                None
            })
            .unwrap_or_else(|| calc_fit_count(width, height, row_height));

            debug_data_push("gallery_init_executed", "null");
            set_gallery(width, height, is_bottom, limit, time);
            // set_gallery(is_bottom);
        });

        Effect::new(move || {
            let Some(gallery_elm) = gallery_ref.get() else {
                return;
            };
            trace!("running gallery reset");

            // let time = get_query_time.get();
            // get_query_tags.track();
            let direction = get_query_direction.get();
            let count = get_query_gallery_count.get();
            let scroll = get_query_scroll.get();

            let new_tags = get_query_tags.get().unwrap_or_default();
            let old_tags_val = old_tags.get_value();
            let tags_are_same = new_tags == old_tags_val;
            if !tags_are_same {
                old_tags.set_value(new_tags);
            }

            // if !gallery_initialized.get_value() {
            //     return;
            // }

            if (direction.is_some()
                || count.is_some()
                || scroll.is_some()
                || gallery_api.is_empty())
                && tags_are_same
            {
                return;
            }

            gallery_api.reset();
            scroll_correction.reset();
            // scroll_correction_enabled.set_value(false);
            set_query_scroll.set(None);
            top_intersector_switch.reset();
            down_intersector_switch.reset();

            let width = gallery_elm.client_width() as u32;
            let height = gallery_elm.client_height() as f64 * 2.0;
            let limit = calc_fit_count(width, height, row_height);
            let time = time_now_ns();

            debug_data_push("gallery_reset_executed", "null");
            set_gallery(width, height, true, limit, time);
            // set_gallery(true);
        });

        gallery_ref.add_mutation_observer(
            move |entries, observer| {
                debug_data_push("gallery_mutated", "true");
                trace!("IT HAS MUTATED");
                let Some(gallery_elm) = gallery_ref.get_untracked() else {
                    trace!("gallery NOT found");
                    return;
                };

                if gallery_api.is_empty() {
                    return;
                }

                if let Some(scroll) = get_query_scroll.get_untracked()
                    && !gallery_initialized.get_value()
                {
                    debug_data_push("gallery_scroll_read", scroll.to_string());
                    gallery_elm.scroll_by_with_x_and_y(0.0, scroll as f64);
                    gallery_initialized.set_value(true);
                } else {
                    set_scroll_top(&gallery_elm);
                }

                scroll_correction.run(&gallery_elm);
                // if scroll_correction_enabled.get_value() {
                //     scroll_correction.run(gallery_elm.clone());
                // } else {
                //     scroll_correction_enabled.set_value(true);
                // }

                top_intersector_switch.reset();
                down_intersector_switch.reset();
                if let Some(first_elm) = gallery_elm.first_element_child() {
                    trace!("mutation hooked to first elm");
                    intersection_top.observe_only(first_elm);
                } else {
                    trace!("mutation NOT hooked to first elm");
                }

                if let Some(last_elm) = gallery_elm.last_element_child() {
                    trace!("mutation hooked to last elm");
                    intersection_down.observe_only(last_elm);
                } else {
                    trace!("mutation NOT hooked to last elm");
                }
            },
            MutationObserverOptions::new()
                .subtree()
                .set_attributes()
                .set_child_list(), // .character_data()
                                   // ,
        );

        let a = view! {
            <div
                id="gallery"
                node_ref=gallery_ref
                class="relative overflow-y-scroll overflow-x-hidden"
            >
                {
                    get_imgs
                }
            </div>
        };

        a
    }

    pub fn elm_id_img_thumbnail(key: impl Into<String>) -> String {
        format!("{}-thumbnail", key.into())
    }

    pub fn elm_id_img_link(key: impl Into<String>) -> String {
        format!("{}-link", key.into())
    }

    // pub fn GalleryImg<FetchBtmFn, FetchTopFn, OnClickFn>(
    #[component]
    pub fn GalleryImg(
        img: Img,
        // index: usize,
        // total_count: usize,
        // run_on_click: OnClickFn,
        // run_fetch_bottom: FetchBtmFn,
        // run_fetch_top: FetchTopFn,
    ) -> impl IntoView
// where
    //     OnClickFn: Fn(MouseEvent, Img) + Send + Sync + 'static + Clone,
    //     FetchBtmFn: Fn() + Send + Sync + 'static + Clone,
    //     FetchTopFn: Fn() + Send + Sync + 'static + Clone,
    {
        // let img_ref = NodeRef::<html::Img>::new();
        // let link_ref = NodeRef::<html::A>::new();
        //
        // let query = use_query_map();
        // let (get_query_scroll, set_query_scroll) = query_signal::<usize>("s");
        // let activated = StoredValue::new(false);
        //
        // img_ref.add_intersection_observer_with_options(
        //     move |entry, _observer| {
        //         let Some(entry) = entry.first() else {
        //             return;
        //         };
        //         let is_intersecting = entry.is_intersecting();
        //
        //         if !is_intersecting {
        //             activated.set_value(true);
        //             return;
        //         }
        //
        //         if !activated.get_value() {
        //             return;
        //         }
        //
        //         activated.set_value(false);
        //
        //         let elm_on_which_fetches = total_count / 3;
        //         if index == total_count.saturating_sub(elm_on_which_fetches)
        //             || index == total_count.saturating_sub(1)
        //         {
        //             run_fetch_bottom();
        //             trace!("intersection fn last {index} is intesecting: {is_intersecting}");
        //         } else if index == elm_on_which_fetches || index == 0 {
        //             run_fetch_top();
        //             trace!("intersection fn first {index} is intesecting: {is_intersecting}");
        //         }
        //     },
        //     IntersectionOptions::<Div>::default(),
        // );

        // let img_key = img.key;
        let view_left = img.view_pos_x;
        let view_top = img.view_pos_y;
        let view_width = img.view_width;
        let view_height = img.view_height;
        let img_width = img.width;
        let img_height = img.height;
        let img_key = img.key.clone();
        let img_key2 = img.key.clone();
        let img_username = img.username.clone();
        let post_link = img.get_post_link();
        // let post_link_with_history = img.get_post_link_with_history(9999);
        let img_link = img.get_img_link();

        let value_left = format!("{view_left}px");
        let value_top = format!("{view_top}px");
        let value_width = format!("{view_width}px");
        let value_height = format!("{view_height}px");
        let value_width2 = value_width.clone();
        let value_height2 = value_height.clone();

        // let on_img_click = move |e: MouseEvent| {
        //     run_on_click(e, img.clone());
        // };
        // node_ref=link_ref
        // on:click=on_img_click

        view! {

            <a
               id=elm_id_img_link(img_key)
               href=post_link
               class="absolute"
               style:left=value_left
               style:top=value_top
               style:width=value_width
               style:height=value_height
            >
                <img
                    id=elm_id_img_thumbnail(img_key2)
                    style:width=value_width2
                    style:height=value_height2
                    // node_ref=img_ref
                    src=img_link
                />
            </a>
        }
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct Img {
        pub key: String,
        pub username: String,
        pub hash: String,
        pub extension: String,
        pub width: u32,
        pub height: u32,
        pub view_width: f64,
        pub view_height: f64,
        pub view_pos_x: f64,
        pub view_pos_y: f64,
        #[serde(serialize_with = "from_u128_custom")]
        pub created_at: u128,
    }

    fn from_u128_custom<S: serde::Serializer>(v: &u128, serializer: S) -> Result<S::Ok, S::Error> {
        let v = v.to_string();
        v.serialize(serializer)
    }

    impl From<UserPost> for Img {
        fn from(user_post: UserPost) -> Self {
            let post_thumbnail = user_post.file.first().cloned().unwrap_or(UserPostFile {
                width: 400,
                height: 400,
                hash: "404".to_string(),
                extension: "webp".to_string(),
            });
            Self {
                key: user_post.key,
                username: user_post.user.username,
                width: post_thumbnail.width,
                height: post_thumbnail.height,
                hash: post_thumbnail.hash,
                extension: post_thumbnail.extension,
                view_width: 0.0,
                view_height: 0.0,
                view_pos_x: 0.0,
                view_pos_y: 0.0,
                created_at: user_post.created_at,
            }
        }
    }

    impl Display for Img {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "Img::new_full({}, {}, {}, {:.64}, {:.64}, {:.64}, {:.64})",
                // "Img::new_full({}, {}, {}, {:.32}, {:.32}, {:.32}, {:.32})",
                self.key,
                self.width,
                self.height,
                self.view_width,
                self.view_height,
                self.view_pos_x,
                self.view_pos_y
            )
        }
    }

    impl ResizableImage for Img {
        fn get_id(&self) -> String {
            self.key.clone()
        }
        fn get_post_link(&self) -> String {
            link_post(&self.username, &self.key)
        }
        fn get_img_link(&self) -> String {
            link_img(&self.hash, &self.extension)
        }
        fn get_width(&self) -> u32 {
            self.width
        }
        fn get_height(&self) -> u32 {
            self.height
        }
        fn set_size(&mut self, view_width: f64, view_height: f64, pos_x: f64, pos_y: f64) {
            self.view_width = view_width;
            self.view_height = view_height;
            self.view_pos_x = pos_x;
            self.view_pos_y = pos_y;
        }

        fn set_pos_x(&mut self, pos_x: f64) {
            self.view_pos_x = pos_x;
        }
        fn set_pos_y(&mut self, pos_y: f64) {
            self.view_pos_y = pos_y;
        }

        fn get_pos_x(&self) -> f64 {
            self.view_pos_x
        }
        fn get_pos_y(&self) -> f64 {
            self.view_pos_y
        }

        fn get_size(&self) -> (u32, u32) {
            (self.width, self.height)
        }

        fn get_view_height(&self) -> f64 {
            self.view_height
        }
    }

    impl Img {
        pub fn new(width: u32, height: u32) -> Self {
            let id = random_u64();

            Self {
                key: id.to_string(),
                username: "bot".to_string(),
                hash: "404".to_string(),
                extension: "webp".to_string(),
                width,
                height,
                view_width: 0.0,
                view_height: 0.0,
                view_pos_x: 0.0,
                view_pos_y: 0.0,
                created_at: 0,
            }
        }

        pub fn rand(id: String) -> Self {
            let width = random_u32_ranged(500, 1000);
            let height = random_u32_ranged(500, 1000);

            Self {
                key: id,
                username: "bot".to_string(),
                hash: "404".to_string(),
                extension: "webp".to_string(),
                width,
                height,
                view_width: 0.0,
                view_height: 0.0,
                view_pos_x: 0.0,
                view_pos_y: 0.0,
                created_at: 0,
            }
        }

        pub fn rand_vec(n: usize) -> Vec<Self> {
            let mut output = Vec::new();
            for i in 0..n {
                output.push(Img::rand(i.to_string()));
            }
            output
        }
    }

    pub trait ResizableImage {
        fn get_id(&self) -> String;
        fn get_post_link(&self) -> String;
        fn get_img_link(&self) -> String;
        fn get_width(&self) -> u32;
        fn get_height(&self) -> u32;
        fn get_size(&self) -> (u32, u32);
        fn get_pos_y(&self) -> f64;
        fn get_pos_x(&self) -> f64;
        fn get_view_height(&self) -> f64;
        fn set_size(&mut self, view_width: f64, view_height: f64, pos_x: f64, pos_y: f64);
        fn set_pos_x(&mut self, pos_x: f64);
        fn set_pos_y(&mut self, pos_y: f64);
        fn scaled_by_height(&self, scaled_height: u32) -> (f64, f64) {
            let (width, height) = self.get_size();
            let ratio = width as f64 / height as f64;
            let scaled_w = width as f64 - (height.saturating_sub(scaled_height) as f64 * ratio);
            (scaled_w, ratio)
        }
    }

    pub fn resize_v2<IMG>(mut imgs: Vec<IMG>, width: u32, row_height: u32) -> Vec<IMG>
    where
        IMG: ResizableImage + Clone + Display + Debug,
    {
        let rows = get_rows_to_bottom(&imgs, 0, width, row_height);
        trace!("rows inbetween: {rows:#?}");
        set_rows_to_bottom(&mut imgs, &rows, width);
        imgs
    }

    pub fn get_total_height<IMG>(imgs: &[IMG]) -> f64
    where
        IMG: ResizableImage + Clone + Display + Debug,
    {
        imgs.last()
            .map(|img| img.get_pos_y() + img.get_view_height())
            .unwrap_or_default()
    }

    pub fn add_imgs_to_bottom<IMG>(
        mut imgs: Vec<IMG>,
        new_imgs: Vec<IMG>,
        width: u32,
        heigth: f64,
        row_height: u32,
    ) -> (Vec<IMG>, f64)
    where
        IMG: ResizableImage + Clone + Display + Debug,
    {
        trace!(
            "INPUT FOR ADD IMGS TO BOTTOM {} x {} x {}\nold_imgs: {}\nnew_imgs: {}",
            width,
            heigth,
            row_height,
            vec_img_to_string(&imgs),
            vec_img_to_string(&new_imgs)
        );
        if new_imgs.is_empty() {
            return (imgs, 0.0);
        }
        let height_before_remove = get_total_height(&imgs);
        trace!("stage 0(KOKheight_before_remove: {height_before_remove}): {imgs:#?}");
        if let Some(cut_index) = remove_until_fit_from_top(&mut imgs, heigth) {
            imgs = imgs[cut_index..].to_vec();
            trace!("stage 1 ({cut_index}): {imgs:#?}");
        }
        normalize_imgs_y_v2(&mut imgs);
        let height_after_remove = get_total_height(&imgs);
        trace!(
            "stage 2(KOKheight_before_remove: {height_before_remove}, height_after_remove: {height_after_remove}): {imgs:#?}"
        );
        let offset = imgs
            .len()
            .checked_sub(1)
            .map(|offset| get_row_start(&mut imgs, offset))
            .unwrap_or_default();
        imgs.extend(new_imgs);
        trace!("stage 4: {imgs:#?}");
        let rows = get_rows_to_bottom(&imgs, offset, width, row_height);
        set_rows_to_bottom(&mut imgs, &rows, width);
        let height_final = get_total_height(&imgs);
        let scroll_by = height_after_remove - height_before_remove;
        trace!(
            "stage 5(KOKheight_before_remove: {height_before_remove}, height_after_remove: {height_after_remove}, height_final: {height_final}, scroll_by: {scroll_by}): {imgs:#?}"
        );
        trace!(
            "INPUT FOR ADD IMGS TO BOTTOM OUTPUT {}\n{}",
            scroll_by,
            vec_img_to_string(&imgs),
        );

        (imgs, scroll_by)
    }

    pub fn add_imgs_to_top<IMG>(
        mut old_imgs: Vec<IMG>,
        mut new_imgs: Vec<IMG>,
        width: u32,
        heigth: f64,
        row_height: u32,
    ) -> (Vec<IMG>, f64)
    where
        IMG: ResizableImage + Clone + Display + Debug,
    {
        trace!(
            "INPUT FOR ADD IMGS TO TOP {} x {} x {}\nold_imgs: {}\nnew_imgs: {}",
            width,
            heigth,
            row_height,
            vec_img_to_string(&old_imgs),
            vec_img_to_string(&new_imgs)
        );
        if new_imgs.is_empty() {
            return (old_imgs, 0.0);
        }
        let height_before_remove = get_total_height(&old_imgs);
        trace!("stage 0: {old_imgs:#?}");
        if let Some(cut_index) = remove_until_fit_from_bottom(&mut old_imgs, heigth) {
            old_imgs = old_imgs[..cut_index].to_vec();
            trace!("stage 1 ({cut_index}): {old_imgs:#?}");
        }

        let height_after_remove = get_total_height(&old_imgs);

        // 2 because .len() twice adds 2
        let offset = (old_imgs.len() + new_imgs.len()).saturating_sub(2);

        // let Some(offset) = old_imgs
        //     .len()
        //     .checked_add(new_imgs.len())
        //     .and_then(|v| v.checked_sub(2))
        // else {
        //     trace!("returning old imgs {old_imgs:#?}");
        //     return (old_imgs, 0.0);
        // };

        new_imgs.extend(old_imgs);
        old_imgs = new_imgs;

        let offset = get_row_end(&mut old_imgs, offset);
        trace!("stage 4(offset: {offset}): {old_imgs:#?}");
        let rows = get_rows_to_top(&old_imgs, offset, width, row_height);
        set_rows_to_top(&mut old_imgs, &rows, width);
        trace!("stage 2: {old_imgs:#?}");
        normalize_imgs_y_v2(&mut old_imgs);
        let height_final = get_total_height(&old_imgs);
        let scroll_by = height_final - height_after_remove;

        trace!(
            "stage 5(KOKheight_before_remove: {height_before_remove}, height_after_remove: {height_after_remove}, height_final: {height_final}, scroll_by: {scroll_by}): {old_imgs:#?}"
        );
        trace!(
            "INPUT FOR ADD IMGS TO TOP OUTPUT {}\n{}",
            scroll_by,
            vec_img_to_string(&old_imgs),
        );
        (old_imgs, scroll_by)
    }

    pub fn normalize_imgs_y_v2<IMG>(imgs: &mut [IMG])
    where
        IMG: ResizableImage,
    {
        if let Some(y) = imgs
            .first()
            .map(|img| img.get_pos_y())
            .and_then(|y| if y == 0.0 { None } else { Some(y) })
        {
            imgs.iter_mut().for_each(|img| {
                img.set_pos_y(img.get_pos_y() - y);
            })
        }
    }

    pub fn normalize_y<IMG>(imgs: &mut [IMG])
    where
        IMG: ResizableImage,
    {
        let Some((first_y, first_height)) =
            imgs.first().map(|v| (v.get_pos_y(), v.get_view_height()))
        else {
            return;
        };
        let needs_normalizing = first_y < 0.0;
        if !needs_normalizing {
            return;
        }

        let mut prev_y: f64 = first_y;
        let mut prev_height: f64 = first_height;
        let mut offset_y: f64 = 0.0;

        for img in imgs {
            let current_y = img.get_pos_y();

            if current_y != prev_y {
                prev_y = current_y;
                offset_y += prev_height;
                prev_height = img.get_view_height();
            }

            img.set_pos_y(offset_y);
        }
    }

    pub fn remove_until_fit_from_bottom<IMG>(imgs: &[IMG], view_height: f64) -> Option<usize>
    where
        IMG: ResizableImage,
    {
        imgs.iter()
            .rev()
            .map(|img| img.get_pos_y() + img.get_view_height())
            .position(|current_row_height| current_row_height <= view_height)
            .and_then(|i| if i == 0 { None } else { Some(i) })
            .inspect(|i| trace!("remove i: {i}"))
            .and_then(|i| imgs.len().checked_sub(i))
    }

    pub fn remove_until_fit_from_top<IMG>(imgs: &[IMG], view_height: f64) -> Option<usize>
    where
        IMG: ResizableImage,
    {
        imgs.last()
            .map(|v| v.get_pos_y() + v.get_view_height())
            .and_then(|total_height| {
                imgs.iter()
                    .map(|img| img.get_pos_y() + img.get_view_height())
                    .position(|current_row_height| total_height - current_row_height < view_height)
                    .inspect(|i| trace!("remove rev i: {i}"))
                    .and_then(|i| if i == 0 { None } else { Some(i) })
            })
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
    pub struct Row {
        pub aspect_ratio: f64,
        pub total_scaled_width: f64,
        pub total_original_width: u32,
        pub start_at: usize,
        pub end_at: usize,
    }

    impl Row {
        pub fn new(
            start_at: usize,
            end_at: usize,
            aspect_ratio: f64,
            total_original_width: u32,
            total_scaled_width: f64,
        ) -> Self {
            Self {
                start_at,
                end_at,
                aspect_ratio,
                total_original_width,
                total_scaled_width,
            }
        }
    }

    pub fn get_rows_to_bottom(
        imgs: &[impl ResizableImage],
        offset: usize,
        max_width: u32,
        row_height: u32,
    ) -> Vec<Row> {
        imgs.iter()
            .enumerate()
            .skip(offset)
            .inspect(|(i, img)| {
                trace!("i={} img_id={}", i, img.get_id());
            })
            .map(|(i, img)| {
                (
                    i,
                    img.get_id(),
                    img.get_width(),
                    img.scaled_by_height(row_height),
                )
            })
            .fold(
                Vec::<Row>::new(),
                |mut rows, (i, id, img_width, (scaled_width, ratio))| {
                    if ratio.is_infinite() {
                        error!("invalid image: id: {id}");
                        return rows;
                    }
                    if let Some(row) = rows.last_mut() {
                        let img_fits_in_row =
                            row.total_scaled_width + scaled_width <= max_width as f64;
                        // if img_fits_in_row && (row.aspect_ratio + ratio) < 5.0 {
                        if img_fits_in_row {
                            row.aspect_ratio += ratio;
                            row.end_at = i;
                            row.total_original_width += img_width;
                            row.total_scaled_width += scaled_width;
                            return rows;
                        }
                    }
                    rows.push(Row {
                        aspect_ratio: ratio,
                        total_scaled_width: scaled_width,
                        total_original_width: img_width,
                        start_at: i,
                        end_at: i,
                    });
                    rows
                },
            )
    }

    pub fn get_rows_to_top(
        imgs: &[impl ResizableImage],
        offset: usize,
        max_width: u32,
        row_height: u32,
    ) -> Vec<Row> {
        imgs.iter()
            .rev()
            .skip(imgs.len().saturating_sub(offset + 1))
            .enumerate()
            .inspect(|(i, img)| {
                trace!("i={} img_id={}", offset.saturating_sub(*i), img.get_id());
            })
            .map(|(i, img)| {
                (
                    offset.saturating_sub(i),
                    img.get_id(),
                    img.get_width(),
                    img.scaled_by_height(row_height),
                )
            })
            .fold(
                Vec::<Row>::new(),
                |mut rows, (i, id, img_width, (scaled_width, ratio))| {
                    if ratio.is_infinite() {
                        error!("invalid image: id: {id}");
                        return rows;
                    }
                    if let Some(row) = rows.last_mut() {
                        let img_fits_in_row =
                            row.total_scaled_width + scaled_width <= max_width as f64;
                        if img_fits_in_row {
                            row.aspect_ratio += ratio;
                            row.start_at = i;
                            row.total_scaled_width += scaled_width;
                            row.total_original_width += img_width;
                            return rows;
                        }
                    }
                    rows.push(Row {
                        aspect_ratio: ratio,
                        total_scaled_width: scaled_width,
                        total_original_width: img_width,
                        start_at: i,
                        end_at: i,
                    });
                    rows
                },
            )
    }

    pub fn set_rows_to_bottom(
        imgs: &mut [impl ResizableImage + Display],
        rows: &[Row],
        max_width: u32,
    ) {
        let mut row_pos_y = rows
            .first()
            .and_then(|row| row.start_at.checked_sub(1))
            .and_then(|i| imgs.get(i))
            .map(|img| img.get_pos_y() + img.get_view_height())
            .unwrap_or_default();
        trace!(
            "row_pos_y: {}, {:#?} {}",
            row_pos_y,
            rows,
            vec_img_to_string(imgs)
        );
        let len = rows.len();
        let last_row_width_is_small = rows
            .last()
            .map(|v| v.total_scaled_width <= max_width as f64)
            .unwrap_or(false);

        let chunks: &[(&[Row], f64)] = if len > 1 && last_row_width_is_small {
            &[
                (&rows[..len - 1], max_width as f64),
                (
                    &rows[len - 1..],
                    rows.last().map(|v| v.total_scaled_width).unwrap(),
                ),
            ]
        } else if last_row_width_is_small {
            &[(rows, rows.last().map(|v| v.total_scaled_width).unwrap())]
        } else {
            &[(rows, max_width as f64)]
        };

        trace!("chunks bottom {chunks:#?}");
        chunks.iter().for_each(|(rows, max_width)| {
            rows.iter().for_each(|row| {
                trace!(
                    "row_height: f64 = max_width as f64 / row.aspect_ratio = {max_width} / {}",
                    row.aspect_ratio
                );
                let row_height: f64 = *max_width / row.aspect_ratio;
                let mut row_pos_x = 0.0;
                imgs[row.start_at..=row.end_at].iter_mut().for_each(|img| {
                    let (width, height) = img.get_size();
                    let new_width = row_height * (width as f64 / height as f64);
                    img.set_size(new_width, row_height, row_pos_x, row_pos_y);
                    row_pos_x += new_width;
                });
                trace!("row_pos_y += row_height = {row_pos_y} += {row_height}");
                row_pos_y += row_height;
            });
        });
    }

    pub fn set_rows_to_top(imgs: &mut [impl ResizableImage], rows: &[Row], max_width: u32) {
        let mut row_pos_y = rows
            .first()
            .and_then(|row| row.end_at.checked_add(1))
            .and_then(|i| imgs.get(i))
            .map(|img| img.get_pos_y())
            .unwrap_or(0.0);
        trace!("row_pos_y: {}, {:#?}", row_pos_y, rows);

        let len = rows.len();
        let last_row_width_is_small = rows
            .last()
            .map(|v| v.total_scaled_width <= max_width as f64)
            .unwrap_or(false);

        let chunks: &[(&[Row], f64)] = if len > 1 && last_row_width_is_small {
            &[
                (&rows[..len - 1], max_width as f64),
                (
                    &rows[len - 1..],
                    rows.last().map(|v| v.total_scaled_width).unwrap(),
                ),
            ]
        } else if last_row_width_is_small {
            &[(rows, rows.last().map(|v| v.total_scaled_width).unwrap())]
        } else {
            &[(rows, max_width as f64)]
        };
        debug!("DEBUGGGGGGGGGGGGGGGG {chunks:#?}");
        trace!("chunks top {chunks:#?}");

        chunks.iter().for_each(|(rows, max_width)| {
            rows.iter().for_each(|row| {
                let row_height: f64 = max_width / row.aspect_ratio;
                let mut row_pos_x = 0.0;
                row_pos_y -= row_height;
                imgs[row.start_at..=row.end_at].iter_mut().for_each(|img| {
                    let (width, height) = img.get_size();
                    let new_width = row_height * (width as f64 / height as f64);
                    img.set_size(new_width, row_height, row_pos_x, row_pos_y);
                    row_pos_x += new_width;
                });
            });
        });
    }

    pub fn get_row_start_or_end(imgs: &[impl ResizableImage], offset: usize, rev: bool) -> usize {
        if rev {
            get_row_start(imgs, offset)
        } else {
            get_row_end(imgs, offset)
        }
    }

    pub fn get_row_end(imgs: &[impl ResizableImage], offset: usize) -> usize {
        imgs.iter()
            .skip(offset)
            .position(|img| img.get_pos_y() != imgs[offset].get_pos_y())
            .map(|i| i + offset)
            .unwrap_or_else(|| imgs.len())
            .saturating_sub(1)
    }

    pub fn get_row_start(imgs: &[impl ResizableImage], offset: usize) -> usize {
        imgs.iter()
            .rev()
            .skip(imgs.len().saturating_sub(offset + 1))
            .position(|img| img.get_pos_y() != imgs[offset].get_pos_y())
            .map(|i| offset.saturating_sub(i) + 1)
            .unwrap_or_default()
    }

    pub fn calc_fit_count(width: u32, height: f64, row_height: u32) -> usize {
        ((width * height as u32) / (row_height * row_height)) as usize * 2
    }

    #[cfg(test)]
    mod resize_tests {

        use crate::view::app::components::gallery::{get_total_height, vec_img_to_string};

        use super::{
            Row, add_imgs_to_bottom, add_imgs_to_top, get_row_end, get_row_start,
            get_rows_to_bottom, get_rows_to_top, normalize_imgs_y_v2, remove_until_fit_from_bottom,
            remove_until_fit_from_top, resize_v2, set_rows_to_bottom, set_rows_to_top,
        };
        use std::fmt::Display;
        use tracing::trace;

        use wasm_bindgen_test::*;

        use super::ResizableImage;

        wasm_bindgen_test_configure!(run_in_browser);

        #[derive(Debug, Clone)]
        struct Img {
            pub id: String,
            pub width: u32,
            pub height: u32,
            pub view_width: f64,
            pub view_height: f64,
            pub view_pos_x: f64,
            pub view_pos_y: f64,
        }

        impl PartialEq for Img {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
                    && self.width == other.width
                    && self.height == other.height
                    && self.view_width == other.view_width
                    && self.view_height == other.view_height
                    && self.view_pos_x == other.view_pos_x
                    && self.view_pos_y == other.view_pos_y
            }
        }

        impl Display for Img {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "Img::new_full({}, {}, {}, {:.64}, {:.64}, {:.64}, {:.64})",
                    // "Img::new_full({}, {}, {}, {:.32}, {:.32}, {:.32}, {:.32})",
                    self.id,
                    self.width,
                    self.height,
                    self.view_width,
                    self.view_height,
                    self.view_pos_x,
                    self.view_pos_y
                )
            }
        }

        impl Img {
            pub fn new(id: usize, width: u32, height: u32) -> Self {
                Self {
                    id: id.to_string(),
                    width,
                    height,
                    view_width: 0.0_f64,
                    view_height: 0.0_f64,
                    view_pos_x: 0.0_f64,
                    view_pos_y: 0.0_f64,
                }
            }

            pub fn new_full(
                id: usize,
                width: u32,
                height: u32,
                view_width: f64,
                view_height: f64,
                view_pos_x: f64,
                view_pos_y: f64,
            ) -> Self {
                Self {
                    id: id.to_string(),
                    width,
                    height,
                    view_width: view_width,
                    view_height: view_height,
                    view_pos_x: view_pos_x,
                    view_pos_y: view_pos_y,
                }
            }
        }

        impl ResizableImage for Img {
            fn get_id(&self) -> String {
                self.id.clone()
            }
            fn get_post_link(&self) -> String {
                "test".to_string()
            }
            fn get_img_link(&self) -> String {
                "test".to_string()
            }
            fn get_width(&self) -> u32 {
                self.width
            }
            fn get_height(&self) -> u32 {
                self.height
            }
            fn get_size(&self) -> (u32, u32) {
                (self.width, self.height)
            }
            fn get_pos_x(&self) -> f64 {
                self.view_pos_x
            }
            fn get_pos_y(&self) -> f64 {
                self.view_pos_y
            }
            fn get_view_height(&self) -> f64 {
                self.view_height
            }
            fn set_size(&mut self, view_width: f64, view_height: f64, pos_x: f64, pos_y: f64) {
                self.view_width = view_width;
                self.view_height = view_height;
                self.view_pos_x = pos_x;
                self.view_pos_y = pos_y;
            }
            fn set_pos_x(&mut self, pos_x: f64) {
                self.view_pos_x = pos_x;
            }
            fn set_pos_y(&mut self, pos_y: f64) {
                self.view_pos_y = pos_y;
            }
        }

        #[test]
        fn test_get_rows_forward() {
            crate::init_test_log();

            let imgs = Vec::from([
                //row 0
                Img::new(0, 1000, 500),
                //row 1
                Img::new(1, 500, 500),
                Img::new(2, 500, 500),
                //row 2
                Img::new(3, 500, 500),
            ]);
            let rows = get_rows_to_bottom(&imgs, 0, 1000, 500);
            let expected_rows = Vec::from([
                Row::new(0, 0, 2.0, 1000, 1000.0),
                Row::new(1, 2, 2.0, 1000, 1000.0),
                Row::new(3, 3, 1.0, 500, 500.0),
            ]);
            assert_eq!(expected_rows, rows);

            let rows = get_rows_to_bottom(&imgs, 1, 1000, 500);
            trace!("{:#?}", rows);
            let expected_rows = Vec::from([
                Row::new(1, 2, 2.0, 1000, 1000.0),
                Row::new(3, 3, 1.0, 500, 500.0),
            ]);
            assert_eq!(expected_rows, rows);
        }

        #[test]
        fn test_get_rows_rev() {
            crate::init_test_log();

            let imgs = Vec::from([
                //row 0
                Img::new(0, 500, 500),
                //row 1
                Img::new(1, 500, 500),
                Img::new(2, 500, 500),
                //row 2
                Img::new(3, 1000, 500),
            ]);
            let rows = get_rows_to_top(&imgs, 2, 1000, 500);
            let expected_imgs = Vec::from([
                Row::new(1, 2, 2.0, 1000, 1000.0),
                Row::new(0, 0, 1.0, 500, 500.0),
            ]);
            assert_eq!(expected_imgs, rows);

            let rows = get_rows_to_top(&imgs, 0, 1000, 500);
            trace!("{:#?}", rows);
            let expected_imgs = Vec::from([Row::new(0, 0, 1.0, 500, 500.0)]);
            assert_eq!(expected_imgs, rows);
        }

        #[test]
        fn test_resize() {
            crate::init_test_log();

            let imgs = Vec::<Img>::from([
                //row 0
            ]);
            let resized_imgs = resize_v2(imgs, 1000, 500);
            trace!("{resized_imgs:#?}");
            // TODO test resize
        }

        #[test]
        fn test_set_rows() {
            crate::init_test_log();

            let rows = Vec::from([
                Row::new(0, 0, 2.0, 1000, 1000.0),
                Row::new(1, 2, 2.0, 1000, 1000.0),
                Row::new(3, 3, 1.0, 500, 500.0),
            ]);

            let mut imgs = Vec::from([
                //row 0
                Img::new(0, 1000, 500),
                //row 1
                Img::new(1, 500, 500),
                Img::new(2, 500, 500),
                //row 2
                Img::new(3, 500, 500),
            ]);

            set_rows_to_bottom(&mut imgs, &rows, 1000);

            let expected_imgs = Vec::from([
                //row 0
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
                //row 1
                Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 500.0),
                //row 2
                Img::new_full(3, 500, 500, 500.0, 500.0, 0.0, 1000.0),
            ]);

            assert_eq!(expected_imgs, imgs);

            let rows = Vec::from([
                Row::new(2, 3, 2.0, 0, 0.0),
                Row::new(4, 4, 2.0, 1000, 1000.0),
            ]);

            let mut imgs = Vec::from([
                //row 0
                Img::new_full(0, 500, 500, 500.0, 500.0, 0.0, 0.0),
                Img::new_full(1, 500, 500, 500.0, 500.0, 500.0, 0.0),
                //row 1
                Img::new(2, 500, 500),
                Img::new(3, 500, 500),
                //row 2
                Img::new(4, 1000, 500),
            ]);

            let expected_imgs = Vec::from([
                //row 0
                Img::new_full(0, 500, 500, 500.0, 500.0, 0.0, 0.0),
                Img::new_full(1, 500, 500, 500.0, 500.0, 500.0, 0.0),
                //row 1
                Img::new_full(2, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(3, 500, 500, 500.0, 500.0, 500.0, 500.0),
                //row 2
                Img::new_full(4, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
            ]);

            set_rows_to_bottom(&mut imgs, &rows, 1000);
            assert_eq!(expected_imgs, imgs);

            let rows = Vec::from([
                Row::new(0, 1, 2.0, 1000, 1000.0),
                Row::new(2, 2, 2.0, 1000, 1000.0),
            ]);

            let mut imgs = Vec::from([
                //row 0
                Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 0.0),
                Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 0.0),
                //row 1
                Img::new(3, 1000, 500),
            ]);

            let expected_imgs = Vec::from([
                //row 0
                Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 0.0),
                Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 0.0),
                //row 2
                Img::new_full(3, 1000, 500, 1000.0, 500.0, 0.0, 500.0),
            ]);

            set_rows_to_bottom(&mut imgs, &rows, 1000);
            assert_eq!(expected_imgs, imgs);

            let rows = Vec::from([
                Row::new(1, 2, 2.0, 1000, 1000.0),
                Row::new(0, 0, 2.0, 1000, 1000.0),
            ]);

            let mut imgs = Vec::from([
                //row 0
                Img::new(0, 1000, 500),
                //row 1
                Img::new(0, 500, 500),
                Img::new(1, 500, 500),
                //row 2
                Img::new_full(2, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(3, 500, 500, 500.0, 500.0, 500.0, 500.0),
                //row 3
                Img::new_full(4, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
            ]);

            let expected_imgs = Vec::from([
                //row 0
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, -500.0),
                //row 1
                Img::new_full(0, 500, 500, 500.0, 500.0, 0.0, 0.0),
                Img::new_full(1, 500, 500, 500.0, 500.0, 500.0, 0.0),
                //row 2
                Img::new_full(2, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(3, 500, 500, 500.0, 500.0, 500.0, 500.0),
                //row 3
                Img::new_full(4, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
            ]);

            set_rows_to_top(&mut imgs, &rows, 1000);
            assert_eq!(expected_imgs, imgs);

            let rows = Vec::from([
                Row::new(3, 4, 2.0, 1000, 1000.0),
                Row::new(2, 2, 2.0, 1000, 1000.0),
                Row::new(0, 1, 2.0, 1000, 1000.0),
            ]);

            let mut imgs = Vec::from([
                //row 0
                Img::new(0, 500, 500),
                Img::new(1, 500, 500),
                //row 1
                Img::new(0, 1000, 500),
                //row 2
                Img::new(0, 500, 500),
                Img::new(1, 500, 500),
                //row 3
                Img::new_full(2, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(3, 500, 500, 500.0, 500.0, 500.0, 500.0),
                //row 4
                Img::new_full(4, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
            ]);

            let expected_imgs = Vec::from([
                //row 0
                Img::new_full(0, 500, 500, 500.0, 500.0, 0.0, -1000.0),
                Img::new_full(1, 500, 500, 500.0, 500.0, 500.0, -1000.0),
                //row 1
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, -500.0),
                //row 2
                Img::new_full(0, 500, 500, 500.0, 500.0, 0.0, 0.0),
                Img::new_full(1, 500, 500, 500.0, 500.0, 500.0, 0.0),
                //row 3
                Img::new_full(2, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(3, 500, 500, 500.0, 500.0, 500.0, 500.0),
                //row 4
                Img::new_full(4, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
            ]);

            set_rows_to_top(&mut imgs, &rows, 1000);
            assert_eq!(expected_imgs, imgs);
        }

        #[test]
        fn test_get_row_end() {
            crate::init_test_log();

            let imgs = Vec::from([
                //row 0
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
                //row 1
                Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 500.0),
                //row 2
                Img::new_full(3, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
            ]);
            assert_eq!(
                [0, 2, 2, 3, 0],
                [
                    get_row_end(&imgs, 0),
                    get_row_end(&imgs, 1),
                    get_row_end(&imgs, 2),
                    get_row_end(&imgs, 3),
                    get_row_end(&([] as [Img; 0]), 0),
                ]
            );
        }

        #[test]
        fn test_get_row_start() {
            crate::init_test_log();

            let imgs = Vec::from([
                //row 0
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
                //row 1
                Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 500.0),
                //row 2
                Img::new_full(3, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
            ]);
            let a = [0, 1, 1, 3, 0];
            let b = [
                get_row_start(&imgs, 0),
                get_row_start(&imgs, 1),
                get_row_start(&imgs, 2),
                get_row_start(&imgs, 3),
                get_row_start(&([] as [Img; 0]), 0),
            ];
            trace!("{:#?} {:#?}", a, b);
            assert_eq!(a, b);
        }

        #[test]
        fn test_add_imgs() {
            crate::init_test_log();

            // TODO, ASSET SCROLL_BY
            trace!("=======UPDATING IMGS=======");
            let imgs = Vec::from([]);
            let new_imgs = Vec::from([Img::new(4, 500, 500)]);
            let expected_imgs = Vec::from([
                //row 1
                Img::new_full(4, 500, 500, 500.0, 500.0, 0.0, 0.0),
            ]);
            let (imgs, scroll_by) = add_imgs_to_top(imgs, new_imgs, 1000, 500.0, 500);
            trace!(scroll_by);
            assert_eq!(expected_imgs, imgs);
            trace!("=======UPDATING IMGS=======");
            let imgs = Vec::from([]);
            let new_imgs = Vec::from([
                Img::new(4, 500, 500),
                Img::new(5, 500, 500),
                Img::new(6, 500, 500),
                Img::new(7, 500, 500),
            ]);
            let expected_imgs = Vec::from([
                //row 1
                Img::new_full(4, 500, 500, 500.0, 500.0, 0.0, 0.0),
                Img::new_full(5, 500, 500, 500.0, 500.0, 500.0, 0.0),
                //row 2
                Img::new_full(6, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(7, 500, 500, 500.0, 500.0, 500.0, 500.0),
            ]);
            let (imgs, scroll_by) = add_imgs_to_top(imgs, new_imgs, 1000, 500.0, 500);
            trace!(scroll_by);
            assert_eq!(expected_imgs, imgs);
            trace!("=======UPDATING IMGS=======");
            let imgs = Vec::from([
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
                Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 500.0),
                Img::new_full(3, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
            ]);
            let new_imgs = Vec::from([
                Img::new(4, 500, 500),
                Img::new(5, 500, 500),
                Img::new(6, 500, 500),
                Img::new(7, 500, 500),
            ]);
            let expected_imgs = Vec::from([
                //row 0
                Img::new_full(3, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
                //row 1
                Img::new_full(4, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(5, 500, 500, 500.0, 500.0, 500.0, 500.0),
                //row 2
                Img::new_full(6, 500, 500, 500.0, 500.0, 0.0, 1000.0),
                Img::new_full(7, 500, 500, 500.0, 500.0, 500.0, 1000.0),
            ]);
            let (imgs, scroll_by) = add_imgs_to_bottom(imgs, new_imgs, 1000, 500.0, 500);
            trace!(scroll_by);
            assert_eq!(expected_imgs, imgs);
            trace!("=======UPDATING IMGS=======");
            let imgs = Vec::from([
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
                Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 500.0),
                Img::new_full(3, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
            ]);
            let new_imgs = Vec::from([
                Img::new(4, 500, 500),
                Img::new(5, 500, 500),
                Img::new(6, 500, 500),
                Img::new(7, 500, 500),
            ]);
            let expected_imgs = Vec::from([
                //row 0
                Img::new_full(4, 500, 500, 500.0, 500.0, 0.0, 0.0),
                Img::new_full(5, 500, 500, 500.0, 500.0, 500.0, 0.0),
                //row 1
                Img::new_full(6, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(7, 500, 500, 500.0, 500.0, 500.0, 500.0),
                //row 2
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
            ]);
            let (imgs, scroll_by) = add_imgs_to_top(imgs, new_imgs, 1000, 500.0, 500);
            trace!(scroll_by);
            assert_eq!(expected_imgs, imgs);
        }

        #[test]
        fn img_remove() {
            crate::init_test_log();

            trace!("=================");
            let mut imgs = [
                //row 0
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
                //row 1
                Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 500.0),
                //row 2
                Img::new_full(3, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
            ];

            let cut_index = remove_until_fit_from_bottom(&mut imgs, 500.0);
            assert_eq!(Some(1), cut_index);

            trace!("=================");
            let mut imgs = [
                //row 0
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
                //row 1
                Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 500.0),
            ];

            let cut_index = remove_until_fit_from_bottom(&mut imgs, 500.0);
            assert_eq!(Some(1), cut_index);

            trace!("=================");
            let mut imgs = [
                //row 0
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
                //row 1
                Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 500.0),
            ];

            let cut_index = remove_until_fit_from_bottom(&mut imgs, 1000.0);
            assert_eq!(None, cut_index);

            trace!("=================");
            let mut imgs = [
                //row 0
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
            ];

            let cut_index = remove_until_fit_from_bottom(&mut imgs, 1000.0);
            assert_eq!(None, cut_index);

            trace!("=================");
            let mut imgs = [
                //row 0
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
                //row 1
                Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 500.0),
            ];

            let cut_index = remove_until_fit_from_bottom(&mut imgs, 1000.0);
            assert_eq!(None, cut_index);

            trace!("=================");
            let mut imgs = [
                //row 0
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
                //row 1
                Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 500.0),
                //row 2
                Img::new_full(3, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
            ];

            let cut_index = remove_until_fit_from_top(&mut imgs, 500.0);
            assert_eq!(Some(3), cut_index);

            trace!("=================");
            let mut imgs = [
                //row 0
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
                //row 1
                Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 500.0),
            ];

            let cut_index = remove_until_fit_from_top(&mut imgs, 500.0);
            assert_eq!(Some(1), cut_index);

            trace!("=================");
            let mut imgs = [
                //row 0
                Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
                //row 1
                Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 500.0),
            ];

            let cut_index = remove_until_fit_from_top(&mut imgs, 1000.0);
            assert_eq!(None, cut_index);
        }

        #[test]
        fn test_normalize_negative() {
            crate::init_test_log();

            trace!("=================");
            let mut imgs = Vec::from([
                //row 0
                Img::new_full(0, 500, 500, 500.0, 500.0, 0.0, -1000.0),
                Img::new_full(1, 500, 500, 500.0, 500.0, 500.0, -1000.0),
                //row 1
                Img::new_full(2, 1000, 500, 1000.0, 500.0, 0.0, -500.0),
                //row 2
                Img::new_full(3, 500, 500, 500.0, 500.0, 0.0, 0.0),
                Img::new_full(4, 500, 500, 500.0, 500.0, 500.0, 0.0),
                //row 3
                Img::new_full(5, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(6, 500, 500, 500.0, 500.0, 500.0, 500.0),
                //row 4
                Img::new_full(7, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
            ]);
            let expected_imgs = Vec::from([
                //row 0
                Img::new_full(0, 500, 500, 500.0, 500.0, 0.0, 0.0),
                Img::new_full(1, 500, 500, 500.0, 500.0, 500.0, 0.0),
                //row 1
                Img::new_full(2, 1000, 500, 1000.0, 500.0, 0.0, 500.0),
                //row 2
                Img::new_full(3, 500, 500, 500.0, 500.0, 0.0, 1000.0),
                Img::new_full(4, 500, 500, 500.0, 500.0, 500.0, 1000.0),
                //row 3
                Img::new_full(5, 500, 500, 500.0, 500.0, 0.0, 1500.0),
                Img::new_full(6, 500, 500, 500.0, 500.0, 500.0, 1500.0),
                //row 4
                Img::new_full(7, 1000, 500, 1000.0, 500.0, 0.0, 2000.0),
            ]);

            normalize_imgs_y_v2(&mut imgs);
            trace!("img {:#?}", imgs);
            assert_eq!(imgs, expected_imgs);

            let mut imgs = Vec::from([
                //row 2
                Img::new_full(3, 500, 500, 500.0, 500.0, 0.0, 1000.0),
                Img::new_full(4, 500, 500, 500.0, 500.0, 500.0, 1000.0),
                //row 3
                Img::new_full(5, 500, 500, 500.0, 500.0, 0.0, 1500.0),
                Img::new_full(6, 500, 500, 500.0, 500.0, 500.0, 1500.0),
                //row 4
                Img::new_full(7, 1000, 500, 1000.0, 500.0, 0.0, 2000.0),
            ]);
            let expected_imgs = Vec::from([
                //row 2
                Img::new_full(3, 500, 500, 500.0, 500.0, 0.0, 0.0),
                Img::new_full(4, 500, 500, 500.0, 500.0, 500.0, 0.0),
                //row 3
                Img::new_full(5, 500, 500, 500.0, 500.0, 0.0, 500.0),
                Img::new_full(6, 500, 500, 500.0, 500.0, 500.0, 500.0),
                //row 4
                Img::new_full(7, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
            ]);

            normalize_imgs_y_v2(&mut imgs);
            trace!("img {:#?}", imgs);
            assert_eq!(imgs, expected_imgs);
        }
    }
}
