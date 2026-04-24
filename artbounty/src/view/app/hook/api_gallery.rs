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
                use_infinite_scroll_fn::InfiniteItem,
            },
        },
        toolbox::prelude::*,
    },
};
use leptos::{
    html::{ElementType, Textarea},
    prelude::*,
};
use tracing::{error, trace, warn};
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
    pub items: RwSignal<Vec<Img>, LocalStorage>,

    // params
    // pub post_key: StoredValue<String, LocalStorage>,
    // pub input_elm: StoredValue::;
    // pub post_key: StoredValue<String, LocalStorage>,
    // pub size: StoredValue<PostContainerSize, LocalStorage>,
    pub fetch_count: usize,
    pub api: API,
}

impl<API: Api> GalleryApi<API> {
    pub fn new(api: API, fetch_count: usize) -> Self {
        Self {
            items: RwSignal::new_local(Vec::new()),
            // size: StoredValue::new_local(PostContainerSize::default()),
            // post_key: StoredValue::new_local(String::new()),
            fetch_count,
            api,
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
        files: Vec<ServerReqImg>,
    ) -> f64 {
        let items = self.items;
        // let size = self.size.get_value();
        // if size.row_height == 0 || size.height == 0.0 || size.width == 0 {
        //     warn!("required params size({size:?} were not set)");
        //     return 0.0;
        // }
        // let post_key = self.post_key.get_value();
        // let limit = self.fetch_count;

        let result = self
            .api
            .add_post(title, description, tags, files)
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
        // is_bottom: bool,
        // time: u128,
        size: GalleryContainerSize,
        time_range: TimeRange,
        order: Order,
        tags: impl Into<String>,
        username: impl Into<String>,
    ) -> f64 {
        let tags = tags.into();
        let username = username.into();
        let items = self.items;
        let is_empty = items.with_untracked(|v| v.is_empty());
        // let post_key = self.post_key.get_value();
        let limit = self.fetch_count;
        // let size = self.size.get_value();
        // if size.row_height == 0 || size.height == 0.0 || size.width == 0 {
        //     warn!("required params size({size:?} were not set)");
        //     return 0.0;
        // }

        let is_bottom = match time_range {
            TimeRange::None => true,
            TimeRange::Less(_) => true,
            TimeRange::LessOrEqual(_) => true,
            TimeRange::More(_) => false,
            TimeRange::MoreOrEqual(_) => false,
        };
        // let time_range = match (is_empty, is_bottom) {
        //     (true, true) => TimeRange::LessOrEqual(()),
        //
        // }

        let result = self
            .api
            .get_posts(limit, time_range, order, tags, username)
            .send_native()
            .await;

        match result {
            Ok(ServerRes::Posts(posts)) => {
                let new_imgs = posts.into_iter().map(Img::from).collect::<Vec<Img>>();
                let old_imgs = items.get_untracked();

                let (resized_imgs, scroll_by) = if is_bottom {
                    add_imgs_to_bottom(old_imgs, new_imgs, size.width, size.height, size.row_height)
                } else {
                    add_imgs_to_top(old_imgs, new_imgs, size.width, size.height, size.row_height)
                };

                items.set(resized_imgs);

                return scroll_by;

                // let fetch_count = self.fetch_count;
                // let len = comments.len();

                // return comments;
                // trace!(
                //     "comments manual (len){len} < (fetch_count){fetch_count} = {}",
                //     len < fetch_count
                // );
                //
                // if len == fetch_count {
                //     finished.set(false);
                // } else if !finished.get_untracked() && len < fetch_count {
                //     finished.set(true);
                // }
                //
                // if len > 0 {
                //     let replies_count = self.replies_count;
                //     self.items.update(|v| {
                //         trace!("comments manual before {v:#?}");
                //         v.extend(comments);
                //         let len = v.len();
                //
                //         trace!("replies count {} {}", replies_count.get_untracked(), len);
                //         // panic!("stop");
                //         if replies_count.get_untracked() < len {
                //             replies_count.set(len);
                //
                //         }
                //         trace!("comments manual after {v:#?}");
                //     });
                // }
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

        self.fetch(size, time_range, Order::ThreeTwoOne, tags, username)
            .await
    }

    pub async fn fetch_top(
        self,
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

        self.fetch(size, time_range, Order::ThreeTwoOne, tags, username)
            .await
    }

    pub async fn fetch_btm_or_top(
        self,
        is_bottom: bool,
        size: GalleryContainerSize,
        current_time: u128,
        tags: impl Into<String>,
        username: impl Into<String>,
    ) -> f64 {
        if is_bottom {
            self.fetch_btm(size, current_time, tags, username).await
        } else {
            self.fetch_top(size, current_time, tags, username).await
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
        api::{
            Order, ServerReqImg, TimeRange, shared::post_comment::UserPostComment,
            tests::ApiTestApp,
        },
        view::{
            app::hook::{
                api_gallery::{GalleryApi, GalleryContainerSize},
                api_post_comments::{CommentKind, CommentKind2, CommentsApi, CommentsApi2},
            },
            logger,
            toolbox::prelude::*,
        },
    };
    use hydration_context::HydrateSharedContext;
    use leptos::prelude::*;
    use std::sync::Arc;
    use surrealdb::types::ToSql;
    use tokio::process::Command;
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
        let mut app = ApiTestApp::new(10).await;

        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let auth_token2 = app
            .register(0, "hey2", "hey2@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        app.api.pre_load_token = auth_token.clone();

        let post_api = GalleryApi::new(&app.api, 10);
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
                size,
                "title1",
                "0",
                "",
                vec![create_img_req("1", 50, 50).await],
            )
            .await;
        app.set_time(2).await;
        post_api
            .post(
                size,
                "title2",
                "0",
                "",
                vec![create_img_req("2", 50, 50).await],
            )
            .await;
        app.set_time(3).await;
        post_api
            .post(
                size,
                "title3",
                "0",
                "",
                vec![create_img_req("3", 50, 50).await],
            )
            .await;
        let items = post_api.items.get_untracked();
        trace!("aaaaa {items:#?}");
        assert_eq!(items.len(), 3);

        app.set_time(4).await;
        let post_api2 = GalleryApi::new(&app.api, 10);
        post_api2.fetch_btm(size, 4, "", "").await;
        let items = post_api2.items.get_untracked();
        assert_eq!(items.len(), 3);

        let post_api3 = GalleryApi::new(&app.api, 2);
        post_api3.fetch_btm(size, 4, "", "").await;
        let items = post_api3.items.get_untracked();
        assert_eq!(items.len(), 2);
        post_api3.fetch_btm(size, 4, "", "").await;
        let items = post_api3.items.get_untracked();
        assert_eq!(items.len(), 3);

        let post_api = GalleryApi::new(&app.api, 2);
        post_api.fetch_top(size, 4, "", "").await;
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 0);

        let post_api = GalleryApi::new(&app.api, 2);
        post_api.fetch_top(size, 0, "", "").await;
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 2);
        post_api.fetch_top(size, 0, "", "").await;
        let items = post_api.items.get_untracked();
        assert_eq!(items.len(), 2);

        let post_api = GalleryApi::new(&app.api, 50);
        post_api.fetch_btm(size, 4, "", "").await;
        let items = post_api.items.get_untracked();
        trace!("ITEMS1: {items:#?}");
        assert_eq!(items.len(), 3);
        post_api.fetch_top(size, 4, "", "").await;
        let items = post_api.items.get_untracked();
        trace!("ITEMS2: {items:#?}");
        assert_eq!(items.len(), 3);

        app.api.pre_load_token = auth_token2.clone();
        let post_api = GalleryApi::new(&app.api, 2);

        app.set_time(5).await;
        post_api
            .post(
                size,
                "title1",
                "0",
                "one two three",
                vec![create_img_req("1", 50, 50).await],
            )
            .await;
        app.set_time(6).await;
        post_api
            .post(
                size,
                "title2",
                "0",
                "one two",
                vec![create_img_req("2", 50, 50).await],
            )
            .await;
        app.set_time(7).await;
        post_api
            .post(
                size,
                "title3",
                "0",
                "one",
                vec![create_img_req("3", 50, 50).await],
            )
            .await;

        app.set_time(8).await;
        let post_api2 = GalleryApi::new(&app.api, 2);
        post_api2.fetch_btm(size, 8, "one", "hey2").await;
        let items = post_api2.items.get_untracked();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].created_at, 7);
        assert_eq!(items[1].created_at, 6);
        post_api2.fetch_btm(size, 8, "one", "hey2").await;
        let items = post_api2.items.get_untracked();
        assert_eq!(items.len(), 3);
        assert_eq!(items[2].created_at, 5);

        app.set_time(9).await;
        let post_api2 = GalleryApi::new(&app.api, 3);
        post_api2.fetch_btm(size, 9, "one two", "hey2").await;
        let items = post_api2.items.get_untracked();
        assert_eq!(items.len(), 2);

        //
    }
}
