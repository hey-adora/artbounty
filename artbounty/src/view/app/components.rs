pub mod nav {
    use crate::{
        api::{Api, ApiWeb},
        path::{PATH_LOGIN, PATH_UPLOAD, link_settings, link_user},
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
            <nav class="text-gray-200 flex gap-2 px-2 h-[3rem] items-center justify-between">
                <a href="/" class="font-black text-[1.3rem]">
                    "ArtBounty"
                </a>
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
                // <a href="/register" class="">"Register"</a>
            </nav>
        }
    }
}
pub mod gallery {
    use crate::api::{Api, ApiWeb, UserPost, UserPostFile};
    use crate::path::{link_img, link_post, link_post_with_history};
    use crate::view::toolbox::prelude::*;
    use chrono::Utc;
    use leptos::html;
    use leptos::{html::Div, prelude::*};
    use leptos_router::hooks::{query_signal, use_params_map, use_query_map};
    use leptos_router::params::Params;
    use std::default::Default;
    use std::time::Duration;
    use std::{
        fmt::{Debug, Display},
        rc::Rc,
    };
    use tracing::{debug, error, trace};
    use web_sys::{HtmlAnchorElement, MouseEvent};

    pub fn vec_img_to_string<IMG: ResizableImage + Display>(imgs: &[IMG]) -> String {
        let mut output = String::new();

        for img in imgs {
            output += &format!("{},\n", img);
        }

        output
    }

    #[component]
    pub fn Gallery(
        #[prop(default = 250)] row_height: u32,
        #[prop(optional)] username: Option<RwSignal<Option<String>>>,
    ) -> impl IntoView {
        let gallery = RwSignal::<Vec<Img>>::new(Vec::new());
        let delayed_scroll = RwSignal::new(0_usize);
        let gallery_ref = NodeRef::<Div>::new();
        let api_top = ApiWeb::new();
        let api_btm = ApiWeb::new();
        let navigate = leptos_router::hooks::use_navigate();
        let (get_query_scroll, set_query_scroll) = query_signal::<usize>("scroll");
        let (get_query_gallery_count, set_query_gallery_count) = query_signal::<usize>("img_count");
        let (get_query_direction, set_query_direction) = query_signal::<String>("direction");
        let (get_query_time, set_query_time) = query_signal::<u128>("time");

        let set_gallery = move |bottom: bool,
                                width: u32,
                                height: f64,
                                time: u128,
                                count: u32,
                                username: Option<String>| {
            let current_img_count = gallery.with_untracked(|v| v.len());
            // api_top.get_posts_newer_or_equal(time, limit)

            if let Some(username) = username {
                if bottom {
                    if current_img_count == 0 {
                        trace!("running get_user_posts_older_or_equal");
                        api_btm.get_user_posts_older_or_equal(time, count, username)
                    } else {
                        trace!("running get_user_posts_older");
                        api_btm.get_user_posts_older(time, count, username)
                    }
                } else {
                    if current_img_count == 0 {
                        trace!("running get_user_posts_newer_or_equal");
                        api_top.get_user_posts_newer_or_equal(time, count, username)
                    } else {
                        trace!("running get_user_posts_newer");
                        api_top.get_user_posts_newer(time, count, username)
                    }
                }
            } else {
                if bottom {
                    if current_img_count == 0 {
                        trace!("running get_posts_older_or_equal");
                        api_btm.get_posts_older_or_equal(time, count)
                    } else {
                        trace!("running get_posts_older");
                        api_btm.get_posts_older(time, count)
                    }
                } else {
                    if current_img_count == 0 {
                        trace!("running get_posts_newer_or_equal");
                        api_top.get_posts_newer_or_equal(time, count)
                    } else {
                        trace!("running get_posts_newer");
                        api_top.get_posts_newer(time, count)
                    }
                }
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

                        let new_imgs = files.into_iter().map(Img::from).collect::<Vec<Img>>();

                        if new_imgs.is_empty() {
                            trace!("RECEIVED EMPTY");
                            return;
                        }

                        let (resized_imgs, scroll_by) = if bottom {
                            add_imgs_to_bottom(prev_imgs, new_imgs, width, height, row_height)
                        } else {
                            add_imgs_to_top(prev_imgs, new_imgs, width, height, row_height)
                        };

                        let first_img_time = (if bottom {
                            resized_imgs.first()
                        } else {
                            resized_imgs.last()
                        })
                        .map(|v| v.created_at);
                        let new_img_count = resized_imgs.len();

                        trace!("using new scroll: {scroll_by}");
                        gallery.set(resized_imgs);
                        if current_img_count > 0 {
                            gallery_elm.scroll_by_with_x_and_y(0.0, scroll_by);
                        }
                        // if let Some(query_scroll) = get_query_scroll.get_untracked()
                        //     && gallery.with_untracked(|v| v.is_empty())
                        // {
                        //     trace!("using initial scroll: {query_scroll}");
                        //     gallery.set(resized_imgs);
                        //     // gallery_elm.scroll_by_with_x_and_y(0.0, query_scroll as f64);
                        //     delayed_scroll.set(query_scroll);
                        // } else {
                        //     trace!("using new scroll: {scroll_by}");
                        //     gallery.set(resized_imgs);
                        //     gallery_elm.scroll_by_with_x_and_y(0.0, scroll_by);
                        // }

                        set_query_direction
                            .set(Some(if bottom { "down" } else { "up" }.to_string()));
                        set_query_time.set(first_img_time);
                        set_query_gallery_count.set(Some(new_img_count));
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
            let user_username = username.get_untracked();
            trace!("gallery fetch top username state: {user_username:?}");
            let user_username = user_username.flatten();

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
            set_gallery(
                false,
                width,
                height * 8.0,
                img.created_at,
                count,
                user_username,
            );
        };

        let run_fetch_bottom = move || {
            let Some(gallery_elm) = gallery_ref.get_untracked() else {
                trace!("gallery NOT found");
                return;
            };
            if api_btm.busy.get_untracked() {
                return;
            }

            let user_username = username.get_untracked();
            trace!("gallery fetch btm username state: {user_username:?}");
            let user_username = user_username.flatten();
            // if username.is_some() && user_username.is_none() {
            //     return;
            // }

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
            set_gallery(
                true,
                width,
                height * 8.0,
                img.created_at,
                count,
                user_username,
            );
        };

        let run_on_click = move |e: MouseEvent, img: Img| {
            // let Some(gallery_elm) = gallery_ref.get_untracked() else {
            //     trace!("gallery NOT found");
            //     return;
            // };
            // if gallery.with_untracked(|v| v.is_empty()) {
            //     return;
            // }

            // e.prevent_default();
            // e.stop_propagation();
            //
            // let scroll_top = gallery_elm.scroll_top() as usize;
            // set_query_scroll.set(Some(scroll_top));
            //
            // let link = link_post(img.username.clone(), img.id.clone());
            // navigate(&link, Default::default());
        };

        let _ = interval::new(
            move || {
                // run_fetch_top();
                // run_fetch_bottom();
                let Some(gallery_elm) = gallery_ref.get_untracked() else {
                    trace!("gallery NOT found");
                    return;
                };

                let user_username = username.get_untracked();
                trace!("gallery watch username state: {user_username:?}");
                // let user_username = user_username.flatten();
                // if username.is_some() && user_username.is_none() {
                //     return;
                // }

                let scroll_top = gallery_elm.scroll_top() as u32;
                let scroll_height = gallery_elm.scroll_height() as u32;
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
            },
            Duration::from_secs(2),
        );

        let _ = interval::new(
            move || {
                // run_fetch_top();
                // run_fetch_bottom();
                let Some(gallery_elm) = gallery_ref.get_untracked() else {
                    trace!("gallery NOT found");
                    return;
                };
                if gallery.with_untracked(|v| v.is_empty()) {
                    return;
                }

                let scroll_top = gallery_elm.scroll_top() as usize;
                // let current_query_scroll = get_query_scroll();
                set_query_scroll.set(Some(scroll_top));
            },
            Duration::from_millis(1000),
        );

        let get_imgs = move || {
            let imgs = gallery.get();
            let total_count = imgs.len();

            imgs.into_iter()
                .enumerate()
                .map({
                    let run_fetch_bottom = run_fetch_bottom.clone();
                    let run_fetch_top = run_fetch_top.clone();
                    let run_on_click = run_on_click.clone();
                    move |(i, img)| view! {<GalleryImg index=i img total_count run_on_click=run_on_click.clone() run_fetch_bottom=run_fetch_bottom.clone() run_fetch_top=run_fetch_top.clone() />}
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
            trace!("running gallery init");

            let user_username = username.get();
            trace!("gallery init username state: {user_username:?}");
            let user_username = user_username.flatten();
            if username.is_some() && user_username.is_none() {
                return;
            }

            let width = gallery_elm.client_width() as u32;
            let height = gallery_elm.client_height() as f64;

            let query_gallery_count = get_query_gallery_count
                .get_untracked()
                .and_then(|v| if v == 0 { None } else { Some(v) });
            let query_direction_is_up = get_query_direction.get_untracked().map(|v| v == "up");
            let query_time = get_query_time.get_untracked();
            let query_scroll = get_query_scroll.get_untracked();

            let (Some(gallery_count), Some(direction_is_up), Some(time), Some(scroll)) = (
                query_gallery_count,
                query_direction_is_up,
                query_time,
                query_scroll,
            ) else {
                let count = calc_fit_count(width, height, row_height) as u32;
                let direction_is_bottom = true;
                let time = Utc::now().timestamp_micros() as u128 * 1000;
                trace!(
                    "initial gallery init - using new params {} {} {} {} {}",
                    direction_is_bottom, width, height, time, count
                );
                set_gallery(
                    direction_is_bottom,
                    width,
                    height,
                    time,
                    count,
                    user_username,
                );
                return;
            };

            delayed_scroll.set(scroll);
            trace!(
                "initial gallery init - using old params {} {} {} {} {}",
                !direction_is_up, width, height, time, gallery_count as u32
            );
            set_gallery(
                !direction_is_up,
                width,
                height,
                time,
                gallery_count as u32,
                user_username,
            );
        });

        Effect::new(move || {
            let Some(gallery_elm) = gallery_ref.get() else {
                return;
            };
            if api_top.busy.get_untracked() {
                return;
            }
            trace!("running gallery reset");

            let user_username = username.get();
            trace!("gallery reset username state: {user_username:?}");
            let user_username = user_username.flatten();
            if username.is_some() && user_username.is_none() {
                return;
            }

            let gallery_count = get_query_gallery_count.with(|v| v.is_some());
            let direction_is_bottom = get_query_direction.with(|v| v.is_some());
            let time = get_query_time.with(|v| v.is_some());
            let current_gallery_count = gallery.with_untracked(|v| v.len());

            if current_gallery_count == 0 || (gallery_count || direction_is_bottom || time) {
                return;
            }

            let width = gallery_elm.client_width() as u32;
            let height = gallery_elm.client_height() as f64;
            let count = calc_fit_count(width, height, row_height) as u32;
            let time = Utc::now().timestamp_micros() as u128 * 1000;

            gallery.set(Vec::new());

            set_gallery(true, width, height, time, count, user_username);
        });

        // let on_scroll = move |e: web_sys::Event| {
        //     let Some(gallery_elm) = gallery_ref.get() else {
        //         return;
        //     };
        //     let scroll_top = gallery_elm.scroll_top() as usize;
        //     set_query_scroll.set(Some(scroll_top));
        // };

        gallery_ref.add_mutation_observer(
            move |entries, observer| {
                trace!("IT HAS MUTATED");
                let Some(gallery_elm) = gallery_ref.get_untracked() else {
                    trace!("gallery NOT found");
                    return;
                };
                let delayed_scroll_value = delayed_scroll.get_untracked();
                if delayed_scroll_value == 0 {
                    return;
                }
                gallery_elm.scroll_by_with_x_and_y(0.0, delayed_scroll_value as f64);
                delayed_scroll.set(0);
            },
            MutationObserverOptions::new().set_child_list(true),
        );

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
    pub fn GalleryImg<FetchBtmFn, FetchTopFn, OnClickFn>(
        img: Img,
        index: usize,
        total_count: usize,
        run_on_click: OnClickFn,
        run_fetch_bottom: FetchBtmFn,
        run_fetch_top: FetchTopFn,
    ) -> impl IntoView
    where
        OnClickFn: Fn(MouseEvent, Img) + Send + Sync + 'static + Clone,
        FetchBtmFn: Fn() + Send + Sync + 'static + Clone,
        FetchTopFn: Fn() + Send + Sync + 'static + Clone,
    {
        let img_ref = NodeRef::<html::Img>::new();
        let link_ref = NodeRef::<html::A>::new();

        let query = use_query_map();
        let (get_query_scroll, set_query_scroll) = query_signal::<usize>("s");
        let activated = StoredValue::new(false);

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

                activated.set_value(false);

                let elm_on_which_fetches = total_count / 3;
                // if index == total_count.saturating_sub(1) && is_intersecting {
                if index == total_count.saturating_sub(elm_on_which_fetches)
                    || index == total_count.saturating_sub(1)
                {
                    run_fetch_bottom();
                    trace!("intersection fn last {index} is intesecting: {is_intersecting}");
                } else if index == elm_on_which_fetches || index == 0 {
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
        let img_id = img.id.clone();
        let img_username = img.username.clone();
        let post_link = img.get_post_link();
        // let post_link_with_history = img.get_post_link_with_history(9999);
        let img_link = img.get_img_link();

        let fn_left = move || format!("{view_left}px");
        let fn_top = move || format!("{view_top}px");
        let fn_width = move || format!("{view_width}px");
        let fn_height = move || format!("{view_height}px");

        // let l = location();

        let on_img_click = move |e: MouseEvent| {
            // e.prevent_default();
            // e.stop_propagation();
            // trace!("img click!!!!!!!!");
            // let link = link_post_with_history(img_username.clone(), img_id.clone(), 999);
            // navigate(&link, Default::default());
            run_on_click(e, img.clone());
            // let Some(link_ref) = link_ref.get_untracked() else {
            //     return;
            // };

            // set_query("scroll", "9999");
            // let href = e.target().map(|v| v);
            // set_query_scroll.set(Some(9999));
            // let issue_list_url = url::Url::parse(
            //     "https://github.com/rust-lang/rust/issues?labels=E-easy&state=open",
            // )
            // .unwrap();

            // l.set_search(value);
            // query.with(|v| v);
        };

        view! {

            <a
               href=post_link
               node_ref=link_ref
               on:click=on_img_click
            >
                <img
                    class="absolute"
                    node_ref=img_ref
                    style:left=fn_left
                    style:top=fn_top
                    style:width=fn_width
                    style:height=fn_height
                    src=img_link
                />
            </a>
        }
    }

    #[derive(Debug, Clone)]
    pub struct Img {
        pub id: String,
        pub username: String,
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

    impl From<UserPost> for Img {
        fn from(user_post: UserPost) -> Self {
            let post_thumbnail = user_post.file.first().cloned().unwrap_or(UserPostFile {
                width: 400,
                height: 400,
                hash: "404".to_string(),
                extension: "webp".to_string(),
            });
            Self {
                id: user_post.id,
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
        fn get_id(&self) -> String {
            self.id.clone()
        }
        fn get_post_link(&self) -> String {
            link_post(&self.username, &self.id)
        }
        // fn get_post_link_with_history(&self, scroll: usize) -> String {
        //     link_post_with_history(&self.username, &self.id, scroll)
        // }
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
                id: id.to_string(),
                // row_id: 0,
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
                id,
                // row_id: 0,
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
        // fn get_post_link_with_history(&self) -> String;
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
            pub id: String,
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
            pub fn new(id: usize, width: u32, height: u32) -> Self {
                Self {
                    id: id.to_string(),
                    width,
                    height,
                    view_width: OrderedFloat(0.0),
                    view_height: OrderedFloat(0.0),
                    view_pos_x: OrderedFloat(0.0),
                    view_pos_y: OrderedFloat(0.0),
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
                    view_width: OrderedFloat(view_width),
                    view_height: OrderedFloat(view_height),
                    view_pos_x: OrderedFloat(view_pos_x),
                    view_pos_y: OrderedFloat(view_pos_y),
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
