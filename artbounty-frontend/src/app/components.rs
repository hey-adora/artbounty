pub mod nav {
    use crate::toolbox::prelude::*;
    use artbounty_shared::fe_router;
    use leptos::prelude::*;
    use log::error;
    use web_sys::SubmitEvent;

    use crate::app::GlobalState;

    #[component]

    pub fn Nav() -> impl IntoView {
        let global_state = expect_context::<GlobalState>();
        let api_logout = artbounty_api::auth::api::logout::client.ground();
        // let is_logged_in = move || global_state.acc.with(|v| v.is_some());
        let logout_or_loading = move || {
            if api_logout.is_pending() {
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

            api_logout.dispatch(artbounty_api::auth::api::logout::Input {});
        };

        Effect::new(move || {
            let Some(result) = api_logout.value() else {
                return;
            };

            match result {
                Ok(_) => {
                    global_state.logout();
                },
                Err(err) => {
                    error!("error logging out {err}");
                }
            }
        });
        // hey69@hey.com

        view! {
            <nav class="text-gray-200 pb-1 flex gap-2 px-2 py-1 items-center justify-between">
                <a href="/" class="font-black text-[1.3rem]">
                    "ArtBounty"
                </a>
                <div class=move||format!("{}", if global_state.acc_pending() { "" } else { "hidden" })>
                    <p>"loading..."</p>
                </div>
                <div class=move||format!("{}", if global_state.is_logged_in() || global_state.acc_pending() { "hidden" } else { "" })>
                    <a href=fe_router::login::PATH>"Login"</a>
                </div>
                <div class=move||format!("flex gap-2 {}", if global_state.is_logged_in() { "" } else { "hidden" })>
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
    use leptos::{html::Div, prelude::*};
    use std::default::Default;
    use std::{
        fmt::{Debug, Display},
        rc::Rc,
    };
    use tracing::{debug, error, trace};

    use crate::toolbox::{prelude::*, random::random_u64};

    pub fn vec_img_to_string<IMG: ResizableImage + Display>(imgs: &[IMG]) -> String {
        let mut output = String::new();

        for img in imgs {
            output += &format!("{},\n", img);
        }

        output
    }

    #[component]
    pub fn Gallery<FetchBtmFn, FetchTopFn>(
        fetch_top: FetchTopFn,
        fetch_bottom: FetchBtmFn,
        #[prop(optional)] fetch_init: Option<Rc<dyn Fn(usize) -> Vec<Img> + Send + Sync + 'static>>,
        #[prop(default = 250)] row_height: u32,
    ) -> impl IntoView
    where
        FetchBtmFn: Fn(usize, Img) -> Vec<Img> + Send + Sync + 'static + Clone,
        FetchTopFn: Fn(usize, Img) -> Vec<Img> + Send + Sync + 'static + Clone,
    {
        let gallery = RwSignal::<Vec<Img>>::new(Vec::new());
        let gallery_ref = NodeRef::<Div>::new();
        // let scroll_offset: StoredValue<f32> = StoredValue::new(0.0_f32);

        gallery_ref.add_resize_observer(move |entry, _observer| {
            trace!("RESIZINGGGGGG");
            let width = entry.content_rect().width() as u32;

            let prev_imgs = gallery.get_untracked();
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
                add_imgs_to_top(prev_imgs, new_imgs, width, heigth * 3.0, row_height);
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
            let (resized_imgs, scroll_by) =
                add_imgs_to_bottom(prev_imgs, new_imgs, width, heigth * 3.0, row_height);
            trace!("scroll master: {scroll_by}");
            gallery.set(resized_imgs);
            gallery_elm.scroll_by_with_x_and_y(0.0, scroll_by);
        };

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
            trace!("ON LOAD");
            let Some(gallery_elm) = gallery_ref.get() else {
                trace!("gallery NOT found");
                return;
            };
            trace!("gallery elm found");
            let width = gallery_elm.client_width() as u32;
            let heigth = gallery_elm.client_height() as f64;

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
            let (resized_imgs, _scroll_by) =
                add_imgs_to_bottom(prev_imgs, new_imgs, width, heigth, row_height);
            gallery.set(resized_imgs);
        });

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
        let img_ref = NodeRef::<Div>::new();

        img_ref.add_intersection_observer_with_options(
            move |entry, _observer| {
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
        let fn_top = move || format!("{view_top}px");
        let fn_width = move || format!("{view_width}px");
        let fn_height = move || format!("{view_height}px");
        let fn_text = move || format!("{img_width}x{img_height}");
        let fn_text2 = move || format!("{view_left}x{view_top}");
        let fn_text3 = move || format!("{img_id}");

        view! {
            <div
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

    pub fn calc_fit_count(width: u32, height: f64, img_height: u32) -> usize {
        ((width * height as u32) / (img_height * img_height)) as usize
    }

    #[cfg(test)]
    mod resize_tests {
        use crate::app::components::gallery::{
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
