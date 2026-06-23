use crate::api::app_state::AppState;
use crate::api::backend::post::get_img_resolution;
use crate::api::{
    AuthToken, ChangeUsernameErr, EmailChangeErr, EmailChangeNewErr, EmailChangeStage,
    EmailChangeTokenErr, Server404Err, ServerAddPostErr, ServerAuthErr, ServerDecodeInviteErr,
    ServerDesErr, ServerErr, ServerErrImg, ServerErrImgMeta, ServerLoginErr, ServerRegistrationErr,
    ServerReq, ServerRes, ServerTokenErr, ServerUpdatePostDescriptionErr, User, UserPost,
    UserPostFile, auth_token_get, hash_password, verify_password,
};
use crate::db::email_change::create_email_change_id;
use crate::db::{AddUserErr, email_change::DBChangeEmailErr};
use crate::db::{DB404Err, DBChangeUsernameErr, DBUserPost, DbEngine, create_user_id};
use crate::db::{DBEmailIsTakenErr, DBUser};
use crate::db::{DBUserPostFile, email_change::DBEmailChange};
use crate::path::{link_settings_form_email_current_confirm, link_settings_form_email_new_confirm};
use crate::valid::auth::{
    proccess_password, proccess_post_description, proccess_post_title, proccess_username,
};
use anyhow::anyhow;
use axum::Extension;
use axum::extract::State;
use axum::response::IntoResponse;
// use axum_extra::extract::CookieJar;
// use axum_extra::extract::cookie::Cookie;
use gxhash::{gxhash64, gxhash128};
use http::header::{AUTHORIZATION, COOKIE};
use http::{HeaderMap, StatusCode};
use image::{ImageFormat, ImageReader};
use leptos_meta::Formatter;
use little_exif::{filetype::FileExtension, metadata::Metadata};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::Duration;
use std::{io::Cursor, path::Path, str::FromStr};
use surrealdb::types::{RecordId, ToSql};
use tokio::fs;
use tracing::{debug, error, info, trace};

pub mod auth;
pub mod change_email;
pub mod change_password;
pub mod change_username;
pub mod post;
pub mod post_comment;
pub mod post_like;

pub fn scale_res_by_width(width: u32, height: u32, new_width: u32) -> (u32, u32) {
    let ratio = height as f32 / width as f32;
    let new_height = ratio * new_width as f32;
    (new_width as u32, new_height as u32)
}

#[test]
fn test_scale_res_by_width() {
    let result = scale_res_by_width(1920, 1080, 1280);
    assert_eq!(result, (1280, 720));

    let result = scale_res_by_width(1080, 1920, 1280);
    assert_eq!(result, (1280, 2275));
}

pub fn scale_res_by_height(width: u32, height: u32, new_height: u32) -> (u32, u32) {
    let ratio = width as f32 / height as f32;
    let new_width = ratio * new_height as f32;
    (new_width as u32, new_height as u32)
}

#[test]
fn test_scale_res_by_height() {
    let result = scale_res_by_height(1920, 1080, 720);
    assert_eq!(result, (1280, 720));
    let result = scale_res_by_height(1080, 1920, 720);
    assert_eq!(result, (405, 720));
}

pub fn scale_resolution(width: u32, height: u32, clamp: u32) -> (u32, u32) {
    match (width, height) {
        (width, height) if width > clamp && width >= height => {
            scale_res_by_width(width, height, clamp)
        }
        (width, height) if height > clamp && height >= width => {
            scale_res_by_height(width, height, clamp)
        }
        (width, height) => (width, height),
    }
}

#[test]
fn test_scale_resolution() {
    let result = scale_resolution(1920, 1080, 1280);
    assert_eq!(result, (1280, 720));
    let result = scale_resolution(1080, 1920, 1280);
    assert_eq!(result, (720, 1280));
    let result = scale_resolution(1280, 720, 1280);
    assert_eq!(result, (1280, 720));
}

pub fn to_thumbnail_file_name(file_name: impl AsRef<str>) -> String {
    format!("{}_thumbnail_default.webp", file_name.as_ref())
}

pub fn to_thumbnail_path(file_path: impl AsRef<OsStr>) -> Result<PathBuf, anyhow::Error> {
    let output = Path::new(file_path.as_ref());
    let output = output.with_extension("");
    let file_name = output
        .file_name()
        .ok_or_else(|| anyhow!("invalid filename"))?
        .to_str()
        .ok_or_else(|| anyhow!("invalid filename"))?;
    let file_name_new = to_thumbnail_file_name(file_name);
    Ok(output.with_file_name(file_name_new).with_extension("webp"))
}

#[test]
fn test_to_thumbnail_path() {
    let file = DBUserPostFile {
        proccesed: false,
        extension: String::from("webp"),
        hash: String::from("one"),
        size_bytes: 1,
        width: 10,
        height: 10,
    };
    let file_path = file.to_file_path("/tmp");
    let thumbnail_path = to_thumbnail_path(file_path).unwrap();
    assert_eq!(
        "/tmp/one_thumbnail_default.webp",
        thumbnail_path.to_str().unwrap()
    );
}

pub struct ProccesedFileResult {
    pub path: PathBuf,
    pub already_existed: bool,
}

pub async fn proccess_post_file(
    arg_input_path: impl AsRef<OsStr>,
    width: u32,
    height: u32,
    resolution_limit: u32,
) -> Result<ProccesedFileResult, anyhow::Error> {
    let arg_input_path = arg_input_path.as_ref();
    let arg_input_path = arg_input_path
        .to_str()
        .ok_or_else(|| anyhow!("invalid filename"))?;

    // TODO fix performance, strings and format and Path are bad
    let mut command = tokio::process::Command::new("ffmpeg");

    let (new_width, new_height) = scale_resolution(width, height, resolution_limit);

    let output_path = to_thumbnail_path(arg_input_path)?;

    if output_path.exists() {
        return Ok(ProccesedFileResult {
            path: output_path,
            already_existed: true,
        });
    }

    let arg_output_path = output_path
        .to_str()
        .ok_or_else(|| anyhow!("invalid filename"))?;

    let arg_scale = format!("scale={new_width}:{new_height}");

    let command = command.args(&[
        "-y",
        "-i",
        arg_input_path,
        "-vf",
        &arg_scale,
        arg_output_path,
    ]);
    trace!("running command {command:?}");
    let result = command.output().await?;

    let result = String::from_utf8(result.stdout)?;
    let result = result.trim();
    trace!("command output {result}");

    Ok(ProccesedFileResult {
        path: output_path,
        already_existed: false,
    })
}

#[tokio::test]
async fn test_proccess_post_file() {
    crate::init_test_log();
    let img_path = "../assets/upload.svg";
    let tmp_path = "/tmp/test_proccess_post_file.svg";
    tokio::fs::copy(img_path, tmp_path).await.unwrap();
    let (width, height) = get_img_resolution(img_path).await.unwrap();
    let output = proccess_post_file(tmp_path, width, height, 1280)
        .await
        .unwrap();
    assert!(output.path.exists());
    assert_eq!(output.already_existed, false);
    let output = proccess_post_file(tmp_path, width, height, 1280)
        .await
        .unwrap();
    assert!(output.path.exists());
    assert_eq!(output.already_existed, true);
    tokio::fs::remove_file(output.path).await.unwrap();
}

pub async fn proccess_post_files(
    db: DbEngine,
    input_path: impl AsRef<str>,
    output_path: impl AsRef<str>,
    resolution_limit: u32,
) -> Result<(), anyhow::Error> {
    let posts = db.get_post_unproccesed().await.unwrap();
    for post in posts {
        for file in &post.file {
            let file_path = file.to_file_path(&input_path);
            let result =
                proccess_post_file(&file_path, file.width, file.height, resolution_limit).await?;
            db.update_post_file_proccesed(post.id.clone(), &file.hash)
                .await?;
            // trace!("file path: {file_path:?}");
            // assert!(file_path.exists());
            // assert_eq!(file.proccesed, false);
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_proccess_post_files() {
    // TODO delete files after test ends
    crate::init_test_log();
    const FILES_PATH: &str = "/tmp/test_proccess_post_files";
    let app = crate::api::tests::ApiTestApp::new_with_exp_and_files(1, FILES_PATH).await;
    // let app = crate::api::tests::ApiTestApp::new(1).await;
    let img_path = "../assets/upload.svg";
    let tmp_path = "/tmp/test_proccess_post_files.svg";
    let tmp_input = "/tmp";
    let tmp_output = "/tmp/";
    tokio::fs::copy(img_path, tmp_path).await.unwrap();

    // file upload by user
    {
        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let user = app.state.db.get_user_by_username("hey").await.unwrap();

        let post = app
            .add_post(0, &auth_token, "title1", "cat", "one")
            .await
            .unwrap();

        let post = app
            .add_post_file(0, &auth_token, post.key.clone(), img_path)
            .await
            .unwrap();
    }

    {
        proccess_post_files(app.state.db.clone(), tmp_input, tmp_output, 1280)
            .await
            .unwrap();
    }

    {
        let posts = app.state.db.get_post_unproccesed().await.unwrap();

        for post in posts {
            for file in post.file {
                let file_path = file.to_file_path(tmp_output);
                let thumbnail_path = file.to_thumbnail_path(tmp_output);

                assert_eq!(file.proccesed, true);
                assert!(file_path.exists());
                assert!(thumbnail_path.exists());
                tokio::fs::remove_file(file_path).await.unwrap();
                tokio::fs::remove_file(thumbnail_path).await.unwrap();
            }
        }
    }

    // tokio::fs::remove_file(output.path).await.unwrap();
}

// #[cfg(test)]
// mod tests {
//     use crate::api::backend::proccess_post_file;

//     #[tokio::test]
//     pub async fn test_proccess_post_file() {
//         crate::init_test_log();
//         // proccess_post_file("../assets/upload.svg").await.unwrap();
//     }
// }

pub async fn get_user(
    State(app_state): State<AppState>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    let ServerReq::GetUser { username } = req else {
        return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
            "expected GetUser, received: {req:?}"
        ))));
    };

    let user = app_state
        .db
        .get_user_by_username(username)
        .await
        .map_err(|err| match err {
            DB404Err::NotFound => Server404Err::NotFound.into(),
            _ => ServerErr::DbErr,
        })?;

    Ok(ServerRes::User {
        username: user.username,
    })
}

pub async fn get_account(
    State(app_state): State<AppState>,
    auth_token: Extension<AuthToken>,
    db_user: Extension<DBUser>,
) -> Result<ServerRes, ServerErr> {
    Ok(ServerRes::Acc {
        key: db_user.id.key.to_sql(),
        username: db_user.username.clone(),
        email: db_user.email.clone(),
    })
}

// email change

pub async fn auth_optional_middleware(
    State(app_state): State<AppState>,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let result = {
        let headers = req.headers();
        // let jar = CookieJar::from_headers(headers);
        check_auth(&app_state, &headers).await
    };

    match result {
        Ok((token, user)) => {
            let extensions = req.extensions_mut();
            extensions.insert(Some::<AuthToken>(token));
            extensions.insert(Some::<DBUser>(user));
        }
        Err(err) => {
            let extensions = req.extensions_mut();
            extensions.insert(None::<AuthToken>);
            extensions.insert(None::<DBUser>);
        }
    }
    next.run(req).await
}

pub async fn auth_middleware(
    State(app_state): State<AppState>,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let result = {
        let headers = req.headers();
        // let jar = CookieJar::from_headers(headers);
        check_auth(&app_state, &headers).await
    };
    match result {
        Ok((token, user)) => {
            {
                let extensions = req.extensions_mut();
                extensions.insert(token);
                extensions.insert(user);
            }
            let response = next.run(req).await;
            return response;
        }
        Err(err) => {
            return err.into_response();
        }
    }
}

pub async fn check_auth(
    app: &AppState,
    headers: &HeaderMap,
) -> Result<(AuthToken, DBUser), ServerErr>
where
    ServerErr: std::error::Error + 'static,
{
    trace!("CHECKING AUTH");
    let secret = app.get_secret().await;
    let token = auth_token_get(headers, COOKIE).ok_or(ServerErr::AuthErr(
        ServerAuthErr::ServerUnauthorizedNoCookie,
    ))?;

    trace!("CHECKING AUTH SESSION");
    let session = app.db.get_session(&token).await.map_err(|err| match err {
        DB404Err::NotFound => ServerErr::from(ServerAuthErr::ServerUnauthorizedInvalidCookie),
        _ => ServerErr::DbErr,
    })?;

    Ok((AuthToken(token), session.user))
}
