use crate::api::app_state::AppState;
use crate::api::shared::post_comment::{PostCommentErrResolver, UserPostComment};
use crate::api::{
    AuthToken, ChangeUsernameErr, EmailChangeErr, EmailChangeNewErr, EmailChangeStage,
    EmailChangeTokenErr, Server404Err, ServerAddPostErr, ServerAuthErr, ServerDecodeInviteErr,
    ServerDesErr, ServerErr, ServerErrImg, ServerErrImgMeta, ServerLoginErr, ServerRegistrationErr,
    ServerReq, ServerRes, ServerTokenErr, User, UserPost, UserPostFile, auth_token_get,
    hash_password, verify_password,
};
use crate::db::DB404Err;
use crate::db::{AddUserErr, DBPostCommentErr, DBUser};
use crate::valid::auth::{proccess_password, proccess_username};
use axum::Extension;
use axum::extract::State;
// use axum_extra::extract::CookieJar;
use http::header::COOKIE;
use tracing::{debug, error, info, trace};

pub async fn update_post_comment(
    State(app): State<AppState>,
    auth_token: Extension<AuthToken>,
    db_user: Extension<DBUser>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    type ResErr = Server404Err;
    //
    let ServerReq::UpdatePostComment { comment_key, text } = req else {
        return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
            "update_post_comment expected UpdatePostComment, received: {req:?}"
        ))));
    };
    let time = app.time().await;

    let comment = app
        .db
        .update_post_comment(time, db_user.id.clone(), comment_key, text)
        .await
        .map_err(|err| match err {
            DB404Err::NotFound => ResErr::NotFound.into(),
            DB404Err::DB(_) => ServerErr::DbErr,
        })?;
    // DBPostCommentErr::PostNotFound(_) => ResErr::NotFound(format!("post \"{post_id}\" not found")).into(),
    let comment = UserPostComment::from(comment);

    // //
    Ok(ServerRes::Comment(comment))
}

pub async fn add_post_comment(
    State(app): State<AppState>,
    auth_token: Extension<AuthToken>,
    db_user: Extension<DBUser>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    type ResErr = Server404Err;
    //
    let ServerReq::AddPostComment {
        post_key,
        comment_key,
        text,
    } = req
    else {
        return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
            "add_post_comment expected AddPostComment, received: {req:?}"
        ))));
    };
    let time = app.time().await;

    let comment = app
        .db
        .add_post_comment(
            time,
            db_user.id.clone(),
            post_key.clone(),
            comment_key,
            text,
        )
        .await
        .map_err(|err| match err {
            DBPostCommentErr::PostNotFound(_) => ResErr::NotFound.into(),
            DBPostCommentErr::DB(_) | DBPostCommentErr::ReplyCommentNotFound(_) => ServerErr::DbErr,
        })?;
    // DBPostCommentErr::PostNotFound(_) => ResErr::NotFound(format!("post \"{post_id}\" not found")).into(),
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
    let ServerReq::GetComments {
        post_key,
        comment_key,
        limit,
        time_range,
        order,
        flatten,
    } = req
    else {
        return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
            "get_post_comment expected PostId, received: {req:?}"
        ))));
    };
    let time = app.time().await;

    let comments: Vec<UserPostComment> = app
        .db
        .get_post_comments(
            time,
            post_key.clone(),
            comment_key.clone(),
            flatten,
            limit,
            time_range,
            order,
        )
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
    let ServerReq::CommentId { comment_id } = req else {
        return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
            "delete_post_comment expected PostId, received: {req:?}"
        ))));
    };
    let time = app.time().await;

    app.db
        .delete_post_comment(db_user.id.clone(), comment_id.clone())
        .await
        .map_err(|err| ServerErr::DbErr)?;

    Ok(ServerRes::Ok)
}

#[cfg(test)]
mod tests {
    use crate::api::{
        Api, Order, ServerRes, TimeRange, shared::post_comment::UserPostComment, tests::ApiTestApp,
    };
    use surrealdb::types::ToSql;
    use tracing::{debug, error, trace};
    use web_sys::console::assert;

    impl ApiTestApp {
        pub async fn update_post_comment(
            &self,
            server_time: u128,
            auth_token: impl AsRef<str>,
            comment_key: impl Into<String>,
            text: impl Into<String>,
        ) -> Option<UserPostComment> {
            self.set_time(server_time).await;
            let result = self
                .api
                .update_post_comment(comment_key, text)
                .send_native_with_token(auth_token)
                .await;

            let Ok(ServerRes::Comment(comment)) = result else {
                return None;
            };

            Some(comment)
        }

        pub async fn add_post_comment(
            &self,
            server_time: u128,
            auth_token: impl AsRef<str>,
            post_key: impl Into<String>,
            comment_key: Option<String>,
            text: impl Into<String>,
        ) -> Option<UserPostComment> {
            self.set_time(server_time).await;
            let result = self
                .api
                .add_post_comment(post_key, comment_key, text)
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
            post_key: impl Into<String>,
            comment_key: Option<String>,
            limit: usize,
            time_range: TimeRange,
            order: Order,
        ) -> Option<Vec<UserPostComment>> {
            self.set_time(server_time).await;
            let result = self
                .api
                .get_post_comment(post_key, comment_key, limit, time_range, order, false)
                .send_native_with_token(auth_token)
                .await;

            let Ok(ServerRes::Comments(comment)) = result else {
                return None;
            };

            Some(comment)
        }
    }

    #[tokio::test]
    async fn api_post_comment_update() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;

        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let auth_token2 = app
            .register(0, "hey2", "hey2@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let post = app.add_post(0, &auth_token).await.unwrap();
        debug!("wtf is that {post:#?}");

        let comment = app
            .add_post_comment(0, &auth_token, post.id.clone(), None, "wowza1".to_string())
            .await
            .unwrap();

        assert_eq!(comment.text, "wowza1");

        let comment = app
            .update_post_comment(0, &auth_token, comment.key.clone(), "wowza2")
            .await
            .unwrap();

        assert_eq!(comment.text, "wowza2");

        let comment = app
            .state
            .db
            .get_post_comment(comment.key.clone())
            .await
            .unwrap();

        assert_eq!(comment.text, "wowza2");

        let comment = app
            .update_post_comment(0, &auth_token2, comment.id.key.to_sql(), "wowza3")
            .await;

        assert!(comment.is_none());

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
            .add_post_comment(0, &auth_token, post.id.clone(), None, "wowza".to_string())
            .await
            .unwrap();

        let comment = app
            .add_post_comment(1, &auth_token, post.id.clone(), None, "wowza2".to_string())
            .await
            .unwrap();

        let comments = app
            .get_post_comments(
                2,
                &auth_token,
                post.id.clone(),
                None::<String>,
                2,
                TimeRange::None,
                Order::ThreeTwoOne,
            )
            .await
            .unwrap();

        assert!(comments.len() == 2);

        let comment_first = comments.first().unwrap();

        assert_eq!(comment_first.text, "wowza2");
    }

    #[tokio::test]
    async fn api_post_comment_reply_test() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;

        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();

        let post = app.add_post(0, &auth_token).await.unwrap();
        debug!("wtf is that {post:#?}");

        let comment = app
            .add_post_comment(0, &auth_token, post.id.clone(), None, "wowza".to_string())
            .await
            .unwrap();

        let comment_reply = app
            .add_post_comment(
                1,
                &auth_token,
                post.id.clone(),
                Some(comment.key.clone()),
                "wowza2".to_string(),
            )
            .await
            .unwrap();

        let comments = app
            .get_post_comments(
                2,
                &auth_token,
                post.id.clone(),
                None::<String>,
                2,
                TimeRange::None,
                Order::ThreeTwoOne,
            )
            .await
            .unwrap();

        assert!(comments.len() == 1);
        let comment_first = comments.first().unwrap();
        assert_eq!(comment_first.text, "wowza");

        let comments = app
            .get_post_comments(
                2,
                &auth_token,
                post.id.clone(),
                Some(comment_first.key.clone()),
                2,
                TimeRange::None,
                Order::ThreeTwoOne,
            )
            .await
            .unwrap();

        assert!(comments.len() == 1);
        let comment_first = comments.first().unwrap();
        assert_eq!(comment_first.text, "wowza2");
    }
}
