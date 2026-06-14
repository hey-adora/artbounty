use crate::{
    api::{
        Api, ApiWeb, Order, ServerErr, ServerReqImg, ServerRes, TimeRange, UserPost,
        shared::post_comment::UserPostComment,
    },
    view::{
        app::{
            components::gallery::{Img, add_imgs_to_bottom, add_imgs_to_top},
            hook::{
                use_future::FutureFn, use_infinite_scroll_basic::InfiniteBasic,
                use_infinite_scroll_fn::InfiniteItem, use_scroll_correction::ScrollCorrection,
            },
        },
        toolbox::prelude::*,
    },
};
use leptos::{
    html::{ElementType, Textarea},
    prelude::*,
};
use tracing::{debug, error, trace, warn};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, HtmlTextAreaElement, MutationObserver, MutationRecord};

#[derive(Clone, Copy, Default, Debug)]
pub struct GalleryContainerSize {
    pub width: u32,
    pub height: f64,
    pub row_height: u32,
}

#[derive(Clone, Copy)]
pub struct GalleryApi<API: Api> {
    // ui
    // pub items: RwSignal<Vec<Img>, LocalStorage>,
    pub items: StoreSignal<Vec<Img>>,
    pub scroll_correction_handle: ScrollCorrection,
    // params
    pub api_top: API,
    pub api_btm: API,
}

// TODO maybe it would be better design to have these as seperate functions, without struct
// abstraction
impl<API: Api> GalleryApi<API> {
    pub fn new(api_top: API, api_btm: API, scroll_correction_handle: ScrollCorrection) -> Self {
        let items = StoreSignal::new_with_formmater(true, "gallery_api_items", Vec::new(), |v| {
            serde_json::to_string(v).unwrap_or_else(|e| e.to_string())
        });
        Self {
            scroll_correction_handle,
            items,
            // items: RwSignal::new_local(Vec::new()),
            api_top,
            api_btm,
        }
    }

    // pub fn observe_only(&self, size: PostContainerSize) {
    //     self.size.set_value(size);
    // }

    pub async fn post(
        &self,
        size: GalleryContainerSize,
        title: impl Into<String>,
        description: impl Into<String>,
        tags: impl Into<String>,
        // files: Vec<ServerReqImg>,
    ) -> f64 {
        let items = self.items;

        let result = self
            .api_top
            .add_post(title, description, tags)
            .send_native()
            .await;

        match result {
            Ok(ServerRes::Post(post)) => {
                let new_img = Img::from(post);
                let new_imgs = Vec::from([new_img]);
                let old_imgs = items.get_untracked();

                trace!("CAN I MAKE THIS OR NOT");
                let (resized_imgs, scroll_by) = add_imgs_to_bottom(
                    old_imgs,
                    new_imgs,
                    size.width,
                    size.height,
                    size.row_height,
                );
                items.set(resized_imgs);
                return scroll_by;
            }
            Ok(err) => {
                let err = format!("post comments basic: unexpected res: {err:?}");
                error!(err);
                // self.err_fetch.set(err);
            }
            Err(err) => {
                let err = format!("post comments basic: {err}");
                error!(err);
                // self.err_fetch.set(err);
            }
        };
        0.0
    }

    pub async fn fetch(
        self,
        limit: usize,
        size: GalleryContainerSize,
        time_range: TimeRange,
        order: Order,
        reverse: bool,
        tags: impl Into<String>,
        username: impl Into<String>,
    ) -> f64 {
        let tags = tags.into();
        let username = username.into();
        let items = self.items;
        let scroll_correction = self.scroll_correction_handle;

        let is_bottom = match time_range {
            TimeRange::None => true,
            TimeRange::Less(_) => true,
            TimeRange::LessOrEqual(_) => true,
            TimeRange::More(_) => false,
            TimeRange::MoreOrEqual(_) => false,
        };

        let api = if is_bottom {
            &self.api_top
        } else {
            &self.api_btm
        };

        let result = api
            .get_posts(limit, time_range, order, tags, username)
            .send_native()
            .await;

        match result {
            Ok(ServerRes::Posts(mut posts)) => {
                if reverse {
                    posts.reverse();
                }

                let new_imgs = posts.into_iter().map(Img::from).collect::<Vec<Img>>();
                let old_imgs = items.get_untracked();

                let (resized_imgs, scroll_by) = if is_bottom {
                    add_imgs_to_bottom(old_imgs, new_imgs, size.width, size.height, size.row_height)
                } else {
                    add_imgs_to_top(old_imgs, new_imgs, size.width, size.height, size.row_height)
                };
                scroll_correction.update();
                items.set(resized_imgs);

                return scroll_by;
            }
            Ok(err) => {
                let err = format!("post comments basic: unexpected res: {err:?}");
                error!(err);
                // self.err_fetch.set(err);
            }
            Err(err) => {
                let err = format!("post comments basic: {err}");
                error!(err);
                // self.err_fetch.set(err);
            }
        };

        0.0
    }

    pub async fn fetch_btm(
        self,
        limit: usize,
        size: GalleryContainerSize,
        current_time: u128,
        tags: impl Into<String>,
        username: impl Into<String>,
    ) -> f64 {
        let time_range = self
            .items
            .with_untracked(|v| v.last().map(|v| v.created_at))
            .map(TimeRange::Less)
            .unwrap_or(TimeRange::LessOrEqual(current_time));

        self.fetch(
            limit,
            size,
            time_range,
            Order::ThreeTwoOne,
            false,
            tags,
            username,
        )
        .await
    }

    pub async fn fetch_top(
        self,
        limit: usize,
        size: GalleryContainerSize,
        current_time: u128,
        tags: impl Into<String>,
        username: impl Into<String>,
    ) -> f64 {
        let time_range = self
            .items
            .with_untracked(|v| v.first().map(|v| v.created_at))
            .map(TimeRange::More)
            .unwrap_or(TimeRange::MoreOrEqual(current_time));
        debug!("time range picked: {time_range:?}");

        self.fetch(
            limit,
            size,
            time_range,
            Order::OneTwoThree,
            true,
            tags,
            username,
        )
        .await
    }

    pub async fn fetch_btm_or_top(
        self,
        is_bottom: bool,
        limit: usize,
        size: GalleryContainerSize,
        current_time: u128,
        tags: impl Into<String>,
        username: impl Into<String>,
    ) -> f64 {
        if is_bottom {
            self.fetch_btm(limit, size, current_time, tags, username)
                .await
        } else {
            self.fetch_top(limit, size, current_time, tags, username)
                .await
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.with_untracked(|v| v.is_empty())
    }

    pub fn reset(&self) {
        self.items.set(Vec::new());
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        api::{ServerReqImg, tests::ApiTestApp},
        view::app::hook::{
            api_gallery::{GalleryApi, GalleryContainerSize},
            use_scroll_correction::ScrollCorrection,
        },
    };
    use hydration_context::HydrateSharedContext;
    use leptos::prelude::*;
    use std::sync::Arc;
    use tracing::{debug, trace};

    use crate::init_test_log;

    pub async fn create_img(name: impl Into<String>, width: u32, height: u32) -> Vec<u8> {
        let mut imgbuf = image::ImageBuffer::new(width, height);
        // Iterate over the coordinates and pixels of the image
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let r = (0.3 * x as f32) as u8;
            let b = (0.3 * y as f32) as u8;
            *pixel = image::Rgb([r, 0, b]);
        }

        // create_dir_all("../target/tmp/").await.unwrap();
        // let path = "../target/tmp/img.png";
        let path = format!("/tmp/{}.png", name.into());
        imgbuf.save(&path).unwrap();

        tokio::fs::read(path).await.unwrap()
    }

    pub async fn create_img_req(name: impl Into<String>, width: u32, height: u32) -> ServerReqImg {
        let name = name.into();
        let v = create_img(name.clone(), width, height).await;
        ServerReqImg {
            path: name,
            data: v,
        }
    }

    #[tokio::test]
    pub async fn hook_gallery_api_post() {
        println!("hello");
        init_test_log();
        let owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
        let scroll_corerction = ScrollCorrection::new();
        let mut app = ApiTestApp::new(10).await;

        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let auth_token2 = app
            .register(0, "hey2", "hey2@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        app.api.auth_token_overwrite = auth_token.clone();

        let post_api = GalleryApi::new(&app.api, &app.api, scroll_corerction.clone());
        let size = GalleryContainerSize {
            width: 100,
            height: 100.0,
            row_height: 50,
        };
        // post_api.observe_only(PostContainerSize {
        //     width: 100,
        //     height: 100.0,
        //     row_height: 50,
        // });

        app.set_time(1).await;
        post_api
            .post(
                size, "title1", "0", "",
                // vec![create_img_req("1", 50, 50).await],
            )
            .await;
        app.set_time(2).await;
        post_api
            .post(
                size, "title2", "0", "",
                // vec![create_img_req("2", 50, 50).await],
            )
            .await;
        app.set_time(3).await;
        post_api
            .post(
                size, "title3", "0", "",
                // vec![create_img_req("3", 50, 50).await],
            )
            .await;
        let items = post_api.items.get_untracked();
        trace!("aaaaa {items:#?}");
        assert_eq!(items.len(), 3);

        app.set_time(4).await;
        let post_api2 = GalleryApi::new(&app.api, &app.api, scroll_corerction.clone());
        post_api2.fetch_btm(10, size, 4, "", "").await;
        let items = post_api2.items.get_untracked();
        assert_eq!(items.len(), 3);

        let post_api3 = GalleryApi::new(&app.api, &app.api, scroll_corerction.clone());
        post_api3.fetch_btm(2, size, 4, "", "").await;
        let items = post_api3.items.get_untracked();
        assert_eq!(items.len(), 2);
        post_api3.fetch_btm(2, size, 4, "", "").await;
        let items = post_api3.items.get_untracked();
        assert_eq!(items.len(), 3);

        let post_api = GalleryApi::new(&app.api, &app.api, scroll_corerction.clone());
        post_api.fetch_top(2, size, 4, "", "").await;
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 0);

        let post_api = GalleryApi::new(&app.api, &app.api, scroll_corerction.clone());
        post_api.fetch_top(2, size, 0, "", "").await;
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].created_at, 2);
        assert_eq!(items[1].created_at, 1);
        post_api.fetch_top(2, size, 0, "", "").await;
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].created_at, 3);
        assert_eq!(items[1].created_at, 2);
        assert_eq!(items[2].created_at, 1);

        let post_api = GalleryApi::new(&app.api, &app.api, scroll_corerction.clone());
        post_api.fetch_btm(3, size, 3, "", "").await;
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].created_at, 3);
        assert_eq!(items[1].created_at, 2);
        assert_eq!(items[2].created_at, 1);
        post_api.items.update_untracked(|v| {
            v.remove(0);
            v.remove(0);
        });
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].created_at, 1);
        post_api.fetch_top(1, size, 4, "", "").await;
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].created_at, 2);
        assert_eq!(items[1].created_at, 1);
        post_api.fetch_top(1, size, 4, "", "").await;
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].created_at, 3);
        assert_eq!(items[1].created_at, 2);
        assert_eq!(items[2].created_at, 1);

        // let items = post_api.items.get_untracked();
        // assert_eq!(items.len(), 2);
        // assert_eq!(items[0].created_at, 2);
        // post_api.fetch_top(1, size, 4, "", "").await;
        // let items = post_api.items.get_untracked();
        // assert_eq!(items.len(), 3);
        // assert_eq!(items[0].created_at, 1);

        // post_api.fetch_top(1, size, 4, "", "").await;
        // let items = post_api.items.get_untracked();
        // assert_eq!(items.len(), 2);

        let post_api = GalleryApi::new(&app.api, &app.api, scroll_corerction.clone());
        post_api.fetch_btm(50, size, 4, "", "").await;
        let items = post_api.items.get_untracked();
        trace!("ITEMS1: {items:#?}");
        assert_eq!(items.len(), 3);
        post_api.fetch_top(50, size, 4, "", "").await;
        let items = post_api.items.get_untracked();
        trace!("ITEMS2: {items:#?}");
        assert_eq!(items.len(), 3);

        app.api.auth_token_overwrite = auth_token2.clone();
        let post_api = GalleryApi::new(&app.api, &app.api, scroll_corerction.clone());

        app.set_time(5).await;
        post_api
            .post(
                size,
                "title1",
                "0",
                "one two three",
                // vec![create_img_req("1", 50, 50).await],
            )
            .await;
        app.set_time(6).await;
        post_api
            .post(
                size, "title2", "0", "one two",
                // vec![create_img_req("2", 50, 50).await],
            )
            .await;
        app.set_time(7).await;
        post_api
            .post(
                size, "title3", "0", "one",
                // vec![create_img_req("3", 50, 50).await],
            )
            .await;

        app.set_time(8).await;
        let post_api2 = GalleryApi::new(&app.api, &app.api, scroll_corerction.clone());
        post_api2.fetch_btm(2, size, 8, "one", "hey2").await;
        let items = post_api2.items.get_untracked();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].created_at, 7);
        assert_eq!(items[1].created_at, 6);
        post_api2.fetch_btm(2, size, 8, "one", "hey2").await;
        let items = post_api2.items.get_untracked();
        assert_eq!(items.len(), 3);
        assert_eq!(items[2].created_at, 5);

        app.set_time(9).await;
        let post_api2 = GalleryApi::new(&app.api, &app.api, scroll_corerction.clone());
        post_api2.fetch_btm(3, size, 9, "one two", "hey2").await;
        let items = post_api2.items.get_untracked();
        assert_eq!(items.len(), 2);

        //
    }
}
