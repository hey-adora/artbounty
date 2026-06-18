use std::error::Error;
use std::ffi::OsStr;
use std::hash::{DefaultHasher, Hasher};
use std::io::Cursor;
use std::ops::{FromResidual, Try};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::str::FromStr;

use crate::api::app_state::AppState;
use crate::api::shared::post_comment::{PostCommentErrResolver, UserPostComment};
use crate::api::{
    AuthToken, ChangeUsernameErr, EmailChangeErr, EmailChangeNewErr, EmailChangeStage,
    EmailChangeTokenErr, Server404Err, ServerAddPostErr, ServerAddPostFileErr, ServerAuthErr,
    ServerDecodeInviteErr, ServerDesErr, ServerErr, ServerErrImg, ServerErrImgMeta, ServerLoginErr,
    ServerRegistrationErr, ServerReq, ServerRes, ServerTokenErr, ServerUpdatePostDescriptionErr,
    ServerUpdatePostTagsErr, ServerUpdatePostTitleErr, User, UserPost, UserPostFile,
    auth_token_get, hash_password, verify_password,
};
use crate::db::{AddUserErr, DBPostCommentErr, DBUser};
use crate::db::{DB404Err, DBUserPostFile};
use crate::valid::SUPPORTED_FILE_EXTENSIONS;
use crate::valid::auth::{
    proccess_password, proccess_post_description, proccess_post_tags, proccess_post_title,
    proccess_username,
};
use anyhow::anyhow;
use axum::body::Body;
use axum::extract::{Multipart, State};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use bytes::Bytes;
use futures::{Stream, TryStreamExt};
use futures_util::StreamExt;
use gxhash::gxhash128;
// use axum_extra::extract::CookieJar;
use http::header::COOKIE;
use image::ImageReader;
use little_exif::filetype::FileExtension;
use std::{io, pin::pin};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use tracing::{debug, error, info, trace};

// use tokio_util::io::StreamReader;

pub async fn get_posts(
    State(app_state): State<AppState>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    let ServerReq::GetPosts2 {
        time,
        order,
        limit,
        tags,
        username,
    } = req
    else {
        return Err(
            ServerDesErr::ServerWrongInput(format!("expected GetPost, received: {req:?}")).into(),
        );
    };
    let post = app_state
        .db
        .post_search(limit as usize, time, order, tags, username)
        .await
        .map_err(|_| ServerErr::DbErr)?
        .into_iter()
        .map(UserPost::from)
        .collect::<Vec<UserPost>>();
    // .map_err(|err| match err {
    //     DB404Err::NotFound => ServerErr::NotFoundErr(Server404Err::NotFound),
    //     _ => ServerErr::DbErr,
    // })?;

    // Ok(ServerRes::Ok)
    Ok(ServerRes::Posts(post))
}

pub async fn get_post(
    State(app_state): State<AppState>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    let ServerReq::PostId { post_key: post_id } = req else {
        return Err(
            ServerDesErr::ServerWrongInput(format!("expected GetPost, received: {req:?}")).into(),
        );
    };
    let post = app_state
        .db
        .get_post(post_id)
        .await
        .map_err(|err| match err {
            DB404Err::NotFound => ServerErr::NotFoundErr(Server404Err::NotFound),
            _ => ServerErr::DbErr,
        })?;

    Ok(ServerRes::Post(post.into()))
}

pub async fn get_posts_newer_or_equal_for_user(
    State(app_state): State<AppState>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    let ServerReq::GetUserPosts {
        time,
        limit,
        username,
    } = req
    else {
        return Err(ServerDesErr::ServerWrongInput(format!(
            "expected GetPostAfter, received: {req:?}"
        ))
        .into());
    };

    let user = app_state
        .db
        .get_user_by_username(username)
        .await
        .map_err(|err| match err {
            DB404Err::NotFound => Server404Err::NotFound.into(),
            DB404Err::DB(_) => ServerErr::DbErr,
        })?;

    let posts = app_state
        .db
        .get_post_newer_or_equal_for_user(time, limit, user.id.clone())
        .await
        .map_err(|_| ServerErr::DbErr)?
        .into_iter()
        .map(UserPost::from)
        .collect::<Vec<UserPost>>();

    Ok(ServerRes::Posts(posts))
}

pub async fn get_posts_older_or_equal_for_user(
    State(app_state): State<AppState>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    let ServerReq::GetUserPosts {
        time,
        limit,
        username,
    } = req
    else {
        return Err(ServerDesErr::ServerWrongInput(format!(
            "expected GetPostAfter, received: {req:?}"
        ))
        .into());
    };

    let user = app_state
        .db
        .get_user_by_username(username)
        .await
        .map_err(|err| match err {
            DB404Err::NotFound => Server404Err::NotFound.into(),
            DB404Err::DB(_) => ServerErr::DbErr,
        })?;

    let posts = app_state
        .db
        .get_post_older_or_equal_for_user(time, limit, user.id.clone())
        .await
        .map_err(|_| ServerErr::DbErr)?
        .into_iter()
        .map(UserPost::from)
        .collect::<Vec<UserPost>>();

    Ok(ServerRes::Posts(posts))
}

pub async fn get_posts_older_for_user(
    State(app_state): State<AppState>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    let ServerReq::GetUserPosts {
        time,
        limit,
        username,
    } = req
    else {
        return Err(ServerDesErr::ServerWrongInput(format!(
            "expected GetPostAfter, received: {req:?}"
        ))
        .into());
    };

    let user = app_state
        .db
        .get_user_by_username(username)
        .await
        .map_err(|err| match err {
            DB404Err::NotFound => Server404Err::NotFound.into(),
            DB404Err::DB(_) => ServerErr::DbErr,
        })?;

    let posts = app_state
        .db
        .get_post_older_for_user(time, limit, user.id.clone())
        .await
        .map_err(|_| ServerErr::DbErr)?
        .into_iter()
        .map(UserPost::from)
        .collect::<Vec<UserPost>>();

    Ok(ServerRes::Posts(posts))
}

pub async fn get_posts_newer_for_user(
    State(app_state): State<AppState>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    let ServerReq::GetUserPosts {
        time,
        limit,
        username,
    } = req
    else {
        return Err(ServerDesErr::ServerWrongInput(format!(
            "expected GetPostAfter, received: {req:?}"
        ))
        .into());
    };

    let user = app_state
        .db
        .get_user_by_username(username)
        .await
        .map_err(|err| match err {
            DB404Err::NotFound => Server404Err::NotFound.into(),
            DB404Err::DB(_) => ServerErr::DbErr,
        })?;

    let posts = app_state
        .db
        .get_post_newer_for_user(time, limit, user.id.clone())
        .await
        .map_err(|_| ServerErr::DbErr)?
        .into_iter()
        .map(UserPost::from)
        .collect::<Vec<UserPost>>();

    Ok(ServerRes::Posts(posts))
}

pub async fn get_posts_newer_or_equal(
    State(app_state): State<AppState>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    let ServerReq::GetPosts { time, limit } = req else {
        return Err(ServerDesErr::ServerWrongInput(format!(
            "expected GetPostAfter, received: {req:?}"
        ))
        .into());
    };
    let posts = app_state
        .db
        .get_post_newer_or_equal(time, limit)
        .await
        .map_err(|_| ServerErr::DbErr)?
        .into_iter()
        .map(UserPost::from)
        .collect::<Vec<UserPost>>();

    Ok(ServerRes::Posts(posts))
}

pub async fn get_posts_older_or_equal(
    State(app_state): State<AppState>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    let ServerReq::GetPosts { time, limit } = req else {
        return Err(ServerDesErr::ServerWrongInput(format!(
            "expected GetPostAfter, received: {req:?}"
        ))
        .into());
    };

    let posts = app_state
        .db
        .get_post_older_or_equal(time, limit)
        .await
        .map_err(|_| ServerErr::DbErr)?
        .into_iter()
        .map(UserPost::from)
        .collect::<Vec<UserPost>>();

    Ok(ServerRes::Posts(posts))
}

pub async fn get_posts_newer(
    State(app_state): State<AppState>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    let ServerReq::GetPosts { time, limit } = req else {
        return Err(ServerDesErr::ServerWrongInput(format!(
            "expected GetPostAfter, received: {req:?}"
        ))
        .into());
    };
    let posts = app_state
        .db
        .get_post_newer(time, limit)
        .await
        .map_err(|_| ServerErr::DbErr)?
        .into_iter()
        .map(UserPost::from)
        .collect::<Vec<UserPost>>();

    Ok(ServerRes::Posts(posts))
}

pub async fn get_posts_older(
    State(app_state): State<AppState>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    let ServerReq::GetPosts { time, limit } = req else {
        return Err(ServerDesErr::ServerWrongInput(format!(
            "expected GetPostAfter, received: {req:?}"
        ))
        .into());
    };

    let posts = app_state
        .db
        .get_post_older(time, limit)
        .await
        .map_err(|_| ServerErr::DbErr)?
        .into_iter()
        .map(UserPost::from)
        .collect::<Vec<UserPost>>();

    Ok(ServerRes::Posts(posts))
}

pub async fn update_post_title(
    State(app): State<AppState>,
    // auth_token: axum::Extension<AuthToken>,
    db_user: Extension<DBUser>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    type ResErr = ServerUpdatePostTitleErr;

    let ServerReq::EditPostTitle {
        post_key,
        new_title,
    } = req
    else {
        return Err(ServerDesErr::ServerWrongInput(format!(
            "expected EditPostDescription, received: {req:?}"
        ))
        .into());
    };

    let new_title = new_title.trim();
    proccess_post_title(new_title).map_err(|err| ResErr::TooLong)?;

    let post = app
        .db
        .update_post_title(0, db_user.id.clone(), post_key, new_title)
        .await
        .map_err(|err| match err {
            DB404Err::NotFound => ResErr::NotFound.into(),
            _ => ServerErr::DbErr,
        })?;
    //

    Ok(ServerRes::Post(post.into()))
}

pub async fn update_post_description(
    State(app): State<AppState>,
    // auth_token: axum::Extension<AuthToken>,
    db_user: Extension<DBUser>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    type ResErr = ServerUpdatePostDescriptionErr;

    let ServerReq::EditPostDescription {
        post_key,
        new_description,
    } = req
    else {
        return Err(ServerDesErr::ServerWrongInput(format!(
            "expected EditPostDescription, received: {req:?}"
        ))
        .into());
    };

    let new_description = new_description.trim();
    proccess_post_description(new_description).map_err(|err| ResErr::TooLong)?;

    let post = app
        .db
        .update_post_description(0, db_user.id.clone(), post_key, new_description)
        .await
        .map_err(|err| match err {
            DB404Err::NotFound => ResErr::NotFound.into(),
            _ => ServerErr::DbErr,
        })?;
    //

    Ok(ServerRes::Post(post.into()))
}

pub async fn update_post_tags(
    State(app): State<AppState>,
    // auth_token: axum::Extension<AuthToken>,
    db_user: Extension<DBUser>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    type ResErr = ServerUpdatePostTagsErr;

    let ServerReq::EditPostTags { post_key, new_tags } = req else {
        return Err(
            ServerDesErr::ServerWrongInput(format!("expected PostId, received: {req:?}")).into(),
        );
    };

    let new_tags = new_tags.trim();
    proccess_post_tags(new_tags).map_err(|err| ResErr::TooLong)?;
    // app.db
    //     .delete_post(db_user.id.clone(), post_key)
    //     .await
    //     .map_err(|_| ServerErr::DbErr)?;

    let post = app
        .db
        .update_post_tags(0, db_user.id.clone(), post_key, new_tags)
        .await
        .map_err(|err| match err {
            DB404Err::NotFound => ResErr::NotFound.into(),
            _ => ServerErr::DbErr,
        })?;
    //

    Ok(ServerRes::Post(post.into()))
}
pub async fn delete_post(
    State(app): State<AppState>,
    auth_token: axum::Extension<AuthToken>,
    db_user: Extension<DBUser>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    let ServerReq::PostId { post_key } = req else {
        return Err(
            ServerDesErr::ServerWrongInput(format!("expected PostId, received: {req:?}")).into(),
        );
    };

    app.db
        .delete_post(db_user.id.clone(), post_key)
        .await
        .map_err(|_| ServerErr::DbErr)?;
    //

    Ok(ServerRes::Ok)
}
// pub async fn test_upload_big_file(mut multipart: Multipart) -> impl IntoResponse {
//     while let Ok(Some(field)) = multipart.next_field().await {
//         trace!("field {field:#?}");
//     }
//     // let mut stream = body.into_data_stream();
//     // while let Some(value) = stream.next().await {
//     //     trace!("wtf: {value:#?}");
//     // }
//     // trace!("wtf {body:#?}");
//
//     "done"
// }
pub struct SavedFile {
    pub hash: String,
    pub saved_path: PathBuf,
    pub size_bytes: usize,
}

#[derive(thiserror::Error, Debug)]
pub enum SaveFileErr {
    #[error("max file size {max_bytes} bytes, upload stopped at {got_bytes} bytes")]
    FileTooBig { max_bytes: usize, got_bytes: usize },

    #[error("io err {0}")]
    IoErr(#[from] std::io::Error),

    #[error(transparent)]
    StreamErr(#[from] anyhow::Error),
}

pub async fn handle_file_saving<S, StreamErr>(
    mut stream: S,
    extension: impl AsRef<str>,
    save_path: impl AsRef<str>,
    max_storage_per_file: usize,
    // used_storage: usize,
    // max_storage: usize,
) -> Result<SavedFile, SaveFileErr>
where
    S: StreamExt + Stream<Item = Result<Bytes, StreamErr>> + Unpin,
    StreamErr: Sync + Send,
    SaveFileErr: From<StreamErr>, // S::Item: Error + Try,
{
    use rand::distr::SampleString;
    let tmp_name = rand::distr::Alphanumeric.sample_string(&mut rand::rng(), 16);
    let file_path_tmp = Path::new("/tmp/").join(&tmp_name).with_extension(".part");
    // extension.as_ref()
    let file = File::create(&file_path_tmp).await?;
    let mut file = BufWriter::new(file);

    let mut hasher = DefaultHasher::new();
    let mut size = 0_usize;

    while let Some(value) = stream.next().await {
        let bytes = value?;
        size += bytes.len();
        if size > max_storage_per_file {
            file.flush().await?;
            drop(file);
            tokio::fs::remove_file(file_path_tmp).await?;
            return Err(SaveFileErr::FileTooBig {
                max_bytes: max_storage_per_file,
                got_bytes: size,
            });
        }
        hasher.write(&bytes);
        file.write(&bytes).await?;
    }

    file.flush().await?;
    let hash = hasher.finish().to_string();

    let file_path = {
        let file_path = Path::new(save_path.as_ref())
            .join(&hash)
            .with_extension(extension.as_ref());
        if file_path.exists() {
            trace!("file removed");
            tokio::fs::remove_file(file_path_tmp).await?;
        } else {
            trace!("file moved");
            tokio::fs::rename(&file_path_tmp, &file_path)
                .await
                .inspect_err(|err| {
                    error!(
                        "move err from {file_path_tmp:?} to {}/{} {err}",
                        std::env::current_dir().unwrap().into_string().unwrap(),
                        file_path.clone().into_string().unwrap(),
                    )
                })?;
        }
        file_path
    };

    Ok(SavedFile {
        hash,
        size_bytes: size,
        saved_path: file_path,
    })
}

pub async fn get_img_resolution(img_path: impl AsRef<str>) -> anyhow::Result<(u32, u32)> {
    let mut command = tokio::process::Command::new("ffprobe");
    let command = command.args(&[
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=width,height",
        "-of",
        "csv=s=x:p=0",
        img_path.as_ref(),
    ]);
    let result = command.output().await?;

    let result = String::from_utf8(result.stdout)?;
    let result = result.trim();
    trace!("command output {result}");

    resolution_from_str(result)
}

//params: RawPathParams,
// pub async fn post_file_add(body: Body) -> impl IntoResponse {
// ) -> Result<ServerRes, ServerErr> {
pub async fn post_file_add(
    State(app): State<AppState>,
    params: axum::extract::RawPathParams,
    db_user: Extension<DBUser>,
    // body: Body,
    mut multipart: Multipart,
) -> impl IntoResponse {
    type Err = ServerAddPostFileErr;

    let max_storage = db_user.max_storage_bytes;
    let max_storage_per_file = db_user.max_storage_per_file_bytes;
    let mut used_storage = db_user.used_storage_bytes;

    let mut inner = async || -> Result<ServerRes, ServerErr> {
        let time = app.clock.now().await;

        let post_key = params
            .iter()
            .find(|(name, _)| *name == "post_id")
            .ok_or(ServerErr::from(Err::ParamNotFoundPostId))
            .map(|(_, value)| value)?;

        trace!("PATH PARAMS {post_key:?}");

        while let Ok(Some(field)) = multipart.next_field().await {
            let file_name = if let Some(file_name) = field.file_name() {
                file_name.to_owned()
            } else {
                continue;
            };

            let Some(extension) = Path::new(&file_name).extension().and_then(|v| v.to_str()) else {
                return Err(ServerErr::from(Err::FileHasNoExtension(
                    file_name.to_string(),
                )));
            };
            let is_supported = SUPPORTED_FILE_EXTENSIONS
                .into_iter()
                .any(|v| *v == extension);
            if !is_supported {
                return Err(ServerErr::from(Err::UnsupportedExtension(
                    extension.to_string(),
                )));
            }

            let storage_left = max_storage.saturating_sub(used_storage);
            let storage_per_file = if storage_left < max_storage_per_file {
                storage_left
            } else {
                max_storage_per_file
            };

            if max_storage_per_file == 0 {
                return Err(ServerErr::from(Err::MaxUserStorageReched {
                    max: max_storage,
                    used: used_storage,
                }));
            }

            let file_path = app.get_file_path().await;
            let stream = field.map_err(io::Error::other);
            let file = handle_file_saving(stream, extension, file_path, storage_per_file)
                .await
                .map_err(|err| match err {
                    SaveFileErr::FileTooBig {
                        got_bytes,
                        max_bytes,
                    } => ServerErr::from(Err::FileTooBig {
                        file_name: file_name.to_string(),
                        max: max_bytes,
                        got: got_bytes,
                    }),
                    SaveFileErr::IoErr(err) => ServerErr::from(Err::IoErr(err.to_string())),
                    SaveFileErr::StreamErr(err) => ServerErr::from(Err::StreamErr(err.to_string())),
                })?;

            let (width, height) = get_img_resolution(file.saved_path.to_str().unwrap())
                .await
                .map_err(|err| ServerErr::from(Err::ReadingResolutionErr(err.to_string())))?;

            let post = app
                .db
                .update_post_file(
                    time,
                    db_user.id.clone(),
                    post_key,
                    file.size_bytes,
                    file.hash,
                    extension,
                    width,
                    height,
                )
                .await
                .map_err(|v| ServerErr::DbErr)?;

            used_storage = post.user.used_storage_bytes;

            // post.user.used_storage_bytes >

            // trace!("{tmp_name}");
        }

        let post = app.db.get_post(post_key).await.map_err(|err| match err {
            DB404Err::NotFound => Err::NotFound.into(),
            DB404Err::DB(_) => ServerErr::DbErr,
        })?;

        Ok(ServerRes::Post(post.into()))
    };
    let result = inner().await;
    Json(result)
    // Ok(ServerRes::Ok)
    // "done"
}

pub fn resolution_from_str(res: impl AsRef<str>) -> anyhow::Result<(u32, u32)> {
    let res = res.as_ref();
    // let width = ['0'; 11];
    // let height = ['0'; 11];
    // let mut index: usize = 0;
    let x_pos = res
        .chars()
        .position(|v| v == 'x')
        .ok_or_else(|| anyhow!("x was not found, example input: 10x10, received: {res}"))?;
    if res.len() <= x_pos + 1 {
        return Err(anyhow!(
            "invalid input, example input: 10x10, received: {res}"
        ));
    }
    let input = &res[..x_pos];
    let width = u32::from_str(input).map_err(|v| anyhow!("input \"{input}\" err: {v}"))?;
    let input = &res[x_pos + 1..];
    let height = u32::from_str(input).map_err(|v| anyhow!("input \"{input}\" err: {v}"))?;

    Ok((width, height))
    // for c in res.chars() {
    //     if c >= '0' {
    //         width
    //     }
    // }
}
// pub async fn post_file_add(
//     State(app): State<AppState>,
//     params: axum::extract::RawPathParams,
//     db_user: Extension<DBUser>,
//     body: Body,
// ) -> Result<ServerRes, ServerErr> {
//     type Err = ServerAddPostFileErr;
//
//     let time = app.clock.now().await;
//     let file_name = app.gen_key().await;
//
//     let post_key = params
//         .iter()
//         .find(|(name, _)| *name == "post_id")
//         .ok_or(Err::ParamNotFoundPostId)
//         .map(|(_, value)| value)?;
//
//     // post_key.
//     let file_path = app.get_file_path().await;
//     let file_path = Path::new(&file_path).join(file_name).with_extension("part");
//
//     trace!("file_path {file_path:?}");
//     trace!("PATH PARAMS {post_key:?}");
//
//     let mut stream = body.into_data_stream();
//     // let body_with_io_error = stream.map_err(io::Error::other);
//     // let mut body_reader = pin!(StreamReader::new(body_with_io_error));
//
//     trace!("hello from streaming");
//
//     // let path = std::path::Path::new("/home/hey/github/artbounty/target/tmp/here.part");
//     let file = File::create(file_path)
//         .await
//         .map_err(|err| Err::IoErr(err.to_string()))?;
//     let mut file = BufWriter::new(file);
//     let mut hasher = DefaultHasher::new();
//     let mut size = 0_u32;
//
//     while let Some(value) = stream.next().await {
//         let bytes = value.map_err(|v| Err::IoErr(v.to_string()))?;
//         size += bytes.len() as u32;
//         hasher.write(&bytes);
//         file.write(&bytes)
//             .await
//             .map_err(|v| Err::IoErr(v.to_string()))?;
//         // trace!("wtf: {value:#?}");
//     }
//     file.flush().await.map_err(|v| Err::IoErr(v.to_string()))?;
//     let hash = hasher.finish().to_string();
//
//     // let hash = "what".to_string();
//
//     // gxhash128(&img_data_org, 0);
//     // file_name
//
//     let post = app
//         .db
//         .update_post_file(
//             time,
//             db_user.id.clone(),
//             post_key,
//             size,
//             hash,
//             "jpg",
//             0,
//             0,
//         )
//         .await
//         .map_err(|v| ServerErr::DbErr)?;
//
//     // tokio::io::copy(&mut body_reader, &mut file).await.unwrap();
//
//     // let post = app
//     //     .db
//     //     .add_post_file(
//     //         time,
//     //         db_user.id.clone(),
//     //         post_key,
//     //         file_hash: impl Into<String>,
//     //         file_extension: impl Into<String>,
//     //         file_width: u32,
//     //         file_height: u32,
//     //
//     //     );
//
//     // let post = app
//     //     .db
//     //     .add_post(
//     //         time,
//     //         &db_user.username,
//     //         &title,
//     //         &description,
//     //         tags,
//     //         0,
//     //         // post_files,
//     //     )
//     //     .await
//     //     .inspect_err(|err| error!("failed to save images {err:?}"))
//     //     .map_err(|_| ServerErr::DbErr)?;
//
//     // while let Some(value) = stream.next().await {
//     //     trace!("wtf: {value:#?}");
//     // }
//     // trace!("wtf {body:#?}");
//
//     Ok(ServerRes::Post(post.into()))
//     // Ok(ServerRes::Ok)
//     // "done"
// }
// pub async fn test_upload_big_file(mut multipart: Multipart) -> impl IntoResponse {
//     trace!("wtf");
//     const UPLOADS_DIRECTORY: &'static str = "/home/hey/github/artbounty/target/tmp/";
//
//     while let Ok(Some(field)) = multipart.next_field().await {
//         let file_name = if let Some(file_name) = field.file_name() {
//             file_name.to_owned()
//         } else {
//             continue;
//         };
//
//         let body_with_io_error = field.map_err(io::Error::other);
//         let mut body_reader = pin!(StreamReader::new(body_with_io_error));
//
//         let path = std::path::Path::new(UPLOADS_DIRECTORY).join(&file_name);
//         let mut file = BufWriter::new(File::create(path).await.unwrap());
//         tokio::io::copy(&mut body_reader, &mut file).await.unwrap();
//
//         trace!("{file_name}");
//     }
//
//     "done"
// }
//
pub async fn add_post(
    State(app): State<AppState>,
    auth_token: axum::Extension<AuthToken>,
    db_user: Extension<DBUser>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    type Err = ServerAddPostErr;
    let ServerReq::AddPost {
        title,
        description,
        tags,
        // files,
    } = req
    else {
        return Err(
            ServerDesErr::ServerWrongInput(format!("expected AddPost, received: {req:?}")).into(),
        );
    };
    let time = app.clock.now().await;

    // let file_path = app.get_file_path().await;

    let tags = tags.trim();
    let title = title.trim();
    let description = description.trim();

    proccess_post_tags(tags).map_err(|err| Err::InvalidTags(err.to_string()))?;
    proccess_post_title(title).map_err(|err| Err::InvalidTitle(err.to_string()))?;
    proccess_post_description(description)
        .map_err(|err| Err::InvalidDescription(err.to_string()))?;

    // let (files, errs) = files
    //     .into_iter()
    //     .map(|v| {
    //         let path = v.path;
    //         let img_data_for_thumbnail = v.data.clone();
    //         let img_data_for_org = v.data;
    //         ImageReader::new(Cursor::new(img_data_for_thumbnail))
    //             .with_guessed_format()
    //             .inspect_err(|err| error!("error guesing the format {err}"))
    //             .map_err(|err| ServerErrImg::ServerImgUnsupportedFormat(err.to_string()))
    //             .and_then(|v| {
    //                 let img_format = v.format().ok_or(ServerErrImg::ServerImgUnsupportedFormat(
    //                     "uwknown".to_string(),
    //                 ))?;
    //                 v.decode()
    //                     .inspect_err(|err| error!("error decoding img {err}"))
    //                     .map_err(|err| ServerErrImg::ServerImgDecodeFailed(err.to_string()))
    //                     .map(|img| (img_format, img))
    //             })
    //             .and_then(|(img_format, img)| {
    //                 let width = img.width();
    //                 let height = img.height();
    //                 webp::Encoder::from_image(&img)
    //                     .inspect_err(|err| error!("failed to create webp encoder {err}"))
    //                     .map_err(|err| {
    //                         ServerErrImg::ServerImgWebPEncoderCreationFailed(err.to_string())
    //                     })
    //                     .and_then(|encoder| {
    //                         encoder
    //                             .encode_simple(false, 90.0)
    //                             .inspect_err(|err| error!("failed to create webp encoder {err:?}"))
    //                             .map_err(|err| {
    //                                 ServerErrImg::ServerImgWebPEncodingFailed(format!("{err:?}"))
    //                             })
    //                     })
    //                     .map(|img| (img_format, (width, height), img))
    //             })
    //             .and_then(|(img_format, (width, height), img_data_thumbnail)| {
    //                 let img_format = img_format.extensions_str()[0];
    //                 let mut img_data_org = img_data_for_org;
    //                 FileExtension::from_str(img_format)
    //                     .map_err(|_| {
    //                         ServerErrImg::ServerImgUnsupportedFormat(img_format.to_string())
    //                     })
    //                     .and_then(|img_format| {
    //                         little_exif::metadata::Metadata::clear_metadata(
    //                             &mut img_data_org,
    //                             img_format,
    //                         )
    //                         .inspect_err(|err| error!("failed to read metadata {err:?}"))
    //                         .map_err(|err| ServerErrImg::ServerImgMetadataReadFail(err.to_string()))
    //                     })
    //                     .map(|_| {
    //                         (
    //                             DBUserPostFile {
    //                                 extension: img_format.to_string(),
    //                                 hash: format!("{:X}", gxhash128(&img_data_org, 0)),
    //                                 width,
    //                                 height,
    //                             },
    //                             img_data_org,
    //                             img_data_thumbnail.to_vec(),
    //                         )
    //                     })
    //             })
    //             .map_err(|err| ServerErrImgMeta { path, err })
    //     })
    //     .fold(
    //         (
    //             Vec::<(DBUserPostFile, Vec<u8>, Vec<u8>)>::new(),
    //             Vec::<ServerErrImgMeta>::new(),
    //         ),
    //         |(mut oks, mut errs), file| {
    //             match file {
    //                 Ok(v) => {
    //                     oks.push(v);
    //                 }
    //                 Err(v) => {
    //                     errs.push(v);
    //                 }
    //             }
    //
    //             (oks, errs)
    //         },
    //     );
    // if !errs.is_empty() {
    //     return Err(ServerAddPostErr::ServerImgErr(errs).into());
    // }

    // let root_path = Path::new(&file_path);
    // let mut output_imgs = Vec::<UserPostFile>::new();
    // for file in &files {
    //     let file_path = root_path.join(format!("{}.{}", &file.0.hash, &file.0.extension));
    //     if file_path.exists() {
    //         trace!(
    //             "file already exists {}",
    //             file_path.to_str().unwrap_or("err")
    //         );
    //         output_imgs.push(file.0.clone().into());
    //         continue;
    //     }
    //
    //     trace!("saving {}", file_path.to_str().unwrap_or("err"));
    //     (match fs::write(&file_path, &file.1).await {
    //         Ok(v) => Ok(v),
    //         Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
    //             fs::create_dir_all(&root_path)
    //                 .await
    //                 .inspect(|_| trace!("created img output dir {:?}", &file_path))
    //                 .inspect_err(|err| error!("error creating img output dir {err}"))
    //                 .map_err(|err| ServerAddPostErr::ServerDirCreationFailed(err.to_string()))?;
    //             fs::write(&file_path, &file.1).await
    //         }
    //         Err(err) => {
    //             //
    //             Err(err)
    //         }
    //     })
    //     .inspect_err(|err| error!("failed to save img to disk {err:?}"))
    //     .map_err(|err| ServerAddPostErr::ServerFSErr(err.to_string()))?;
    //     output_imgs.push(file.0.clone().into());
    // }

    // let post_files = files
    //     .into_iter()
    //     .map(|v| v.0)
    //     .collect::<Vec<DBUserPostFile>>();
    let post = app
        .db
        .add_post(
            time,
            &db_user.username,
            title,
            description,
            tags,
            0,
            // post_files,
        )
        .await
        .inspect_err(|err| error!("failed to save images {err:?}"))
        .map_err(|_| ServerErr::DbErr)?;

    Ok(ServerRes::Post(post.into()))
}
#[cfg(test)]
mod tests {
    use async_stream::stream;
    use axum::Router;
    use bytes::Bytes;
    use futures::StreamExt;
    use std::hash::{DefaultHasher, Hasher};
    use std::path::Path;
    use std::pin::pin;
    use std::sync::Arc;
    use std::time::Duration;
    use surrealdb::types::{RecordId, ToSql};
    use tokio::fs::{self, create_dir_all};

    use axum_test::TestServer;
    use gxhash::gxhash128;
    use tokio::sync::Mutex;
    use tracing::{debug, error, trace};

    use crate::api::app_state::AppState;
    use crate::api::backend::post::{SaveFileErr, handle_file_saving, resolution_from_str};
    use crate::api::shared::post_comment::UserPostComment;
    use crate::api::tests::ApiTestApp;
    use crate::api::{
        Api, ApiTest, EmailChangeErr, EmailChangeNewErr, EmailChangeStage, EmailChangeTokenErr,
        Order, PostLikeErr, Server404Err, ServerAddPostFileErr, ServerAuthErr, ServerErr,
        ServerLoginErr, ServerRegistrationErr, ServerReqImg, ServerRes, ServerSendInviteErr,
        TimeRange, UserPost,
    };
    use crate::db::DB404Err;
    use crate::db::email_change::create_email_change_id;
    use crate::db::post_comment::DBPostComment;
    use crate::db::{DBEmailIsTakenErr, DBUser, email_change::DBEmailChange};
    use crate::server::create_api_router;

    #[tokio::test]
    async fn api_post_get_test() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;
        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        app.add_post(0, &auth_token, "title1", "cat", "one two three")
            .await
            .unwrap();
        app.add_post(1, &auth_token, "title2", "cat", "one two")
            .await
            .unwrap();
        app.add_post(2, &auth_token, "title3", "cat", "one")
            .await
            .unwrap();

        let posts = app
            .get_posts(
                3,
                auth_token,
                3,
                TimeRange::Less(3),
                Order::ThreeTwoOne,
                "one",
                "hey",
            )
            .await
            .unwrap();

        assert_eq!(posts.len(), 3);
    }

    #[tokio::test]
    async fn api_post_delete_test() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;
        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let post0 = app
            .add_post(0, &auth_token, "title1", "cat", "one")
            .await
            .unwrap();

        let posts = app.state.db.get_post_all().await.unwrap();
        assert_eq!(posts.len(), 1);

        app.delete_post(1, auth_token, post0.key.to_sql())
            .await
            .unwrap();

        let posts = app.state.db.get_post_all().await.unwrap();
        assert_eq!(posts.len(), 0);
    }

    #[tokio::test]
    async fn resolution_from_str_test() {
        let result = resolution_from_str("10x10").unwrap();
        assert_eq!(result, (10, 10));
        let result = resolution_from_str("10x1").unwrap();
        assert_eq!(result, (10, 1));
        let result = resolution_from_str("10x");
        assert!(result.is_err());
        let result = resolution_from_str("x");
        assert!(result.is_err());
        let result = resolution_from_str("10z10");
        assert!(result.is_err());
        let result = resolution_from_str("999999999999999x1000000000000000000000");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn handle_file_saving_test() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;

        let stream = stream! {
            for i in 0..3 {
                yield Ok::<bytes::Bytes, anyhow::Error>(bytes::Bytes::from(vec![i]));
            }
        };
        let stream = pin!(stream);

        let save_path = &app.state.settings.site.files_path;
        let result = handle_file_saving(stream, "jpg", save_path, 10)
            .await
            .unwrap();

        assert_eq!(result.size_bytes, 3);

        let file_path = Path::new(&result.saved_path);
        let file = tokio::fs::read(file_path).await.unwrap();

        let mut hasher = DefaultHasher::new();
        hasher.write(&file);
        let hash = hasher.finish().to_string();

        assert_eq!(result.size_bytes, file.len());
        assert_eq!(result.hash, hash);

        tokio::fs::remove_file(&result.saved_path).await.unwrap();

        let stream = stream! {
            for i in 0..3 {
                yield Ok::<bytes::Bytes, anyhow::Error>(bytes::Bytes::from(vec![i]));
            }
        };
        let stream = pin!(stream);

        let save_path = &app.state.settings.site.files_path;
        let result = handle_file_saving(stream, "jpg", save_path, 2).await;
        assert!(matches!(
            result,
            Err(SaveFileErr::FileTooBig {
                max_bytes,
                got_bytes
            })
        ));
    }

    #[tokio::test]
    async fn api_post_file_add_fail() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;

        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let user = app.state.db.get_user_by_username("hey").await.unwrap();

        let post = app
            .add_post(0, &auth_token, "title1", "cat", "one")
            .await
            .unwrap();

        let upload_fn = async |max_storage: usize, max_file: usize, file_path: &str| {
            let user = app
                .state
                .db
                .update_user_storage(0, user.id.clone(), max_storage, max_file)
                .await
                .unwrap();

            let result = app
                .add_post_file(0, &auth_token, post.key.clone(), file_path)
                .await
                .map_err(|err| err.downcast::<ServerErr>().unwrap());

            result
        };

        for (max_storage, max_file_size) in [(3605, 3605), (3605, 3606), (3606, 3605)] {
            let result = upload_fn(max_storage, max_file_size, "../assets/favicon.ico")
                .await
                .err()
                .unwrap();

            assert!(matches!(
                result,
                ServerErr::AddPostFileErr(ServerAddPostFileErr::FileTooBig {
                    file_name,
                    max,
                    got
                })
            ));
        }

        let result = upload_fn(3606, 3606, "../assets/favicon.ico")
            .await
            .unwrap();

        assert_eq!(result.user.used_storage_bytes, 3606);
    }

    #[tokio::test]
    async fn api_post_file_add() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;

        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let post = app
            .add_post(0, &auth_token, "title1", "cat", "one")
            .await
            .unwrap();

        assert_eq!(post.file.len(), 0);

        let files = [
            (45, 45, "../assets/favicon.ico"),
            (15, 15, "../assets/upload.svg"),
        ];
        for (i, (width, height, file_path)) in files.into_iter().enumerate() {
            let post = app
                .add_post_file(0, &auth_token, post.key.clone(), file_path)
                .await
                .unwrap();
            let file = tokio::fs::read(file_path).await.unwrap();
            let mut hasher = DefaultHasher::new();
            hasher.write(&file);
            let hash = hasher.finish().to_string();

            let total_size = post.file.iter().fold(0_usize, |a, b| a + b.size_bytes);
            assert_eq!(post.file.len(), i + 1); // +1 because its length
            assert_eq!(post.file[i].proccesed, false);
            assert_eq!(post.file[i].size_bytes, file.len());
            assert_eq!(post.file[i].hash, hash);
            assert_eq!(post.file[i].width, width);
            assert_eq!(post.file[i].height, height);
            assert_eq!(post.user.used_storage_bytes, total_size);

            {
                let extension = Path::new(&file_path).extension().unwrap();
                let file_path = Path::new(&app.state.settings.site.files_path)
                    .join(hash)
                    .with_extension(extension);
                let file = tokio::fs::read(file_path).await.unwrap();

                let mut hasher = DefaultHasher::new();
                hasher.write(&file);
                let hash = hasher.finish().to_string();

                assert_eq!(post.file[i].size_bytes, file.len());
                assert_eq!(post.file[i].hash, hash);
            }
        }
    }

    #[tokio::test]
    async fn api_post_test() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;
        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        app.add_post(0, &auth_token, "title1", "cat", "one")
            .await
            .unwrap();
        app.expect_posts(0, 0, 1, 0, 1).await.unwrap();
        app.add_post(1, &auth_token, "title2", "cat", "one")
            .await
            .unwrap();
        app.expect_posts(0, 1, 2, 0, 1).await.unwrap();
        app.expect_posts(1, 0, 1, 1, 2).await.unwrap();
        app.expect_posts(2, 0, 0, 2, 2).await.unwrap();
    }

    #[tokio::test]
    async fn security_api_post_update_description() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;
        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let post = app
            .add_post(0, &auth_token, "title1", "cat", "")
            .await
            .unwrap();

        let big_description = (0..10000).into_iter().map(|_| 'a').collect::<String>();

        let result = app
            .update_post_description(0, &auth_token, post.key.clone(), big_description)
            .await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn api_post_update_title() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;
        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let post = app
            .add_post(0, &auth_token, "title1", "cat", "")
            .await
            .unwrap();

        assert_eq!(post.title, "title1");

        let post = app
            .update_post_title(0, &auth_token, post.key.clone(), "title2")
            .await
            .unwrap();

        assert_eq!(post.title, "title2");

        let posts = app
            .get_posts(
                0,
                &auth_token,
                2,
                TimeRange::MoreOrEqual(0),
                Order::OneTwoThree,
                "",
                "",
            )
            .await
            .unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].title, "title2");
    }

    #[tokio::test]
    async fn api_post_update_description() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;
        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let post = app
            .add_post(0, &auth_token, "title1", "cat", "")
            .await
            .unwrap();

        assert_eq!(post.description, "cat");

        let post = app
            .update_post_description(0, &auth_token, post.key.clone(), "one")
            .await
            .unwrap();

        assert_eq!(post.description, "one");

        let posts = app
            .get_posts(
                0,
                &auth_token,
                2,
                TimeRange::MoreOrEqual(0),
                Order::OneTwoThree,
                "",
                "",
            )
            .await
            .unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].description, "one");
    }

    #[tokio::test]
    async fn api_post_update_tags() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;
        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let post = app
            .add_post(0, &auth_token, "title1", "cat", "")
            .await
            .unwrap();

        assert_eq!(post.tags, "");

        let post = app
            .update_post_tags(0, &auth_token, post.key.clone(), "one")
            .await
            .unwrap();

        assert_eq!(post.tags, "one");

        let posts = app
            .get_posts(
                0,
                &auth_token,
                2,
                TimeRange::MoreOrEqual(0),
                Order::OneTwoThree,
                "",
                "",
            )
            .await
            .unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].tags, "one");
    }
}
