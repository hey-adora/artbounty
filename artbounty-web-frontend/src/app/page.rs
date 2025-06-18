pub mod home {
    use std::rc::Rc;

    use crate::{
        app::{
            GlobalState,
            components::{
                gallery::{Gallery, Img},
                nav::Nav,
            },
        },
        toolbox::prelude::*,
    };
    use leptos::prelude::*;
    use reactive_stores::Store;
    use tracing::trace;
    use web_sys::{HtmlDivElement, HtmlElement};

    #[component]
    pub fn Page() -> impl IntoView {
        let main_ref = NodeRef::new();
        let global_state = expect_context::<GlobalState>();
        let fake_imgs = RwSignal::new(Vec::<Img>::new());
        // let imgs = global_state.imgs;

        main_ref.on_file_drop(async |event, data| {
            for file in data.get_files() {
                let stream = file.get_file_stream()?;
                let mut data = Vec::<u8>::new();
                while let Some(chunk) = stream.get_stream_chunk().await? {
                    chunk.push_to_vec(&mut data);
                }
                let data_str = String::from_utf8_lossy(&data);
                trace!("file: {}", data_str);
            }

            Ok(())
        });

        Effect::new(move || {
            let imgs = Img::rand_vec(200);
            fake_imgs.set(imgs);
        });

        let fetch_init = Rc::new(move |count| -> Vec<Img> {
            trace!("gog1");
            if count == 0 || count > fake_imgs.with(|v| v.len()) {
                Vec::new()
            } else {
                fake_imgs.with(|v| v[..count].to_vec())
            }
        });

        let fetch_bottom = move |count: usize, last_img: Img| -> Vec<Img> {
            trace!("gogbtm");

            fake_imgs
                .with_untracked(|imgs| {
                    imgs.iter()
                        .position(|img| img.id == last_img.id)
                        .and_then(|pos_start| {
                            let len = imgs.len();
                            if len == pos_start + 1 {
                                return None;
                            }
                            let pos_end = pos_start + count;
                            let pos_end = if pos_start + count > len {
                                len
                            } else {
                                pos_end
                            };
                            Some(imgs[pos_start..pos_end].to_vec())
                        })
                })
                .unwrap_or_default()
        };
        let fetch_top = move |count: usize, last_img: Img| -> Vec<Img> {
            trace!("gogtop");

            fake_imgs
                .with_untracked(|imgs| {
                    imgs.iter()
                        .position(|img| img.id == last_img.id)
                        .and_then(|pos_end| {
                            trace!("FETCH_TOP: POS_END {pos_end}");
                            if pos_end == 0 {
                                return None;
                            }
                            let pos_start = pos_end.saturating_sub(count);
                            Some(imgs[pos_start..pos_end].to_vec())
                        })
                })
                .unwrap_or_default()
        };

        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] h-screen">
                <Nav/>
                <Gallery fetch_init fetch_bottom fetch_top />
            </main>
        }
    }
}

pub mod login {
    use crate::{
        app::{
            GlobalState,
            components::{gallery::Gallery, nav::Nav},
        },
        toolbox::prelude::*,
    };
    use leptos::prelude::*;
    use reactive_stores::Store;
    use tracing::trace;
    use web_sys::{HtmlDivElement, HtmlElement};

    #[component]
    pub fn Page() -> impl IntoView {
        let main_ref = NodeRef::new();
        let global_state = expect_context::<GlobalState>();
        // let imgs = global_state.imgs;

        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] h-screen">
                <Nav/>
                <div>"hello login"</div>
            </main>
        }
    }
}
