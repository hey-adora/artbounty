use leptos::prelude::*;

use crate::{
    api::{Api, Server404Err, ServerErr, ServerUpdatePostDescriptionErr},
    path::{link_home, link_img, link_user},
};
use tracing::{error, info, trace, warn};

#[derive(Clone, Copy)]
pub struct PostApi<API: Api> {
    // ui
    // pub items: RwSignal<Vec<Img>, LocalStorage>,
    pub err_general: RwSignal<String, LocalStorage>,
    pub err_tags: RwSignal<String, LocalStorage>,
    pub err_description: RwSignal<String, LocalStorage>,
    pub live_description_length: RwSignal<usize, LocalStorage>,
    pub imgs_links: RwSignal<Vec<(String, f64)>, LocalStorage>,
    pub title: RwSignal<String, LocalStorage>,
    pub author: RwSignal<String, LocalStorage>,
    pub author_link: RwSignal<String, LocalStorage>,
    pub tags: RwSignal<String, LocalStorage>,
    // pub tags_is_empty: RwSignal<bool, LocalStorage>,
    pub update_tags_mode: RwSignal<bool, LocalStorage>,
    pub update_description_mode: RwSignal<bool, LocalStorage>,
    // pub tags_is_e: RwSignal<String, LocalStorage>,
    pub description: RwSignal<String, LocalStorage>,
    // pub description_is_empty: RwSignal<bool, LocalStorage>,
    pub favorites: RwSignal<u64, LocalStorage>,
    pub post_state: RwSignal<PostState, LocalStorage>,

    pub api: API,
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    PartialOrd,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum PostState {
    #[default]
    Loading,
    Normal,
    NotFound,
    Deleted,
}

impl<API: Api> PostApi<API> {
    pub fn new(api: API) -> Self {
        Self {
            // items: RwSignal::new_local(Vec::new()),
            imgs_links: RwSignal::new_local(Vec::<(String, f64)>::new()),
            title: RwSignal::new_local(String::from("loading...")),
            author: RwSignal::new_local(String::from("loading...")),
            author_link: RwSignal::new_local(link_home()),
            tags: RwSignal::new_local(String::new()),
            live_description_length: RwSignal::new_local(0),
            err_general: RwSignal::new_local(String::new()),
            err_tags: RwSignal::new_local(String::new()),
            err_description: RwSignal::new_local(String::new()),
            update_tags_mode: RwSignal::new_local(false),
            update_description_mode: RwSignal::new_local(false),
            description: RwSignal::new_local(String::from("loading...")),
            // description_is_empty: RwSignal::new_local(true),
            favorites: RwSignal::new_local(0_u64),
            post_state: RwSignal::new_local(PostState::Loading),
            api,
        }
    }

    pub async fn update_description(
        self,
        post_key: impl Into<String>,
        description: impl Into<String>,
    ) -> Option<()> {
        self.err_description.update(|v| v.clear());

        let result = self
            .api
            .update_post_description(post_key, description)
            .send_native()
            .await;

        match result {
            Ok(crate::api::ServerRes::Post(v)) => {
                self.live_description_length.set(v.description.len());
                self.description.set(v.description);
                self.update_description_mode.set(false);
                return Some(());
            }
            Ok(res) => {
                let err = format!("wrong res, expected Post, got {:?}", res);
                error!(err);
                self.err_description.set(err);
            }
            Err(ServerErr::UpdatePostErr(ServerUpdatePostDescriptionErr::TooLong)) => {
                self.err_description.set("Description is too long".to_string());
            }
            Err(ServerErr::UpdatePostErr(ServerUpdatePostDescriptionErr::NotFound)) => {
                self.post_state.set(PostState::NotFound);
                self.err_description.set("post not found".to_string());
            }
            Err(err) => {
                let err = format!("unexpected err {:#?}", { err });
                error!(err);
                self.err_description.set(err);
            }
        }

        None
    }

    pub async fn update_tags(
        self,
        post_key: impl Into<String>,
        tags: impl Into<String>,
    ) -> Option<()> {
        self.err_tags.update(|v| v.clear());

        let result = self
            .api
            .update_post_tags(post_key, tags)
            .send_native()
            .await;

        match result {
            Ok(crate::api::ServerRes::Post(v)) => {
                self.tags.set(v.tags);
                self.update_tags_mode.set(false);
                return Some(());
            }
            Ok(res) => {
                let err = format!("wrong res, expected Post, got {:?}", res);
                error!(err);
                self.err_tags.set(err);
            }
            Err(ServerErr::NotFoundErr(Server404Err::NotFound)) => {
                self.post_state.set(PostState::NotFound);
                self.err_tags.set("post not found".to_string());
            }
            Err(err) => {
                let err = format!("unexpected err {:#?}", { err });
                error!(err);
                self.err_tags.set(err);
            }
        }

        None
    }

    pub async fn delete(self, post_id: impl Into<String>) -> Option<()> {
        let post_id = post_id.into();
        let result = self.api.delete_post(post_id).send_native().await;

        match result {
            Ok(crate::api::ServerRes::Ok) => {
                self.post_state.set(PostState::Deleted);
                return Some(());
            }
            Ok(res) => {
                error!("wrong res, expected Post, got {:?}", res);
            }
            Err(ServerErr::NotFoundErr(Server404Err::NotFound)) => {
                self.post_state.set(PostState::NotFound);
            }
            Err(err) => {
                error!("unexpected err {:#?}", { err });
            }
        }

        None
    }

    pub async fn get(self, post_id: impl Into<String>) {
        let post_id = post_id.into();
        // let (Some(username), Some(post_id)) = (param_username(), param_post.get()) else {
        //     return;
        // };

        let result = self.api.get_post(post_id).send_native().await;
        match result {
            Ok(crate::api::ServerRes::Post(post)) => {
                self.title.set(post.title);
                self.author.set(post.user.username.clone());
                self.author_link.set(link_user(post.user.username));
                self.tags.set(post.tags);
                self.live_description_length.set(post.description.len());
                self.description.set(post.description);
                // if post.description.is_empty() {
                //     self.description.set("No description.".to_string());
                //     // self.description_is_empty.set(true);
                // } else {
                //     self.description.set(post.description);
                //     // self.description_is_empty.set(false);
                // }

                self.favorites.set(post.favorites);
                self.imgs_links.set(
                    post.file
                        .into_iter()
                        .map(|file| {
                            (
                                link_img(file.hash, file.extension),
                                file.width as f64 / file.height as f64,
                            )
                        })
                        .collect(),
                );
                self.post_state.set(PostState::Normal);
            }
            Ok(res) => {
                let err = format!("wrong res, expected Post, got {:?}", res);
                error!(err);
                self.err_general.set(err);
            }
            Err(ServerErr::NotFoundErr(Server404Err::NotFound)) => {
                self.post_state.set(PostState::NotFound);
                self.err_general.set(PostState::NotFound.to_string());
            }
            Err(err) => {
                let err = format!("unexpected err {:#?}", { err });
                error!(err);
                self.err_general.set(err);
            }
        }
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
                api_gallery::{GalleryApi, GalleryContainerSize, tests::create_img_req},
                api_post::PostApi,
                api_post_comments::{CommentKind, CommentKind2, CommentsApi, CommentsApi2},
                use_scroll_correction::ScrollCorrection,
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

    #[tokio::test]
    pub async fn hook_post_api_update_description() {
        println!("hello");
        init_test_log();
        let owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
        let mut app = ApiTestApp::new(10).await;
        let scroll_correction = ScrollCorrection::new();
        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();
        app.api.auth_token_overwrite = auth_token.clone();
        let post_key = {
            let gallery_api = GalleryApi::new(&app.api, &app.api, scroll_correction.clone());
            app.set_time(1).await;
            gallery_api
                .post(
                    GalleryContainerSize {
                        width: 100,
                        height: 100.0,
                        row_height: 50,
                    },
                    "title1",
                    "0",
                    "",
                    vec![create_img_req("1", 50, 50).await],
                )
                .await;
            gallery_api.items.get()[0].clone().key
        };

        // testing normal 
        let post_api = PostApi::new(&app.api);
        post_api.get(&post_key).await;
        assert_eq!(post_api.live_description_length.get(), 1);
        assert_eq!(post_api.description.get(), "0");
        post_api.update_description_mode.set(true);
        assert!(post_api.err_description.get().is_empty());

        post_api.update_description(&post_key, "22").await;
        assert_eq!(post_api.live_description_length.get(), 2);
        assert_eq!(post_api.description.get(), "22");
        assert_eq!(post_api.update_tags_mode.get(), false);
        assert!(post_api.err_description.get().is_empty());

        let post_api = PostApi::new(&app.api);
        post_api.get(&post_key).await;
        assert_eq!(post_api.live_description_length.get(), 2);
        assert_eq!(post_api.description.get(), "22");

        post_api.delete(&post_key).await;

        post_api.update_description(&post_key, "2").await;
        assert_eq!(post_api.live_description_length.get(), 2);
        assert!(!post_api.err_description.get().is_empty());


        // let items = gallery_api.items.get();
        // assert_eq!(items.len(), 1);
    }

    #[tokio::test]
    pub async fn hook_post_api_update_tags() {
        println!("hello");
        init_test_log();
        let owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
        let mut app = ApiTestApp::new(10).await;
        let scroll_correction = ScrollCorrection::new();
        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();
        app.api.auth_token_overwrite = auth_token.clone();
        let post_key = {
            let gallery_api = GalleryApi::new(&app.api, &app.api, scroll_correction.clone());
            app.set_time(1).await;
            gallery_api
                .post(
                    GalleryContainerSize {
                        width: 100,
                        height: 100.0,
                        row_height: 50,
                    },
                    "title1",
                    "0",
                    "",
                    vec![create_img_req("1", 50, 50).await],
                )
                .await;
            gallery_api.items.get()[0].clone().key
        };

        // testing err
        let post_api = PostApi::new(&app.api);
        post_api.get("invalid").await;
        assert!(!post_api.err_general.get().is_empty());
        assert_eq!(post_api.tags.get(), "");

        // testing normal 
        let post_api = PostApi::new(&app.api);
        post_api.get(&post_key).await;
        assert_eq!(post_api.tags.get(), "");
        post_api.update_tags_mode.set(true);
        assert!(post_api.err_tags.get().is_empty());

        post_api.update_tags(&post_key, "one").await;
        assert_eq!(post_api.tags.get(), "one");
        assert_eq!(post_api.update_tags_mode.get(), false);

        let post_api = PostApi::new(&app.api);
        post_api.get(&post_key).await;
        assert_eq!(post_api.tags.get(), "one");


        // let items = gallery_api.items.get();
        // assert_eq!(items.len(), 1);
    }

    #[tokio::test]
    pub async fn hook_post_api_delete() {
        println!("hello");
        init_test_log();
        let owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
        let mut app = ApiTestApp::new(10).await;

        let scroll_correction = ScrollCorrection::new();

        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        app.api.auth_token_overwrite = auth_token.clone();

        let gallery_api = GalleryApi::new(&app.api, &app.api, scroll_correction.clone());
        let size = GalleryContainerSize {
            width: 100,
            height: 100.0,
            row_height: 50,
        };

        app.set_time(1).await;
        gallery_api
            .post(
                size,
                "title1",
                "0",
                "",
                vec![create_img_req("1", 50, 50).await],
            )
            .await;
        let post0 = gallery_api.items.get()[0].clone();

        let post_all = app.state.db.get_post_all().await.unwrap();
        assert_eq!(post_all.len(), 1);

        let post_api = PostApi::new(&app.api);
        post_api.get(&post0.key).await;

        let result = post_api.delete(post0.key.clone()).await;
        assert!(result.is_some());

        let post_all = app.state.db.get_post_all().await.unwrap();
        assert_eq!(post_all.len(), 0);
    }

    #[tokio::test]
    pub async fn hook_post_api_post() {
        println!("hello");
        init_test_log();
        let owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));
        let mut app = ApiTestApp::new(10).await;

        let scroll_correction = ScrollCorrection::new();

        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        app.api.auth_token_overwrite = auth_token.clone();

        let gallery_api = GalleryApi::new(&app.api, &app.api, scroll_correction.clone());
        let size = GalleryContainerSize {
            width: 100,
            height: 100.0,
            row_height: 50,
        };
        app.set_time(1).await;
        gallery_api
            .post(
                size,
                "title1",
                "0",
                "",
                vec![create_img_req("1", 50, 50).await],
            )
            .await;
        let post0 = gallery_api.items.get()[0].clone();

        let post_api = PostApi::new(&app.api);
        post_api.get(&post0.key).await;
        assert_eq!(post_api.title.get_untracked(), "title1");

        //
    }
}
