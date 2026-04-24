use axum::{Extension, extract::State};
use thiserror::Error;
use tracing::{debug, trace};

use crate::{
    api::{
        AuthToken, PostLikeErr, Server404Err, ServerDesErr, ServerErr, ServerReq, ServerRes,
        app_state::AppState,
    },
    db::{DB404Err, DBPostLikeErr, DBUser, post_like::create_post_like_id},
};

pub async fn add_post_like(
    State(app): State<AppState>,
    auth_token: Extension<AuthToken>,
    db_user: Extension<DBUser>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    type ResErr = PostLikeErr;
    //
    let ServerReq::PostId { post_key: post_id } = req else {
        return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
            "expected PostId, received: {req:?}"
        ))));
    };
    let time = app.time().await;

    app.db
        .add_post_like(time, db_user.id.clone(), post_id.clone())
        .await
        .map_err(|err| match err {
            DBPostLikeErr::PostNotFound(_) => ResErr::PostNotFound(post_id.clone()).into(),
            DBPostLikeErr::PostWasAlreadyLiked => ResErr::PostAlreadyLiked(post_id.clone()).into(),
            DBPostLikeErr::DB(_) => ServerErr::DbErr,
        })?;

    Ok(ServerRes::Ok)
}

pub async fn check_post_like(
    State(app): State<AppState>,
    auth_token: Extension<AuthToken>,
    db_user: Extension<DBUser>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    let ServerReq::PostId { post_key: post_id } = req else {
        return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
            "expected PostId, received: {req:?}"
        ))));
    };
    let time = app.time().await;

    let err = app
        .db
        .check_post_like(time, db_user.id.clone(), post_id.clone())
        .await;
    match err {
        Ok(v) => Ok(ServerRes::Condition(true)),
        Err(DB404Err::NotFound) => Ok(ServerRes::Condition(false)),
        Err(DB404Err::DB(_)) => Err(ServerErr::DbErr),
    }
}

pub async fn delete_post_like(
    State(app): State<AppState>,
    auth_token: Extension<AuthToken>,
    db_user: Extension<DBUser>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    type ResErr = Server404Err;

    let ServerReq::PostId { post_key: post_id } = req else {
        return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
            "expected PostId, received: {req:?}"
        ))));
    };
    let time = app.time().await;

    app.db
        .delete_post_like(db_user.id.clone(), post_id.clone())
        .await
        .map_err(|err| ServerErr::DbErr)?;

    Ok(ServerRes::Ok)
}

#[cfg(test)]
pub mod tests {
    use surrealdb::types::{RecordId, ToSql};
    use tokio::fs::{self, create_dir_all};

    use tracing::{debug, error, trace};

    use crate::api::app_state::AppState;
    use crate::api::shared::post_comment::UserPostComment;
    use crate::api::tests::ApiTestApp;
    use crate::db::{DBUser, DBEmailIsTakenErr, email_change::DBEmailChange};

    #[tokio::test]
    async fn api_post_like_test() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;

        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let post = app.add_post(0, &auth_token, "title1", "cat", "one").await.unwrap();
        debug!("wtf is that {post:#?}");

        app.check_post_like(0, &auth_token, post.key.clone(), false)
            .await
            .unwrap();
        app.add_post_like(0, &auth_token, post.key.clone())
            .await
            .unwrap();
        app.check_post_like(0, &auth_token, post.key.clone(), true)
            .await
            .unwrap();
        app.add_post_like_err_already_liked(0, &auth_token, post.key.clone())
            .await
            .unwrap();
        app.add_post_like_err_not_found(0, &auth_token, "none")
            .await
            .unwrap();
        app.delete_post_like(0, &auth_token, post.key.clone())
            .await
            .unwrap();
        app.check_post_like(0, &auth_token, post.key.clone(), false)
            .await
            .unwrap();
    }
}
