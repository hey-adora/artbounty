
        use std::f64::consts::SQRT_2;

        use axum::{Extension, extract::State};
        use thiserror::Error;
        use tracing::{debug, trace};

        use crate::{
            api::{
                AuthToken, PostLikeErr, Server404Err, ServerDesErr, ServerErr, ServerReq,
                ServerRes, app_state::AppState,
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
            let ServerReq::PostId { post_id } = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected PostId, received: {req:?}"
                ))));
            };
            let time = app.time().await;

            app.db
                .add_post_like(time, db_user.id.clone(), &post_id)
                .await
                .map_err(|err| match err {
                    DBPostLikeErr::PostNotFound(_) => ResErr::PostNotFound(post_id.clone()).into(),
                    DBPostLikeErr::PostWasAlreadyLiked => {
                        ResErr::PostAlreadyLiked(post_id.clone()).into()
                    }
                    DBPostLikeErr::DB(_) => ServerErr::DbErr,
                })?;

            // //
            Ok(ServerRes::Ok)
        }

        pub async fn check_post_like(
            State(app): State<AppState>,
            auth_token: Extension<AuthToken>,
            db_user: Extension<DBUser>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            // type ResErr = Server404Err;
            //
            let ServerReq::PostId { post_id } = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected PostId, received: {req:?}"
                ))));
            };
            let time = app.time().await;

            let err = app
                .db
                .check_post_like(time, db_user.id.clone(), post_id.clone())
                .await;
            // .map_err(|err| )?;
            match err {
                Ok(v) => Ok(ServerRes::Condition(true)),
                Err(DB404Err::NotFound) => Ok(ServerRes::Condition(false)),
                // DBPostLikeErr::PostWasAlreadyLiked => {
                //     ResErr::PostAlreadyLiked(post_id.clone()).into()
                // }
                Err(DB404Err::DB(_)) => Err(ServerErr::DbErr),
            }

            // //
            // Ok(ServerRes::Condition(true))
        }

        pub async fn delete_post_like(
            State(app): State<AppState>,
            auth_token: Extension<AuthToken>,
            db_user: Extension<DBUser>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            type ResErr = Server404Err;
            //
            let ServerReq::PostId { post_id } = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected PostId, received: {req:?}"
                ))));
            };
            let time = app.time().await;

            app.db
                .delete_post_like(db_user.id.clone(), post_id.clone())
                .await
                .map_err(|err| ServerErr::DbErr)?;

            // //
            Ok(ServerRes::Ok)
        }
