use crate::api::app_state::AppState;
use crate::api::{
    AuthToken, ChangeUsernameErr, EmailChangeErr, EmailChangeNewErr, EmailChangeStage,
    EmailChangeTokenErr, Server404Err, ServerAddPostErr, ServerAuthErr, ServerDecodeInviteErr,
    ServerDesErr, ServerErr, ServerErrImg, ServerErrImgMeta, ServerLoginErr, ServerRegistrationErr,
    ServerReq, ServerRes, ServerTokenErr, ServerUpdatePostDescriptionErr, User, UserPost,
    UserPostFile, auth_token_get, hash_password, verify_password,
};
use crate::db::email_change::create_email_change_id;
use crate::db::{AddUserErr, email_change::DBChangeEmailErr};
use crate::db::{DB404Err, DBChangeUsernameErr, create_user_id};
use crate::db::{DBEmailIsTakenErr, DBUser};
use crate::db::{DBUserPostFile, email_change::DBEmailChange};
use crate::path::{link_settings_form_email_current_confirm, link_settings_form_email_new_confirm};
use crate::valid::auth::{
    proccess_password, proccess_post_description, proccess_post_title, proccess_username,
};
use axum::Extension;
use axum::extract::State;
use axum::response::IntoResponse;
// use axum_extra::extract::CookieJar;
// use axum_extra::extract::cookie::Cookie;
use gxhash::{gxhash64, gxhash128};
use http::header::{AUTHORIZATION, COOKIE};
use http::{HeaderMap, StatusCode};
use image::{ImageFormat, ImageReader};
use little_exif::{filetype::FileExtension, metadata::Metadata};
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

//

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
