use crate::api::app_state::AppState;
use crate::api::shared::post_comment::{PostCommentErrResolver, UserPostComment};
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
            "add_post_comment expected AddPostComment, received: {req:?}"
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
    let ServerReq::GetComments { post_id, limit, time_range } = req else {
        return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
            "get_post_comment expected PostId, received: {req:?}"
        ))));
    };
    let time = app.time().await;

    let comments: Vec<UserPostComment> = app
        .db
        .get_post_comments(time, &post_id, limit, time_range)
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
            "delete_post_comment expected PostId, received: {req:?}"
        ))));
    };
    let time = app.time().await;

    app.db
        .delete_post_comment(db_user.id.clone(), &post_id)
        .await
        .map_err(|err| ServerErr::DbErr)?;

    Ok(ServerRes::Ok)
}

#[cfg(test)]
mod tests {
    use crate::api::{Api, ServerRes, TimeRange, shared::post_comment::UserPostComment, tests::ApiTestApp};
    use tracing::{debug, error, trace};
    use web_sys::console::assert;

    impl ApiTestApp {
        pub async fn add_post_comment(
            &self,
            server_time: u128,
            auth_token: impl AsRef<str>,
            post_id: impl Into<String>,
            text: impl Into<String>,
        ) -> Option<UserPostComment> {
            self.set_time(server_time).await;
            let result = self
                .api
                .add_post_comment(post_id, text)
                .send_native_with_token(auth_token)
                .await;

            let Ok(ServerRes::Comment(comment)) = result else {
                return None;
            };

            Some(comment)
        }

        pub async fn get_post_comments(
            &self,
            server_time: u128,
            auth_token: impl AsRef<str>,
            post_id: impl Into<String>,
            limit: usize,
            time_range: TimeRange,
        ) -> Option<Vec<UserPostComment>> {
            self.set_time(server_time).await;
            let result = self
                .api
                .get_post_comment(post_id, limit, time_range)
                .send_native_with_token(auth_token)
                .await;

            let Ok(ServerRes::Comments(comment)) = result else {
                return None;
            };

            Some(comment)
        }
    }

    #[tokio::test]
    async fn api_post_comment_test() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;

        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let post = app.add_post(0, &auth_token).await.unwrap();
        debug!("wtf is that {post:#?}");

        let comment = app
            .add_post_comment(0, &auth_token, post.id.clone(), "wowza".to_string())
            .await.unwrap();

        let comment = app
            .add_post_comment(1, &auth_token, post.id.clone(), "wowza2".to_string())
            .await.unwrap();

        let comments = app
            .get_post_comments(2, &auth_token, post.id.clone(), 2, TimeRange::None)
            .await.unwrap();

        assert!(comments.len() == 2);

        let comment_first = comments.first().unwrap();

        assert_eq!(comment_first.text, "wowza2");

    }
}
