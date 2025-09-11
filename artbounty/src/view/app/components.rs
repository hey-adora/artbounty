pub mod nav {
    use crate::{
        api::{Api, ApiWeb},
        path::PATH_LOGIN,
        view::{app::GlobalState, toolbox::prelude::*},
    };
    use leptos::prelude::*;
    use log::error;
    use web_sys::SubmitEvent;

    #[component]
    pub fn Nav() -> impl IntoView {
        let global_state = expect_context::<GlobalState>();
        let api = ApiWeb::new();
        // let api_logout = controller::auth::route::logout::client.ground();
        // let is_logged_in = move || global_state.acc.with(|v| v.is_some());
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

        view! {
            <nav class="text-gray-200 pb-1 flex gap-2 px-2 py-1 items-center justify-between">
                <a href="/" class="font-black text-[1.3rem]">
                    "ArtBounty"
                </a>
                <div class=move||format!("{}", if global_state.acc_pending() { "" } else { "hidden" })>
                    <p>"loading..."</p>
                </div>
                <div class=move||format!("{}", if global_state.is_logged_in() || global_state.acc_pending() { "hidden" } else { "" })>
                    <a href=PATH_LOGIN>"Login"</a>
                </div>
                <div class=move||format!("flex gap-2 {}", if global_state.is_logged_in() { "" } else { "hidden" })>
                    <a href="/post">"U"</a>
                    <a href=move||format!("/u/{}", acc_username())>{acc_username}</a>
                    <form method="POST" action="" on:submit=on_logout >
                        <input type="submit" value=logout_or_loading class="transition-all duration-300 ease-in hover:font-bold"/>
                    </form>
                </div>
                // <a href="/register" class="">"Register"</a>
            </nav>
        }
    }
}
pub mod gallery {
    use crate::api::{Api, ApiWeb};
    use crate::view::toolbox::prelude::*;
    use chrono::Utc;
    use leptos::html;
    use leptos::{html::Div, prelude::*};
    use std::default::Default;
    use std::time::Duration;
    use std::{
        fmt::{Debug, Display},
        rc::Rc,
    };
    use tracing::{debug, error, trace};

    pub fn vec_img_to_string<IMG: ResizableImage + Display>(imgs: &[IMG]) -> String {
        let mut output = String::new();

        for img in imgs {
            output += &format!("{},\n", img);
        }

        output
    }

    #[component]
    pub fn Gallery(
        // fetch_top: FetchTopFn,
        // fetch_bottom: FetchBtmFn,
        // #[prop(optional)] fetch_init: Option<Rc<dyn Fn(usize) -> Vec<Img> + Send + Sync + 'static>>,
        #[prop(default = 250)] row_height: u32,
    ) -> impl IntoView
// where
    //     FetchBtmFn: (Fn(usize, Img) -> FetchBtmFnFut) + Send + Sync + Clone + 'static,
    //     FetchBtmFnFut: Future<Output = Vec<Img>>  + Send + Sync + 'static,
    //     FetchTopFn: Fn(usize, Img) -> Vec<Img> + Send + Sync + 'static + Clone,
    {
        let gallery = RwSignal::<Vec<Img>>::new(Vec::new());
        let gallery_ref = NodeRef::<Div>::new();
        // let newest_img_hash = StoredValue::new(String::new());
        // let oldest_img_hash = StoredValue::new(String::new());
        // let scroll_offset: StoredValue<f32> = StoredValue::new(0.0_f32);
        // let api_post_get_after = controller::post::route::get_after::client.ground();
        let api_top = ApiWeb::new();
        let api_btm = ApiWeb::new();

        let set_gallery = move |bottom: bool, width: u32, height: f64, time: u128, count: u32| {
            // let gallery = gallery.clone();
            // let gallery_ref = gallery_ref.clone();
            if bottom {
                api_btm.get_posts_older(time, count)
            } else {
                api_top.get_posts_newer(time, count)
            }
            .send_web(move |result| async move {
                match result {
                    Ok(crate::api::ServerRes::Posts(files)) => {
                        let (Some(prev_imgs), Some(gallery_elm)) = (
                            gallery.try_get_untracked(),
                            gallery_ref.try_get_untracked().flatten(),
                        ) else {
                            return;
                        };

                        let new_imgs = files
                            .iter()
                            .map(|post| Img {
                                id: 0,
                                hash: post.hash.clone(),
                                extension: post.extension.clone(),
                                width: post.width,
                                height: post.height,
                                view_width: 0.0,
                                view_height: 0.0,
                                view_pos_x: 0.0,
                                view_pos_y: 0.0,
                                created_at: post.created_at,
                            })
                            .collect::<Vec<Img>>();

                        if new_imgs.is_empty() {
                            trace!("RECEIVED EMPTY");
                            return;
                        }

                        let (resized_imgs, scroll_by) = if bottom {
                            add_imgs_to_bottom(prev_imgs, new_imgs, width, height, row_height)
                        } else {
                            add_imgs_to_top(prev_imgs, new_imgs, width, height, row_height)
                        };
                        gallery.set(resized_imgs);
                        gallery_elm.scroll_by_with_x_and_y(0.0, scroll_by);
                    }
                    Ok(_) => unreachable!(),
                    Err(err) => error!("{}", err.to_string()),
                }
            });
        };


        gallery_ref.add_resize_observer(move |entry, _observer| {
            trace!("RESIZINGGGGGG");
            let width = entry.content_rect().width() as u32;

            let prev_imgs = gallery.get_untracked();
            trace!("stage r1: width:{width} {prev_imgs:#?} ");
            let resized_imgs = resize_v2(prev_imgs, width, row_height);
            trace!("stage r2 {resized_imgs:#?}");
            gallery.set(resized_imgs);
        });

        let run_fetch_top = move || {
            let Some(gallery_elm) = gallery_ref.get_untracked() else {
                trace!("gallery NOT found");
                return;
            };
            if api_top.busy.get_untracked() {
                return;
            }

            trace!("gallery elm found");
            let width = gallery_elm.client_width() as u32;
            let height = gallery_elm.client_height() as f64;
            let is_empty = gallery.with_untracked(|v| v.is_empty());
            let count = calc_fit_count(width, height, row_height) as u32;
            if is_empty || count == 0 {
                return;
            }
            let gallery = gallery.get_untracked();
            let Some(img) = gallery.first() else {
                return;
            };
            // if img.hash == newest_img_hash.get_value() {
            //     trace!("skipping fetching new imgs");
            //     return;
            // }
            // newest_img_hash.set_value(img.hash.clone());
            // let time = Utc::now().timestamp_micros() as u128 * 1000;
            set_gallery(false, width, height * 8.0, img.created_at, count);

            // gallery.set(resized_imgs);
            // gallery_elm.scroll_by_with_x_and_y(0.0, scroll_by);
            // let new_imgs = fetch_top(count, prev_imgs.first().cloned().unwrap());
            // if new_imgs.is_empty() {
            //     return;
            // }
            // let (resized_imgs, scroll_by) =
            //     add_imgs_to_top(prev_imgs, new_imgs, width, heigth * 3.0, row_height);
            // trace!("scroll master: {scroll_by}");
        };

        let run_fetch_bottom = move || {
            // if api_post_get_after.is_pending() {
            //     return;
            // }
            let Some(gallery_elm) = gallery_ref.get_untracked() else {
                trace!("gallery NOT found");
                return;
            };
            if api_btm.busy.get_untracked() {
                return;
            }
            trace!("gallery elm found");
            let width = gallery_elm.client_width() as u32;
            let height = gallery_elm.client_height() as f64;
            let is_empty = gallery.with_untracked(|v| v.is_empty());
            let count = calc_fit_count(width, height, row_height) as u32;
            if is_empty || count == 0 {
                return;
            }
            let gallery = gallery.get_untracked();
            let Some(img) = gallery.last() else {
                return;
            };
            // if img.hash == oldest_img_hash.get_value() {
            //     trace!("skipping fetching older imgs");
            //     return;
            // }
            // oldest_img_hash.set_value(img.hash.clone());
            // last.c

            // let time = Utc::now().timestamp_micros() as u128 * 1000;
            set_gallery(true, width, height * 8.0, img.created_at, count);

            // api.get_posts_after(time, count)
            //     .send_web(move |result| async move {
            //         match result {
            //             Ok(crate::api::ServerRes::Posts(files)) => {
            //                 let Some(prev_imgs) = gallery.try_get_untracked() else {
            //                     return;
            //                 };
            //
            //                 let new_imgs = files
            //                     .iter()
            //                     .map(|post| Img {
            //                         id: 0,
            //                         hash: post.hash.clone(),
            //                         extension: post.extension.clone(),
            //                         width: post.width,
            //                         height: post.height,
            //                         view_width: 0.0,
            //                         view_height: 0.0,
            //                         view_pos_x: 0.0,
            //                         view_pos_y: 0.0,
            //                         created_at: post.created_at,
            //                     })
            //                     .collect::<Vec<Img>>();
            //
            //                 let (resized_imgs, scroll_by) =
            //                     add_imgs_to_bottom(prev_imgs, new_imgs, width, heigth, row_height);
            //                 gallery.set(resized_imgs);
            //                 gallery_elm.scroll_by_with_x_and_y(0.0, scroll_by);
            //             }
            //             Ok(_) => unreachable!(),
            //             Err(err) => error!("{}", err.to_string()),
            //         }
            //     });
            // api_post_get_after.dispatch_and_run(
            //     controller::post::route::get_after::Input { time, limit: count },
            //     move |files| {
            //         let files = files.clone();
            //         async move {
            //             // gallery.set(vec![Img {
            //             //     id: 0,
            //             //     hash: "404".to_string(),
            //             //     extension: "webp".to_string(),
            //             //     width: 300,
            //             //     height: 200,
            //             //     view_width: 0.0,
            //             //     view_height: 0.0,
            //             //     view_pos_x: 0.0,
            //             //     view_pos_y: 0.0,
            //             // }]);
            //             match files {
            //                 Ok(files) => {}
            //                 Err(err) => {
            //                     error!("posts api err: {err}");
            //                 }
            //             }
            //         }
            //     },
            // );
            // let new_imgs = fetch_bottom(count, prev_imgs.last().cloned().unwrap());
            // if new_imgs.is_empty() {
            //     return;
            // }
            // let (resized_imgs, scroll_by) =
            //     add_imgs_to_bottom(prev_imgs, new_imgs, width, heigth * 3.0, row_height);
            // trace!("scroll master: {scroll_by}");
            // gallery.set(resized_imgs);
            // gallery_elm.scroll_by_with_x_and_y(0.0, scroll_by);
        };

        let _ = interval::new(move || {
            // run_fetch_top();
            // run_fetch_bottom();
            let Some(gallery_elm) = gallery_ref.get_untracked() else {
                trace!("gallery NOT found");
                return;
            };

            let scroll_top= gallery_elm.scroll_top() as u32;
            let scroll_height= gallery_elm.scroll_height() as u32;
            let width = gallery_elm.client_width() as u32;
            let height = gallery_elm.client_height() as u32;

            if scroll_top < row_height {
                trace!("INTERVAL FETCH TOP");
                run_fetch_top();
            }

            if scroll_height.saturating_sub(scroll_top + height) < row_height {
                trace!("INTERVAL FETCH BTM");
                run_fetch_bottom();
            }

        }, Duration::from_secs(2));

        let get_imgs = move || {
            let imgs = gallery.get();
            let total_count = imgs.len();

            imgs.into_iter()
                .enumerate()
                .map({
                    let run_fetch_bottom = run_fetch_bottom.clone();
                    let run_fetch_top = run_fetch_top.clone();
                    move |(i, img)| view! {<GalleryImg index=i img total_count run_fetch_bottom=run_fetch_bottom.clone() run_fetch_top=run_fetch_top.clone() />}
                })
                .collect_view()
        };

        Effect::new(move || {
            let Some(gallery_elm) = gallery_ref.get() else {
                return;
            };
            if api_top.busy.get_untracked() {
                return;
            }

            let width = gallery_elm.client_width() as u32;
            let height = gallery_elm.client_height() as f64;
            let count = calc_fit_count(width, height, row_height) as u32;
            let time = Utc::now().timestamp_micros() as u128 * 1000;

            set_gallery(true, width, height, time, count);
            // api_post_get_after.dispatch_and_run(
            //     controller::post::route::get_after::Input { time, limit: count },
            //     move |files| {
            //         let files = files.clone();
            //         async move {
            //             // gallery.set(vec![Img {
            //             //     id: 0,
            //             //     hash: "404".to_string(),
            //             //     extension: "webp".to_string(),
            //             //     width: 300,
            //             //     height: 200,
            //             //     view_width: 0.0,
            //             //     view_height: 0.0,
            //             //     view_pos_x: 0.0,
            //             //     view_pos_y: 0.0,
            //             // }]);
            //             match files {
            //                 Ok(files) => {
            //                     let Some(prev_imgs) = gallery.try_get_untracked() else {
            //                         return;
            //                     };
            //
            //                     let new_imgs = files
            //                         .posts
            //                         .iter()
            //                         .map(|post| Img {
            //                             id: 0,
            //                             hash: post.hash.clone(),
            //                             extension: post.extension.clone(),
            //                             width: post.width,
            //                             height: post.height,
            //                             view_width: 0.0,
            //                             view_height: 0.0,
            //                             view_pos_x: 0.0,
            //                             view_pos_y: 0.0,
            //                             created_at: post.created_at,
            //                         })
            //                         .collect::<Vec<Img>>();
            //
            //                     let (resized_imgs, _scroll_by) = add_imgs_to_bottom(
            //                         prev_imgs, new_imgs, width, heigth, row_height,
            //                     );
            //                     gallery.set(resized_imgs);
            //                 }
            //                 Err(err) => {
            //                     error!("posts api err: {err}");
            //                 }
            //             }
            //         }
            //     },
            // );
        });

        // Effect::new(move || {
        //     trace!("ON LOAD");
        //     if api_post_get_after.is_pending() {
        //         return;
        //     }
        //     let (Some(gallery_elm), Some(files)) = (gallery_ref.get(), api_post_get_after.value())
        //     else {
        //         // trace!("gallery NOT found");
        //         return;
        //     };
        //     trace!("gallery elm found");
        //     match files {
        //         Ok(files) => {
        //             let width = gallery_elm.client_width() as u32;
        //             let heigth = gallery_elm.client_height() as f64;
        //
        //             let prev_imgs = gallery.get_untracked();
        //             let count = calc_fit_count(width, heigth, row_height);
        //
        //             let (resized_imgs, _scroll_by) =
        //                 add_imgs_to_bottom(prev_imgs, new_imgs, width, heigth, row_height);
        //             gallery.set(resized_imgs);
        //         }
        //         Err(err) => {
        //             error!("posts api err: {err}");
        //         }
        //     }
        //
        //     // let new_imgs = match fetch_init.clone() {
        //     //     Some(fetch_init) => {
        //     //         let imgs = fetch_init(count);
        //     //         if imgs.is_empty() {
        //     //             return;
        //     //         }
        //     //         imgs
        //     //     }
        //     //     None => Img::rand_vec(count),
        //     // };
        //     // let (resized_imgs, _scroll_by) =
        //     //     add_imgs_to_bottom(prev_imgs, new_imgs, width, heigth, row_height);
        //     // gallery.set(resized_imgs);
        // });
        //
        let onscroll = move |e| {
            let Some(gallery_elm) = gallery_ref.get() else {
                return;
            };
            if api_top.busy.get_untracked() {
                return;
            }

            let width = gallery_elm.client_width() as u32;
            let height = gallery_elm.client_height() as f64;
            let count = calc_fit_count(width, height, row_height) as u32;
            let time = Utc::now().timestamp_micros() as u128 * 1000;

            // let prev_imgs = gallery.get_untracked();
            // let Some(time) = prev_imgs.last().map(|v| v.created_at) else {
            //     return;
            // };
            trace!("SCROLLINGGG");
        };
        // on:scroll=onscroll

        let a = view! {
            <div
                id="gallery"
                node_ref=gallery_ref
                on:scroll=onscroll
                class="relative overflow-y-scroll overflow-x-hidden"
            >
                {
                    get_imgs
                }
            </div>
        };

        a
    }

    #[component]
    pub fn GalleryImg<FetchBtmFn, FetchTopFn>(
        img: Img,
        index: usize,
        total_count: usize,
        run_fetch_bottom: FetchBtmFn,
        run_fetch_top: FetchTopFn,
    ) -> impl IntoView
    where
        FetchBtmFn: Fn() + Send + Sync + 'static + Clone,
        FetchTopFn: Fn() + Send + Sync + 'static + Clone,
    {
        let img_ref = NodeRef::<html::Img>::new();

        let activated = StoredValue::new(false);
        // let activated = StoredValue::new(None::<bool>);

        img_ref.add_intersection_observer_with_options(
            move |entry, _observer| {
                let is_intersecting = entry.is_intersecting();

                if !is_intersecting {
                    activated.set_value(true);
                    return;
                }

                if !activated.get_value() {
                    return;
                }

                let quarter = total_count / 4;
                // if index == total_count.saturating_sub(1) && is_intersecting {
                if index == total_count.saturating_sub(quarter)
                    || index == total_count.saturating_sub(1)
                {
                    run_fetch_bottom();
                    trace!("intersection fn last {index} is intesecting: {is_intersecting}");
                } else if index == quarter || index == 0 {
                    run_fetch_top();
                    trace!("intersection fn first {index} is intesecting: {is_intersecting}");
                }
            },
            IntersectionOptions::<Div>::default(),
        );

        let view_left = img.view_pos_x;
        let view_top = img.view_pos_y;
        let view_width = img.view_width;
        let view_height = img.view_height;
        let img_width = img.width;
        let img_height = img.height;
        let img_id = img.id;
        let link = img.get_link();
        // let link = "/file/404.webp".to_string();

        let fn_background = move || format!("rgb({}, {}, {})", 50, 50, 50);
        let fn_left = move || format!("{view_left}px");
        let fn_top = move || format!("{view_top}px");
        let fn_width = move || format!("{view_width}px");
        let fn_height = move || format!("{view_height}px");
        let fn_text = move || format!("{img_width}x{img_height}");
        let fn_text2 = move || format!("{view_left}x{view_top}");
        let fn_text3 = move || format!("{img_id}");
        // let link = move || link;

        view! {

            <img
                class="absolute"
                node_ref=img_ref
                style:left=fn_left
                style:top=fn_top
                style:width=fn_width
                style:height=fn_height
                src=link
            />
            // <div
            //     class="text-white grid place-items-center bg-blue-950 absolute border border-red-600 overflow-hidden"
            //
            //     node_ref=img_ref
            //     style:background-color=fn_background
            //     style:left=fn_left
            //     style:top=fn_top
            //     style:width=fn_width
            //     style:height=fn_height
            // >
            //     // <div>
            //     //     <div>{fn_text3}</div>
            //     //     <div>{fn_text}</div>
            //     //     <div>{fn_text2}</div>
            //     // </div>
            // </div>
        }
    }

    #[derive(Debug, Clone)]
    pub struct Img {
        pub id: u64,
        pub hash: String,
        pub extension: String,
        // pub row_id: usize,
        pub width: u32,
        pub height: u32,
        pub view_width: f64,
        pub view_height: f64,
        pub view_pos_x: f64,
        pub view_pos_y: f64,
        pub created_at: u128,
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

    impl ResizableImage for Img {
        fn get_id(&self) -> u64 {
            self.id
        }
        fn get_link(&self) -> String {
            format!("/file/{}.{}", self.hash, self.extension)
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
                id,
                // row_id: 0,
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

        pub fn rand(id: usize) -> Self {
            let width = random_u32_ranged(500, 1000);
            let height = random_u32_ranged(500, 1000);

            Self {
                id: id as u64,
                // row_id: 0,
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
                output.push(Img::rand(i));
            }
            output
        }
    }

    pub trait ResizableImage {
        fn get_id(&self) -> u64;
        fn get_link(&self) -> String;
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
        mut imgs: Vec<IMG>,
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
            vec_img_to_string(&imgs),
            vec_img_to_string(&new_imgs)
        );
        if new_imgs.is_empty() {
            return (imgs, 0.0);
        }
        let height_before_remove = get_total_height(&imgs);
        trace!("stage 0: {imgs:#?}");
        if let Some(cut_index) = remove_until_fit_from_bottom(&mut imgs, heigth) {
            imgs = imgs[..cut_index].to_vec();
            trace!("stage 1 ({cut_index}): {imgs:#?}");
        }

        let height_after_remove = get_total_height(&imgs);
        let Some(offset) = imgs
            .len()
            .checked_add(new_imgs.len())
            .and_then(|v| v.checked_sub(2))
        else {
            return (imgs, 0.0);
        };

        new_imgs.extend(imgs);
        imgs = new_imgs;

        let offset = get_row_end(&mut imgs, offset);
        trace!("stage 4(offset: {offset}): {imgs:#?}");
        let rows = get_rows_to_top(&imgs, offset, width, row_height);
        set_rows_to_top(&mut imgs, &rows, width);
        trace!("stage 2: {imgs:#?}");
        normalize_imgs_y_v2(&mut imgs);
        let height_final = get_total_height(&imgs);
        let scroll_by = height_final - height_after_remove;

        trace!(
            "stage 5(KOKheight_before_remove: {height_before_remove}, height_after_remove: {height_after_remove}, height_final: {height_final}, scroll_by: {scroll_by}): {imgs:#?}"
        );
        trace!(
            "INPUT FOR ADD IMGS TO TOP OUTPUT {}\n{}",
            scroll_by,
            vec_img_to_string(&imgs),
        );
        (imgs, scroll_by)
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
                        // if img_fits_in_row && (row.aspect_ratio + ratio) < 5.0 {
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
        ((width * height as u32) / (row_height * row_height)) as usize
    }

    #[cfg(test)]
    mod resize_tests {
        use crate::view::app::components::gallery::{get_total_height, vec_img_to_string};

        use super::{
            Row, add_imgs_to_bottom, add_imgs_to_top, get_row_end, get_row_start,
            get_rows_to_bottom, get_rows_to_top, normalize_imgs_y_v2, remove_until_fit_from_bottom,
            remove_until_fit_from_top, resize_v2, set_rows_to_bottom, set_rows_to_top,
        };
        // use leptos::prelude::*;
        use ordered_float::OrderedFloat;
        use pretty_assertions::assert_eq;
        use std::fmt::Display;
        use test_log::test;
        use tracing::trace;

        use wasm_bindgen_test::*;

        use super::ResizableImage;

        wasm_bindgen_test_configure!(run_in_browser);

        #[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
        struct Img {
            pub id: u32,
            pub width: u32,
            pub height: u32,
            pub view_width: OrderedFloat<f64>,
            pub view_height: OrderedFloat<f64>,
            pub view_pos_x: OrderedFloat<f64>,
            pub view_pos_y: OrderedFloat<f64>,
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
                // write!(
                //     f,
                //     "Img {{ width: {}, height: {}, view_width: OrderedFloat({:.5}), view_height: OrderedFloat({:.5}), view_pos_x: OrderedFloat({:.5}), view_pos_y: OrderedFloat({:.5})}}",
                //     self.width,
                //     self.height,
                //     self.view_width,
                //     self.view_height,
                //     self.view_pos_x,
                //     self.view_pos_y
                // )
            }
        }

        impl Img {
            pub const fn new(id: u32, width: u32, height: u32) -> Self {
                Self {
                    id,
                    width,
                    height,
                    view_width: OrderedFloat(0.0),
                    view_height: OrderedFloat(0.0),
                    view_pos_x: OrderedFloat(0.0),
                    view_pos_y: OrderedFloat(0.0),
                }
            }

            pub const fn new_full(
                id: u32,
                width: u32,
                height: u32,
                view_width: f64,
                view_height: f64,
                view_pos_x: f64,
                view_pos_y: f64,
            ) -> Self {
                Self {
                    id,
                    width,
                    height,
                    view_width: OrderedFloat(view_width),
                    view_height: OrderedFloat(view_height),
                    view_pos_x: OrderedFloat(view_pos_x),
                    view_pos_y: OrderedFloat(view_pos_y),
                }
            }
        }

        impl ResizableImage for Img {
            fn get_id(&self) -> u64 {
                self.id as u64
            }
            fn get_link(&self) -> String {
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
                *self.view_pos_x
            }
            fn get_pos_y(&self) -> f64 {
                *self.view_pos_y
            }
            fn get_view_height(&self) -> f64 {
                *self.view_height
            }
            fn set_size(&mut self, view_width: f64, view_height: f64, pos_x: f64, pos_y: f64) {
                *self.view_width = view_width;
                *self.view_height = view_height;
                self.view_pos_x = OrderedFloat::from(pos_x);
                self.view_pos_y = OrderedFloat::from(pos_y);
            }
            fn set_pos_x(&mut self, pos_x: f64) {
                *self.view_pos_x = pos_x;
            }
            fn set_pos_y(&mut self, pos_y: f64) {
                *self.view_pos_y = pos_y;
            }
        }

        #[test]
        fn test_get_rows_forward() {
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
            let imgs = Vec::<Img>::from([
                //row 0
            ]);
            let resized_imgs = resize_v2(imgs, 1000, 500);
            trace!("{resized_imgs:#?}");
            // TODO test resize
        }

        #[test]
        fn test_set_rows() {
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
            // TODO, ASSET SCROLL_BY
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
            trace!("=======UPDATING IMGS=======");
            #[rustfmt::skip]
            let imgs = Vec::from([
                Img::new_full(0, 806, 595, 525.9883961169446138228522613644599914550781250000000000000000000000, 388.2916819970000119610631372779607772827148437500000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 455, 234, 755.0116038830556135508231818675994873046875000000000000000000000000, 388.2916819970000119610631372779607772827148437500000000000000000000, 525.9883961169446138228522613644599914550781250000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 657, 281, 613.5186845168799436578410677611827850341796875000000000000000000000, 262.4029685681023806864686775952577590942382812500000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 388.2916819969996140571311116218566894531250000000000000000000000000),
                Img::new_full(0, 669, 263, 667.4813154831197152816457673907279968261718750000000000000000000000, 262.4029685681023806864686775952577590942382812500000000000000000000, 613.5186845168799436578410677611827850341796875000000000000000000000, 388.2916819969996140571311116218566894531250000000000000000000000000),
                Img::new_full(0, 701, 275, 1281.0000000000000000000000000000000000000000000000000000000000000000, 502.5320970042795352128450758755207061767578125000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 650.6946505651021652738563716411590576171875000000000000000000000000),
                Img::new_full(0, 326, 125, 1281.0000000000000000000000000000000000000000000000000000000000000000, 491.1809815950920210525509901344776153564453125000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 1153.2267475693815867998637259006500244140625000000000000000000000000),
                Img::new_full(0, 331, 118, 773.0390697229383931698976084589958190917968750000000000000000000000, 275.5849251580263512551027815788984298706054687500000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 1644.4077291644734941655769944190979003906250000000000000000000000000),
                Img::new_full(0, 1920, 1076, 491.7500523265897527380730025470256805419921875000000000000000000000, 275.5849251580263512551027815788984298706054687500000000000000000000, 773.0390697229383931698976084589958190917968750000000000000000000000, 1644.4077291644734941655769944190979003906250000000000000000000000000),
                Img::new_full(0, 31, 527, 16.2108779504721383091236930340528488159179687500000000000000000000, 275.5849251580263512551027815788984298706054687500000000000000000000, 1264.7891220495280322211328893899917602539062500000000000000000000000, 1644.4077291644734941655769944190979003906250000000000000000000000000),
                Img::new_full(0, 564, 744, 318.3707760585990058643801603466272354125976562500000000000000000000, 419.9784705453859032786567695438861846923828125000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 1919.9926543225001296377740800380706787109375000000000000000000000000),
                Img::new_full(0, 564, 564, 419.9784705453859032786567695438861846923828125000000000000000000000, 419.9784705453859032786567695438861846923828125000000000000000000000, 318.3707760585990058643801603466272354125976562500000000000000000000, 1919.9926543225001296377740800380706787109375000000000000000000000000),
                Img::new_full(0, 1013, 784, 542.6507533960151477003819309175014495849609375000000000000000000000, 419.9784705453859032786567695438861846923828125000000000000000000000, 738.3492466039849659864557906985282897949218750000000000000000000000, 1919.9926543225001296377740800380706787109375000000000000000000000000),
                Img::new_full(0, 359, 150, 822.1912991656735130163724534213542938232421875000000000000000000000, 343.5339690107270484986656811088323593139648437500000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 2339.9711248678859192295931279659271240234375000000000000000000000000),
                Img::new_full(0, 1202, 900, 458.8087008343266006704652681946754455566406250000000000000000000000, 343.5339690107270484986656811088323593139648437500000000000000000000, 822.1912991656735130163724534213542938232421875000000000000000000000, 2339.9711248678859192295931279659271240234375000000000000000000000000),
                Img::new_full(0, 971, 277, 1281.0000000000000000000000000000000000000000000000000000000000000000, 365.4346035015448137528437655419111251831054687500000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 2683.5050938786125698243267834186553955078125000000000000000000000000),
                Img::new_full(0, 1292, 657, 1281.0000000000000000000000000000000000000000000000000000000000000000, 651.4063467492260315339080989360809326171875000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 3048.9396973801576677942648530006408691406250000000000000000000000000),
                Img::new_full(0, 1403, 400, 1002.8922979334291767372633330523967742919921875000000000000000000000, 285.9279537942777551506878808140754699707031250000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 3700.3460441293836993281729519367218017578125000000000000000000000000),
                Img::new_full(0, 1138, 1170, 278.1077020665709937929932493716478347778320312500000000000000000000, 285.9279537942777551506878808140754699707031250000000000000000000000, 1002.8922979334291767372633330523967742919921875000000000000000000000, 3700.3460441293836993281729519367218017578125000000000000000000000000),
                Img::new_full(0, 461, 196, 635.0291527921731358219403773546218872070312500000000000000000000000, 269.9907027055660364567302167415618896484375000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 3986.2739979236612271051853895187377929687500000000000000000000000000),
                Img::new_full(0, 1178, 1020, 311.8127919481929097855754662305116653442382812500000000000000000000, 269.9907027055660364567302167415618896484375000000000000000000000000, 635.0291527921731358219403773546218872070312500000000000000000000000, 3986.2739979236612271051853895187377929687500000000000000000000000000),
                Img::new_full(0, 1781, 1439, 334.1580552596338407056464347988367080688476562500000000000000000000, 269.9907027055660364567302167415618896484375000000000000000000000000, 946.8419447403659887640969827771186828613281250000000000000000000000, 3986.2739979236612271051853895187377929687500000000000000000000000000),
                Img::new_full(0, 76, 62, 243.8280626684967842265905346721410751342773437500000000000000000000, 198.9123669137736953871353762224316596984863281250000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 4256.2647006292272635619156062602996826171875000000000000000000000000),
                Img::new_full(0, 590, 365, 321.5295793948670848294568713754415512084960937500000000000000000000, 198.9123669137736953871353762224316596984863281250000000000000000000, 243.8280626684967842265905346721410751342773437500000000000000000000, 4256.2647006292272635619156062602996826171875000000000000000000000000),
                Img::new_full(0, 1738, 1440, 240.0761761778740606132487300783395767211914062500000000000000000000, 198.9123669137736953871353762224316596984863281250000000000000000000, 565.3576420633638690560474060475826263427734375000000000000000000000, 4256.2647006292272635619156062602996826171875000000000000000000000000),
                Img::new_full(0, 591, 1280, 91.8415694109689439983412739820778369903564453125000000000000000000, 198.9123669137736953871353762224316596984863281250000000000000000000, 805.4338182412379865127149969339370727539062500000000000000000000000, 4256.2647006292272635619156062602996826171875000000000000000000000000),
            ]);

            #[rustfmt::skip]
            let new_imgs = Vec::from([
                Img::new_full(0, 1414, 866, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 1429, 876, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 1316, 866, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 580, 537, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 1935, 1130, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 129, 70, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 942, 722, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 418, 79, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 1542, 975, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 970, 641, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 1270, 718, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 2549, 1406, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 2555, 1440, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 76, 126, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 630, 585, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 1253, 1241, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 1, 19, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 833, 505, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 838, 507, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 630, 433, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 418, 368, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 1653, 754, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 692, 555, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 1607, 1189, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 644, 453, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 458, 412, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 961, 116, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
            ]);

            #[rustfmt::skip]
            let expected_imgs = Vec::from([
                Img::new_full(0, 1414, 866, 408.1986143187067455073702149093151092529296875000000000000000000000, 250.0000000000000284217094304040074348449707031250000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 1429, 876, 493.8964669661540938250254839658737182617187500000000000000000000000, 302.7664835985661397899093572050333023071289062500000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 250.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 1316, 866, 460.0931783091374427385744638741016387939453125000000000000000000000, 302.7664835985661397899093572050333023071289062500000000000000000000, 493.8964669661540938250254839658737182617187500000000000000000000000, 250.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 580, 537, 327.0103547247082929061434697359800338745117187500000000000000000000, 302.7664835985661397899093572050333023071289062500000000000000000000, 953.9896452752915365635999478399753570556640625000000000000000000000, 250.0000000000000000000000000000000000000000000000000000000000000000),
                Img::new_full(0, 1935, 1130, 451.3561332277247402089415118098258972167968750000000000000000000000, 263.5826514456480254011694341897964477539062500000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 552.7664835985660829464904963970184326171875000000000000000000000000),
                Img::new_full(0, 129, 70, 485.7451719498370721339597366750240325927734375000000000000000000000, 263.5826514456480254011694341897964477539062500000000000000000000000, 451.3561332277247402089415118098258972167968750000000000000000000000, 552.7664835985660829464904963970184326171875000000000000000000000000),
                Img::new_full(0, 942, 722, 343.8986948224383013439364731311798095703125000000000000000000000000, 263.5826514456480254011694341897964477539062500000000000000000000000, 937.1013051775618123429012484848499298095703125000000000000000000000, 552.7664835985660829464904963970184326171875000000000000000000000000),
                Img::new_full(0, 418, 79, 1281.0000000000000000000000000000000000000000000000000000000000000000, 242.1028708133971463212219532579183578491210937500000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 816.3491350442145630950108170509338378906250000000000000000000000000),
                Img::new_full(0, 1542, 975, 654.6308094626538149896077811717987060546875000000000000000000000000, 413.9202589014834074987447820603847503662109375000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 1058.4520058576117662596516311168670654296875000000000000000000000000),
                Img::new_full(0, 970, 641, 626.3691905373461850103922188282012939453125000000000000000000000000, 413.9202589014834074987447820603847503662109375000000000000000000000, 654.6308094626538149896077811717987060546875000000000000000000000000, 1058.4520058576117662596516311168670654296875000000000000000000000000),
                Img::new_full(0, 1270, 718, 632.6063263419844133750302717089653015136718750000000000000000000000, 357.6467262311376771322102285921573638916015625000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 1472.3722647590948326978832483291625976562500000000000000000000000000),
                Img::new_full(0, 2549, 1406, 648.3936736580155866249697282910346984863281250000000000000000000000, 357.6467262311376771322102285921573638916015625000000000000000000000, 632.6063263419844133750302717089653015136718750000000000000000000000, 1472.3722647590948326978832483291625976562500000000000000000000000000),
                Img::new_full(0, 2555, 1440, 509.1506096590884453689795918762683868408203125000000000000000000000, 286.9576821561985866537725087255239486694335937500000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 1830.0189909902328508906066417694091796875000000000000000000000000000),
                Img::new_full(0, 76, 126, 173.0855860624689910309825791046023368835449218750000000000000000000, 286.9576821561985866537725087255239486694335937500000000000000000000, 509.1506096590884453689795918762683868408203125000000000000000000000, 1830.0189909902328508906066417694091796875000000000000000000000000000),
                Img::new_full(0, 630, 585, 309.0313500143677174492040649056434631347656250000000000000000000000, 286.9576821561985866537725087255239486694335937500000000000000000000, 682.2361957215574648216716013848781585693359375000000000000000000000, 1830.0189909902328508906066417694091796875000000000000000000000000000),
                Img::new_full(0, 1253, 1241, 289.7324542640748177291243337094783782958984375000000000000000000000, 286.9576821561985866537725087255239486694335937500000000000000000000, 991.2675457359251822708756662905216217041015625000000000000000000000, 1830.0189909902328508906066417694091796875000000000000000000000000000),
                Img::new_full(0, 1, 19, 14.0169620389277245209314060048200190067291259765625000000000000000, 266.3222787396267676740535534918308258056640625000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 2116.9766731464314943877980113029479980468750000000000000000000000000),
                Img::new_full(0, 833, 505, 439.2999172081367760256398469209671020507812500000000000000000000000, 266.3222787396267676740535534918308258056640625000000000000000000000, 14.0169620389277245209314060048200190067291259765625000000000000000, 2116.9766731464314943877980113029479980468750000000000000000000000000),
                Img::new_full(0, 838, 507, 440.1934311317696710830205120146274566650390625000000000000000000000, 266.3222787396267676740535534918308258056640625000000000000000000000, 453.3168792470644916647870559245347976684570312500000000000000000000, 2116.9766731464314943877980113029479980468750000000000000000000000000),
                Img::new_full(0, 630, 433, 387.4896896211659509390301536768674850463867187500000000000000000000, 266.3222787396267676740535534918308258056640625000000000000000000000, 893.5103103788342195912264287471771240234375000000000000000000000000, 2116.9766731464314943877980113029479980468750000000000000000000000000),
                Img::new_full(0, 418, 368, 318.0418025891976867569610476493835449218750000000000000000000000000, 279.9985247675232926667376887053251266479492187500000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 2383.2989518860586031223647296428680419921875000000000000000000000000),
                Img::new_full(0, 1653, 754, 613.8429196826472207249025814235210418701171875000000000000000000000, 279.9985247675232926667376887053251266479492187500000000000000000000, 318.0418025891976867569610476493835449218750000000000000000000000000, 2383.2989518860586031223647296428680419921875000000000000000000000000),
                Img::new_full(0, 692, 555, 349.1152777281551493615552317351102828979492187500000000000000000000, 279.9985247675232926667376887053251266479492187500000000000000000000, 931.8847222718449074818636290729045867919921875000000000000000000000, 2383.2989518860586031223647296428680419921875000000000000000000000000),
                Img::new_full(0, 1607, 1189, 445.6665292193769687401072587817907333374023437500000000000000000000, 329.7433125338141053362051025032997131347656250000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 2663.2974766535817252588458359241485595703125000000000000000000000000),
                Img::new_full(0, 644, 453, 468.7741573328394792952167335897684097290039062500000000000000000000, 329.7433125338141053362051025032997131347656250000000000000000000000, 445.6665292193769687401072587817907333374023437500000000000000000000, 2663.2974766535817252588458359241485595703125000000000000000000000000),
                Img::new_full(0, 458, 412, 366.5593134477836088080948684364557266235351562500000000000000000000, 329.7433125338141053362051025032997131347656250000000000000000000000, 914.4406865522164480353239923715591430664062500000000000000000000000, 2663.2974766535817252588458359241485595703125000000000000000000000000),
                Img::new_full(0, 961, 116, 1281.0000000000000000000000000000000000000000000000000000000000000000, 154.6264308012486878851632354781031608581542968750000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 2993.0407891873956032213754951953887939453125000000000000000000000000),
                Img::new_full(0, 806, 595, 525.9883961169446138228522613644599914550781250000000000000000000000, 388.2916819970000119610631372779607772827148437500000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 3147.6672199886443195282481610774993896484375000000000000000000000000),
                Img::new_full(0, 455, 234, 755.0116038830556135508231818675994873046875000000000000000000000000, 388.2916819970000119610631372779607772827148437500000000000000000000, 525.9883961169446138228522613644599914550781250000000000000000000000, 3147.6672199886443195282481610774993896484375000000000000000000000000),
                Img::new_full(0, 657, 281, 613.5186845168799436578410677611827850341796875000000000000000000000, 262.4029685681023806864686775952577590942382812500000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 3535.9589019856443883327301591634750366210937500000000000000000000000),
                Img::new_full(0, 669, 263, 667.4813154831197152816457673907279968261718750000000000000000000000, 262.4029685681023806864686775952577590942382812500000000000000000000, 613.5186845168799436578410677611827850341796875000000000000000000000, 3535.9589019856443883327301591634750366210937500000000000000000000000),
                Img::new_full(0, 701, 275, 1281.0000000000000000000000000000000000000000000000000000000000000000, 502.5320970042795352128450758755207061767578125000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 3798.3618705537469395494554191827774047851562500000000000000000000000),
                Img::new_full(0, 326, 125, 1281.0000000000000000000000000000000000000000000000000000000000000000, 491.1809815950920210525509901344776153564453125000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 4300.8939675580259063281118869781494140625000000000000000000000000000),
                Img::new_full(0, 331, 118, 1281.0000000000000000000000000000000000000000000000000000000000000000, 456.6706948640483005874557420611381530761718750000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 4792.0749491531187231885269284248352050781250000000000000000000000000),
                Img::new_full(0, 1920, 1076, 878.7227620879992855407181195914745330810546875000000000000000000000, 492.4508812534829189644369762390851974487304687500000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 5248.7456440171663416549563407897949218750000000000000000000000000000),
                Img::new_full(0, 31, 527, 28.9676988972637019514877465553581714630126953125000000000000000000, 492.4508812534829189644369762390851974487304687500000000000000000000, 878.7227620879992855407181195914745330810546875000000000000000000000, 5248.7456440171663416549563407897949218750000000000000000000000000000),
                Img::new_full(0, 564, 744, 373.3095390147370267186488490551710128784179687500000000000000000000, 492.4508812534829189644369762390851974487304687500000000000000000000, 907.6904609852630301247700117528438568115234375000000000000000000000, 5248.7456440171663416549563407897949218750000000000000000000000000000),
                Img::new_full(0, 564, 564, 273.4010155969262996222823858261108398437500000000000000000000000000, 273.4010155969262996222823858261108398437500000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 5741.1965252706495448364876210689544677734375000000000000000000000000),
                Img::new_full(0, 1013, 784, 353.2592204077631663494685199111700057983398437500000000000000000000, 273.4010155969262996222823858261108398437500000000000000000000000000, 273.4010155969262996222823858261108398437500000000000000000000000000, 5741.1965252706495448364876210689544677734375000000000000000000000000),
                Img::new_full(0, 359, 150, 654.3397639953103634979925118386745452880859375000000000000000000000, 273.4010155969262996222823858261108398437500000000000000000000000000, 626.6602360046895228151697665452957153320312500000000000000000000000, 5741.1965252706495448364876210689544677734375000000000000000000000000),
                Img::new_full(0, 1202, 900, 353.4098358210686683378298766911029815673828125000000000000000000000, 264.6163496164407433752785436809062957763671875000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 6014.5975408675758444587700068950653076171875000000000000000000000000),
                Img::new_full(0, 971, 277, 927.5901641789313316621701233088970184326171875000000000000000000000, 264.6163496164407433752785436809062957763671875000000000000000000000, 353.4098358210686683378298766911029815673828125000000000000000000000, 6014.5975408675758444587700068950653076171875000000000000000000000000),
                Img::new_full(0, 1292, 657, 1281.0000000000000000000000000000000000000000000000000000000000000000, 651.4063467492260315339080989360809326171875000000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 6279.2138904840167015208862721920013427734375000000000000000000000000),
                Img::new_full(0, 1403, 400, 1002.8922979334291767372633330523967742919921875000000000000000000000, 285.9279537942777551506878808140754699707031250000000000000000000000, 0.0000000000000000000000000000000000000000000000000000000000000000, 6930.6202372332427330547943711280822753906250000000000000000000000000),
                Img::new_full(0, 1138, 1170, 278.1077020665709937929932493716478347778320312500000000000000000000, 285.9279537942777551506878808140754699707031250000000000000000000000, 1002.8922979334291767372633330523967742919921875000000000000000000000, 6930.6202372332427330547943711280822753906250000000000000000000000000),
            ]);
            let (imgs, scroll_by) = add_imgs_to_top(imgs, new_imgs, 1281, 4086.0, 250);
            let total_h = get_total_height(&imgs);
            // trace!("{scroll_by} - {}", vec_img_to_string(&imgs));
            assert_eq!(expected_imgs, imgs);
        }

        #[test]
        fn img_remove() {
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
