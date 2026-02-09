
        use std::f64::consts::SQRT_2;

        use axum::{Extension, extract::State};
        use thiserror::Error;
        use tracing::{debug, trace};

        use crate::{
            api::{
                AuthToken, ChangePasswordErr, ChangeUsernameErr, Server404Err, ServerDesErr,
                ServerErr, ServerErrImg, ServerReq, ServerRes, app_state::AppState, hash_password,
                verify_password,
            },
            db::{DB404Err, DBChangeUsernameErr, DBUser},
            valid::auth::proccess_password,
        };

        pub async fn send_password_change(
            State(app): State<AppState>,
            auth_token: Extension<Option<AuthToken>>,
            db_user: Extension<Option<DBUser>>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            let ServerReq::EmailAddress { email } = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected None, received: {req:?}"
                ))));
            };
            let time = app.time().await;

            let user = app.db.get_user_by_email(&email).await;

            let user = match user {
                Ok(v) => v,
                Err(DB404Err::NotFound) => {
                    // so people couldnt exploit what accounts exists
                    return Ok(ServerRes::Ok);
                }
                Err(DB404Err::DB(_)) => {
                    return Err(ServerErr::DbErr);
                }
            };

            let exp = app.new_exp().await;
            let result = app
                .db
                .add_confirm_email(time, &email, exp)
                .await
                .map_err(|err| ServerErr::DbErr)?;

            let confirm_key = result.id.key().to_string();

            if db_user
                .as_ref()
                .map(|v| v.email == email)
                .unwrap_or_default()
            {
                app.send_email_change_password(time, &email, confirm_key)
                    .await?;
            } else {
                app.send_email_reset_password(time, &email, confirm_key)
                    .await?;
            }

            Ok(ServerRes::Ok)
        }

        pub async fn confirm_password_change(
            State(app): State<AppState>,
            // auth_token: Extension<AuthToken>,
            // db_user: Extension<DBUser>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            type ResErr = ChangePasswordErr;

            let ServerReq::ChangePassword {
                confirm_key,
                new_password,
            } = req
            else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected None, received: {req:?}"
                ))));
            };
            let time = app.time().await;

            let new_password =
                proccess_password(new_password, None).map_err(ResErr::InvalidPassword)?;

            let confirm_email = app
                .db
                .get_confirm_email_by_key(time, &confirm_key)
                .await
                .map_err(|err| match err {
                    DB404Err::NotFound => ServerErr::from(ResErr::NotFound),
                    DB404Err::DB(_) => ServerErr::DbErr,
                })?;
            let email = confirm_email.to_email;

            let new_password =
                hash_password(new_password).map_err(|_| ServerErr::InternalServerErr)?;

            let db_user = app
                .db
                .update_user_password_by_email(time, email, new_password)
                .await
                .map_err(|err| match err {
                    DB404Err::NotFound => ServerErr::from(ResErr::NotFound),
                    DB404Err::DB(_) => ServerErr::DbErr,
                })?;

            app.db
                .delete_session_user(db_user.id)
                .await
                .map_err(|_err| ServerErr::DbErr)?;

            // let user = match user {
            //     Ok(v) => v,
            //     Err(DB404Err::NotFound) => {
            //         return Ok(ServerRes::Ok);
            //     }
            //     Err(DB404Err::DB(_)) => {
            //         return Err(ServerErr::DbErr);
            //     }
            // };

            // verify_password(password, db_user.password.clone())
            //     .inspect_err(|err| trace!("passwords verification failed {err}"))
            //     .map_err(|_| ServerErr::ChangeUsernameErr(ChangeUsernameErr::WrongCredentials))?;

            // let exp = app.new_exp().await;
            // let result = app
            //     .db
            //     .add_confirm_email(time, &email, exp)
            //     .await
            //     .map_err(|err| ServerErr::DbErr)?;
            //
            // let confirm_key = result.id.key().to_string();
            //
            // app.send_email_change_password(time, &email, confirm_key)
            //     .await?;

            Ok(ServerRes::Ok)
        }
