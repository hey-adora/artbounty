use std::io::Cursor;
use std::path::Path;
use std::str::FromStr;

use crate::api::app_state::AppState;
use crate::api::shared::post_comment::{PostCommentErrResolver, UserPostComment};
use crate::api::{
    AuthToken, ChangeUsernameErr, EmailChangeErr, EmailChangeNewErr, EmailChangeStage,
    EmailChangeTokenErr, Server404Err, ServerAddPostErr, ServerAddPostFileErr, ServerAuthErr,
    ServerDecodeInviteErr, ServerDesErr, ServerErr, ServerErrImg, ServerErrImgMeta, ServerLoginErr,
    ServerRegistrationErr, ServerReq, ServerRes, ServerTokenErr, ServerUpdatePostDescriptionErr,
    User, UserPost, UserPostFile, auth_token_get, hash_password, verify_password,
};
use crate::db::{AddUserErr, DBPostCommentErr, DBUser};
use crate::db::{DB404Err, DBUserPostFile};
use crate::valid::auth::{
    proccess_password, proccess_post_description, proccess_post_title, proccess_username,
};
use axum::Extension;
use axum::body::Body;
use axum::extract::{Multipart, State};
use axum::response::IntoResponse;
use futures::TryStreamExt;
use futures_util::StreamExt;
use gxhash::gxhash128;
// use axum_extra::extract::CookieJar;
use http::header::COOKIE;
use image::ImageReader;
use little_exif::filetype::FileExtension;
use std::{io, pin::pin};
use tokio::fs;
use tokio::{fs::File, io::BufWriter};
use tokio::io::AsyncWriteExt;
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

    let new_description =
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
    type ResErr = Server404Err;

    let ServerReq::EditPostTags { post_key, new_tags } = req else {
        return Err(
            ServerDesErr::ServerWrongInput(format!("expected PostId, received: {req:?}")).into(),
        );
    };

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

//params: RawPathParams,
// pub async fn post_file_add(body: Body) -> impl IntoResponse {
pub async fn post_file_add(
    State(app): State<AppState>,
    params: axum::extract::RawPathParams,
    db_user: Extension<DBUser>,
    body: Body,
) -> Result<ServerRes, ServerErr> {
    type Err = ServerAddPostFileErr;

    let time = app.clock.now().await;
    let file_name = app.gen_key().await;

    let post_key = params
        .iter()
        .find(|(name, _)| *name == "post_id")
        .ok_or(Err::ParamNotFoundPostId)
        .map(|(_, value)| value)?;



    let file_path = app.get_file_path().await;
    let file_path = Path::new(&file_path).join(file_name).with_extension("part");

    trace!("file_path {file_path:?}");
    trace!("PATH PARAMS {post_key:?}");

    let mut stream = body.into_data_stream();
    // let body_with_io_error = stream.map_err(io::Error::other);
    // let mut body_reader = pin!(StreamReader::new(body_with_io_error));

    trace!("hello from streaming");

    // let path = std::path::Path::new("/home/hey/github/artbounty/target/tmp/here.part");
    let mut file = BufWriter::new(File::create(file_path).await.unwrap());
    while let Some(value) = stream.next().await {
        let bytes = value.map_err(|v| Err::IoErr(v.to_string()))?;
        file.write(&bytes).await.map_err(|v| Err::IoErr(v.to_string()))?;
        // trace!("wtf: {value:#?}");
    }
    file.flush().await.map_err(|v| Err::IoErr(v.to_string()))?;
    // tokio::io::copy(&mut body_reader, &mut file).await.unwrap();

    // let post = app
    //     .db
    //     .add_post_file(
    //         time,
    //         db_user.id.clone(),
    //         post_key,
    //         file_hash: impl Into<String>,
    //         file_extension: impl Into<String>,
    //         file_width: u32,
    //         file_height: u32,
    //
    //     );

    // let post = app
    //     .db
    //     .add_post(
    //         time,
    //         &db_user.username,
    //         &title,
    //         &description,
    //         tags,
    //         0,
    //         // post_files,
    //     )
    //     .await
    //     .inspect_err(|err| error!("failed to save images {err:?}"))
    //     .map_err(|_| ServerErr::DbErr)?;


    // while let Some(value) = stream.next().await {
    //     trace!("wtf: {value:#?}");
    // }
    // trace!("wtf {body:#?}");

    // Ok(ServerRes::Post(post.into()))
    Ok(ServerRes::Ok)
    // "done"
}
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
    let file_path = app.get_file_path().await;

    let title = proccess_post_title(title)
        .map_err(|err| ServerAddPostErr::InvalidTitle(err.to_string()))?;
    let description = proccess_post_description(description)
        .map_err(|err| ServerAddPostErr::InvalidDescription(err.to_string()))?;

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
            &title,
            &description,
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
    use axum::Router;
    use std::path::Path;
    use std::sync::Arc;
    use std::time::Duration;
    use surrealdb::types::{RecordId, ToSql};
    use tokio::fs::{self, create_dir_all};

    use axum_test::TestServer;
    use gxhash::gxhash128;
    use tokio::sync::Mutex;
    use tracing::{debug, error, trace};

    use crate::api::app_state::AppState;
    use crate::api::shared::post_comment::UserPostComment;
    use crate::api::tests::ApiTestApp;
    use crate::api::{
        Api, ApiTest, EmailChangeErr, EmailChangeNewErr, EmailChangeStage, EmailChangeTokenErr,
        Order, PostLikeErr, Server404Err, ServerAuthErr, ServerErr, ServerLoginErr,
        ServerRegistrationErr, ServerReqImg, ServerRes, ServerSendInviteErr, TimeRange, UserPost,
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

        let post = app
            .add_post_file(
                0,
                auth_token,
                post.key.clone(),
                "/home/hey/github/artbounty/flake.nix",
            )
            .await
            .unwrap();

        // assert_eq!(post.file.len(), 1);
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
