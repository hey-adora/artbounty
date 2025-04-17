pub mod gallery {
    use futures::io::Cursor;
    use leptos::{
        html::{self, Div, Main, div},
        prelude::*,
        tachys::html::node_ref::NodeRefContainer,
    };
    use ordered_float::OrderedFloat;
    use std::fmt::Debug;
    use std::{default::Default, time::Duration};
    use tracing::{debug, trace};
    use web_sys::HtmlDivElement;

    use crate::toolbox::{prelude::*, random::random_u64};

    pub const NEW_IMG_HEIGHT: u32 = 250;

    #[component]
    pub fn Gallery(imgs: RwSignal<Vec<Img>>) -> impl IntoView {
        let gallery_ref = NodeRef::<Div>::new();
        let top_bar_ref = NodeRef::<Div>::new();
        let first_ref = NodeRef::<Div>::new();
        let top_bar_is_visible = StoredValue::new(false);
        let scroll_offset: StoredValue<f32> = StoredValue::new(0.0_f32);

        let handle = interval::new(
            move || {
                let Some(gallery_elm) = gallery_ref.get_untracked() else {
                    return;
                };
                let width = gallery_elm.client_width() as u32;
                //return;
                let Some(first_ref) = first_ref.get_untracked() else {
                    let mut new_imgs = Img::rand_vec(1);
                    resize(&mut new_imgs, NEW_IMG_HEIGHT, width, 0, false);
                    imgs.set(new_imgs);
                    return;
                }; 
                if !top_bar_is_visible.get_value() {
                    return; 
                }
               
                let scroll_top = gallery_elm.scroll_top();

                // return;
                let mut new_imgs = Img::rand_vec(10);
                imgs.update(|old_imgs| {

                    let old_imgs_len = old_imgs.len();
                    // if old_imgs_len >= 10 {
                    //     return;
                    // }
                    let new_imgs_len = new_imgs.len();
                    let offset = new_imgs_len.saturating_sub(old_imgs_len);
                    new_imgs.extend_from_slice(old_imgs);
                    *old_imgs = new_imgs;

                    let y = if old_imgs_len > 0 {
                        debug!("running {NEW_IMG_HEIGHT} {width} {offset} {}", false);
                        resize(old_imgs, NEW_IMG_HEIGHT, width, offset, true)
                    } else {
                        debug!("running {NEW_IMG_HEIGHT} {width} {} {}", 0, false);
                        resize(old_imgs, NEW_IMG_HEIGHT, width, 0, false)
                    };

                    let diff = (y - scroll_offset.get_value()).abs();
                    scroll_offset.set_value(y);
                    let scroll_by = scroll_top as f64 + (diff / 2.0) as f64;
                    trace!("SCROLL_BY: {} + ({} / 2.0) = {}", scroll_top, diff,scroll_by );
                    trace!("totalllllllllllllll y: {y} diff: {diff} scroll_top: {scroll_top} scroll_by: {scroll_by}");
                    gallery_elm.scroll_by_with_x_and_y(0.0, diff as f64);
                   
                });


                //first_ref.scroll_into_view();
                //trace!("beep boop");
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
            let Some(gallery_elm) = gallery_ref.get_untracked() else {
                return;
            };
            let width = gallery_elm.client_width() as u32;
            let scroll_top = gallery_elm.scroll_top();
            let mut new_imgs = Vec::from([
                Img::new(1000, 200),
                Img::new(100, 1000),
                Img::new(1000, 100),
            ]);

            imgs.update(|v| {
                let old_imgs_count = v.len();
                let new_imgs_count = new_imgs.len();
                let offset = new_imgs.len();
                //let offset = new_imgs.len().saturating_sub(v.len());
                //new_imgs.extend_from_slice(v);
                //new_imgs.extend_from_slice(v);

                let y = resize(&mut new_imgs, NEW_IMG_HEIGHT, width, 0, false);
                let diff = (y - scroll_offset.get_value()).abs();
                // if old_y > 0 {
                //     let diff = old_y - y;
                //     println!("old_y: {}",);
                // }
                scroll_offset.set_value(y);
                let scroll_by = scroll_top as f64 + (diff / 2.0) as f64;
                trace!("SCROLL_BY: {} + ({} / 2.0) = {}", scroll_top, diff,scroll_by );
                trace!("totalllllllllllllll y: {y} diff: {diff} scroll_top: {scroll_top} scroll_by: {scroll_by}");
                gallery_elm.scroll_by_with_x_and_y(0.0, diff as f64);

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
                class="transition-all duration-300 ease-in-out text-white grid place-items-center bg-blue-950 absolute border border-red-600 overflow-hidden"
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
        pub row_id: usize,
        pub width: u32,
        pub height: u32,
        pub view_width: RwSignal<f32>,
        pub view_height: RwSignal<f32>,
        pub view_pos_x: RwSignal<f32>,
        pub view_pos_y: RwSignal<f32>,
    }

    impl ResizableImage for Img {
        fn set_size(&mut self, view_width: f32, view_height: f32, pos_x: f32, pos_y: f32) {
            self.view_width.set(view_width);
            self.view_height.set(view_height);
            self.view_pos_x.set(pos_x);
            self.view_pos_y.set(pos_y);
        }

        fn set_pos_y(&mut self, pos_y: f32) {
            self.view_pos_y.set(pos_y);
        }

        fn get_pos_y(&self) -> f32 {
            self.view_pos_y.get_untracked()
        }

        fn get_size(&self) -> (u32, u32) {
            (self.width, self.height)
        }

        // fn get_view_width(&self) -> f32 {
        //     self.view_width.get_untracked()
        // }

        fn get_view_height(&self) -> f32 {
            self.view_height.get_untracked()
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
                view_width: RwSignal::new(0.0),
                view_height: RwSignal::new(0.0),
                view_pos_x: RwSignal::new(0.0),
                view_pos_y: RwSignal::new(0.0),
            }
        }

        pub fn rand() -> Self {
            let id = random_u64();
            let width = random_u32_ranged(500, 1000);
            let height = random_u32_ranged(500, 1000);

            Self {
                id,
                row_id: 0,
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

    pub trait ResizableImage {
        fn get_size(&self) -> (u32, u32);
        fn get_pos_y(&self) -> f32;
        // fn get_view_width(&self) -> f32;
        fn get_view_height(&self) -> f32;
        fn set_size(&mut self, view_width: f32, view_height: f32, pos_x: f32, pos_y: f32);
        fn set_pos_y(&mut self, pos_y: f32);
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
        //let needs_normalizing = imgs.first().map(|v| v.get_pos_y() < 0.0 ).unwrap_or(false);
        if !needs_normalizing {
            return;
        }

        //let len = imgs.len();
        let mut prev_y: f32 = first_y;
        let mut prev_height: f32 = first_height;
        //let mut current_height: f32 = first_height;
        let mut offset_y: f32 = 0.0;

        // for i in 0..len {
        //     let current_y = {
        //         let img = &imgs[i];
        //         img.get_pos_y()
        //     };

        //     if current_y != prev_y {
        //         let current_height = img.get_view_height();
        //         let prev_height = &imgs[i - 1];
        //         prev_y = current_y;
        //         offset_y += current_height;
        //     }

        //     img.set_pos_y(offset_y);
        // }
        for img in imgs {
            let current_y = img.get_pos_y();

            if current_y != prev_y {
                prev_y = current_y;
                offset_y += prev_height;
                prev_height = img.get_view_height();
            }

            img.set_pos_y(offset_y);
        }

        // let Some(first) = imgs.first() else {
        //     return;
        // };

        // let first_y = first.get_pos_y();

        // if first_y >= 0.0 {
        //     return;
        // }

        // for img in imgs {
        //     let current_y = img.get_pos_y();
        //     let new_y = current_y - first_y;
        //     img.set_pos_y(new_y);
        // }
    }

    pub fn resize<IMG>(
        imgs: &mut [IMG],
        row_height: u32,
        max_width: u32,
        offset: usize,
        rev: bool,
    ) -> f32
    where
        IMG: ResizableImage,
    {
        let len = imgs.len();
        let mut total_w = 0;
        let mut total_ratio = 0.0;
        let mut pos_y: f32 = 0.0;
        let mut cursor_row_start: usize = offset;
        let mut cursor_row_end: usize = cursor_row_start;

        // find start of the row
        // or end of a row in rev
        if cursor_row_end < len {
            let current_pos_y = imgs[offset].get_pos_y();
            //let mut sub_cursor = cursor;

            let mut new_offset = offset;
            loop {
                trace!(
                    "find first row loop break: ({new_offset} == 0 && !{rev}) = {} || ({rev} && {new_offset} >= {len}) = {}",
                    (new_offset == 0 && !rev),
                    (rev && new_offset >= len)
                );
                if (new_offset == 0 && !rev) || (rev && new_offset >= len) {
                    break;
                }

                let prev_pos_y = imgs[new_offset].get_pos_y();
                trace!(
                    "finding first row {new_offset} : {prev_pos_y} != {current_pos_y} == {}",
                    prev_pos_y != current_pos_y
                );
                if prev_pos_y != current_pos_y {
                    break;
                }

                if rev {
                    new_offset += 1;
                } else {
                    new_offset -= 1;
                }
            }

            if new_offset != offset {
                if rev {
                    trace!("before setting end: {cursor_row_end}");
                    cursor_row_end = new_offset.saturating_sub(1);
                    trace!("after setting end: {cursor_row_end}");
                    cursor_row_start = cursor_row_end;
                } else {
                    trace!("before setting end: {cursor_row_end}");
                    cursor_row_end = (new_offset + 1).clamp(0, len.saturating_sub(1));
                    trace!("after setting end: {cursor_row_end}");
                    cursor_row_start = cursor_row_end;
                }
            }
          

            // if cursor_row_end != 0 && cursor_row_end < len {
            //     if rev {
            //         trace!("before setting end: {cursor_row_end}");
            //         cursor_row_end -= 1;
            //         trace!("after setting end: {cursor_row_end}");
            //         cursor_row_start = cursor_row_end;
            //     } else {
            //         trace!("before setting end: {cursor_row_end}");
            //         cursor_row_end += 1;
            //         trace!("after setting end: {cursor_row_end}");
            //         cursor_row_start = cursor_row_end;
            //     }
            // }

        //    if i > 0 {
        //     if rev {
        //         trace!("before setting end: {cursor_row_end}");
        //         cursor_row_end -= 1;
        //         trace!("after setting end: {cursor_row_end}");
        //         cursor_row_start = cursor_row_end;
        //     } else {
        //         trace!("before setting end: {cursor_row_end}");
        //         cursor_row_end += 1;
        //         trace!("after setting end: {cursor_row_end}");
        //         cursor_row_start = cursor_row_end;
        //     }
        //    }
        }

        trace!("start: {cursor_row_start}, end: {cursor_row_end}");

        //read the first image
        if cursor_row_end < len {
            let img = &imgs[cursor_row_end];
            let (width, height) = img.get_size();
            let ratio = width as f32 / height as f32;
            let scaled_w = width - (height.saturating_sub(row_height) as f32 * ratio) as u32;

            total_w = scaled_w;
            total_ratio = ratio;
            pos_y = img.get_pos_y();

            if rev {
                cursor_row_end = cursor_row_end.saturating_sub(1);
            } else {
                cursor_row_end += 1;
            }
        }

        // do the actual resizing
        loop {
            if cursor_row_end >= len || (rev && cursor_row_end == 0) {
                break;
            }

            let (scaled_w, ratio, img_fits_in_row) = {
                trace!("pos_y: {}", pos_y);
                let img = &imgs[cursor_row_end];
                let (width, height) = img.get_size();
                let ratio = width as f32 / height as f32;
                let scaled_w = width - (height.saturating_sub(row_height) as f32 * ratio) as u32;
                let img_fits_in_row = total_w + scaled_w <= max_width;
                (scaled_w, ratio, img_fits_in_row)
            };

            if !img_fits_in_row {
                let row_height: f32 = max_width as f32 / total_ratio;
                trace!(
                    "row_height {}-{} = {} / {} = {} ",
                    cursor_row_start, cursor_row_end, max_width, total_ratio, row_height,
                );
                //let mut pos_x: f32 = if rev { max_width as f32 } else { 0.0 };
                let mut pos_x: f32 = if rev { max_width as f32 } else { 0.0 };

                //let mut i: usize = row_start;
                loop {
                    if cursor_row_start == cursor_row_end {
                        break;
                    }

                    let img = &mut imgs[cursor_row_start];
                    let (width, height) = img.get_size();
                    let new_width = row_height * (width as f32 / height as f32);
                    let new_height = row_height;

                    if rev {
                        trace!(
                            "!!!!!!!!!!!!!setting pos_x: {pos_x} - {new_width} = {}",
                            pos_x - new_width
                        );
                        pos_x -= new_width;
                    }

                    trace!("how is this possible?");

                    img.set_size(new_width, new_height, pos_x, pos_y);

                    if rev {
                        cursor_row_start -= 1;
                    } else {
                        pos_x += new_width;
                        cursor_row_start += 1;
                    }
                }

                trace!("pos_y1: {}", pos_y);

                total_w = 0;
                total_ratio = 0.0;
                if rev {
                    pos_y -= row_height;
                } else {
                    pos_y += row_height;
                }
                trace!("pos_y: {}", pos_y);
            }

            total_w += scaled_w;
            total_ratio += ratio;

            if rev {
                cursor_row_end -= 1;
            } else {
                cursor_row_end += 1;
            }
        }
        // finish unfilled row resizing
        if total_w != 0 {
            let row_gap = max_width.saturating_sub(total_w);
            let missing_imgs = row_gap.saturating_div(row_height);
            let row_ratio = total_ratio + missing_imgs as f32;
            let row_height: f32 = max_width as f32 / row_ratio;
            let mut pos_x: f32 = if rev { max_width as f32 } else { 0.0 };

            let mut i = cursor_row_start;
            loop {
                trace!("in last loop");
                if (i == len) {
                    break;
                }

                trace!("resizing: {i}");

                let img = &mut imgs[i];
                let (width, height) = img.get_size();
                let new_width = row_height * (width as f32 / height as f32);
                let new_height = row_height;

                if rev {
                    pos_x -= new_width;
                }

                img.set_size(new_width, new_height, pos_x, pos_y);

                if rev {
                    if i == 0 {
                        break;
                    }
                    i -= 1;
                } else {
                    pos_x += new_width;
                    i += 1;
                }
            }
        }

        normalize_y(imgs);
        //        imgs.pos_y

        imgs.last()
            .map(|v| v.get_view_height() + v.get_pos_y())
            .unwrap_or(0.0)
    }

    #[cfg(test)]
    mod resize_tests {
        use crate::app::components::gallery::resize;
        use ordered_float::OrderedFloat;
        use pretty_assertions::{assert_eq, assert_ne};
        use std::str::FromStr;
        use test_log::test;

        use super::ResizableImage;

        #[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
        struct Img {
            pub width: u32,
            pub height: u32,
            pub view_width: OrderedFloat<f32>,
            pub view_height: OrderedFloat<f32>,
            pub pos_x: OrderedFloat<f32>,
            pub pos_y: OrderedFloat<f32>,
        }

        impl Img {
            pub const fn new(width: u32, height: u32) -> Self {
                Self {
                    width,
                    height,
                    view_width: OrderedFloat(0.0),
                    view_height: OrderedFloat(0.0),
                    pos_x: OrderedFloat(0.0),
                    pos_y: OrderedFloat(0.0),
                }
            }

            pub const fn new_with_y(width: u32, height: u32, y: f32) -> Self {
                Self {
                    width,
                    height,
                    view_width: OrderedFloat(0.0),
                    view_height: OrderedFloat(0.0),
                    pos_x: OrderedFloat(0.0),
                    pos_y: OrderedFloat(y),
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
            fn get_size(&self) -> (u32, u32) {
                (self.width, self.height)
            }
            fn get_pos_y(&self) -> f32 {
                *self.pos_y
            }
            // fn get_view_width(&self) -> f32 {
            //     *self.view_width
            // }
            fn get_view_height(&self) -> f32 {
                *self.view_height
            }
            fn set_size(&mut self, view_width: f32, view_height: f32, pos_x: f32, pos_y: f32) {
                println!("setting yyyyyyyyyyy {}", pos_y);
                println!("setting xxxxxxxxxxx {}", pos_x);
                *self.view_width = view_width;
                *self.view_height = view_height;
                self.pos_x = OrderedFloat::from(pos_x);
                self.pos_y = OrderedFloat::from(pos_y);
            }
            fn set_pos_y(&mut self, pos_y: f32) {
                *self.pos_y = pos_y;
            }
        }

        #[test]
        fn resize_imgs_two() {
            let mut imgs = [Img::new(640, 480), Img::new(720, 1280)];
            resize(&mut imgs, 200, 1000, 0, false);
            assert_eq!(
                [
                    Img {
                        width: 640,
                        height: 480,
                        view_width: OrderedFloat(272.34042),
                        view_height: OrderedFloat(204.25531),
                        pos_x: OrderedFloat(0.0),
                        pos_y: OrderedFloat(0.0),
                    },
                    Img {
                        width: 720,
                        height: 1280,
                        view_width: OrderedFloat(114.893616),
                        view_height: OrderedFloat(204.25531),
                        pos_x: OrderedFloat(272.34042),
                        pos_y: OrderedFloat(0.0),
                    }
                ],
                imgs
            )
        }

        #[test]
        fn resize_imgs_three() {
            let mut imgs = Vec::from([
                Img::new(1000, 200),
                Img::new(100, 1000),
                Img::new(1000, 100),
            ]);
            resize(&mut imgs, 200, 1000, 0, false);

            assert_eq!(
                [
                    Img {
                        width: 1000,
                        height: 200,
                        view_width: OrderedFloat(1000.0),
                        view_height: OrderedFloat(200.0),
                        pos_x: OrderedFloat(0.0),
                        pos_y: OrderedFloat(0.0),
                    },
                    Img {
                        width: 100,
                        height: 1000,
                        view_width: OrderedFloat(1000.0),
                        view_height: OrderedFloat(10000.0),
                        pos_x: OrderedFloat(0.0),
                        pos_y: OrderedFloat(200.0),
                    },
                    Img {
                        width: 1000,
                        height: 100,
                        view_width: OrderedFloat(1000.0),
                        view_height: OrderedFloat(100.0),
                        pos_x: OrderedFloat(0.0),
                        pos_y: OrderedFloat(10200.0),
                    },
                ],
                *imgs
            )
        }

        #[test]
        fn resize_imgs_single_normal() {
            let mut imgs = [Img::new(640, 480)];
            resize(&mut imgs, 200, 1000, 0, false);
            assert_eq!(
                [Img {
                    width: 640,
                    height: 480,
                    view_width: OrderedFloat(307.69232),
                    view_height: OrderedFloat(230.76923),
                    pos_x: OrderedFloat(0.0),
                    pos_y: OrderedFloat(0.0),
                },],
                imgs
            )
        }

    

        #[test]
        fn resize_imgs_single_rev() {
            let mut imgs = [Img::new(640, 480)];
            resize(&mut imgs, 200, 1000, 0, true);
            assert_eq!(
                [Img {
                    width: 640,
                    height: 480,
                    view_width: OrderedFloat(307.69232),
                    view_height: OrderedFloat(230.76923),
                    pos_x: OrderedFloat(692.3077),
                    pos_y: OrderedFloat(0.0),
                },],
                imgs,
            )
        }

        #[test]
        fn resize_imgs_rev_from_top() {
            //simple_logger::SimpleLogger::new().init().unwrap();
            // tracing_subscriber::fmt()
            //     .event_format(
            //         tracing_subscriber::fmt::format()
            //             .with_file(true)
            //             .with_line_number(true),
            //     )
            //     .with_env_filter(tracing_subscriber::EnvFilter::from_str("artbounty=trace"))
            //     .try_init()
            //     .unwrap();

            let mut imgs = [
                Img::new_with_y(640, 480, 0.0),
                Img::new_with_y(640, 480, 0.0),
                Img::new_with_y(19200, 1080, 0.0),
                Img::new_with_y(1280, 720, 0.0),
                Img::new_with_y(720, 1280, 0.0),
            ];
            resize(&mut imgs, 200, 1000, 0, true);
            assert_eq!(
                [
                    Img {
                        width: 640,
                        height: 480,
                        view_width: OrderedFloat(307.69232),
                        view_height: OrderedFloat(230.76923),
                        pos_x: OrderedFloat(384.61536),
                        pos_y: OrderedFloat(0.0),
                    },
                    Img {
                        width: 640,
                        height: 480,
                        view_width: OrderedFloat(307.69232),
                        view_height: OrderedFloat(230.76923),
                        pos_x: OrderedFloat(692.3077),
                        pos_y: OrderedFloat(0.0),
                    },
                    Img {
                        width: 19200,
                        height: 1080,
                        view_width: OrderedFloat(1000.0),
                        view_height: OrderedFloat(56.249996),
                        pos_x: OrderedFloat(0.0),
                        pos_y: OrderedFloat(230.76923),
                    },
                    Img {
                        width: 1280,
                        height: 720,
                        view_width: OrderedFloat(759.6439),
                        view_height: OrderedFloat(427.2997),
                        pos_x: OrderedFloat(0.0),
                        pos_y: OrderedFloat(287.01923),
                    },
                    Img {
                        width: 720,
                        height: 1280,
                        view_width: OrderedFloat(240.3561),
                        view_height: OrderedFloat(427.2997),
                        pos_x: OrderedFloat(759.6439),
                        pos_y: OrderedFloat(287.01923),
                    },
                ],
                imgs
            )
        }

        #[test]
        fn resize_imgs_rev_from_bottom() {
            let mut imgs = [
                Img::new_with_y(640, 480, 0.0),
                Img::new_with_y(640, 480, 0.0),
                Img::new_with_y(19200, 1080, 0.0),
                Img::new_with_y(1280, 720, 0.0),
                Img::new_with_y(720, 1280, 0.0),
            ];
            let offset = imgs.len().saturating_sub(1);
            resize(&mut imgs, 200, 1000, offset, true);
            assert_eq!(
                [
                    Img {
                        width: 640,
                        height: 480,
                        view_width: OrderedFloat(307.69232),
                        view_height: OrderedFloat(230.76923),
                        pos_x: OrderedFloat(384.61536),
                        pos_y: OrderedFloat(0.0),
                    },
                    Img {
                        width: 640,
                        height: 480,
                        view_width: OrderedFloat(307.69232),
                        view_height: OrderedFloat(230.76923),
                        pos_x: OrderedFloat(692.3077),
                        pos_y: OrderedFloat(0.0),
                    },
                    Img {
                        width: 19200,
                        height: 1080,
                        view_width: OrderedFloat(1000.0),
                        view_height: OrderedFloat(56.249996),
                        pos_x: OrderedFloat(0.0),
                        pos_y: OrderedFloat(230.76923),
                    },
                    Img {
                        width: 1280,
                        height: 720,
                        view_width: OrderedFloat(759.6439),
                        view_height: OrderedFloat(427.2997),
                        pos_x: OrderedFloat(0.0),
                        pos_y: OrderedFloat(287.01923),
                    },
                    Img {
                        width: 720,
                        height: 1280,
                        view_width: OrderedFloat(240.3561),
                        view_height: OrderedFloat(427.2997),
                        pos_x: OrderedFloat(759.6439),
                        pos_y: OrderedFloat(287.01923),
                    },
                ],
                imgs
            )
        }

        #[test]
        fn resize_imgs_offset_rev() {
            let mut imgs = [
                Img::new_with_y(640, 480, 0.0),
                Img::new_with_y(640, 480, 0.0),
                Img::new_with_y(19200, 1080, -5.0),
                Img::new_with_y(1280, 720, 0.0),
                Img::new_with_y(720, 1280, 0.0),
            ];
            resize(&mut imgs, 200, 1000, 2, true);
            assert_eq!(
                [
                    Img {
                        width: 640,
                        height: 480,
                        view_width: OrderedFloat(307.69232),
                        view_height: OrderedFloat(230.76923),
                        pos_x: OrderedFloat(384.61536),
                        pos_y: OrderedFloat(0.0),
                    },
                    Img {
                        width: 640,
                        height: 480,
                        view_width: OrderedFloat(307.69232),
                        view_height: OrderedFloat(230.76923),
                        pos_x: OrderedFloat(692.3077),
                        pos_y: OrderedFloat(0.0),
                    },
                    Img {
                        width: 19200,
                        height: 1080,
                        view_width: OrderedFloat(1000.0),
                        view_height: OrderedFloat(56.249996),
                        pos_x: OrderedFloat(0.0),
                        pos_y: OrderedFloat(230.76923),
                    },
                    Img {
                        width: 1280,
                        height: 720,
                        view_width: OrderedFloat(0.0),
                        view_height: OrderedFloat(0.0),
                        pos_x: OrderedFloat(0.0),
                        pos_y: OrderedFloat(287.01923),
                    },
                    Img {
                        width: 720,
                        height: 1280,
                        view_width: OrderedFloat(0.0),
                        view_height: OrderedFloat(0.0),
                        pos_x: OrderedFloat(0.0),
                        pos_y: OrderedFloat(287.01923),
                    },
                ],
                imgs
            )
        }

        #[test]
        fn resize_imgs_offset() {
            let mut imgs = [
                Img::new_with_y(640, 480, 0.0),
                Img::new_with_y(640, 480, 0.0),
                Img::new_with_y(19200, 1080, -5.0),
                Img::new_with_y(1280, 720, 0.0),
                Img::new_with_y(720, 1280, 0.0),
            ];
            resize(&mut imgs, 200, 1000, 2, false);
            assert_eq!(
                [
                    Img {
                        width: 640,
                        height: 480,
                        view_width: OrderedFloat(0.0),
                        view_height: OrderedFloat(0.0),
                        pos_x: OrderedFloat(0.0),
                        pos_y: OrderedFloat(0.0),
                    },
                    Img {
                        width: 640,
                        height: 480,
                        view_width: OrderedFloat(0.0),
                        view_height: OrderedFloat(0.0),
                        pos_x: OrderedFloat(0.0),
                        pos_y: OrderedFloat(0.0),
                    },
                    Img {
                        width: 19200,
                        height: 1080,
                        view_width: OrderedFloat(1000.0),
                        view_height: OrderedFloat(56.249996),
                        pos_x: OrderedFloat(0.0),
                        pos_y: OrderedFloat(-5.0),
                    },
                    Img {
                        width: 1280,
                        height: 720,
                        view_width: OrderedFloat(409.6),
                        view_height: OrderedFloat(230.40001),
                        pos_x: OrderedFloat(0.0),
                        pos_y: OrderedFloat(51.249996),
                    },
                    Img {
                        width: 720,
                        height: 1280,
                        view_width: OrderedFloat(129.6),
                        view_height: OrderedFloat(230.40001),
                        pos_x: OrderedFloat(409.6),
                        pos_y: OrderedFloat(51.249996),
                    },
                ],
                imgs
            )
        }
    }
}
