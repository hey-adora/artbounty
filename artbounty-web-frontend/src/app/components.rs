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
                // return;
                let Some(first_ref) = first_ref.get_untracked() else {
                    let mut new_imgs = Img::rand_vec(1);
                    // resize(&mut new_imgs, NEW_IMG_HEIGHT, width, 0);
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
                    // resize(&mut new_imgs, NEW_IMG_HEIGHT, width, 0);
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
        pub row_id: usize,
        pub width: u32,
        pub height: u32,
        pub view_width: RwSignal<f32>,
        pub view_height: RwSignal<f32>,
        pub view_pos_x: RwSignal<f32>,
        pub view_pos_y: RwSignal<f32>,
    }

    // impl ResizableImage for Img {
    //     fn get_size(&self) -> (u32, u32) {
    //         (self.width, self.height)
    //     }
    //     fn get_pos_y(&self) -> f32 {
    //         self.view_pos_y.get_untracked()
    //     }
    //     fn set_size(&mut self, view_width: f32, view_height: f32, pos_x: f32, pos_y: f32) {
    //         self.view_width.set(view_width);
    //         self.view_height.set(view_height);
    //         self.view_pos_x.set(pos_x);
    //         self.view_pos_y.set(pos_y);
    //     }
    // }

    impl Img {
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
        fn get_view_width(&self) -> f32;
        fn get_view_height(&self) -> f32;
        fn set_size(&mut self, view_width: f32, view_height: f32, pos_x: f32, pos_y: f32);
    }

    pub fn resize<IMG>(imgs: &mut [IMG], row_height: u32, max_width: u32, offset: usize)
    where
        IMG: ResizableImage,
    {
        let mut total_w = 0;
        let mut total_ratio = 0.0;
        let mut row_start = offset;
        let mut pos_y: f32 = 0.0;

        let len = imgs.len();

        let mut cursor: usize = offset;

        if cursor > 0 && cursor < len {
            // let mut sub_cursor: usize = cursor.saturating_sub(1);
            let mut iter = imgs.iter().rev().skip(cursor);
            if let Some(pos_y) = iter.next().map(|v| v.get_pos_y()) {
                row_start = iter
                    .position(|v| v.get_pos_y() != pos_y)
                    .unwrap_or_default();
                cursor = row_start;
                println!("wtf?: {:?}", row_start);
            };
            // let a = first.get_pos_y();
            // let current_img = &imgs[sub_cursor];
            // let current_pos_y: f32 = current_img.get_pos_y();
            // // let mut total_width = 0.0;
            // // let steps_taken_len: usize = 0;
            // loop {
            //     let img = &imgs[sub_cursor];
            //     let pos_y = img.get_pos_y();
            //     // let view_width = img.get_view_width();
            //     if pos_y != current_pos_y {
            //         break;
            //     }

            //     // total_width += view_width;
            //     // println!("moving one row back");

            //     if sub_cursor <= 0 {
            //         break;
            //     }

            //     sub_cursor -= 1;
            // }

            // let traversed_len = cursor - sub_cursor;
            // if traversed_len > 0 {
            //     println!(
            //         "traversed len: {} and width: {}",
            //         traversed_len, total_width
            //     );

            //     let max_width = max_width
            //         .saturating_sub(total_width as u32)
            //         .clamp(row_height, max_width);

            //     let mut offset_cursor = cursor;
            //     let img = &imgs[cursor];
            //     let row_height = img.get_view_height();
            //     let total_ratio = max_width as f32 / row_height;
            //     let mut total_w: u32 = 0;
            //     // let row_height: f32 = max_width as f32 / total_ratio;
            //     // for sub_cursor in offset..len {}

            //     loop {
            //         if sub_cursor >= len {
            //             break;
            //         }
            //         let img = &imgs[cursor];
            //         let (width, height) = img.get_size();
            //         let ratio = width as f32 / height as f32;
            //         let scaled_w = row_height * ratio;
            //         // let scaled_w =
            //         //     width - (height.saturating_sub(row_height) as f32 * total_ratio) as u32;

            //         total_w += scaled_w as u32;

            //         sub_cursor += 1;
            //     }
            // }

            // println!("where the row starts: {}", sub_cursor);
        }

        loop {
            if cursor >= len {
                break;
            }

            let img = &imgs[cursor];
            let (width, height) = img.get_size();
            let ratio = width as f32 / height as f32;
            let scaled_w = width - (height.saturating_sub(row_height) as f32 * ratio) as u32;
            let img_fits_in_row = total_w + scaled_w <= max_width;

            if !img_fits_in_row {
                let row_height: f32 = max_width as f32 / total_ratio;
                let mut pos_x: f32 = 0.0;
                for i in row_start..cursor {
                    let img = &mut imgs[i];
                    let (width, height) = img.get_size();
                    let new_width = row_height * (width as f32 / height as f32);
                    let new_height = row_height;
                    img.set_size(new_width, new_height, pos_x, pos_y);

                    pos_x += new_width;
                }

                row_start = cursor;
                total_w = 0;
                total_ratio = 0.0;
                pos_y += row_height;
            }

            total_w += scaled_w;
            total_ratio += ratio;

            cursor += 1;
        }
        if total_w != 0 {
            let row_gap = max_width.saturating_sub(total_w);
            let missing_imgs = row_gap.saturating_div(row_height);
            let row_ratio = total_ratio + missing_imgs as f32;
            let row_height: f32 = max_width as f32 / row_ratio;
            let mut pos_x: f32 = 0.0;

            for i in row_start..len {
                let img = &mut imgs[i];
                let (width, height) = img.get_size();
                let new_width = row_height * (width as f32 / height as f32);
                let new_height = row_height;
                img.set_size(new_width, new_height, pos_x, pos_y);

                pos_x += new_width;
            }
        }
    }

    #[cfg(test)]
    mod resize_tests {
        use ordered_float::OrderedFloat;

        use crate::app::components::gallery::resize;

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
        const SAMPLE_FOUR_MAX_WIDTH: u32 = 1000;
        const SAMPLE_FOUR_ROW_HEIGHT: u32 = 200;
        const SAMPLE_FOUR_NEW: [Img; 4] = [
            Img::new(640, 480),
            Img::new(1920, 1080),
            Img::new(1280, 720),
            Img::new(720, 1280),
        ];
        const SAMPLE_FOUR_RESIZED: [Img; 4] = [
            Img {
                width: 640,
                height: 480,
                view_width: OrderedFloat(272.7273),
                view_height: OrderedFloat(204.54546),
                pos_x: OrderedFloat(0.0),
                pos_y: OrderedFloat(0.0),
            },
            Img {
                width: 1920,
                height: 1080,
                view_width: OrderedFloat(363.63638),
                view_height: OrderedFloat(204.54546),
                pos_x: OrderedFloat(272.7273),
                pos_y: OrderedFloat(0.0),
            },
            Img {
                width: 1280,
                height: 720,
                view_width: OrderedFloat(363.63638),
                view_height: OrderedFloat(204.54546),
                pos_x: OrderedFloat(636.36365),
                pos_y: OrderedFloat(0.0),
            },
            Img {
                width: 720,
                height: 1280,
                view_width: OrderedFloat(123.287674),
                view_height: OrderedFloat(219.17809),
                pos_x: OrderedFloat(0.0),
                pos_y: OrderedFloat(204.54546),
            },
        ];
        const SAMPLE_FIVE_RESIZED: [Img; 5] = [
            Img {
                width: 6400,
                height: 480,
                view_width: OrderedFloat(272.7273),
                view_height: OrderedFloat(204.54546),
                pos_x: OrderedFloat(0.0),
                pos_y: OrderedFloat(0.0),
            },
            Img {
                width: 640,
                height: 480,
                view_width: OrderedFloat(272.7273),
                view_height: OrderedFloat(204.54546),
                pos_x: OrderedFloat(0.0),
                pos_y: OrderedFloat(0.0),
            },
            Img {
                width: 1920,
                height: 1080,
                view_width: OrderedFloat(363.63638),
                view_height: OrderedFloat(204.54546),
                pos_x: OrderedFloat(272.7273),
                pos_y: OrderedFloat(0.0),
            },
            Img {
                width: 1280,
                height: 720,
                view_width: OrderedFloat(363.63638),
                view_height: OrderedFloat(204.54546),
                pos_x: OrderedFloat(636.36365),
                pos_y: OrderedFloat(0.0),
            },
            Img {
                width: 720,
                height: 1280,
                view_width: OrderedFloat(123.287674),
                view_height: OrderedFloat(219.17809),
                pos_x: OrderedFloat(0.0),
                pos_y: OrderedFloat(204.54546),
            },
        ];

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
            fn get_view_width(&self) -> f32 {
                *self.view_width
            }
            fn get_view_height(&self) -> f32 {
                *self.view_height
            }
            fn set_size(&mut self, view_width: f32, view_height: f32, pos_x: f32, pos_y: f32) {
                *self.view_width = view_width;
                *self.view_height = view_height;
                self.pos_x = OrderedFloat::from(pos_x);
                self.pos_y = OrderedFloat::from(pos_y);
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
        fn resize_all_four() {
            let mut imgs = SAMPLE_FOUR_NEW.clone();
            resize(&mut imgs, SAMPLE_FOUR_ROW_HEIGHT, SAMPLE_FOUR_MAX_WIDTH, 0);
            assert_eq!(imgs, SAMPLE_FOUR_RESIZED,);
        }

        #[test]
        fn resize_second_row() {
            let mut imgs = SAMPLE_FIVE_RESIZED.clone();
            resize(&mut imgs, SAMPLE_FOUR_ROW_HEIGHT, SAMPLE_FOUR_MAX_WIDTH, 3);
            assert_eq!(imgs, SAMPLE_FIVE_RESIZED)
        }
    }
}
