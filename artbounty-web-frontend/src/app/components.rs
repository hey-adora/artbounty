pub mod nav {
    use leptos::prelude::*;

    #[component]
    pub fn Nav() -> impl IntoView {
        view! {
            <nav class="text-gray-200 pb-1 flex gap-2 px-2 py-1 items-center justify-between">
                <a href="/" class="font-black text-xl">
                    "ArtBounty"
                </a>
                <a href="/login">"Login"</a>
            </nav>
        }
    }
}
pub mod gallery {
    use futures::io::Cursor;
    use itertools::{FoldWhile, Itertools};
    use leptos::{
        html::{self, Div, Main, div},
        prelude::*,
        tachys::html::node_ref::NodeRefContainer,
    };
    use ordered_float::OrderedFloat;
    use std::{default::Default, time::Duration};
    use std::{
        fmt::{Debug, Display},
        rc::Rc,
    };
    use tracing::{debug, error, trace, trace_span};
    use web_sys::HtmlDivElement;

    use crate::toolbox::{prelude::*, random::random_u64};

    // pub const NEW_IMG_HEIGHT: u32 = 250;

    pub fn vec_img_to_string<IMG: ResizableImage + Display>(imgs: &[IMG]) -> String {
        let mut output = String::new();

        for img in imgs {
            output += &format!("{},\n", img);
        }

        output
    }

    #[component]
    pub fn Gallery<FETCH_BTM_FN, FETCH_TOP_FN>(
        fetch_top: FETCH_TOP_FN,
        fetch_bottom: FETCH_BTM_FN,
        #[prop(optional)] fetch_init: Option<Rc<dyn Fn(usize) -> Vec<Img> + Send + Sync + 'static>>,
        #[prop(default = 250)] row_height: u32,
    ) -> impl IntoView
    where
        FETCH_BTM_FN: Fn(usize, Img) -> Vec<Img> + Send + Sync + 'static + Clone,
        FETCH_TOP_FN: Fn(usize, Img) -> Vec<Img> + Send + Sync + 'static + Clone,
    {
        let gallery = RwSignal::<Vec<Img>>::new(Vec::new());
        let gallery_ref = NodeRef::<Div>::new();
        // let scroll_offset: StoredValue<f32> = StoredValue::new(0.0_f32);

        gallery_ref.add_resize_observer(move |entry, observer| {
            trace!("RESIZINGGGGGG");
            let width = entry.content_rect().width() as u32;
            let heigth = entry.content_rect().height() as f32;

            let prev_imgs = gallery.get_untracked();
            // let new_imgs = Vec::from([Img::rand()]);
            trace!("stage r1: width:{width} {prev_imgs:#?} ");
            let resized_imgs = resize_v2(prev_imgs, width, 250);
            trace!("stage r2 {resized_imgs:#?}");
            gallery.set(resized_imgs);
        });

        let run_fetch_top = move || {
            let Some(gallery_elm) = gallery_ref.get_untracked() else {
                trace!("gallery NOT found");
                return;
            };
            trace!("gallery elm found");
            let width = gallery_elm.client_width() as u32;
            let heigth = gallery_elm.client_height() as f64;
            let scroll_heigth = gallery_elm.scroll_height() as f64;
            let scroll_top = gallery_elm.scroll_top() as f64;
            let is_empty = gallery.with_untracked(|v| v.is_empty());
            let count = calc_fit_count(width, heigth, row_height);
            if is_empty || count == 0 {
                return;
            }
            let prev_imgs = gallery.get_untracked();
            let new_imgs = fetch_top(count, prev_imgs.first().cloned().unwrap());
            if new_imgs.is_empty() {
                return;
            }
            let (resized_imgs, scroll_by) =
                add_imgs_to_top(prev_imgs, new_imgs, width, heigth, row_height);
            trace!("scroll master: {scroll_by}");
            gallery.set(resized_imgs);
            gallery_elm.scroll_by_with_x_and_y(0.0, scroll_by);
        };

        let run_fetch_bottom = move || {
            let Some(gallery_elm) = gallery_ref.get_untracked() else {
                trace!("gallery NOT found");
                return;
            };
            trace!("gallery elm found");
            let width = gallery_elm.client_width() as u32;
            let heigth = gallery_elm.client_height() as f64;
            let scroll_heigth = gallery_elm.scroll_height() as f64;
            let scroll_top = gallery_elm.scroll_top() as f64;
            let is_empty = gallery.with_untracked(|v| v.is_empty());
            let count = calc_fit_count(width, heigth, row_height);
            if is_empty || count == 0 {
                return;
            }
            let prev_imgs = gallery.get_untracked();
            let new_imgs = fetch_bottom(count, prev_imgs.last().cloned().unwrap());
            if new_imgs.is_empty() {
                return;
            }
            // let prev_total_height = get_total_height(&prev_imgs);
            // let _span = trace_span!("ON_SCROLL", scroll_top).entered();
            // let new_total_height = get_total_height(&resized_imgs);
            // let diff = new_total_height - prev_total_height;
            let (resized_imgs, scroll_by) =
                add_imgs_to_bottom(prev_imgs, new_imgs, width, heigth, row_height);
            trace!("scroll master: {scroll_by}");
            gallery.set(resized_imgs);
            gallery_elm.scroll_by_with_x_and_y(0.0, scroll_by);
        };

        let get_imgs = move || {
            let mut imgs = gallery.get();
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

        // let update_imgs = move |gallery_elm: &HtmlDivElement,
        //                         remove_at_top: bool,
        //                         width: f32,
        //                         heigth: f32,
        //                         scroll_heigth: f32,
        //                         scroll_top: f32| {};

        Effect::new(move || {
            trace!("ON LOAD");
            let Some(gallery_elm) = gallery_ref.get() else {
                trace!("gallery NOT found");
                return;
            };
            trace!("gallery elm found");
            let width = gallery_elm.client_width() as u32;
            let heigth = gallery_elm.client_height() as f64;
            let scroll_heigth = gallery_elm.scroll_height() as f64;
            let scroll_top = gallery_elm.scroll_top() as f64;

            let prev_imgs = gallery.get_untracked();
            let count = calc_fit_count(width, heigth, row_height);
            let new_imgs = match fetch_init.clone() {
                Some(fetch_init) => {
                    let imgs = fetch_init(count);
                    if imgs.is_empty() {
                        return;
                    }
                    imgs
                }
                None => Img::rand_vec(count),
            };
            let (resized_imgs, scroll_by) =
                add_imgs_to_bottom(prev_imgs, new_imgs, width, heigth, row_height);
            gallery.set(resized_imgs);

            // update_imgs(
            //     &gallery_elm,
            //     false,
            //     width,
            //     heigth,
            //     scroll_heigth,
            //     scroll_top,
            // );
        });

        // let on_scroll = move |_: web_sys::Event| {
        //     trace!("SCROLLINGGG");
        //     // trace!("ON SCROLL");
        //     let Some(gallery_elm) = gallery_ref.get_untracked() else {
        //         trace!("gallery NOT found");
        //         return;
        //     };
        //     trace!("gallery elm found");
        //     let width = gallery_elm.client_width() as u32;
        //     let heigth = gallery_elm.client_height() as f64;
        //     let scroll_heigth = gallery_elm.scroll_height() as f64;
        //     let scroll_top = gallery_elm.scroll_top() as f64;
        //     let is_empty = gallery.with_untracked(|v| v.is_empty());
        //     let count = calc_fit_count(width, heigth, row_height);
        //     if is_empty || count == 0 {
        //         return;
        //     }

        //     // let scroll_at_bottom = scroll_heigth / 2.0 < scroll_top + heigth / 2.0;
        //     let scroll_at_top = scroll_top <= heigth;
        //     let scroll_at_bottom = scroll_heigth - (scroll_top + heigth) <= 100.0;
        //     if scroll_at_top && !is_empty && fetch_top.is_some() {
        //         let prev_imgs = gallery.get_untracked();
        //         let first_img = prev_imgs.first().cloned().unwrap();
        //         let fetch_top = fetch_top.clone().unwrap();
        //         let new_imgs = fetch_top(count, first_img);
        //         // return;

        //         // let new_imgs = Img::rand_vec(50);

        //         // let prev_total_height = get_total_height(&prev_imgs);
        //         let (resized_imgs, scroll_by) =
        //             add_imgs_to_top(prev_imgs, new_imgs, width, heigth, row_height);
        //         // let new_total_height = get_total_height(&resized_imgs);
        //         // let diff = prev_total_height - new_total_height;
        //         // trace!("scroll master: {diff}");
        //         gallery_elm.scroll_by_with_x_and_y(0.0, scroll_by);
        //         gallery.set(resized_imgs);
        //         // gallery.set(resized_imgs);
        //         //     update_imgs(
        //         //         &gallery_elm,
        //         //         false,
        //         //         width,
        //         //         heigth,
        //         //         scroll_heigth,
        //         //         scroll_top,
        //         //     );
        //     }

        //     if scroll_at_bottom {
        //         let prev_imgs = gallery.get_untracked();
        //         let new_imgs = match fetch_bottom.clone() {
        //             Some(fetch_bottom) => fetch_bottom(count, prev_imgs.last().cloned().unwrap()),
        //             None => Img::rand_vec(count),
        //         };
        //         // let prev_total_height = get_total_height(&prev_imgs);
        //         let _span = trace_span!("ON_SCROLL", scroll_top).entered();
        //         let (resized_imgs, scroll_by) =
        //             add_imgs_to_bottom(prev_imgs, new_imgs, width, heigth, row_height);
        //         // let new_total_height = get_total_height(&resized_imgs);
        //         // let diff = new_total_height - prev_total_height;
        //         trace!("scroll master: {scroll_by}");
        //         gallery.set(resized_imgs);
        //         gallery_elm.scroll_by_with_x_and_y(0.0, scroll_by);
        //         // update_imgs(&gallery_elm, true, width, heigth, scroll_heigth, scroll_top);
        //         // let len = gallery.with(|imgs| imgs.len());
        //         // trace!("img count: {len}");
        //     }
        // };

        let a = view! {
            <div
                id="gallery"
                node_ref=gallery_ref
                // on:scroll=on_scroll
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
    pub fn GalleryImg<FETCH_BTM_FN, FETCH_TOP_FN>(
        img: Img,
        index: usize,
        total_count: usize,
        run_fetch_bottom: FETCH_BTM_FN,
        run_fetch_top: FETCH_TOP_FN,
    ) -> impl IntoView
    where
        FETCH_BTM_FN: Fn() -> () + Send + Sync + 'static + Clone,
        FETCH_TOP_FN: Fn() -> () + Send + Sync + 'static + Clone,
    {
        let img_ref = NodeRef::<Div>::new();

        img_ref.add_intersection_observer_with_options(
            move |entry, observer| {
                let is_intersecting = entry.is_intersecting();
                if index == total_count.saturating_sub(1) && is_intersecting {
                    run_fetch_bottom();
                    trace!("intersection fn last {index} is intesecting: {is_intersecting}");
                } else if index == 0 && is_intersecting {
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

        let fn_background = move || format!("rgb({}, {}, {})", 50, 50, 50);
        let fn_left = move || format!("{view_left}px");
        // let fn_top = move || format!("{}px", view_top.get() + 100.0);
        let fn_top = move || format!("{view_top}px");
        let fn_width = move || format!("{view_width}px");
        let fn_height = move || format!("{view_height}px");
        let fn_text = move || format!("{img_width}x{img_height}");
        let fn_text2 = move || format!("{view_left}x{view_top}");
        let fn_text3 = move || format!("{img_id}");

        view! {
            <div
                // class="transition-all duration-300 ease-in-out text-white grid place-items-center bg-blue-950 absolute border border-red-600 overflow-hidden"
                class="text-white grid place-items-center bg-blue-950 absolute border border-red-600 overflow-hidden"

                node_ref=img_ref
                style:background-color=fn_background
                style:left=fn_left
                style:top=fn_top
                style:width=fn_width
                style:height=fn_height
            >
                <div>
                    <div>{fn_text3}</div>
                    <div>{fn_text}</div>
                    <div>{fn_text2}</div>
                </div>
            </div>
        }
    }

    #[derive(Debug, Clone)]
    pub struct Img {
        pub id: u64,
        pub row_id: usize,
        pub width: u32,
        pub height: u32,
        pub view_width: f64,
        pub view_height: f64,
        pub view_pos_x: f64,
        pub view_pos_y: f64,
    }

    impl Display for Img {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            //  write!(f, "Img {{id: {}, row_id: {}, width: {}, height: {}, view_width: OrderedFloat({:.5}), view_height: OrderedFloat({:.5}), view_pos_x: OrderedFloat({:.5}), view_pos_y: OrderedFloat({:.5})}}", self.id, self.row_id, self.width, self.height, self.view_width.get(), self.view_height.get(), self.view_pos_x.get(), self.view_pos_y.get())
            write!(
                f,
                "Img::new_full({}, {}, {}, {:.5}, {:.5}, {:.5}, {:.5})",
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

        // fn get_view_width(&self) -> f32 {
        //     self.view_width.get_untracked()
        // }

        fn get_view_height(&self) -> f64 {
            self.view_height
        }
    }

    impl Img {
        pub fn new(width: u32, height: u32) -> Self {
            let id = random_u64();

            Self {
                id,
                row_id: 0,
                width,
                height,
                view_width: 0.0,
                view_height: 0.0,
                view_pos_x: 0.0,
                view_pos_y: 0.0,
            }
        }

        pub fn rand(id: usize) -> Self {
            // let id = random_u64();
            let width = random_u32_ranged(500, 1000);
            let height = random_u32_ranged(500, 1000);

            Self {
                id: id as u64,
                row_id: 0,
                width,
                height,
                view_width: 0.0,
                view_height: 0.0,
                view_pos_x: 0.0,
                view_pos_y: 0.0,
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
        fn get_width(&self) -> u32;
        fn get_height(&self) -> u32;
        fn get_size(&self) -> (u32, u32);
        fn get_pos_y(&self) -> f64;
        fn get_pos_x(&self) -> f64;
        // fn get_view_width(&self) -> f32;
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

    pub fn get_total_height<IMG>(mut imgs: &[IMG]) -> f64
    where
        IMG: ResizableImage + Clone + Display + Debug,
    {
        imgs.last()
            .map(|img| img.get_pos_y() + img.get_view_height())
            .unwrap_or_default() as f64
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
        let height_before_remove = get_total_height(&imgs);
        trace!("stage 0(KOKheight_before_remove: {height_before_remove}): {imgs:#?}");
        if let Some(cut_index) = remove_until_fit_from_top(&mut imgs, heigth * 3.0) {
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
        // let Some(offset) = imgs
        //     .len()
        //     .checked_sub(1)
        //     .map(|offset| get_row_start(&mut imgs, offset))
        // else {
        //     return imgs;
        // };
        // trace!("stage 3: {imgs:#?}");
        imgs.extend(new_imgs);
        trace!("stage 4: {imgs:#?}");
        let rows = get_rows_to_bottom(&imgs, offset, width, row_height);
        set_rows_to_bottom(&mut imgs, &rows, width);
        let height_final = get_total_height(&imgs);
        // let scroll_by =
        //     (height_final - height_before_remove) - (height_before_remove - height_after_remove);
        // let scroll_by =
        //     (height_before_remove - height_final) - (height_before_remove - height_after_remove);
        let scroll_by = height_after_remove - height_before_remove;
        trace!(
            "stage 5(KOKheight_before_remove: {height_before_remove}, height_after_remove: {height_after_remove}, height_final: {height_final}, scroll_by: {scroll_by}): {imgs:#?}"
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
        if new_imgs.is_empty() {
            return (imgs, 0.0);
        }
        let height_before_remove = get_total_height(&imgs);
        trace!("stage 0: {imgs:#?}");
        if let Some(cut_index) = remove_until_fit_from_bottom(&mut imgs, heigth * 3.0) {
            imgs = imgs[..cut_index].to_vec();
            trace!("stage 1 ({cut_index}): {imgs:#?}");
        }

        let height_after_remove = get_total_height(&imgs);
        let Some(offset) = imgs.len().strict_add(new_imgs.len()).checked_sub(2) else {
            return (imgs, 0.0);
        };
        // .inspect(|offset| trace!(offset))

        new_imgs.extend(imgs);
        imgs = new_imgs;

        let offset = get_row_end(&mut imgs, offset);
        trace!("stage 4(offset: {offset}): {imgs:#?}");
        let rows = get_rows_to_top(&imgs, offset, width, row_height);
        set_rows_to_top(&mut imgs, &rows, width);
        trace!("stage 2: {imgs:#?}");
        normalize_imgs_y_v2(&mut imgs);
        let height_final = get_total_height(&imgs);
        // let scroll_by = (height_final - height_before_remove) ;
        // let scroll_by =
        //     (height_final - height_before_remove) + (height_before_remove + height_after_remove);
        let scroll_by = height_final - height_after_remove;

        trace!(
            "stage 5(KOKheight_before_remove: {height_before_remove}, height_after_remove: {height_after_remove}, height_final: {height_final}, scroll_by: {scroll_by}): {imgs:#?}"
        );
        // trace!("stage 5: {imgs:#?}");
        (imgs, scroll_by)
        // trace!("stage 3: {imgs:#?}");
        // trace!("stage 4(offset: {offset}): {imgs:#?}");
    }

    // fn update_imgs<IMG: ResizableImage + Clone + Display>(
    //     mut imgs: Vec<IMG>,
    //     mut new_imgs: Vec<IMG>,
    //     row_height: u32,
    //     remove_at_top: bool,
    //     width: u32,
    //     heigth: f32,
    //     scroll_heigth: f32,
    //     scroll_top: f32,
    // ) -> (Vec<IMG>, f64) {
    //     // let width = width as u32;
    //     // let can_fit_count = calc_fit_count(width, heigth as u32, NEW_IMG_HEIGHT) * 2;
    //     // let mut new_imgs = Img::rand_vec(can_fit_count as usize);
    //     trace!("stage 1 - input - {}", vec_img_to_string(&imgs));

    //     let removed_height = if let Some((cut_at, removed_height)) =
    //         remove_imgs(&imgs, heigth, scroll_heigth, remove_at_top)
    //     {
    //         if remove_at_top {
    //             debug!("cutting images at {cut_at}..{}", imgs.len());
    //             imgs = imgs[cut_at..].to_vec();
    //         } else {
    //             debug!("cutting images at {}..={cut_at}", 0);
    //             imgs = imgs[..=cut_at].to_vec();
    //         }
    //         removed_height
    //     } else {
    //         0.0
    //     };
    //     trace!("stage 2 - removed - {}", vec_img_to_string(&imgs));

    //     let old_imgs_len = imgs.len();
    //     let new_imgs_len = new_imgs.len();
    //     let offset = new_imgs_len.saturating_sub(old_imgs_len);

    //     if remove_at_top {
    //         imgs.extend(new_imgs);
    //     } else {
    //         new_imgs.extend(imgs);
    //         imgs = new_imgs;
    //     }
    //     trace!("stage 3 - added - {}", vec_img_to_string(&imgs));

    //     let y = if old_imgs_len > 0 {
    //         if remove_at_top {
    //             debug!("running {row_height} {width} {offset} {}", false);
    //             resize(&mut imgs, row_height, width, offset, false)
    //         } else {
    //             debug!("running {row_height} {width} {offset} {}", true);
    //             resize(&mut imgs, row_height, width, offset, true)
    //         }
    //     } else {
    //         debug!("running {row_height} {width} {} {}", 0, false);
    //         resize(&mut imgs, row_height, width, 0, false)
    //     };

    //     trace!("stage 4 {}", vec_img_to_string(&imgs));
    //     if remove_at_top {
    //         let diff = y - scroll_heigth;
    //         let new_scroll_top = diff - removed_height - 100.0;
    //         trace!(
    //             "SCROLL: y {y}: removed height {removed_height} : scroll top {scroll_top}: scroll height {scroll_heigth}: diff {diff}: new_scroll_top {new_scroll_top}"
    //         );
    //         // gallery_elm.scroll_by_with_x_and_y(0.0, );
    //         (imgs, new_scroll_top as f64)
    //     } else {
    //         let diff = y - scroll_heigth;
    //         let new_scroll_top = diff + removed_height + 100.0;
    //         trace!(
    //             "SCROLL: y {y}: removed height {removed_height} : scroll top {scroll_top}: scroll height {scroll_heigth}: diff {diff}: new_scroll_top {new_scroll_top}"
    //         );
    //         // gallery_elm.scroll_by_with_x_and_y(0.0, new_scroll_top as f64);
    //         (imgs, new_scroll_top as f64)
    //     }
    // }

    pub fn normalize_imgs_y_v2<IMG>(imgs: &mut [IMG])
    where
        IMG: ResizableImage,
    {
        imgs.first()
            .map(|img| img.get_pos_y())
            .and_then(|y| if y == 0.0 { None } else { Some(y) })
            .map(|y| {
                imgs.iter_mut().for_each(|img| {
                    img.set_pos_y(img.get_pos_y() - y);
                })
            });
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

    // pub fn scroll_height_calc<IMG>(
    //     imgs: &[IMG],
    // ) -> f32
    // where
    //     IMG: ResizableImage {
    //         imgs
    //     }

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
            .map(|i| imgs.len().strict_sub(i))
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

    // pub fn remove_imgs<IMG>(
    //     imgs: &[IMG],
    //     view_height: f32,
    //     scroll_height: f32,
    //     from_top: bool,
    // ) -> Option<(usize, f32)>
    // where
    //     IMG: ResizableImage,
    // {
    //     let len = imgs.len();
    //     let span = trace_span!("IMG_REMOVAL", len, view_height, scroll_height, from_top).entered();

    //     let Some((last_y, last_height)) = (if from_top {
    //         imgs.first().map(|v| (v.get_pos_y(), v.get_view_height()))
    //     } else {
    //         imgs.last().map(|v| (v.get_pos_y(), v.get_view_height()))
    //     }) else {
    //         return None;
    //     };

    //     let mut prev_y: f32 = last_y;
    //     let mut removed_y = last_height;
    //     let mut removed_row_index = 0;
    //     let mut cut_index = if from_top { 0 } else { len - 1 };
    //     let available_height_for_removal = scroll_height - view_height;

    //     if available_height_for_removal - removed_y < 0.0 {
    //         trace!("nothing to remove");
    //         return None;
    //     }

    //     loop {
    //         let Some(img) = imgs.get(cut_index) else {
    //             trace!("end reached");
    //             break;
    //         };
    //         let id = img.get_id();
    //         let current_y = img.get_pos_y();
    //         let is_new_row = current_y != prev_y;
    //         if is_new_row {
    //             removed_row_index += 1;
    //         }
    //         let span = trace_span!(
    //             "LOOP",
    //             id,
    //             removed_row_index,
    //             cut_index,
    //             prev_y,
    //             removed_y,
    //             available_height_for_removal
    //         )
    //         .entered();

    //         if is_new_row {
    //             prev_y = current_y;
    //             let current_height = img.get_view_height();
    //             let cant_remove_more =
    //                 available_height_for_removal - (removed_y + current_height) < 0.0;

    //             if cant_remove_more {
    //                 trace!(
    //                     "cant remove {} = overflow {}",
    //                     current_height,
    //                     available_height_for_removal - (removed_y + current_height)
    //                 );
    //                 return Some((if from_top { cut_index } else { cut_index }, removed_y));
    //             }
    //             trace!("removing +{current_height}");
    //             removed_y += current_height;
    //         } else {
    //             trace!("skipping");
    //         }

    //         if from_top {
    //             if cut_index == len {
    //                 error!("cut index overflow: {}/{}", cut_index + 1, len);
    //                 return None;
    //             }
    //             cut_index += 1;
    //         } else {
    //             if cut_index == 0 {
    //                 error!("cut index overflow: {}/{}", -1, len);
    //                 return None;
    //             }
    //             cut_index -= 1;
    //         }
    //     }

    //     None
    // }

    // pub fn get_row_start_or_end<IMG>(imgs: &[IMG], selected_img_index: usize, rev: bool) -> usize
    // where
    //     IMG: ResizableImage,
    // {
    //     let len = imgs.len();
    //     let last_img_index = len.saturating_sub(1);
    //     if (selected_img_index == 0 && rev) || (selected_img_index == last_img_index && !rev) {
    //         return selected_img_index;
    //     }
    //     let selected_img = &imgs[selected_img_index];
    //     let selected_pos_y = selected_img.get_pos_y();

    //     let mut i: usize = selected_img_index;
    //     let step = if rev { -1 } else { 1 };
    //     loop {
    //         let next_i = i.strict_add_signed(step);
    //         let img = &imgs[next_i];
    //         if img.get_pos_y() != selected_pos_y {
    //             break;
    //         }
    //         i = next_i;
    //     }
    //     i
    // }

    //         if rev {
    // itertools::Either::Left(imgs.iter().rev())
    //         } else {
    // itertools::Either::Right(imgs.iter())
    //         }.position(|img| img.get_pos_y() != imgs[offset]).map(f)

    //         if rev {
    //         } else {
    // itertools::Either::Right(imgs.iter())
    //         }.position(|img| img.get_pos_y() != imgs[offset]).map(f)

    // .position(|img| {
    //     trace!(
    //         "{} {} = {} != {} = {}",
    //         img.get_id(),
    //         imgs[offset].get_id(),
    //         img.get_pos_y(),
    //         imgs[offset].get_pos_y(),
    //         img.get_pos_y() != imgs[offset].get_pos_y()
    //     );

    //     img.get_pos_y() != imgs[offset].get_pos_y()
    // })

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
                        if img_fits_in_row && (row.aspect_ratio + ratio) < 5.0 {
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

    // append_or_add_row(img, rows, &mut row_width, row_height, max_width, i)
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
                        if img_fits_in_row && (row.aspect_ratio + ratio) < 5.0 {
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

    // pub fn append_or_add_row(
    //     img: &impl ResizableImage,
    //     mut rows: Vec<Row>,
    //     row_width_total: &mut f64,
    //     row_height: u32,
    //     max_width: u32,
    //     i: usize,
    // ) -> Vec<Row> {
    //     let row = rows.last_mut().unwrap();
    //     let (scaled_width, ratio) = img.scaled_by_height(row_height);
    //     let img_fits_in_row = *row_width_total + scaled_width <= max_width as f64;
    //     if img_fits_in_row {
    //         row.aspect_ratio += ratio;
    //         row.end_at = i;
    //         *row_width_total += scaled_width;
    //     } else {
    //         rows.push(Row {
    //             aspect_ratio: ratio,
    //             start_at: i,
    //             end_at: i,
    //         });
    //         *row_width_total = scaled_width;
    //     }

    //     rows
    // }

    // pub fn get_rows_rev(
    //     imgs: &[impl ResizableImage],
    //     // offset: usize,
    //     max_width: u32,
    //     row_height: u32,
    // ) -> Vec<Row> {
    //     imgs.iter()
    //         // .rev()
    //         .enumerate()
    //         // .skip(imgs.len().saturating_sub(offset + 1))
    //         .inspect(|(i, img)| {
    //             // trace!("i={} img_id={}", i, img.get_id());
    //         })
    //         .fold(
    //             (
    //                 Vec::<Row>::from([Row {
    //                     // start_at: offset,
    //                     // end_at: offset,
    //                     ..Default::default()
    //                 }]),
    //                 0.0,
    //             ),
    //             |(mut rows, mut row_width), (i, img)| {
    //                 // let i = offset.saturating_sub(i);
    //                 let (width, height) = img.get_size();
    //                 let ratio = width as f32 / height as f32;
    //                 let scaled_w =
    //                     width as f32 - (height.saturating_sub(row_height) as f32 * ratio);
    //                 let img_fits_in_row = row_width + scaled_w <= max_width as f32;
    //                 let row = rows.last_mut().unwrap();
    //                 if img_fits_in_row {
    //                     row.aspect_ratio += ratio;
    //                     row.end_at += 1;
    //                     row_width += scaled_w;
    //                 } else {
    //                     rows.push(Row {
    //                         aspect_ratio: ratio,
    //                         start_at: i,
    //                         end_at: i,
    //                     });
    //                     row_width = scaled_w;
    //                 }
    //                 trace!("i={} img_id={}, {:#?}", i, img.get_id(), rows);
    //                 (rows, row_width)
    //             },
    //         )
    //         .0
    // }

    pub fn set_rows_to_bottom(
        imgs: &mut [impl ResizableImage + Display],
        rows: &[Row],
        max_width: u32,
    ) {
        // let Some() =
        // else {
        //     return;
        // };
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
        // let last_row_width_is_small = rows
        //     .last()
        //     .map(|v| v.total_width <= max_width as f64)
        //     .unwrap_or(false);
        // let chunks: &[(&[Row], f64)] = &[(rows, max_width as f64)];
        trace!("chunks {chunks:#?}");
        // trace!("chunks2 {chunks2:#?}");
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

        rows.iter().for_each(|row| {
            let row_height: f64 = max_width as f64 / row.aspect_ratio;
            let mut row_pos_x = 0.0;
            row_pos_y -= row_height;
            imgs[row.start_at..=row.end_at].iter_mut().for_each(|img| {
                let (width, height) = img.get_size();
                let new_width = row_height * (width as f64 / height as f64);
                img.set_size(new_width, row_height, row_pos_x, row_pos_y);
                row_pos_x += new_width;
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

    // pub fn get_row_start_v2(imgs: &mut [impl ResizableImage], offset: usize) -> usize {
    //     imgs.iter()
    //         .rev()
    //         .position(|img| img.get_pos_y() != imgs[offset].get_pos_y())
    //         .unwrap_or_else(|| imgs.len())
    // }

    // pub fn resize<IMG>(
    //     imgs: &mut [IMG],
    //     row_height: u32,
    //     max_width: u32,
    //     offset: usize,
    //     rev: bool,
    // ) -> f32
    // where
    //     IMG: ResizableImage,
    // {
    //     let len = imgs.len();
    //     let mut total_w = 0;
    //     let mut total_ratio = 0.0;
    //     let mut pos_y: f64 = 0.0;
    //     let mut cursor_row_start: usize = offset;
    //     let mut cursor_row_end: usize = cursor_row_start;

    //     let _span = trace_span!("IMG_RESIZE", len, row_height,).entered();

    //     // find start of the row
    //     // or end of a row in rev
    //     let start_or_end_str = if rev { "START" } else { "END" };
    //     let stage_span = trace_span!("FIND ROW ", "{}", start_or_end_str).entered();
    //     if cursor_row_end < len {
    //         let current_pos_y = imgs[offset].get_pos_y();
    //         //let mut sub_cursor = cursor;

    //         let mut new_offset = offset;

    //         loop {
    //             // trace!(
    //             //     "find first row loop break: ({new_offset} == 0 && !{rev}) = {} || ({rev} && {new_offset} >= {len}) = {}",
    //             //     (new_offset == 0 && !rev),
    //             //     (rev && new_offset >= len)
    //             // );
    //             let end_of_array = (new_offset == 0 && !rev) || (rev && new_offset >= len);
    //             if end_of_array {
    //                 trace!("end of array {}/{}", new_offset, len);
    //                 break;
    //             }

    //             let img = &imgs[new_offset];
    //             let prev_pos_y = img.get_pos_y();
    //             let id = img.get_id();
    //             let is_new_row = prev_pos_y != current_pos_y;

    //             let _span = trace_span!(
    //                 "LOOP",
    //                 id,
    //                 new_offset,
    //                 cursor_row_start,
    //                 cursor_row_end,
    //                 total_w,
    //                 total_ratio,
    //                 pos_y,
    //             )
    //             .entered();
    //             // trace!(
    //             //     "finding first row {new_offset} : {prev_pos_y} != {current_pos_y} == {}",
    //             //     prev_pos_y != current_pos_y
    //             // );
    //             if is_new_row {
    //                 trace!("row {} at {}", start_or_end_str, new_offset);
    //                 break;
    //             } else {
    //                 trace!("skipping {}", new_offset);
    //             }

    //             if rev {
    //                 new_offset += 1;
    //             } else {
    //                 new_offset -= 1;
    //             }
    //         }

    //         if new_offset != offset {
    //             if rev {
    //                 trace!("before setting end: {cursor_row_end}");
    //                 cursor_row_end = new_offset.saturating_sub(1);
    //                 trace!("after setting end: {cursor_row_end}");
    //                 cursor_row_start = cursor_row_end;
    //             } else {
    //                 trace!("before setting end: {cursor_row_end}");
    //                 cursor_row_end = (new_offset + 1).clamp(0, len.saturating_sub(1));
    //                 trace!("after setting end: {cursor_row_end}");
    //                 cursor_row_start = cursor_row_end;
    //             }
    //         }

    //         // if cursor_row_end != 0 && cursor_row_end < len {
    //         //     if rev {
    //         //         trace!("before setting end: {cursor_row_end}");
    //         //         cursor_row_end -= 1;
    //         //         trace!("after setting end: {cursor_row_end}");
    //         //         cursor_row_start = cursor_row_end;
    //         //     } else {
    //         //         trace!("before setting end: {cursor_row_end}");
    //         //         cursor_row_end += 1;
    //         //         trace!("after setting end: {cursor_row_end}");
    //         //         cursor_row_start = cursor_row_end;
    //         //     }
    //         // }

    //         //    if i > 0 {
    //         //     if rev {
    //         //         trace!("before setting end: {cursor_row_end}");
    //         //         cursor_row_end -= 1;
    //         //         trace!("after setting end: {cursor_row_end}");
    //         //         cursor_row_start = cursor_row_end;
    //         //     } else {
    //         //         trace!("before setting end: {cursor_row_end}");
    //         //         cursor_row_end += 1;
    //         //         trace!("after setting end: {cursor_row_end}");
    //         //         cursor_row_start = cursor_row_end;
    //         //     }
    //         //    }
    //     }
    //     stage_span.exit();

    //     // trace!("start: {cursor_row_start}, end: {cursor_row_end}");

    //     //read the first image
    //     let stage_span = trace_span!("READ FIRST IMG").entered();
    //     if cursor_row_end < len {
    //         let img = &imgs[cursor_row_end];
    //         let (width, height) = img.get_size();
    //         let ratio = width as f64 / height as f64;
    //         let scaled_w = width - (height.saturating_sub(row_height) as f64 * ratio) as u32;

    //         total_w = scaled_w;
    //         total_ratio = ratio;
    //         pos_y = if rev { img.get_pos_y() } else { 0.0_f64 };
    //         trace!(total_w, total_ratio, pos_y);

    //         if rev {
    //             cursor_row_end = cursor_row_end.saturating_sub(1);
    //         } else {
    //             cursor_row_end += 1;
    //         }
    //     }
    //     stage_span.exit();

    //     // do the actual resizing
    //     let stage_span = trace_span!("RESIZE").entered();
    //     loop {
    //         if cursor_row_end >= len || (rev && cursor_row_end == 0) {
    //             trace!("end of array {}/{}", cursor_row_end, len);
    //             break;
    //         }

    //         let (id, scaled_w, ratio, img_fits_in_row) = {
    //             let img = &imgs[cursor_row_end];
    //             let id = img.get_id();
    //             let (width, height) = img.get_size();
    //             let ratio = width as f32 / height as f32;
    //             let scaled_w = width - (height.saturating_sub(row_height) as f32 * ratio) as u32;
    //             let img_fits_in_row = total_w + scaled_w <= max_width;
    //             (id, scaled_w, ratio, img_fits_in_row)
    //         };

    //         let _span = trace_span!(
    //             "LOOP",
    //             id,
    //             cursor_row_start,
    //             cursor_row_end,
    //             total_w,
    //             total_ratio,
    //             pos_y,
    //         )
    //         .entered();

    //         if !img_fits_in_row {
    //             trace!(
    //                 "resizing imgs from {} to {}",
    //                 cursor_row_start, cursor_row_end
    //             );
    //             let row_height: f64 = max_width as f64 / total_ratio;
    //             let mut pos_x: f64 = if rev { max_width as f64 } else { 0.0_f64 };

    //             loop {
    //                 if cursor_row_start == cursor_row_end {
    //                     break;
    //                 }

    //                 let img = &mut imgs[cursor_row_start];
    //                 let (width, height) = img.get_size();
    //                 let new_width = row_height * (width as f64 / height as f64);
    //                 let new_height = row_height;

    //                 if rev {
    //                     pos_x -= new_width;
    //                 }

    //                 img.set_size(new_width, new_height, pos_x, pos_y);

    //                 if rev {
    //                     cursor_row_start -= 1;
    //                 } else {
    //                     pos_x += new_width;
    //                     cursor_row_start += 1;
    //                 }
    //             }

    //             total_w = 0;
    //             total_ratio = 0.0;
    //             if rev {
    //                 pos_y -= row_height;
    //             } else {
    //                 pos_y += row_height;
    //             }
    //         } else {
    //             trace!("adding for resize {}", cursor_row_end);
    //         }

    //         total_w += scaled_w;
    //         total_ratio += ratio;

    //         if rev {
    //             cursor_row_end -= 1;
    //         } else {
    //             cursor_row_end += 1;
    //         }
    //     }

    //     // finish unfilled row resizing
    //     if total_w != 0 {
    //         let row_gap = max_width.saturating_sub(total_w);
    //         let missing_imgs = row_gap.saturating_div(row_height);
    //         let row_ratio = total_ratio + missing_imgs as f32;
    //         let row_height: f32 = max_width as f32 / row_ratio;
    //         let mut pos_x: f32 = if rev { max_width as f32 } else { 0.0 };

    //         let mut i = cursor_row_start;

    //         trace!("resizing imgs from {} to {}", i, len);
    //         loop {
    //             // trace!("in last loop");
    //             if (i == len) {
    //                 break;
    //             }

    //             trace!("resizing: {i}");

    //             let img = &mut imgs[i];
    //             let (width, height) = img.get_size();
    //             let new_width = row_height * (width as f32 / height as f32);
    //             let new_height = row_height;

    //             if rev {
    //                 pos_x -= new_width;
    //             }

    //             img.set_size(new_width, new_height, pos_x, pos_y);

    //             if rev {
    //                 if i == 0 {
    //                     break;
    //                 }
    //                 i -= 1;
    //             } else {
    //                 pos_x += new_width;
    //                 i += 1;
    //             }
    //         }
    //     }
    //     stage_span.exit();

    //     normalize_y(imgs);
    //     //        imgs.pos_y

    //     imgs.last()
    //         .map(|v| v.get_view_height() + v.get_pos_y())
    //         .unwrap_or(0.0)
    // }

    pub fn calc_fit_count(width: u32, height: f64, img_height: u32) -> usize {
        ((width * height as u32) / (img_height * img_height)) as usize
    }

    #[cfg(test)]
    mod resize_tests {
        use crate::{
            app::components::gallery::{
                Gallery, Row, add_imgs_to_bottom, add_imgs_to_top, get_row_end, get_row_start,
                get_row_start_or_end, get_rows_to_bottom, get_rows_to_top, normalize_imgs_y_v2,
                remove_until_fit_from_bottom, remove_until_fit_from_top, resize_v2,
                set_rows_to_bottom, set_rows_to_top,
            },
            logger,
        };
        use leptos::{mount::mount_to, prelude::*, task::tick};
        use ordered_float::OrderedFloat;
        use pretty_assertions::{assert_eq, assert_ne};
        use std::{fmt::Display, str::FromStr};
        use test::Bencher;
        use test_log::test;
        use tracing::{level_filters::LevelFilter, trace};
        use wasm_bindgen::JsCast;
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
                //  write!(f, "Img {{id: {}, row_id: {}, width: {}, height: {}, view_width: OrderedFloat({:.5}), view_height: OrderedFloat({:.5}), view_pos_x: OrderedFloat({:.5}), view_pos_y: OrderedFloat({:.5})}}", self.id, self.row_id, self.width, self.height, self.view_width.get(), self.view_height.get(), self.view_pos_x.get(), self.view_pos_y.get())
                write!(
                    f,
                    "Img {{ width: {}, height: {}, view_width: OrderedFloat({:.5}), view_height: OrderedFloat({:.5}), view_pos_x: OrderedFloat({:.5}), view_pos_y: OrderedFloat({:.5})}}",
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

            pub const fn new_with_y(id: u32, width: u32, height: u32, y: f64) -> Self {
                Self {
                    id,
                    width,
                    height,
                    view_width: OrderedFloat(0.0),
                    view_height: OrderedFloat(0.0),
                    view_pos_x: OrderedFloat(0.0),
                    view_pos_y: OrderedFloat(y),
                }
            }

            pub fn ratio(&self) -> f32 {
                self.width as f32 / self.height as f32
            }

            pub fn get_scaled_width(&self, desired_height: u32) -> u32 {
                let ratio = self.ratio();
                self.width - (self.height.saturating_sub(desired_height) as f32 * ratio) as u32
            }
            pub fn rand(id: u32) -> Self {
                let width = (id + 500) % 500;
                let height = (id + 500) % 500;
                // let width = rand::random_range(500..=1000);
                // let height = rand::random_range(500..=1000);

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

            pub fn rand_vec(n: usize) -> Vec<Self> {
                let mut output = Vec::new();
                for id in 0..n {
                    output.push(Img::rand(id as u32));
                }
                output
            }
        }

        impl ResizableImage for Img {
            fn get_id(&self) -> u64 {
                self.id as u64
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
            // fn get_view_width(&self) -> f32 {
            //     *self.view_width
            // }
            fn get_view_height(&self) -> f64 {
                *self.view_height
            }
            fn set_size(&mut self, view_width: f64, view_height: f64, pos_x: f64, pos_y: f64) {
                // println!("setting yyyyyyyyyyy {}", pos_y);
                // println!("setting xxxxxxxxxxx {}", pos_x);
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

        // pub const STATIC_UNIT: &&() = &&();

        // #[inline(never)]
        // pub fn lifetime_translator_mut<'a, 'b, T: ?Sized>(
        //     _val_a: &'a &'b (),
        //     val_b: &'b mut T,
        // ) -> &'a mut T {
        //     val_b
        // }

        // pub fn null_mut<'a, T: 'static>() -> &'a mut T {
        //     transmute(0usize)
        // }

        // pub fn transmute<A, B>(obj: A) -> B {
        //     use std::hint::black_box;

        //     // The layout of `DummyEnum` is approximately
        //     // DummyEnum {
        //     //     is_a_or_b: u8,
        //     //     data: usize,
        //     // }
        //     // Note that `data` is shared between `DummyEnum::A` and `DummyEnum::B`.
        //     // This should hopefully be more reliable than spamming the stack with a value and hoping the memory
        //     // is placed correctly by the compiler.
        //     #[allow(dead_code)]
        //     enum DummyEnum<A, B> {
        //         A(Option<Box<A>>),
        //         B(Option<Box<B>>),
        //     }

        //     #[inline(never)]
        //     fn transmute_inner<A, B>(dummy: &mut DummyEnum<A, B>, obj: A) -> B {
        //         let DummyEnum::B(ref_to_b) = dummy else {
        //             unreachable!()
        //         };
        //         let ref_to_b = expand_mut(ref_to_b);
        //         *dummy = DummyEnum::A(Some(Box::new(obj)));
        //         black_box(dummy);

        //         *ref_to_b.take().unwrap()
        //     }

        //     transmute_inner(black_box(&mut DummyEnum::B(None)), obj)
        // }

        // pub fn expand_mut<'a, 'b, T: ?Sized>(x: &'a mut T) -> &'b mut T {
        //     let f: for<'x> fn(_, &'x mut T) -> &'b mut T = lifetime_translator_mut;
        //     f(STATIC_UNIT, x)
        // }

        // pub fn segfault() -> ! {
        //     let null = null_mut::<u8>();
        //     *null = 42;

        //     unreachable!("Sorry, your platform is too strong.")
        // }

        // #[test]
        // fn seg() {
        //     segfault();
        // }

        // #[wasm_bindgen_test]
        // async fn test_component_gallery() {
        //     console_error_panic_hook::set_once();
        //     logger::simple_shell_logger_init();

        //     mount_to_body(|| view! { <Gallery /> });

        //     let document = document();
        //     let window = window();
        //     let gallery = document.get_element_by_id("gallery").unwrap();

        //     tick().await;
        //     gallery.set_scroll_top(500);
        //     tick().await;
        //     tick().await;
        //     gallery
        //         .dispatch_event(&web_sys::Event::new("resize").unwrap())
        //         .unwrap();
        //     tick().await;
        //     gallery
        //         .dispatch_event(&web_sys::Event::new("resize").unwrap())
        //         .unwrap();
        //     tick().await;

        //     let html1 = gallery.outer_html();
        //     let html2 = view! { <div>"hello"</div> }.build().outer_html();

        //     trace!("wow");
        //     assert_eq!(html2, html1);
        // }

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
            let mut imgs = Vec::<Img>::from([
                //row 0
            ]);
            let resized_imgs = resize_v2(imgs, 1000, 500);
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
                Row::new(4, 4, 2.0, 1000, 500.0),
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
                Img::new_full(4, 1000, 500, 500.0, 250.0, 0.0, 1000.0),
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

            let rows = Vec::from([Row::new(1, 2, 2.0, 0, 0.0), Row::new(0, 0, 2.0, 0, 0.0)]);

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
                Row::new(3, 4, 2.0, 0, 0.0),
                Row::new(2, 2, 2.0, 0, 0.0),
                Row::new(0, 1, 2.0, 0, 0.0),
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
            assert_eq!(expected_imgs, imgs);
        }

        // #[test]
        // fn update_imgs_test() {
        //     trace!("=======UPDATING IMGS=======");
        //     let imgs = Vec::from([
        //         Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
        //         Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 500.0),
        //         Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 500.0),
        //         Img::new_full(3, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
        //     ]);
        //     let new_imgs = Vec::from([
        //         Img::new(4, 500, 500),
        //         Img::new(5, 500, 500),
        //         Img::new(6, 500, 500),
        //         Img::new(7, 500, 500),
        //     ]);
        //     let (imgs, scroll) = update_imgs(imgs, new_imgs, 500, false, 1000, 500.0, 1500.0, 0.0);
        //     assert_eq!(
        //         Vec::from([
        //             Img::new_full(4, 500, 500, 500.0, 500.0, 0.0, 0.0),
        //             Img::new_full(5, 500, 500, 500.0, 500.0, 500.0, 0.0),
        //             Img::new_full(6, 500, 500, 500.0, 500.0, 0.0, 500.0),
        //             Img::new_full(7, 500, 500, 500.0, 500.0, 500.0, 500.0),
        //             Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
        //         ]),
        //         imgs
        //     );
        //     trace!("=======UPDATING IMGS=======");
        //     let imgs = Vec::from([
        //         Img::new_full(0, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
        //         Img::new_full(1, 500, 500, 500.0, 500.0, 0.0, 500.0),
        //         Img::new_full(2, 500, 500, 500.0, 500.0, 500.0, 500.0),
        //         Img::new_full(3, 1000, 500, 1000.0, 500.0, 0.0, 1000.0),
        //     ]);
        //     let new_imgs = Vec::from([
        //         Img::new(4, 500, 500),
        //         Img::new(5, 500, 500),
        //         Img::new(6, 500, 500),
        //         Img::new(7, 500, 500),
        //     ]);
        //     let (imgs, scroll) = update_imgs(imgs, new_imgs, 500, true, 1000, 500.0, 1500.0, 0.0);
        //     assert_eq!(
        //         Vec::from([
        //             Img::new_full(3, 1000, 500, 1000.0, 500.0, 0.0, 0.0),
        //             Img::new_full(4, 500, 500, 500.0, 500.0, 0.0, 500.0),
        //             Img::new_full(5, 500, 500, 500.0, 500.0, 500.0, 500.0),
        //             Img::new_full(6, 500, 500, 500.0, 500.0, 0.0, 1000.0),
        //             Img::new_full(7, 500, 500, 500.0, 500.0, 500.0, 1000.0),
        //         ]),
        //         imgs
        //     );
        // }

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

        // #[test]
        // fn resize_imgs_two() {
        //     let mut imgs = [Img::new(0, 640, 480), Img::new(1, 720, 1280)];
        //     resize(&mut imgs, 200, 1000, 0, false);
        //     assert_eq!(
        //         [
        //             Img {
        //                 id: 0,
        //                 width: 640,
        //                 height: 480,
        //                 view_width: OrderedFloat(272.34042),
        //                 view_height: OrderedFloat(204.25531),
        //                 view_pos_x: OrderedFloat(0.0),
        //                 view_pos_y: OrderedFloat(0.0),
        //             },
        //             Img {
        //                 id: 1,
        //                 width: 720,
        //                 height: 1280,
        //                 view_width: OrderedFloat(114.893616),
        //                 view_height: OrderedFloat(204.25531),
        //                 view_pos_x: OrderedFloat(272.34042),
        //                 view_pos_y: OrderedFloat(0.0),
        //             }
        //         ],
        //         imgs
        //     )
        // }

        // #[test]
        // fn resize_imgs_three() {
        //     let mut imgs = Vec::from([
        //         Img::new(0, 1000, 200),
        //         Img::new(1, 100, 1000),
        //         Img::new(2, 1000, 100),
        //     ]);
        //     resize(&mut imgs, 200, 1000, 0, false);

        //     assert_eq!(
        //         [
        //             Img {
        //                 id: 0,
        //                 width: 1000,
        //                 height: 200,
        //                 view_width: OrderedFloat(1000.0),
        //                 view_height: OrderedFloat(200.0),
        //                 view_pos_x: OrderedFloat(0.0),
        //                 view_pos_y: OrderedFloat(0.0),
        //             },
        //             Img {
        //                 id: 1,
        //                 width: 100,
        //                 height: 1000,
        //                 view_width: OrderedFloat(1000.0),
        //                 view_height: OrderedFloat(10000.0),
        //                 view_pos_x: OrderedFloat(0.0),
        //                 view_pos_y: OrderedFloat(200.0),
        //             },
        //             Img {
        //                 id: 2,
        //                 width: 1000,
        //                 height: 100,
        //                 view_width: OrderedFloat(1000.0),
        //                 view_height: OrderedFloat(100.0),
        //                 view_pos_x: OrderedFloat(0.0),
        //                 view_pos_y: OrderedFloat(10200.0),
        //             },
        //         ],
        //         *imgs
        //     )
        // }

        // #[bench]
        // fn resize_bench(b: &mut Bencher) {
        //     b.iter(|| {
        //         let mut imgs = Img::rand_vec(10000);
        //         // let mut imgs = [Img::new(0, 640, 480)];
        //         resize(&mut imgs, 200, 1000, 0, false);
        //     });
        // }

        // #[test]
        // fn resize_imgs_single_normal() {
        //     let mut imgs = [Img::new(0, 640, 480)];
        //     resize(&mut imgs, 200, 1000, 0, false);
        //     assert_eq!(
        //         [Img {
        //             id: 0,
        //             width: 640,
        //             height: 480,
        //             view_width: OrderedFloat(307.69232),
        //             view_height: OrderedFloat(230.76923),
        //             view_pos_x: OrderedFloat(0.0),
        //             view_pos_y: OrderedFloat(0.0),
        //         },],
        //         imgs
        //     )
        // }

        // #[test]
        // fn resize_imgs_single_rev() {
        //     let mut imgs = [Img::new(0, 640, 480)];
        //     resize(&mut imgs, 200, 1000, 0, true);
        //     assert_eq!(
        //         [Img {
        //             id: 0,
        //             width: 640,
        //             height: 480,
        //             view_width: OrderedFloat(307.69232),
        //             view_height: OrderedFloat(230.76923),
        //             view_pos_x: OrderedFloat(692.3077),
        //             view_pos_y: OrderedFloat(0.0),
        //         },],
        //         imgs,
        //     )
        // }

        // #[test]
        // fn resize_imgs_rev_from_top() {
        //     //simple_logger::SimpleLogger::new().init().unwrap();
        //     // tracing_subscriber::fmt()
        //     //     .event_format(
        //     //         tracing_subscriber::fmt::format()
        //     //             .with_file(true)
        //     //             .with_line_number(true),
        //     //     )
        //     //     .with_env_filter(tracing_subscriber::EnvFilter::from_str("artbounty=trace"))
        //     //     .try_init()
        //     //     .unwrap();

        //     let mut imgs = [
        //         Img::new_with_y(0, 640, 480, 0.0),
        //         Img::new_with_y(1, 640, 480, 0.0),
        //         Img::new_with_y(2, 19200, 1080, 0.0),
        //         Img::new_with_y(3, 1280, 720, 0.0),
        //         Img::new_with_y(4, 720, 1280, 0.0),
        //     ];
        //     resize(&mut imgs, 200, 1000, 0, true);
        //     assert_eq!(
        //         [
        //             Img {
        //                 id: 0,
        //                 width: 640,
        //                 height: 480,
        //                 view_width: OrderedFloat(307.69232),
        //                 view_height: OrderedFloat(230.76923),
        //                 view_pos_x: OrderedFloat(384.61536),
        //                 view_pos_y: OrderedFloat(0.0),
        //             },
        //             Img {
        //                 id: 1,
        //                 width: 640,
        //                 height: 480,
        //                 view_width: OrderedFloat(307.69232),
        //                 view_height: OrderedFloat(230.76923),
        //                 view_pos_x: OrderedFloat(692.3077),
        //                 view_pos_y: OrderedFloat(0.0),
        //             },
        //             Img {
        //                 id: 2,
        //                 width: 19200,
        //                 height: 1080,
        //                 view_width: OrderedFloat(1000.0),
        //                 view_height: OrderedFloat(56.249996),
        //                 view_pos_x: OrderedFloat(0.0),
        //                 view_pos_y: OrderedFloat(230.76923),
        //             },
        //             Img {
        //                 id: 3,
        //                 width: 1280,
        //                 height: 720,
        //                 view_width: OrderedFloat(759.6439),
        //                 view_height: OrderedFloat(427.2997),
        //                 view_pos_x: OrderedFloat(0.0),
        //                 view_pos_y: OrderedFloat(287.01923),
        //             },
        //             Img {
        //                 id: 4,
        //                 width: 720,
        //                 height: 1280,
        //                 view_width: OrderedFloat(240.3561),
        //                 view_height: OrderedFloat(427.2997),
        //                 view_pos_x: OrderedFloat(759.6439),
        //                 view_pos_y: OrderedFloat(287.01923),
        //             },
        //         ],
        //         imgs
        //     )
        // }

        // #[test]
        // fn resize_imgs_rev_from_bottom() {
        //     let mut imgs = [
        //         Img::new_with_y(0, 640, 480, 0.0),
        //         Img::new_with_y(1, 640, 480, 0.0),
        //         Img::new_with_y(2, 19200, 1080, 0.0),
        //         Img::new_with_y(3, 1280, 720, 0.0),
        //         Img::new_with_y(4, 720, 1280, 0.0),
        //     ];
        //     let offset = imgs.len().saturating_sub(1);
        //     resize(&mut imgs, 200, 1000, offset, true);
        //     assert_eq!(
        //         [
        //             Img {
        //                 id: 0,
        //                 width: 640,
        //                 height: 480,
        //                 view_width: OrderedFloat(307.69232),
        //                 view_height: OrderedFloat(230.76923),
        //                 view_pos_x: OrderedFloat(384.61536),
        //                 view_pos_y: OrderedFloat(0.0),
        //             },
        //             Img {
        //                 id: 1,
        //                 width: 640,
        //                 height: 480,
        //                 view_width: OrderedFloat(307.69232),
        //                 view_height: OrderedFloat(230.76923),
        //                 view_pos_x: OrderedFloat(692.3077),
        //                 view_pos_y: OrderedFloat(0.0),
        //             },
        //             Img {
        //                 id: 2,
        //                 width: 19200,
        //                 height: 1080,
        //                 view_width: OrderedFloat(1000.0),
        //                 view_height: OrderedFloat(56.249996),
        //                 view_pos_x: OrderedFloat(0.0),
        //                 view_pos_y: OrderedFloat(230.76923),
        //             },
        //             Img {
        //                 id: 3,
        //                 width: 1280,
        //                 height: 720,
        //                 view_width: OrderedFloat(759.6439),
        //                 view_height: OrderedFloat(427.2997),
        //                 view_pos_x: OrderedFloat(0.0),
        //                 view_pos_y: OrderedFloat(287.01923),
        //             },
        //             Img {
        //                 id: 4,
        //                 width: 720,
        //                 height: 1280,
        //                 view_width: OrderedFloat(240.3561),
        //                 view_height: OrderedFloat(427.2997),
        //                 view_pos_x: OrderedFloat(759.6439),
        //                 view_pos_y: OrderedFloat(287.01923),
        //             },
        //         ],
        //         imgs
        //     )
        // }

        // #[test]
        // fn resize_imgs_offset_rev() {
        //     let mut imgs = [
        //         Img::new_with_y(0, 640, 480, 0.0),
        //         Img::new_with_y(1, 640, 480, 0.0),
        //         Img::new_with_y(2, 19200, 1080, -5.0),
        //         Img::new_with_y(3, 1280, 720, 0.0),
        //         Img::new_with_y(4, 720, 1280, 0.0),
        //     ];
        //     resize(&mut imgs, 200, 1000, 2, true);
        //     assert_eq!(
        //         [
        //             Img {
        //                 id: 0,
        //                 width: 640,
        //                 height: 480,
        //                 view_width: OrderedFloat(307.69232),
        //                 view_height: OrderedFloat(230.76923),
        //                 view_pos_x: OrderedFloat(384.61536),
        //                 view_pos_y: OrderedFloat(0.0),
        //             },
        //             Img {
        //                 id: 1,
        //                 width: 640,
        //                 height: 480,
        //                 view_width: OrderedFloat(307.69232),
        //                 view_height: OrderedFloat(230.76923),
        //                 view_pos_x: OrderedFloat(692.3077),
        //                 view_pos_y: OrderedFloat(0.0),
        //             },
        //             Img {
        //                 id: 2,
        //                 width: 19200,
        //                 height: 1080,
        //                 view_width: OrderedFloat(1000.0),
        //                 view_height: OrderedFloat(56.249996),
        //                 view_pos_x: OrderedFloat(0.0),
        //                 view_pos_y: OrderedFloat(230.76923),
        //             },
        //             Img {
        //                 id: 3,
        //                 width: 1280,
        //                 height: 720,
        //                 view_width: OrderedFloat(0.0),
        //                 view_height: OrderedFloat(0.0),
        //                 view_pos_x: OrderedFloat(0.0),
        //                 view_pos_y: OrderedFloat(287.01923),
        //             },
        //             Img {
        //                 id: 4,
        //                 width: 720,
        //                 height: 1280,
        //                 view_width: OrderedFloat(0.0),
        //                 view_height: OrderedFloat(0.0),
        //                 view_pos_x: OrderedFloat(0.0),
        //                 view_pos_y: OrderedFloat(287.01923),
        //             },
        //         ],
        //         imgs
        //     )
        // }

        // #[test]
        // fn resize_imgs_offset() {
        //     let mut imgs = [
        //         Img::new_with_y(0, 640, 480, 0.0),
        //         Img::new_with_y(1, 640, 480, 0.0),
        //         Img::new_with_y(2, 19200, 1080, -5.0),
        //         Img::new_with_y(3, 1280, 720, 0.0),
        //         Img::new_with_y(4, 720, 1280, 0.0),
        //     ];
        //     resize(&mut imgs, 200, 1000, 2, false);
        //     assert_eq!(
        //         [
        //             Img {
        //                 id: 0,
        //                 width: 640,
        //                 height: 480,
        //                 view_width: OrderedFloat(0.0),
        //                 view_height: OrderedFloat(0.0),
        //                 view_pos_x: OrderedFloat(0.0),
        //                 view_pos_y: OrderedFloat(0.0),
        //             },
        //             Img {
        //                 id: 1,
        //                 width: 640,
        //                 height: 480,
        //                 view_width: OrderedFloat(0.0),
        //                 view_height: OrderedFloat(0.0),
        //                 view_pos_x: OrderedFloat(0.0),
        //                 view_pos_y: OrderedFloat(0.0),
        //             },
        //             Img {
        //                 id: 2,
        //                 width: 19200,
        //                 height: 1080,
        //                 view_width: OrderedFloat(1000.0),
        //                 view_height: OrderedFloat(56.249996),
        //                 view_pos_x: OrderedFloat(0.0),
        //                 view_pos_y: OrderedFloat(-5.0),
        //             },
        //             Img {
        //                 id: 3,
        //                 width: 1280,
        //                 height: 720,
        //                 view_width: OrderedFloat(409.6),
        //                 view_height: OrderedFloat(230.40001),
        //                 view_pos_x: OrderedFloat(0.0),
        //                 view_pos_y: OrderedFloat(51.249996),
        //             },
        //             Img {
        //                 id: 4,
        //                 width: 720,
        //                 height: 1280,
        //                 view_width: OrderedFloat(129.6),
        //                 view_height: OrderedFloat(230.40001),
        //                 view_pos_x: OrderedFloat(409.6),
        //                 view_pos_y: OrderedFloat(51.249996),
        //             },
        //         ],
        //         imgs
        //     )
        // }
    }
}
