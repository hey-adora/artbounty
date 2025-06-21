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
    use web_sys::{HtmlDivElement, HtmlElement, SubmitEvent};

    #[component]
    pub fn Page() -> impl IntoView {
        let main_ref = NodeRef::new();
        let global_state = expect_context::<GlobalState>();
        let on_login = |e: SubmitEvent| {
            e.prevent_default();

            trace!("oh hello");
        };
        // let imgs = global_state.imgs;

        view! {
            <main node_ref=main_ref class="grid grid-rows-[auto_1fr] h-screen  ">
                <Nav/>
                <div class="grid place-items-center text-white">
                    <div class="bg-gray-900  flex flex-col gap-4 px-3 py-4">
                        <h1 class="text-2xl font-bold">"Register"</h1>
                        <form method="POST" action="" on:submit=on_login class="flex flex-col gap-2">
                            <div class="flex flex-col gap-0">
                                <label>"Username"</label>
                                <input type="text" class="border-b-2 border-white" />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label>"Password"</label>
                                <input type="password" class="border-b-2 border-white" />
                            </div>
                            <div class="flex flex-col gap-0">
                                <label>"Invite Token"</label>
                                <input type="password" class="border-b-2 border-white" />
                            </div>
                            <input type="submit" value="Register" class="border-2 border-white mt-2"/>
                        </form>
                    </div>
                </div>
            </main>
        }
    }
}
