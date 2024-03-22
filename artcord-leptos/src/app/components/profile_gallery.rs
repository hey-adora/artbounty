use crate::app::components::navbar::shrink_nav;
use crate::app::global_state::GlobalState;
use artcord_state::message::prod_client_msg::ClientMsg;
use artcord_state::message::prod_server_msg::{ServerMsg, UserGalleryResponse, UserResponse};
use chrono::Utc;
use leptos::ev::resize;
use leptos::html::Section;
use leptos::logging::log;
use leptos::*;
use leptos_router::{use_location, use_params_map};
use leptos_use::{use_event_listener, use_window};
use web_sys::Event;
use tracing::{debug, error, trace};

use crate::app::utils::{
    calc_fit_count, resize_imgs, LoadingNotFound, SelectedImg, ServerMsgImgResized, NEW_IMG_HEIGHT,
};

//F: Fn(ServerMsgImgResized) -> IV + 'static, IV: IntoView
#[component]
pub fn ProfileGallery() -> impl IntoView {
    let params = use_params_map();
    let global_state = use_context::<GlobalState>().expect("Failed to provide global state");
    let gallery_section = create_node_ref::<Section>();
    let nav_tran = global_state.nav_tran;
    let global_gallery_imgs = global_state.page_profile.gallery_imgs;
    let global_gallery_user = global_state.page_profile.user;
    let selected_img: RwSignal<Option<SelectedImg>> = create_rw_signal(None);
    let loaded_sig = global_state.page_profile.gallery_loaded;
    // let _connection_load_state_name = SERVER_MSG_PROFILE_IMGS_NAME;
    let location = use_location();

    let ws_gallery = global_state.ws.create_singleton();
    let ws_user = global_state.ws.create_singleton();

    let on_click = move |img: ServerMsgImgResized| {
        selected_img.set(Some(SelectedImg {
            org_url: img.display_high.clone(),
            author_name: img.user.name.clone(),
            author_pfp: format!("/assets/gallery/pfp_{}.webp", img.user.id.clone()),
            author_id: img.user_id.clone(),
            width: img.width,
            height: img.height,
        }))
    };

    let on_fetch = move || {
        let current_loading_state = loaded_sig.get();
        // if current_loading_state == LoadingNotFound::NotLoaded {
            
        // }

        let Some(section) = gallery_section.get_untracked() else {
            return;
        };

        let last = global_gallery_imgs.with_untracked(|imgs| imgs.last().map(|img|img.created_at)).unwrap_or(Utc::now().timestamp_millis());
        let client_height = section.client_height();
        let client_width = section.client_width();

        let Some(new_user) = params.with(|p| p.get("id").cloned()) else {
            return;
        };
        
        let current_user = global_gallery_user.get();

        let same_user = current_user.map(|user| user.id == new_user).unwrap_or(false);

        let fetch = {
            debug!("profile_gallery: fetching imgs");
            //let new_user = new_user.clone();
            move |user_id: String| {
                let msg = ClientMsg::UserGalleryInit {
                    amount: calc_fit_count(client_width as u32, client_height as u32) * 2,
                    from: last,
                    user_id,
                };
        
                let _ = ws_gallery.send_once(msg, move |server_msg| {
                    if let ServerMsg::UserGallery(response) = server_msg {
                        match response {
                            UserGalleryResponse::Imgs(new_imgs) => {
                                let new_imgs = new_imgs
                                    .iter()
                                    .map(|img| ServerMsgImgResized::from(img.to_owned()))
                                    .collect::<Vec<ServerMsgImgResized>>();
        
                                global_gallery_imgs.update(|imgs| {
                                    imgs.extend(new_imgs);
                                    //let document = document();
                                    //let gallery_section = document.get_element_by_id("gallery_section");
                                    let gallery_section = gallery_section.get_untracked();
                                    let Some(gallery_section) = gallery_section else {
                                        return;
                                    };
                                    let width = gallery_section.client_width() as u32;
                                    resize_imgs(NEW_IMG_HEIGHT, width, imgs);
                                });
                                loaded_sig.set(LoadingNotFound::Loaded);
                            }
                            UserGalleryResponse::UserNotFound => {
                                //error!("received wrong response for user_gallery: {:?}", &server_msg);
                                loaded_sig.set(LoadingNotFound::NotFound);
                            }
                        }
                    } else {
                        loaded_sig.set(LoadingNotFound::Error);
                    }
                }).inspect_err(|err| error!("profile_gallery: failed to send msg: {err}"));
            }
        };

        if same_user {
            fetch(new_user);
        } else {
            debug!("profile_gallery: fetching user");
            loaded_sig.set(LoadingNotFound::Loading);
            global_gallery_imgs.set(Vec::new());

            let msg = ClientMsg::User { user_id: new_user };
            let send_result = ws_user.send_once(msg, move |server_msg| {
                if let ServerMsg::User(response) = server_msg {
                    match response {
                        UserResponse::User(user) => {
                            let user_id = user.id.clone();
                            global_gallery_user.set(Some(user));
                            fetch(user_id);
                        }
                        UserResponse::UserNotFound => {
                            loaded_sig.set(LoadingNotFound::NotFound);
                        }
                    }
                } else {
                    error!("profile_gallery: received wrong response for user: {:?}", server_msg);
                    loaded_sig.set(LoadingNotFound::Error);
                }
            });
        }


        
        //log!("USER GALLERY FETCH: {:#?}", &msg);
        //global_state.socket_send(&msg);
    };

    create_effect(move |_| {
        let loaded = loaded_sig.with_untracked(|state| *state == LoadingNotFound::Loaded);
        if !loaded {
            return;
        }
        let _ = location.pathname.get();
        let _ = location.hash.get();

        global_gallery_imgs.update(|imgs| {
            let section = gallery_section.get_untracked();
            if let Some(section) = section {
                let width = section.client_width() as u32;

                resize_imgs(NEW_IMG_HEIGHT, width, imgs);
            };
        });
    });

    let section_scroll = move |_: Event| {
        // if !global_state.socket_state_is_ready(connection_load_state_name) {
        //     return;
        // }

        // let Some(last) = global_gallery_imgs.with_untracked(|imgs| match imgs.last() {
        //     Some(l) => Some(l.created_at),
        //     None => None,
        // }) else {
        //     return;
        // };

        let Some(section) = gallery_section.get_untracked() else {
            return;
        };

        let scroll_top = section.scroll_top();
        let client_height = section.client_height();
        let scroll_height = section.scroll_height();
        //let client_width = section.client_width();

        shrink_nav(nav_tran, scroll_top as u32);

        let left = scroll_height - (client_height + scroll_top);

        if left < client_height {
            //global_state.socket_state_used(connection_load_state_name);
            on_fetch();
        }
    };

    create_effect(move |_| {
        let _ = use_event_listener(use_window(), resize, move |_| {
            // log!("TRYING TO RESIZE");
            global_gallery_imgs.update(|imgs| {
                let section = gallery_section.get_untracked();
                if let Some(section) = section {
                    let width = section.client_width() as u32;

                    // log!("RESIZING!!!!!!!!!!!!");
                    resize_imgs(NEW_IMG_HEIGHT, width, imgs);
                };
            });
        });
    });

    // create_effect(move |_| {
    //     let Some(new_user) = params.with(|p| p.get("id").cloned()) else {
    //         return;
    //     };

    //     let user = global_state.page_profile.user.get();

    //     let same_user = if let Some(ref user) = user {
    //         new_user == user.id
    //     } else {
    //         false
    //     };
    //     log!("ONE {} {:?}", same_user, user);
    //     // if !global_state.socket_state_is_ready(SERVER_MSG_PROFILE) {
    //     //     return;
    //     // }

    //     if !same_user {
    //         let _msg = ClientMsg::User {
    //             user_id: String::from(new_user),
    //         };
    //         //global_state.socket_send(&msg);
    //     }
    // });

    create_effect(move |_| {
        // let loaded = !loaded_sig.with(|state| *state == LoadingNotFound::NotLoaded);
        // if loaded {
        //     return;
        // }

        // let user = global_state.page_profile.user.get();
        // log!("TWO {:?}", user);
        // let Some(_user) = user else {
        //     return;
        // };

        // if !global_state.socket_state_is_ready(SERVER_MSG_PROFILE_IMGS_NAME) {
        //     return;
        // }

        // let Some(section) = gallery_section.get_untracked() else {
        //     return;
        // };

        // let client_height = section.client_height();
        // let client_width = section.client_width();

        // // //global_state.socket_state_used(connection_load_state_name);

        // // global_gallery_imgs.set(vec![]);

        // loaded_sig.set(LoadingNotFound::Loading);

        on_fetch();
    });

    view! {
        {
            move || {
                match selected_img.get() {
                    Some(img) => Some(view! {
                        <div on:click=move |_| { selected_img.set(None); } class=" absolute grid grid-rows-[1fr] left-0 top-0 w-screen h-[100dvh] place-items-center bg-gradient-to-br from-mid-purple/50 to-dark-purple/50 z-[150] ">
                            <div on:click=move |e| { e.stop_propagation();  } >
                                <div class="flex justify-between items-center rounded-t-lg bg-dark-purple pl-2">
                                       <div class="flex gap-2">
                                            <div>"By "</div>
                                            <img class="border border-low-purple rounded-full bg-mid-purple h-[25px] " src=img.author_pfp/>
                                            <a href=move||format!("/user/{}", img.author_id)>{img.author_name}</a>
                                       </div>
                                     <img on:click=move |_| { selected_img.set(None); } class="cursor-pointer border-2 border-low-purple rounded-full bg-mid-purple w-[30px] h-[30px] p-1 m-2" src="/assets/x.svg"/>
                                </div>
                                <img class="bg-mid-purple object-contain " alt="loading..." style=move|| format!("max-height: calc(100dvh - 70px); max-width: 100vw; height: min({1}px, calc(100vw * ( {1} / {0} ))); aspect-ratio: {0} / {1};", img.width, img.height)  src=img.org_url/>
                            </div>
                        </div> }),
                None => None
                }
            }
        }
        <section id="profile_gallery_section" on:scroll=section_scroll _ref=gallery_section class="relative content-start overflow-x-hidden overflow-y-scroll h-full" >
            <Show when=move|| global_state.socket_connected.get() fallback=move || { "Connecting..." }>
                <Show when=move||loaded_sig.with(|state| *state == LoadingNotFound::NotLoaded || *state == LoadingNotFound::Loading) >
                  <div>"LOADING..."</div>
                </Show>
                <Show when=move||loaded_sig.with(|state| *state == LoadingNotFound::NotFound) >
                  <div>"No Images Found."</div>
                </Show>
            </Show>

            <For each=move || global_gallery_imgs.get().into_iter().enumerate()  key=|state| state.1.id.clone() let:data > {
                    let img = data.1;
                    let i = data.0;
                    let height = img.new_height;
                    let width = img.new_width;
                    let top = img.top;
                    let left = img.left;
                    let bg_img = format!("url('{}')", &img.display_preview);

                        view! {
                            <div
                                class="absolute bg-center bg-contain transition-all bg-no-repeat flex-shrink-0 font-bold grid place-items-center border hover:shadow-glowy hover:z-[99]  duration-300 bg-mid-purple  border-low-purple"
                                style:height=move || format!("{}px", height.get())
                                style:width=move || format!("{}px", width.get())
                                style:top=move || format!("{}px", top.get())
                                style:left=move || format!("{}px", left.get())
                                style:background-image=bg_img
                                on:click=  move |_| on_click(global_gallery_imgs.with_untracked(|imgs|imgs[i].clone()))
                            >
                            </div>
                        }
                    }

            </For>
        </section>
    }
}
