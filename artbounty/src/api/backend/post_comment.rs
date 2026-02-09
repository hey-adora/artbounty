use crate::api::app_state::AppState;
use crate::api::post_comment::{PostCommentErrResolver, UserPostComment};
use crate::api::{
    AuthToken, ChangeUsernameErr, EmailChangeErr, EmailChangeNewErr, EmailChangeStage,
    EmailChangeTokenErr, EmailToken, Server404Err, ServerAddPostErr, ServerAuthErr,
    ServerDecodeInviteErr, ServerDesErr, ServerErr, ServerErrImg, ServerErrImgMeta, ServerLoginErr,
    ServerRegistrationErr, ServerReq, ServerRes, ServerTokenErr, User, UserPost, UserPostFile,
    auth_token_get, decode_token, encode_token, hash_password, verify_password,
};
use crate::db::DB404Err;
use crate::db::{AddUserErr, DBPostCommentErr, DBUser};
use crate::valid::auth::{proccess_password, proccess_username};
use axum::Extension;
use axum::extract::State;
// use axum_extra::extract::CookieJar;
use http::header::COOKIE;
use tracing::{debug, error, info, trace};

pub async fn add_post_comment(
    State(app): State<AppState>,
    auth_token: Extension<AuthToken>,
    db_user: Extension<DBUser>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    type ResErr = Server404Err;
    //
    let ServerReq::AddPostComment { post_id, text } = req else {
        return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
            "expected AddPostComment, received: {req:?}"
        ))));
    };
    let time = app.time().await;

    let comment = app
        .db
        .add_post_comment(time, db_user.id.clone(), &post_id, None, text)
        .await
        .map_err(|err| match err {
            DBPostCommentErr::PostNotFound(_) => ResErr::NotFound.into(),
            // DBPostCommentErr::PostNotFound(_) => ResErr::NotFound(format!("post \"{post_id}\" not found")).into(),
            DBPostCommentErr::DB(_) | DBPostCommentErr::ReplyCommentNotFound(_) => ServerErr::DbErr,
        })?;
    let comment = UserPostComment::from(comment);

    // //
    Ok(ServerRes::Comment(comment))
}

pub async fn get_post_comment(
    State(app): State<AppState>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    type ResErr = Server404Err;
    //
    let ServerReq::PostId { post_id } = req else {
        return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
            "expected AddPostComment, received: {req:?}"
        ))));
    };
    let time = app.time().await;

    let comments: Vec<UserPostComment> = app
        .db
        .get_post_comments(time, &post_id)
        .await
        .map_err(|err| match err {
            DB404Err::NotFound => ResErr::NotFound.into(),
            // DBPostCommentErr::PostNotFound(_) => ResErr::NotFound(format!("post \"{post_id}\" not found")).into(),
            DB404Err::DB(_) => ServerErr::DbErr,
        })?
        .into_iter()
        .map(|v| UserPostComment::from(v))
        .collect();

    // //
    Ok(ServerRes::Comments(comments))
}

pub async fn delete_post_comment(
    State(app): State<AppState>,
    db_user: Extension<DBUser>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    type ResErr = Server404Err;
    //
    let ServerReq::PostId { post_id } = req else {
        return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
            "expected AddPostComment, received: {req:?}"
        ))));
    };
    let time = app.time().await;

    app.db
        .delete_post_comment(db_user.id.clone(), &post_id)
        .await
        .map_err(|err| ServerErr::DbErr)?;

    Ok(ServerRes::Ok)
}
