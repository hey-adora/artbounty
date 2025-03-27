pub mod gallery {
    use leptos::{
        html::{self, Div, Main, div},
        prelude::*,
        tachys::html::node_ref::NodeRefContainer,
    };
    use ordered_float::OrderedFloat;
    use std::fmt::Debug;
    use std::{default::Default, time::Duration};
    use tracing::trace;
    use web_sys::HtmlDivElement;

    use crate::toolbox::{prelude::*, random::random_u64};

    pub const NEW_IMG_HEIGHT: u32 = 250;

    #[component]
    pub fn Gallery(imgs: RwSignal<Vec<Img>>) -> impl IntoView {
        let gallery_ref = NodeRef::<Div>::new();
        let top_bar_ref = NodeRef::<Div>::new();
        let first_ref = NodeRef::<Div>::new();
        let top_bar_is_visible = StoredValue::new(false);

        let handle = interval::new(
            move || {
                let Some(width) = gallery_ref.get_untracked().map(|v| v.client_width() as u32)
                else {
                    return;
                };
                return;
                let Some(first_ref) = first_ref.get_untracked() else {
                    let mut new_imgs = Img::rand_vec(1);
                    resize_imgs(NEW_IMG_HEIGHT, width, &mut new_imgs);
                    imgs.set(new_imgs);
                    return;
                };
                if !top_bar_is_visible.get_value() {
                    return;
                }
                let mut new_imgs = Img::rand_vec(1);
                imgs.update(|v| {
                    let old_imgs_count = v.len();
                    let new_imgs_count = new_imgs.len();
                    new_imgs.extend_from_slice(v);
                    // fast_img_resize(NEW_IMG_HEIGHT, width, &mut new_imgs);
                    *v = new_imgs;
                });
                first_ref.scroll_into_view();
                trace!("beep boop");
            },
            Duration::from_secs(1),
        )
        .unwrap();

        //let imggg = RwSignal::<Vec<(usize, Img)>>::new(Vec::new());

        // Effect::new(move || {
        //     let Some(gallery_elm) = gallery_ref.get() else {
        //         return;
        //     };
        //     let resize_observer = resize_observer::new_raw(move |entries, observer| {
        //         imgs.update_untracked(|imgs| {
        //             // let Some(width) = gallery_ref.get_untracked().map(|v| v.client_width() as u32)
        //             // else {
        //             //     return;
        //             // };
        //             //resize_imgs(NEW_IMG_HEIGHT, width, imgs);
        //         });
        //         trace!("yo yo yo");
        //     });
        //     let intersection_observer = intersection_observer::new(move |entries, observer| {});
        //     resize_observer.observe(&gallery_elm);
        // });

        gallery_ref.add_resize_observer(move |entry, observer| {
            let width = entry.content_rect().width();
            imgs.update_untracked(|imgs| {
                // fast_img_resize(NEW_IMG_HEIGHT, width as u32, imgs);
            });
        });

        top_bar_ref.observe_intersection_with_options(
            move |entry, observer| {
                // let Some(first_ref) = first_ref.get_untracked() else {
                //     return;
                // };
                let is_interescting = entry.is_intersecting();
                top_bar_is_visible.set_value(is_interescting);
                // if is_interescting {
                //     let mut new_imgs = Img::rand_vec(1);
                //     imgs.update(|v| {
                //         new_imgs.extend_from_slice(v);
                //         *v = new_imgs;
                //     });
                // }
                // if is_interescting {
                //     first_ref.scroll_into_view();
                // }
                // trace!("wowza, its intersecting: {}", is_interescting);
            },
            intersection_observer::Options::<Div>::default().set_threshold(0.1),
        );

        let get_imgs = move || {
            let mut imgs = imgs.get();
            let Some(width) = gallery_ref.get().map(|v| v.client_width() as u32) else {
                return Vec::new();
            };
            // trace!("resizing!!!! {}", width);
            // if width > 0 {
            //     resize_imgs(NEW_IMG_HEIGHT, width, &mut imgs);
            // }

            imgs.into_iter().enumerate().collect::<Vec<(usize, _)>>()
        };

        let big_btn = move |_| {
            let Some(width) = gallery_ref.get_untracked().map(|v| v.client_width() as u32) else {
                return;
            };
            let mut new_imgs = Img::rand_vec(1);
            imgs.update(|v| {
                let old_imgs_count = v.len();
                let new_imgs_count = new_imgs.len();
                new_imgs.extend_from_slice(v);
                // fast_img_resize(NEW_IMG_HEIGHT, width, &mut new_imgs[..new_imgs_count]);
                *v = new_imgs;
            });
        };

        let a = view! {
            <div
                id="gallery"
                node_ref=gallery_ref
                class="relative overflow-y-scroll overflow-x-hidden"
            >
                // style:width=move || format!("{}px", gallery_wdith.get())
                <div node_ref=top_bar_ref class="bg-red-600 h-[100px] w-full ">
                    <button on:click=big_btn>"click me"</button>
                </div>
                <For
                    each=get_imgs
                    key=|img| img.1.id
                    children=move |(i, img)| {
                        view! { <GalleryImg index=i img first_ref /> }
                    }
                />
            </div>
        };

        a
    }

    #[component]
    pub fn GalleryImg(img: Img, index: usize, first_ref: NodeRef<Div>) -> impl IntoView {
        let gallery_img_ref = NodeRef::<Div>::new();

        gallery_img_ref.on_load(move |e| {
            trace!("did i load or what? o.O");
        });

        Effect::new(move || {
            if index != 0 {
                return;
            }

            let Some(gallery_img_ref) = gallery_img_ref.get() else {
                return;
            };
            first_ref.load(&gallery_img_ref);
            trace!("FIRST REF SET");
            // gallery_img_ref.scroll_into_view();
            // if let Some(node_ref) = node_ref {

            //     node_ref.track();
            //     trace!("tracking!");
            // }
        });

        let view_left = img.view_pos_x;
        let view_top = img.view_pos_y;
        let view_width = img.view_width;
        let view_height = img.view_height;
        let img_width = img.width;
        let img_height = img.height;

        let fn_background =
            move || format!("rgb({}, {}, {})", random_u8(), random_u8(), random_u8());
        let fn_left = move || format!("{}px", view_left.get());
        let fn_top = move || format!("{}px", view_top.get() + 100.0);
        let fn_width = move || format!("{}px", view_width.get());
        let fn_height = move || format!("{}px", view_height.get());
        let fn_text = move || format!("{}x{}", img_width, img_height);
        let fn_text2 = move || format!("{}x{}", view_left.get(), view_top.get() + 100.0);

        view! {
            <div
                node_ref=gallery_img_ref
                // node_ref=first_ref
                class="text-white grid place-items-center bg-blue-950 absolute border border-red-600 overflow-hidden"
                style:background-color=fn_background
                style:left=fn_left
                style:top=fn_top
                style:width=fn_width
                style:height=fn_height
            >
                <div>
                    <div>{fn_text}</div>
                    <div>{fn_text2}</div>
                </div>
            </div>
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Img {
        pub id: u64,
        pub width: u32,
        pub height: u32,
        pub view_width: RwSignal<f32>,
        pub view_height: RwSignal<f32>,
        pub view_pos_x: RwSignal<f32>,
        pub view_pos_y: RwSignal<f32>,
    }

    impl Img {
        pub fn rand() -> Self {
            let id = random_u64();
            let width = random_u32_ranged(500, 1000);
            let height = random_u32_ranged(500, 1000);

            Self {
                id,
                width,
                height,
                view_width: RwSignal::new(0.0),
                view_height: RwSignal::new(0.0),
                view_pos_x: RwSignal::new(0.0),
                view_pos_y: RwSignal::new(0.0),
            }
        }

        pub fn rand_vec(n: usize) -> Vec<Self> {
            let mut output = Vec::new();
            for _ in 0..n {
                output.push(Img::rand());
            }
            output
        }
    }

    pub fn resize_img(
        top: &mut f32,
        max_width: u32,
        new_row_start: usize,
        new_row_end: usize,
        imgs: &[Img],
    ) {
        let mut total_ratio: f32 = 0f32;

        for i in new_row_start..(new_row_end + 1) {
            let (width, height) = (imgs[i].width, imgs[i].height);
            total_ratio += width as f32 / height as f32;
        }
        let optimal_height: f32 = max_width as f32 / total_ratio;
        let mut left: f32 = 0.0;

        for i in new_row_start..(new_row_end + 1) {
            let (width, height) = (imgs[i].width, imgs[i].height);
            let new_width = optimal_height * (width as f32 / height as f32);
            let new_height = optimal_height;
            imgs[i].view_width.set(new_width);
            imgs[i].view_height.set(new_height);
            imgs[i].view_pos_x.set(left);
            imgs[i].view_pos_y.set(*top);
            left += new_width;
        }
        *top += optimal_height;
    }

    pub fn resize_img2(
        top: &mut f32,
        max_width: u32,
        new_row_start: usize,
        new_row_end: usize,
        imgs: &[Img],
    ) {
        let mut optimal_count =
            (max_width as i32 / NEW_IMG_HEIGHT as i32) - (new_row_end - new_row_start) as i32;
        if optimal_count < 0 {
            optimal_count = 0;
        }
        let mut total_ratio: f32 = optimal_count as f32;
        if max_width < NEW_IMG_HEIGHT * 3 {
            total_ratio = 0.0;
        }

        for i in new_row_start..(new_row_end + 1) {
            let (width, height) = (imgs[i].width, imgs[i].height);
            total_ratio += width as f32 / height as f32;
        }
        let optimal_height: f32 = max_width as f32 / total_ratio;
        let mut left: f32 = 0.0;

        for i in new_row_start..(new_row_end + 1) {
            let (width, height) = (imgs[i].width, imgs[i].height);
            let new_width = optimal_height * (width as f32 / height as f32);
            let new_height = optimal_height;
            imgs[i].view_width.set(new_width);
            imgs[i].view_height.set(new_height);
            imgs[i].view_pos_x.set(left);
            imgs[i].view_pos_y.set(*top);

            left += new_width;
        }

        *top += optimal_height;
    }

    pub fn resize_imgs(new_height: u32, max_width: u32, imgs: &[Img]) -> () {
        // debug!("utils: resizing started: count: {}", imgs.len());
        let loop_start = 0;
        let loop_end = imgs.len();
        let mut new_row_start: usize = 0;
        let mut new_row_end: usize = if loop_end > 0 { loop_end - 1 } else { 0 };
        let mut current_row_filled_width: u32 = 0;
        let mut top: f32 = 0.0;

        for index in loop_start..loop_end {
            let org_img = &imgs[index];
            let (width, height) = (org_img.width, org_img.height);
            let ratio: f32 = width as f32 / height as f32;
            let height_diff: u32 = if height < new_height {
                0
            } else {
                height - new_height
            };
            let new_width: u32 = width - (height_diff as f32 * ratio) as u32;
            if (current_row_filled_width + new_width) <= max_width {
                current_row_filled_width += new_width;
                new_row_end = index;
                if index == loop_end - 1 {
                    resize_img2(&mut top, max_width, new_row_start, new_row_end, imgs);
                }
            } else {
                if index != 0 {
                    resize_img(&mut top, max_width, new_row_start, new_row_end, imgs);
                }
                new_row_start = index;
                new_row_end = index;
                current_row_filled_width = new_width;
                if index == loop_end - 1 {
                    resize_img2(&mut top, max_width, new_row_start, new_row_end, imgs);
                }
            }
        }

        // debug!("utils: resizing ended: count: {}", imgs.len());
    }

    pub trait ResizableImage {
        fn get_width(&self) -> u32;
        fn get_height(&self) -> u32;
        fn set_view_width(&mut self, width: f32);
        fn set_view_height(&mut self, height: f32);
        fn set_view_pos_x(&mut self, x: f32);
        fn set_view_pos_y(&mut self, y: f32);
    }

    #[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
    pub struct ImgRow {
        pub row_start: usize,
        pub row_end: usize,
        pub row_width: u32,
        pub ratio: OrderedFloat<f32>,
        pub pos_x: OrderedFloat<f32>,
        pub pos_y: OrderedFloat<f32>,
        // pub scaled_width: u32,
        // pub ratio: f32,
    }

    pub struct ImgSize {
        width: usize,
        height: usize,
    }

    pub fn get_imgs_rows<IMG>(imgs: &mut [IMG], row_height: u32, max_width: u32) -> Vec<ImgRow>
    where
        IMG: ResizableImage,
    {
        let mut rows = Vec::<ImgRow>::new();
        let mut total_w = 0;
        let mut total_ratio = 0.0;
        let mut row_start = 0;
        let mut pos_x: f32 = 0.0;
        let mut pos_y: f32 = 0.0;
        let last = imgs.len();

        let mut cursor: usize = 0;
        loop {
            if cursor >= last {
                break;
            }

            let img = &imgs[cursor];
            let width = img.get_width();
            let height = img.get_height();
            let ratio = width as f32 / height as f32;
            let scaled_w = width - (height.saturating_sub(row_height) as f32 * ratio) as u32;
            let img_fits_in_row = total_w + scaled_w <= max_width;

            if !img_fits_in_row {
                // let row_height: f32 = max_width as f32 / *row.ratio;
                let row_height: f32 = max_width as f32 / total_ratio;
                // let row_len = row.row_end - row.row_start;
                // println!("row len: {}", row_len);
                let mut offest_x: f32 = 0.0;
                for i in row_start..cursor {
                    let img = &mut imgs[i];
                    let width = img.get_width();
                    let height = img.get_height();
                    let new_width = row_height * (width as f32 / height as f32);
                    let new_height = row_height;
                    img.set_view_width(new_width);
                    img.set_view_height(new_height);
                }

                rows.push(ImgRow {
                    row_start,
                    row_end: cursor,
                    row_width: total_w,
                    pos_x: OrderedFloat::from(pos_x),
                    pos_y: OrderedFloat::from(pos_y),
                    ratio: OrderedFloat::from(total_ratio),
                });
                row_start = cursor;
                total_w = 0;
                total_ratio = 0.0;
            }

            total_w += scaled_w;
            total_ratio += ratio;

            cursor += 1;
        }
        for (cursor, img) in imgs.iter().enumerate() {}
        if total_w != 0 {
            let optimal_row_len = max_width / row_height;
            let row_gap = max_width.saturating_sub(total_w);
            let missing_imgs = row_gap.saturating_div(row_height);
            let row_ratio = total_ratio + missing_imgs as f32;
            let row_height: f32 = max_width as f32 / row_ratio;

            // println!("last row {} {} {:?}", len, missing_imgs, row);

            for i in row_start..last {
                let img = &mut imgs[i];
                let width = img.get_width();
                let height = img.get_height();
                let new_width = row_height * (width as f32 / height as f32);
                let new_height = row_height;
                img.set_view_width(new_width);
                img.set_view_height(new_height);
            }

            rows.push(ImgRow {
                row_start,
                row_end: last,
                row_width: total_w,
                pos_x: OrderedFloat::from(pos_x),
                pos_y: OrderedFloat::from(pos_y),
                ratio: OrderedFloat::from(total_ratio),
            });
        }

        rows
    }

    pub fn do_the_resize<IMG>(imgs: &mut [IMG], row_height: u32, max_width: u32, rows: &[ImgRow])
    where
        IMG: ResizableImage,
    {
        let rows_len = rows.len();
        let more_than_one = rows_len > 1;
        let len = if more_than_one {
            rows_len - 1
        } else {
            rows_len
        };
        for i in 0..len {
            let row = &rows[i];
            let row_height: f32 = max_width as f32 / *row.ratio;
            let row_len = row.row_end - row.row_start;
            println!("row len: {}", row_len);
            let mut offest_x: f32 = 0.0;
            for i in row.row_start..row.row_end {
                let img = &mut imgs[i];
                let width = img.get_width();
                let height = img.get_height();
                let new_width = row_height * (width as f32 / height as f32);
                let new_height = row_height;
                img.set_view_width(new_width);
                img.set_view_height(new_height);
            }
        }
        if more_than_one {
            let optimal_row_len = max_width / row_height;
            let row = &rows[len];
            let row_gap = max_width.saturating_sub(row.row_width);
            let missing_imgs = row_gap.saturating_div(row_height);
            let row_ratio = *row.ratio + missing_imgs as f32;
            let row_height: f32 = max_width as f32 / row_ratio;

            println!("last row {} {} {:?}", len, missing_imgs, row);

            for i in row.row_start..row.row_end {
                let img = &mut imgs[i];
                let width = img.get_width();
                let height = img.get_height();
                let new_width = row_height * (width as f32 / height as f32);
                let new_height = row_height;
                img.set_view_width(new_width);
                img.set_view_height(new_height);
            }
        }
        // println!("optimal row len: {}", optimal_row_len);
    }

    pub fn gen_imgs_rows<IMG>(row_height: u32, max_width: u32, imgs: &mut [IMG])
    where
        IMG: ResizableImage,
    {
        // let mut output = Vec::new();

        let mut current_row_filled_width: u32 = 0;
        let mut total_ratio = 0.0;
        let mut row_start: usize = 0;
        let mut row_end: usize = 0;
        let mut offset_y = 0.0;
        let len = imgs.len();
        let last = len.saturating_sub(1);

        for row_cursor in 0..len {
            let img = &imgs[row_cursor];
            let width = img.get_width();
            let height = img.get_height();
            let ratio = width as f32 / height as f32;
            let new_width = width - (height.saturating_sub(row_height) as f32 * ratio) as u32;

            let fits_in_row = current_row_filled_width + new_width <= max_width;
            if fits_in_row {
                total_ratio += ratio;
                current_row_filled_width += new_width;
                // row_filled = true;
            } else {
                let row_height: f32 = max_width as f32 / total_ratio;
                let mut offest_x: f32 = 0.0;
                for i in row_start..row_cursor {
                    let img = &mut imgs[i];
                    let width = img.get_width();
                    let height = img.get_height();
                    let new_width = row_height * (width as f32 / height as f32);
                    let new_height = row_height;
                    img.set_view_width(new_width);
                    img.set_view_height(new_height);
                    img.set_view_pos_x(offest_x);
                    img.set_view_pos_y(offset_y);
                    offest_x += new_width;
                }
                println!(
                    "new: {}-{} {} {} optimal_height: {}",
                    row_start, row_cursor, current_row_filled_width, total_ratio, row_height
                );

                offset_y += row_height;
                total_ratio = ratio;
                current_row_filled_width = new_width;
                row_start = row_cursor;
            }

            // let (new_width, ratio) = {
            //     let img = &imgs[i];
            //     let width = img.get_width();
            //     let height = img.get_height();
            //     let ratio = width as f32 / height as f32;
            //     // let height_diff = img.height.saturating_sub(img.height.saturating_sub(new_height));
            //     let height_diff: u32 = if height < new_height {
            //         0
            //     } else {
            //         height - new_height
            //     };
            //     let new_width = width - (height_diff as f32 * ratio) as u32;
            //     (new_width, ratio)
            // };
        }

        // output
    }

    pub fn fast_img_resize<IMG>(new_height: u32, max_width: u32, imgs: &mut [IMG])
    where
        IMG: ResizableImage,
    {
        let mut current_row_filled_width: u32 = 0;
        let mut optimal_count = 0;
        let mut offset_y: f32 = 0.0;
        let mut total_ratio = 0.0;
        let mut row_filled = false;
        let mut i: usize = 0;
        let len = imgs.len();
        let last = len.saturating_sub(1);
        // loop {
        //     if i == len {
        //         break;
        //     }
        //     let img = imgs[i];

        //     i += 1;
        // }
        let mut row_start: usize = 0;
        // let apply_resize = |row_cursor: usize| {
        //     let row_height: f32 = max_width as f32 / total_ratio;
        //     let mut offest_x: f32 = 0.0;
        //     for i in row_start..row_cursor {
        //         let img = &mut imgs[i];
        //         let width = img.width;
        //         let height = img.height;
        //         let new_width = row_height * (width as f32 / height as f32);
        //         let new_height = row_height;
        //         img.view_width.set(new_width);
        //         img.view_height.set(new_height);
        //         img.view_pos_x.set(offest_x);
        //         img.view_pos_y.set(offset_y);
        //         offest_x += new_width;
        //     }
        // };

        // for row_cursor in 0..len {
        //     let (new_width, ratio) = {
        //         let img = &imgs[row_cursor];
        //         let width = img.get_width();
        //         let height = img.get_height();
        //         let ratio = width as f32 / height as f32;
        //         // let height_diff = img.height.saturating_sub(img.height.saturating_sub(new_height));
        //         let height_diff: u32 = if height < new_height {
        //             0
        //         } else {
        //             height - new_height
        //         };
        //         let new_width = width - (height_diff as f32 * ratio) as u32;
        //         (new_width, ratio)
        //     };
        //     let is_last = row_cursor == last;
        //     let fits_in_row = current_row_filled_width + new_width <= max_width;
        //     if fits_in_row {
        //         total_ratio += ratio;
        //         current_row_filled_width += new_width;
        //         // row_filled = true;
        //     }
        //     if is_last {
        //         println!("wowza ");
        //     }
        //     if !fits_in_row || is_last {
        //         let row_height: f32 = max_width as f32 / total_ratio;
        //         let mut offest_x: f32 = 0.0;
        //         for i in row_start..row_cursor {
        //             let img = &mut imgs[i];
        //             let width = img.width;
        //             let height = img.height;
        //             let new_width = row_height * (width as f32 / height as f32);
        //             let new_height = row_height;
        //             img.view_width.set(new_width);
        //             img.view_height.set(new_height);
        //             img.view_pos_x.set(offest_x);
        //             img.view_pos_y.set(offset_y);
        //             offest_x += new_width;
        //         }
        //         println!(
        //             "new: {}-{} {} {} optimal_height: {}",
        //             row_start, row_cursor, current_row_filled_width, total_ratio, row_height
        //         );

        //         offset_y += row_height;
        //         total_ratio = ratio;
        //         current_row_filled_width = new_width;
        //         row_start = row_cursor;
        //     }
        // }
    }

    pub fn calc_fit_count(width: u32, height: u32) -> u32 {
        (width * height) / (NEW_IMG_HEIGHT * NEW_IMG_HEIGHT)
    }

    #[cfg(test)]
    mod resize_tests {
        use ordered_float::OrderedFloat;

        use crate::app::components::gallery::{ImgRow, do_the_resize, get_imgs_rows};

        use super::{ResizableImage, fast_img_resize};

        #[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
        struct Img {
            pub width: u32,
            pub height: u32,
            pub view_width: OrderedFloat<f32>,
            pub view_height: OrderedFloat<f32>,
        }

        impl Img {
            pub fn new(width: u32, height: u32) -> Self {
                Self {
                    width,
                    height,
                    view_width: OrderedFloat::from(0.0),
                    view_height: OrderedFloat::from(0.0),
                }
            }

            pub fn ratio(&self) -> f32 {
                self.width as f32 / self.height as f32
            }

            pub fn get_scaled_width(&self, desired_height: u32) -> u32 {
                let ratio = self.ratio();
                self.width - (self.height.saturating_sub(desired_height) as f32 * ratio) as u32
            }
        }

        impl ResizableImage for Img {
            fn get_width(&self) -> u32 {
                self.width
            }
            fn get_height(&self) -> u32 {
                self.height
            }
            fn set_view_width(&mut self, width: f32) {
                *self.view_width = width;
            }
            fn set_view_height(&mut self, height: f32) {
                *self.view_height = height;
            }
            fn set_view_pos_x(&mut self, x: f32) {
                todo!()
            }
            fn set_view_pos_y(&mut self, y: f32) {
                todo!()
            }
        }
        // #[test]
        // fn gen_row_single() {
        //     let max_width: u32 = 1000;
        //     let row_height: u32 = 200;
        //     let imgs = [Img::new(640, 480)];

        //     let rows1 = get_imgs_rows(&imgs, row_height, max_width);

        //     assert_eq!(
        //         rows1,
        //         Vec::from([ImgRow {
        //             row_start: 0,
        //             row_end: 1,
        //             row_width: 1,
        //             ratio: OrderedFloat::from(1.3333334),
        //         },])
        //     )
        // }

        // #[test]
        // fn gen_row_two() {
        //     let max_width: u32 = 1000;
        //     let row_height: u32 = 200;
        //     let imgs = [
        //         Img::new(640, 480),
        //         Img::new(1920, 1080),
        //         Img::new(1280, 720),
        //         Img::new(720, 1280),
        //     ];

        //     let rows1 = get_imgs_rows(&imgs, row_height, max_width);

        //     assert_eq!(
        //         rows1,
        //         Vec::from([
        //             ImgRow {
        //                 row_start: 2,
        //                 row_end: 3,
        //                 row_width: 0,
        //                 ratio: OrderedFloat::from(4.888889),
        //             },
        //             ImgRow {
        //                 row_start: 3,
        //                 row_end: 4,
        //                 row_width: 0,
        //                 ratio: OrderedFloat::from(0.5625),
        //             },
        //         ])
        //     )
        // }

        #[test]
        fn resize_rows() {
            let max_width: u32 = 1000;
            let row_height: u32 = 200;
            let mut imgs = [
                Img::new(640, 480),
                Img::new(1920, 1080),
                Img::new(1280, 720),
                Img::new(720, 1280),
            ];

            let rows = get_imgs_rows(&mut imgs, row_height, max_width);
            // do_the_resize(&mut imgs, row_height, max_width, &rows);

            assert_eq!(
                imgs,
                [
                    Img {
                        height: 0,
                        width: 0,
                        view_width: OrderedFloat::from(0.0),
                        view_height: OrderedFloat::from(0.0),
                    },
                    Img {
                        height: 0,
                        width: 0,
                        view_width: OrderedFloat::from(0.0),
                        view_height: OrderedFloat::from(0.0),
                    },
                    Img {
                        height: 0,
                        width: 0,
                        view_width: OrderedFloat::from(0.0),
                        view_height: OrderedFloat::from(0.0),
                    },
                    Img {
                        height: 0,
                        width: 0,
                        view_width: OrderedFloat::from(0.0),
                        view_height: OrderedFloat::from(0.0),
                    },
                ]
            )
        }
    }
}
