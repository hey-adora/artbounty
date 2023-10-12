use leptos::ev::{load, resize};
use leptos::html::Section;
use leptos::logging::log;
use leptos::*;
use leptos_use::{use_event_listener, use_window};
use rand::prelude::*;

fn render_gallery(max_width: i32, images: &mut [(i32, i32)]) -> () {
    let mut new_row_start: usize = 0;
    let mut new_row_end: usize = images.len();
    let mut current_row_filled_width: i32 = 0;
    let new_height: i32 = match max_width {
        _ => 250,
    };
    let mut index: usize = 0;
    let end: usize = images.len();
    while index < end {
        let (w, h) = &images[index];
        let width: i32 = *w;
        let height: i32 = *h;
        let ratio: f32 = width as f32 / height as f32;
        let height_diff: i32 = height - new_height;
        let new_width: i32 = width - (height_diff as f32 * ratio) as i32;

        for index in new_row_start..new_row_end {
            if (current_row_filled_width + new_width) <= max_width {
                current_row_filled_width += new_width;
                new_row_end = index;
            } else {
                let mut total_ratio: f32 = 0f32;
                for i in new_row_start..(new_row_end + 1) {
                    let (prev_img_w, prev_img_h) = &images[i];
                    total_ratio += *prev_img_w as f32 / *prev_img_h as f32;
                }
                let optimal_height: f32 = max_width as f32 / total_ratio;
                for i in new_row_start..(new_row_end + 1) {
                    let (prev_img_w, prev_img_h) = &images[i];
                    let ratio = *prev_img_w as f32 / *prev_img_h as f32;
                    let new_prev_img_w: f32 = optimal_height * ratio;
                    let new_prev_img_h: f32 = optimal_height;
                    images[i].0 = new_prev_img_w as i32;
                    images[i].1 = new_prev_img_h as i32;
                }
                new_row_start = index;
                new_row_end = index;
                current_row_filled_width = new_width;
            }
        }

        index += 1;
    }
}

fn render_gallery3(max_width: i32, images: &Vec<(i32, i32)>) -> Vec<(i32, i32)> {
    let max_width = max_width - 48;
    let mut resized_images: Vec<(i32, i32)> = Vec::new();
    let mut new_row_start = 0;
    let mut new_row_end = 0;
    let mut current_row_filled_width: i32 = 0;
    let new_height: i32 = match max_width {
        _ => max_width,
    };
    for (index, (w, h)) in images.iter().enumerate() {
        let width: i32 = w.to_owned();
        let height: i32 = h.to_owned();
        let ratio: f32 = width as f32 / height as f32;
        let height_diff: i32 = height - new_height;
        let new_width: i32 = width - (height_diff as f32 * ratio) as i32;
        if (current_row_filled_width + new_width) <= max_width {
            current_row_filled_width += new_width;
            new_row_end = index;
        } else {
            let mut total_ratio: f32 = 0f32;
            for i in new_row_start..(new_row_end + 1) {
                let (prev_img_w, prev_img_h) = resized_images[i];
                total_ratio += prev_img_w as f32 / prev_img_h as f32;
            }
            let optimal_height: f32 = max_width as f32 / total_ratio;
            for i in new_row_start..(new_row_end + 1) {
                let (prev_img_w, prev_img_h) = resized_images[i];
                let ratio = prev_img_w as f32 / prev_img_h as f32;
                let new_prev_img_w: f32 = optimal_height * ratio;
                let new_prev_img_h: f32 = optimal_height;
                resized_images[i].0 = new_prev_img_w as i32;
                resized_images[i].1 = new_prev_img_h as i32;
            }
            new_row_start = index;
            new_row_end = index;
            current_row_filled_width = new_width;
        }
        resized_images.push((new_width, new_height));
    }
    resized_images
}

#[component]
pub fn GalleryPage() -> impl IntoView {
    let (gallery_images, set_gallery_images): (
        ReadSignal<Vec<(i32, i32)>>,
        WriteSignal<Vec<(i32, i32)>>,
    ) = create_signal::<Vec<(i32, i32)>>(
        (0..25)
            .map(|_| {
                (
                    rand::thread_rng().gen_range(500..1000),
                    rand::thread_rng().gen_range(500..1000),
                )
            })
            .collect(),
    );

    let (gallery_width, set_gallery_width): (ReadSignal<i32>, WriteSignal<i32>) =
        create_signal::<i32>(0);

    let gallery_section = create_node_ref::<Section>();
    let resize_images = move || {
        let section = gallery_section.get_untracked();
        if let Some(section) = section {
            let width = section.offset_width();
            set_gallery_width(width);

            set_gallery_images.update(move |imgs| {
                log!("{}", width);
                render_gallery(gallery_width.get_untracked(), imgs);
            });
        };
    };

    create_effect(move |_| {
        let _ = use_event_listener(use_window(), load, move |_| {
            resize_images();
            log!("LOADED");
        });
    });

    create_effect(move |_| {
        resize_images();

        let _ = use_event_listener(use_window(), resize, move |_| resize_images());
    });

    view! {
        <section on:resize=move |_| { log!("test resize") } _ref=gallery_section class="line-bg max-w-screen overflow-hidden  content-start flex flex-wrap  " style=move|| format!("min-height: calc(100vh - 100px)")>
            { move || {

                  gallery_images.get().into_iter().map(|(w, h)|{

                    view! {
                        <div
                            class="flex-shrink-0 font-bold grid place-items-center shadow-glowy  bg-mid-purple border-4 border-low-purple"
                            style:height=move || format!("{}px", h)
                            style:width=move || format!("{}px", w)
                        >
                            <div class="flex flex-col text-center justify-center gap-2">
                                <h3>{w}x{h}</h3>
                                <h3>{w as f32 /h as f32}</h3>
                            </div>
                        </div>
                    } }).collect_view()

            }
        }
        </section>
    }
}
