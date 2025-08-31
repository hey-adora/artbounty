use std::time::Duration;

use chrono::{DateTime, Utc};


#[derive(
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    PartialEq,
)]
pub struct Post {
    pub hash: String,
    pub extension: String,
    pub width: u32,
    pub height: u32,
    pub created_at: u128,
}

pub mod route {
    pub mod get_after {
        use thiserror::Error;

        use crate::{
            controller::{
                encode::{ResErr, send_web},
                post::Post,
            },
            path::{PATH_API, PATH_API_POST_GET_AFTER},
        };

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
            PartialEq,
        )]
        pub struct Input {
            pub time: u128,
            pub limit: u32,
        }

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
            PartialEq,
        )]
        pub struct ServerOutput {
            pub posts: Vec<Post>,
        }

        pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
            send_web::<ServerOutput, ServerErr>(PATH_API_POST_GET_AFTER, &input).await
        }

        #[derive(
            Debug,
            Error,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
            PartialEq,
        )]
        pub enum ServerErr {
            #[error("internal server error")]
            ServerErr,
        }

        #[cfg(feature = "ssr")]
        pub async fn server(
            axum::extract::State(app_state): axum::extract::State<
                crate::controller::app_state::AppState,
            >,
            jar: axum_extra::extract::cookie::CookieJar,
            // username: Extension<String>,
            multipart: axum::extract::Multipart,
        ) -> impl axum::response::IntoResponse {
            use crate::controller::encode::encode_server_output;

            encode_server_output(
                (async || -> Result<ServerOutput, ResErr<ServerErr>> {
                    use std::time::Duration;

                    use tracing::trace;

                    use crate::controller::encode::decode_multipart;

                    let input = decode_multipart::<Input, ServerErr>(multipart).await?;
                    trace!("{input:?}");
                    // let time = app_state.clock.now().await;
                    // trace!("time");
                    // tokio::time::sleep(Duration::from_secs(2)).await;
                    let posts = app_state
                        .db
                        .get_post_after(input.time, input.limit)
                        .await
                        .map_err(|_| ServerErr::ServerErr)?
                        .into_iter()
                        .map(|post| {
                            use std::time::Duration;
                            use chrono::{Utc, DateTime};

                            // let a = Duration::from_nanos(0);
                            // let b = a.as_nanos();
                            // let created_at: DateTime<Utc> = post.created_at.to_string();

                            post.file
                                .first()
                                .cloned()
                                .map(|post_file| Post {
                                    hash: post_file.hash,
                                    extension: post_file.extension,
                                    width: post_file.width,
                                    height: post_file.height,
                                    created_at: post.created_at,
                                })
                                .unwrap_or(Post {
                                    hash: "404".to_string(),
                                    extension: "webp".to_string(),
                                    width: 300,
                                    height: 200,
                                    created_at: 0,
                                })
                        })
                        .collect::<Vec<Post>>();

                    Ok(ServerOutput { posts })
                })()
                .await,
            )
        }

        #[cfg(test)]
        pub async fn test_send(
            server: &axum_test::TestServer,
            time: u128,
        ) -> (http::HeaderMap, Result<ServerOutput, ResErr<ServerErr>>) {
            use tracing::trace;

            use crate::controller::encode::send_builder;

            let input = Input { time, limit: 100 };
            let path = format!("{}{}", PATH_API, PATH_API_POST_GET_AFTER);
            let builder = server.reqwest_post(&path);
            let res = send_builder::<ServerOutput, ServerErr>(builder, &input).await;
            trace!("RESPONSE: {res:#?}");
            res
        }

        #[cfg(test)]
        mod api {
            use std::path::Path;
            use std::sync::Arc;
            use std::time::Duration;
            use tokio::fs;

            use axum_test::TestServer;
            use gxhash::gxhash128;
            use pretty_assertions::assert_eq;
            use test_log::test;
            use tokio::sync::Mutex;
            use tracing::trace;

            use crate::controller;
            use crate::controller::app_state::AppState;
            use crate::controller::auth::test_extract_cookie;
            use crate::controller::clock::get_timestamp;
            use crate::server::create_api_router;

            #[test(tokio::test)]
            async fn post_get_after() {
                let current_time = Duration::from_nanos(1);
                let time = Arc::new(Mutex::new(current_time));
                let app_state = AppState::new_testng(time).await;
                let my_app = create_api_router().with_state(app_state.clone());

                let server = TestServer::builder()
                    .http_transport()
                    .build(my_app)
                    .unwrap();

                {
                    let time = app_state.clock.now().await;
                    // let exp = time + Duration::from_secs(60 * 30);

                    controller::auth::route::invite::test_send(&server, "hey1@hey.com")
                        .await
                        .1
                        .unwrap();
                    let invite = app_state
                        .db
                        .get_invite("hey1@hey.com", current_time.as_nanos())
                        .await
                        .unwrap();
                    let (cookies, res) = controller::auth::route::register::test_send(
                        &server,
                        "hey",
                        &invite.token_raw,
                        "wowowowow123@",
                    )
                    .await;
                    let token = test_extract_cookie(&cookies).unwrap();
                    let dir = std::env::current_dir()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    trace!("current working dir: {dir}");
                    // crate::auth::api::post::test_send(&server, [ format!("{}../assets/favicon.ico", dir) ], token)
                    let res = controller::post::route::add::test_send(
                        &server,
                        "hello",
                        "stuff",
                        ["/mnt/hdd2/pictures/me/EX6P5GmWsAE3uij.jpg"],
                        token,
                    )
                    .await
                    .1
                    .unwrap();
                    trace!("post api server ouput: {res:#?}");

                    let res2 = controller::post::route::get_after::test_send(&server, time.as_nanos())
                        .await
                        .1
                        .unwrap();

                    // res2.posts;
                    assert_eq!(res.posts, res2.posts);

                    let res2 = controller::post::route::get_after::test_send(&server, 0)
                        .await
                        .1
                        .unwrap();

                    let res2 = controller::post::route::get_after::test_send(&server, 2)
                        .await
                        .1
                        .unwrap();

                    // for img in res.posts {
                    //     let file_path = format!(
                    //         "{}/{}.{}",
                    //         &app_state.settings.site.files_path, img.hash, img.extension
                    //     );
                    //     let file_path = Path::new(&file_path);
                    //     let data = fs::read(file_path).await.unwrap();
                    //     let new_hash = format!("{:X}", gxhash128(&data, 0));
                    //     assert_eq!(img.hash, new_hash);
                    //     fs::remove_file(&file_path).await.unwrap();
                    // }
                }
            }
        }
    }
    pub mod add {
        use thiserror::Error;
        use tracing::{error, trace};

        use crate::{controller::{
            encode::{send_web, ResErr},
            post::Post,
        }, path::PATH_API_POST_ADD};

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
            PartialEq,
        )]
        pub struct Input {
            pub title: String,
            pub description: String,
            pub files: Vec<Vec<u8>>,
        }

        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
            PartialEq,
        )]
        pub struct ServerOutput {
            pub posts: Vec<Post>,
        }

        // #[derive(
        //     Debug,
        //     Clone,
        //     serde::Serialize,
        //     serde::Deserialize,
        //     rkyv::Archive,
        //     rkyv::Serialize,
        //     rkyv::Deserialize,
        //     PartialEq,
        // )]
        // pub struct ServerOutputImg {
        //     pub hash: String,
        //     pub extension: String,
        // }

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerOutput {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::encode_result;

                let bytes = encode_result::<ServerOutput, ServerErr>(&Ok(self));
                (axum::http::StatusCode::OK, bytes).into_response()
            }
        }

        #[derive(
            Debug,
            Error,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
            PartialEq,
        )]
        pub enum ServerErr {
            #[error("internal server error")]
            ServerErr,

            #[error("unauthorized")]
            Unauthorized,

            #[error("img errors")]
            ImgErrors(Vec<ServerErrImg>),

            #[error("failed to create output dir for imgs")]
            ImgFailedToCreateOutputDir(String),

            #[error("failed to save images metadata")]
            ImgFailedToSaveImgMeta,

            #[error("failed to save images to disk {0}")]
            ImgFailedToSaveImgToDisk(String),
        }

        #[derive(
            Debug,
            Error,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            rkyv::Archive,
            rkyv::Serialize,
            rkyv::Deserialize,
            PartialEq,
        )]
        pub enum ServerErrImg {
            #[error("unsupported format {0}")]
            UnsupportedFormat(String),

            #[error("failed to decode {0}")]
            FailedToDecode(String),

            #[error("failed to create webp encoder {0}")]
            FailedToCreateWebpEncoder(String),

            #[error("failed to encode webp {0}")]
            FailedToEncodeWebp(String),

            #[error("failed to read metadata {0}")]
            FailedToReadMetadata(String),
        }

        #[cfg(feature = "ssr")]
        impl axum::response::IntoResponse for ServerErr {
            fn into_response(self) -> axum::response::Response {
                use crate::controller::encode::encode_result;

                let status = match &self {
                    // ServerErr::NoCookie => axum::http::StatusCode::OK,
                    _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                };
                let bytes = encode_result::<ServerOutput, ServerErr>(&Err(ResErr::ServerErr(self)));
                (status, bytes).into_response()
            }
        }

        pub async fn client(input: Input) -> Result<ServerOutput, ResErr<ServerErr>> {
            send_web::<ServerOutput, ServerErr>(PATH_API_POST_ADD, &input).await
        }

        #[cfg(feature = "ssr")]
        pub async fn server(
            axum::extract::State(app_state): axum::extract::State<
                crate::controller::app_state::AppState,
            >,
            jar: axum_extra::extract::cookie::CookieJar,
            // username: Extension<String>,
            multipart: axum::extract::Multipart,
        ) -> impl axum::response::IntoResponse {
            // let username = &*username;
            trace!("executing post api");
            use std::{io::Cursor, path::Path, str::FromStr};

            use crate::{controller::encode::encode_server_output_custom, db::post::PostFile};
            use axum_extra::extract::cookie::Cookie;
            use gxhash::{gxhash64, gxhash128};
            use http::header::AUTHORIZATION;
            use image::{ImageFormat, ImageReader};
            use little_exif::{filetype::FileExtension, metadata::Metadata};
            use tokio::fs;

            let result = (async || -> Result<ServerOutput, ResErr<ServerErr>> {
                use crate::controller::{auth::check_auth, encode::decode_multipart};

                let auth_token = check_auth(&app_state, &jar).await?;

                let input = decode_multipart::<Input, ServerErr>(multipart).await?;
                let time = app_state.clock.now().await;
                // trace!("{input:?}");
                let files = input
                    .files
                    .into_iter()
                    .map(|v| {
                        let img_data_for_thumbnail = v.clone();
                        let img_data_for_org = v;
                        ImageReader::new(Cursor::new(img_data_for_thumbnail))
                            .with_guessed_format()
                            .inspect_err(|err| error!("error guesing the format {err}"))
                            .map_err(|err| ServerErrImg::UnsupportedFormat(err.to_string()))
                            .and_then(|v| {
                                let img_format = v.format().ok_or(
                                    ServerErrImg::UnsupportedFormat("uwknown".to_string()),
                                )?;
                                v.decode()
                                    .inspect_err(|err| error!("error decoding img {err}"))
                                    .map_err(|err| ServerErrImg::FailedToDecode(err.to_string()))
                                    .map(|img| (img_format, img))
                            })
                            .and_then(|(img_format, img)| {
                                let width = img.width();
                                let height = img.height();
                                webp::Encoder::from_image(&img)
                                    .inspect_err(|err| {
                                        error!("failed to create webp encoder {err}")
                                    })
                                    .map_err(|err| ServerErrImg::FailedToDecode(err.to_string()))
                                    .and_then(|encoder| {
                                        encoder
                                            .encode_simple(false, 90.0)
                                            .inspect_err(|err| {
                                                error!("failed to create webp encoder {err:?}")
                                            })
                                            .map_err(|err| {
                                                ServerErrImg::FailedToEncodeWebp(format!("{err:?}"))
                                            })
                                    })
                                    .map(|img| (img_format, (width, height), img))
                            })
                            .and_then(|(img_format, (width, height), img_data_thumbnail)| {
                                let img_format = img_format.extensions_str()[0];
                                let mut img_data_org = img_data_for_org;
                                FileExtension::from_str(img_format)
                                    .map_err(|_| {
                                        ServerErrImg::UnsupportedFormat(img_format.to_string())
                                    })
                                    .and_then(|img_format| {
                                        little_exif::metadata::Metadata::clear_metadata(
                                            &mut img_data_org,
                                            img_format,
                                        )
                                        .inspect_err(|err| {
                                            error!("failed to read metadata {err:?}")
                                        })
                                        .map_err(|err| {
                                            ServerErrImg::FailedToReadMetadata(err.to_string())
                                        })
                                    })
                                    .map(|_| {
                                        (
                                            PostFile {
                                                extension: img_format.to_string(),
                                                hash: format!("{:X}", gxhash128(&img_data_org, 0)),
                                                width,
                                                height,
                                            },
                                            img_data_org,
                                            img_data_thumbnail.to_vec(),
                                        )
                                    })
                            })
                    })
                    .fold(
                        (
                            Vec::<(PostFile, Vec<u8>, Vec<u8>)>::new(),
                            Vec::<ServerErrImg>::new(),
                        ),
                        |(mut oks, mut errs), file| {
                            match file {
                                Ok(v) => {
                                    oks.push(v);
                                }
                                Err(v) => {
                                    errs.push(v);
                                }
                            }

                            (oks, errs)
                        },
                    );
                if !files.1.is_empty() {
                    return Err(ResErr::from(ServerErr::ImgErrors(files.1)));
                }

                let files = files.0;
                let root_path = Path::new(&app_state.settings.site.files_path);
                let mut output_imgs = Vec::<Post>::new();
                for file in &files {
                    let file_path =
                        root_path.join(format!("{}.{}", &file.0.hash, &file.0.extension));
                    if file_path.exists() {
                        trace!(
                            "file already exists {}",
                            file_path.to_str().unwrap_or("err")
                        );
                        output_imgs.push(Post {
                            hash: file.0.hash.clone(),
                            extension: file.0.extension.clone(),
                            width: file.0.width,
                            height: file.0.height,
                            created_at: time.as_nanos(),
                        });
                        continue;
                    }

                    trace!("saving {}", file_path.to_str().unwrap_or("err"));
                    (match fs::write(&file_path, &file.1).await {
                        Ok(v) => Ok(v),
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                            fs::create_dir_all(&root_path)
                                .await
                                .inspect(|_| trace!("created img output dir {:?}", &file_path))
                                .inspect_err(|err| error!("error creating img output dir {err}"))
                                .map_err(|err| {
                                    ServerErr::ImgFailedToCreateOutputDir(err.to_string())
                                })?;
                            fs::write(&file_path, &file.1).await
                        }
                        Err(err) => {
                            //
                            Err(err)
                        }
                    })
                    .inspect_err(|err| error!("failed to save img to disk {err:?}"))
                    .map_err(|err| ServerErr::ImgFailedToSaveImgToDisk(err.to_string()))?;
                    output_imgs.push(Post {
                        hash: file.0.hash.clone(),
                        extension: file.0.extension.clone(),
                        width: file.0.width,
                        height: file.0.height,
                        created_at: time.as_nanos(),
                    });
                }

                let post_files = files.into_iter().map(|v| v.0).collect::<Vec<PostFile>>();
                app_state
                    .db
                    .add_post(
                        time.as_nanos(),
                        &auth_token.username,
                        &input.title,
                        &input.description,
                        post_files,
                    )
                    .await
                    .inspect_err(|err| error!("failed to save images {err:?}"))
                    .map_err(|_| ServerErr::ImgFailedToSaveImgMeta)?;

                Result::<ServerOutput, ResErr<ServerErr>>::Ok(ServerOutput { posts: output_imgs })
            })()
            .await;

            encode_server_output_custom(result)
        }

        #[cfg(test)]
        pub async fn test_send<
            Token: Into<String>,
            Files: AsRef<[File]>,
            File: AsRef<std::path::Path>,
        >(
            server: &axum_test::TestServer,
            title: impl Into<String>,
            description: impl Into<String>,
            file_paths: Files,
            token: Token,
        ) -> (http::HeaderMap, Result<ServerOutput, ResErr<ServerErr>>) {
            use std::{ffi::OsStr, path::Path};

            use tokio::fs;

            use crate::{controller::encode::send_builder, path::{PATH_API, PATH_API_POST_ADD}};

            let mut files = Vec::new();
            let paths = file_paths.as_ref();
            let title = title.into();
            let description = description.into();
            for path in paths {
                let path = path.as_ref();
                trace!("reading path: {path:?}");
                let data = fs::read(path).await.unwrap();
                files.push(data);
            }

            let input = Input {
                title,
                description,
                files,
            };
            let path = format!("{}{}", PATH_API, PATH_API_POST_ADD);
            let token: String = token.into();
            let builder = server.reqwest_post(&path).header(
                http::header::COOKIE,
                format!("authorization=Bearer%3D{}%3B%20Secure%3B%20HttpOnly", token),
            );
            let res = send_builder::<ServerOutput, ServerErr>(builder, &input).await;
            trace!("RESPONSE: {res:#?}");
            res
        }

        #[cfg(test)]
        mod api {
            use std::path::Path;
            use std::sync::Arc;
            use std::time::Duration;
            use tokio::fs;

            use axum_test::TestServer;
            use gxhash::gxhash128;
            use test_log::test;
            use tokio::sync::Mutex;
            use tracing::trace;

            use crate::controller;
            use crate::controller::app_state::AppState;
            use crate::controller::auth::test_extract_cookie;
            use crate::controller::clock::get_timestamp;
            use crate::server::create_api_router;

            #[test(tokio::test)]
            async fn post_add() {
                let current_time = get_timestamp();
                let time = Arc::new(Mutex::new(current_time));
                let app_state = AppState::new_testng(time).await;
                let my_app = create_api_router().with_state(app_state.clone());

                let server = TestServer::builder()
                    .http_transport()
                    .build(my_app)
                    .unwrap();

                {
                    let time = app_state.clock.now().await;
                    let exp = time + Duration::from_secs(60 * 30);

                    controller::auth::route::invite::test_send(&server, "hey1@hey.com")
                        .await
                        .1
                        .unwrap();
                    let invite = app_state
                        .db
                        .get_invite("hey1@hey.com", current_time.as_nanos())
                        .await
                        .unwrap();
                    let (cookies, res) = controller::auth::route::register::test_send(
                        &server,
                        "hey",
                        &invite.token_raw,
                        "wowowowow123@",
                    )
                    .await;
                    let token = test_extract_cookie(&cookies).unwrap();
                    let dir = std::env::current_dir()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    trace!("current working dir: {dir}");
                    // crate::auth::api::post::test_send(&server, [ format!("{}../assets/favicon.ico", dir) ], token)
                    let res = controller::post::route::add::test_send(
                        &server,
                        "hello",
                        "stuff",
                        ["/mnt/hdd2/pictures/me/EX6P5GmWsAE3uij.jpg"],
                        token,
                    )
                    .await
                    .1
                    .unwrap();
                    trace!("post api server ouput: {res:#?}");

                    for img in res.posts {
                        let file_path = format!(
                            "{}/{}.{}",
                            &app_state.settings.site.files_path, img.hash, img.extension
                        );
                        let file_path = Path::new(&file_path);
                        let data = fs::read(file_path).await.unwrap();
                        let new_hash = format!("{:X}", gxhash128(&data, 0));
                        assert_eq!(img.hash, new_hash);
                        fs::remove_file(&file_path).await.unwrap();
                    }
                }
            }
        }
    }
}
