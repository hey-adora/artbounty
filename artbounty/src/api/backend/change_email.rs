
        use crate::api::app_state::AppState;
        use crate::api::{
            AuthToken, EmailChangeErr, EmailChangeNewErr, EmailChangeStage, EmailChangeTokenErr,
            EmailToken, Server404Err, ServerAddPostErr, ServerAuthErr, ServerDecodeInviteErr,
            ServerDesErr, ServerErr, ServerErrImg, ServerErrImgMeta, ServerLoginErr,
            ServerRegistrationErr, ServerReq, ServerRes, ServerTokenErr, User, UserPost,
            UserPostFile, auth_token_get, decode_token, encode_token, hash_password,
            verify_password,
        };
        use crate::db::email_change::{DBChangeEmailErr, create_email_change_id};
        use crate::db::{AddUserErr, DBUser};
        use crate::db::{DB404Err, EmailIsTakenErr};
        use crate::valid::auth::{proccess_password, proccess_username};
        use axum::Extension;
        use axum::extract::State;
        // use axum_extra::extract::CookieJar;
        use http::header::COOKIE;
        use tracing::{debug, error, info, trace};

        pub async fn send_email_change(
            State(app): State<AppState>,
            auth_token: Extension<AuthToken>,
            db_user: Extension<DBUser>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            let ServerReq::None = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected None, received: {req:?}"
                ))));
            };

            let time = app.time().await;

            let (confirm_token, exp) = app.new_token(&db_user.email).await?;

            let email_change = app
                .db
                .add_email_change(
                    time,
                    db_user.id.clone(),
                    db_user.email.clone(),
                    confirm_token.clone(),
                    exp,
                )
                .await
                .map_err(|_| ServerErr::DbErr)?;

            let stage = EmailChangeStage::from(&email_change);
            match &stage {
                EmailChangeStage::ConfirmEmail { .. } => (),
                _ => {
                    return Err(ServerErr::EmailChange(EmailChangeErr::InvalidStage(
                        "email change is already initialized".to_string(),
                    )));
                }
            }
            // stage.is_stage(EmailChangeStage::ConfirmEmail, EmailChangeErr::InvalidStage)?;

            app.send_email_change(
                time,
                email_change.current.email.clone(),
                &email_change.id,
                email_change.current.token_raw,
                email_change.current.email,
                email_change.expires,
            )
            .await?;

            Ok(ServerRes::EmailChangeStage(stage))
        }

        pub async fn confirm_email_change(
            State(app_state): State<AppState>,
            auth_token: Extension<AuthToken>,
            db_user: Extension<DBUser>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            let ServerReq::ConfirmToken { token } = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected ConfirmKind, received: {req:?}"
                ))));
            };
            let time = app_state.clock.now().await;

            let result = app_state
                .db
                .get_email_change_by_current_token(time, db_user.id.clone(), token)
                .await
                .map_err(|e| match e {
                    DB404Err::NotFound => ServerErr::from(EmailChangeTokenErr::TokenInvalid),
                    DB404Err::DB(_) => ServerErr::DbErr,
                })?;

            let stage = EmailChangeStage::from(&result);
            match stage {
                EmailChangeStage::ConfirmEmail { .. } => (),
                stage => {
                    return Err(ServerErr::from(EmailChangeTokenErr::InvalidStage(format!(
                        "wrong stage, expected ConfirmEmail, got {stage}"
                    ))));
                }
            }

            // if result.current.token_raw == token

            // let (id, expires) = match stage {
            //     EmailChangeStage::ConfirmEmail { id, expires } => (id, expires),
            //     _ => {
            //         return Err(ServerErr::EmailChange(EmailChangeErr::InvalidStage(
            //             "email change is already initialized".to_string(),
            //         )));
            //     }
            // };

            // let stage = app_state.get_email_change_status(time, &db_user).await?;
            // let (email_change, stage) =
            //     .get_email_change_status_compare(
            //         time,
            //         &db_user,
            //         EmailChangeStage::ConfirmEmail,
            //         EmailChangeTokenErr::InvalidStage,
            //     )
            //     .await?;

            // if email_change.current.token_raw != token {
            //     return Err(EmailChangeTokenErr::TokenInvalid.into());
            // }

            let result = app_state
                .db
                .update_email_change_confirm_current(time, result.id.clone())
                .await
                .map_err(|_| ServerErr::DbErr)?;

            let stage = EmailChangeStage::from(&result);

            Ok(ServerRes::EmailChangeStage(stage))
        }

        pub async fn send_email_new(
            State(app): State<AppState>,
            auth_token: Extension<AuthToken>,
            db_user: Extension<DBUser>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            type ResErr = EmailChangeErr;

            let ServerReq::EmailAddressWithId { id, email } = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected AddPost, received: {req:?}"
                ))));
            };

            let time = app.time().await;

            let result = app
                .db
                .get_email_change(time, create_email_change_id(&id))
                .await
                .map_err(|e| match e {
                    DB404Err::NotFound => ServerErr::from(ResErr::InvalidStage(
                        "email change is not initialized".to_string(),
                    )),
                    DB404Err::DB(_) => ServerErr::DbErr,
                })?;

            let stage = EmailChangeStage::from(&result);
            let (id, expires) = match stage {
                EmailChangeStage::EnterNewEmail {
                    id,
                    old_email,
                    expires,
                } => (create_email_change_id(id), expires),
                err => {
                    return Err(ServerErr::EmailChange(ResErr::InvalidStage(format!(
                        "expected stage EnterNewEmail, got {err:?}"
                    ))));
                }
            };
            // let (email_change, stage) = app
            //     .get_email_change_status_compare(
            //         time,
            //         &db_user,
            //         EmailChangeStage::EnterNewEmail,
            //         EmailChangeNewErr::InvalidStage,
            //     )
            //     .await?;

            let (confirm_token, exp) = app.new_token(&db_user.email).await?;

            let result = app
                .db
                .update_email_change_add_new(time, id, email.clone(), confirm_token.clone())
                .await
                .map_err(|err| match err {
                    EmailIsTakenErr::EmailIsTaken(email) => {
                        ServerErr::from(EmailChangeNewErr::EmailIsTaken(email))
                    }
                    EmailIsTakenErr::DB(_) => ServerErr::DbErr,
                })?;

            // let stage = EmailChangeStage::from(&result);

            // let token = result
            //     .new
            //     .as_ref()
            //     .ok_or_else(|| EmailChangeNewErr::InvalidStage(format!("expected NewConfirm")))?;

            let stage = EmailChangeStage::from(&result);
            match &stage {
                EmailChangeStage::ConfirmNewEmail { .. } => (),
                err => {
                    return Err(ServerErr::EmailChange(ResErr::InvalidStage(format!(
                        "expected NewConfirm, got {err:?}"
                    ))));
                }
            }

            app.send_email_new(
                time,
                email.clone(),
                &result.id,
                result.new.as_ref().unwrap().token_raw.clone(),
                result.current.email,
                expires,
            )
            .await?;

            Ok(ServerRes::EmailChangeStage(stage))
        }

        pub async fn confirm_email_new(
            State(app): State<AppState>,
            auth_token: Extension<AuthToken>,
            db_user: Extension<DBUser>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            type ErrRes = EmailChangeTokenErr;
            // let _span = tracing::trace_span!("confirm_email_new").entered();
            let ServerReq::ConfirmToken { token } = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected ConfirmKind, received: {req:?}"
                ))));
            };
            let time = app.clock.now().await;
            trace!("time {}", time);

            let result = app
                .db
                .get_email_change_by_new_token(time, db_user.id.clone(), token)
                .await
                .map_err(|e| match e {
                    DB404Err::NotFound => ServerErr::from(ErrRes::TokenInvalid),
                    DB404Err::DB(_) => ServerErr::DbErr,
                })?;

            let stage = EmailChangeStage::from(&result);
            match stage {
                EmailChangeStage::ConfirmNewEmail { .. } => (),
                stage => {
                    return Err(ServerErr::from(ErrRes::InvalidStage(format!(
                        "wrong stage, expected ConfirmEmail, got {stage}"
                    ))));
                }
            }

            // let (email_change, stage) = app
            //     .get_email_change_status_compare(
            //         time,
            //         &db_user,
            //         EmailChangeStage::ConfirmNewEmail,
            //         EmailChangeTokenErr::InvalidStage,
            //     )
            //     .await?;

            // trace!("stage {}", stage);
            //
            // if email_change
            //     .new
            //     .as_ref()
            //     .map(|v| v.token_raw != token)
            //     .unwrap_or_default()
            // {
            //     return Err(EmailChangeTokenErr::TokenInvalid.into());
            // }

            let result = app
                .db
                .update_email_change_confirm_new(time, result.id.clone())
                .await
                .map_err(|_| ServerErr::DbErr)?;

            let stage = EmailChangeStage::from(&result);
            match stage {
                EmailChangeStage::ReadyToComplete { .. } => (),
                stage => {
                    return Err(ServerErr::from(ErrRes::InvalidStage(format!(
                        "wrong stage, expected ConfirmEmail, got {stage}"
                    ))));
                }
            }

            Ok(ServerRes::EmailChangeStage(stage))
        }

        pub async fn change_email(
            State(app): State<AppState>,
            auth_token: Extension<AuthToken>,
            db_user: Extension<DBUser>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            type ResErr = EmailChangeNewErr;

            let ServerReq::Id { id } = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected ChangeUsername, received: {req:?}"
                ))));
            };
            let time = app.clock.now().await;

            trace!("1");

            let email_change = app
                .db
                .get_email_change(time, create_email_change_id(&id))
                .await
                .map_err(|e| match e {
                    DB404Err::NotFound => ServerErr::from(ServerErr::from(ResErr::InvalidStage(
                        format!("state not initialized, expected ReadyToConfirm"),
                    ))),
                    DB404Err::DB(_) => ServerErr::DbErr,
                })?;

            trace!("2");

            let stage = EmailChangeStage::from(&email_change);
            match stage {
                EmailChangeStage::ReadyToComplete { .. } => (),
                stage => {
                    return Err(ServerErr::from(ResErr::InvalidStage(format!(
                        "wrong stage, expected ReadyToConfirm, got {stage}"
                    ))));
                }
            }

            trace!("3");

            // let (email_change, stage) = app
            //     .get_email_change_status_compare(
            //         time,
            //         &db_user,
            //         EmailChangeStage::ReadyToComplete,
            //         EmailChangeNewErr::InvalidStage,
            //     )
            //     .await?;

            let new = email_change
                .new
                .as_ref()
                .ok_or_else(|| ResErr::InvalidStage(format!("expected ReadyToConfirm")))?;

            trace!("5");

            let result = app
                .db
                .update_user_email(db_user.id.clone(), new.email.clone(), time)
                .await
                .map_err(|err| match err {
                    DBChangeEmailErr::EmailIsTaken(email) => {
                        ServerErr::from(ResErr::EmailIsTaken(email))
                    }
                    _ => ServerErr::DbErr,
                })?;

            trace!("6");

            let result = app
                .db
                .update_email_change_complete(time, email_change.id.clone())
                .await
                .map_err(|_| ServerErr::DbErr)?;

            trace!("7");

            let stage = EmailChangeStage::from(&result);
            match stage {
                EmailChangeStage::Complete { .. } => (),
                stage => {
                    return Err(ServerErr::from(ResErr::InvalidStage(format!(
                        "wrong stage, expected ReadyToConfirm, got {stage}"
                    ))));
                }
            }

            trace!("8");

            Ok(ServerRes::EmailChangeStage(stage))
        }

        // pub async fn confirm_action(
        //     State(app_state): State<AppState>,
        //     auth_token: Extension<AuthToken>,
        //     db_user: Extension<DBUser>,
        //     req: ServerReq,
        // ) -> Result<ServerRes, ServerErr> {
        //     let ServerReq::ConfirmToken { token } = req else {
        //         return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
        //             "expected ConfirmKind, received: {req:?}"
        //         ))));
        //     };
        //     let time = app_state.clock.now().await.as_nanos();
        //
        //
        //     Ok(ServerRes::Ok)
        // }

        pub async fn resend_email_change(
            State(app): State<AppState>,
            auth_token: Extension<AuthToken>,
            db_user: Extension<DBUser>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            type ResErr = EmailChangeNewErr;

            let ServerReq::Id { id } = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected None, received: {req:?}"
                ))));
            };

            let time = app.time().await;

            let result = app
                .db
                .get_email_change(time, create_email_change_id(&id))
                .await
                .map_err(|e| match e {
                    DB404Err::NotFound => ServerErr::from(ServerErr::from(ResErr::InvalidStage(
                        format!("state not initialized, expected ReadyToConfirm"),
                    ))),
                    DB404Err::DB(_) => ServerErr::DbErr,
                })?;

            let stage = EmailChangeStage::from(&result);
            match stage {
                EmailChangeStage::ConfirmEmail { .. } => (),
                stage => {
                    return Err(ServerErr::from(ResErr::InvalidStage(format!(
                        "wrong stage, expected ReadyToConfirm, got {stage}"
                    ))));
                }
            }

            // let (email_change, stage) =
            //     app.get_email_change_status(time, &db_user)
            //         .await?
            //         .ok_or(ServerErr::EmailChange(EmailChangeErr::InvalidStage(
            //             "email change not started".to_string(),
            //         )))?;

            // stage.is_stage(
            //     EmailChangeStage::ConfirmEmail,
            //     EmailChangeNewErr::InvalidStage,
            // )?;

            app.send_email_change(
                time,
                result.current.email.clone(),
                &result.id,
                result.current.token_raw,
                result.current.email,
                result.expires,
            )
            .await?;

            Ok(ServerRes::EmailChangeStage(stage))
        }

        pub async fn resend_email_new(
            State(app): State<AppState>,
            auth_token: Extension<AuthToken>,
            db_user: Extension<DBUser>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            type ResErr = EmailChangeNewErr;

            let ServerReq::Id { id } = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected None, received: {req:?}"
                ))));
            };

            let time = app.time().await;
            let result = app
                .db
                .get_email_change(time, create_email_change_id(&id))
                .await
                .map_err(|e| match e {
                    DB404Err::NotFound => ServerErr::from(ServerErr::from(ResErr::InvalidStage(
                        format!("state not initialized, expected ReadyToConfirm"),
                    ))),
                    DB404Err::DB(_) => ServerErr::DbErr,
                })?;

            let stage = EmailChangeStage::from(&result);
            match stage {
                EmailChangeStage::ConfirmNewEmail { .. } => (),
                stage => {
                    return Err(ServerErr::from(ResErr::InvalidStage(format!(
                        "wrong stage, expected ReadyToConfirm, got {stage}"
                    ))));
                }
            }

            let new = result
                .new
                .as_ref()
                .ok_or_else(|| ResErr::InvalidStage(format!("expected ReadyToConfirm")))?;

            // let (email_change, stage) =
            //     app.get_email_change_status(time, &db_user)
            //         .await?
            //         .ok_or(ServerErr::EmailChange(EmailChangeErr::InvalidStage(
            //             "email change not started".to_string(),
            //         )))?;
            //
            // stage.is_stage(
            //     EmailChangeStage::ConfirmNewEmail,
            //     EmailChangeNewErr::InvalidStage,
            // )?;
            // let new_email = email_change.new.unwrap().email;

            app.send_email_new(
                time,
                new.email.clone(),
                &result.id,
                new.token_raw.clone(),
                result.current.email,
                result.expires,
            )
            .await?;

            Ok(ServerRes::EmailChangeStage(stage))
        }

        pub async fn cancel_email_change(
            State(app): State<AppState>,
            auth_token: Extension<AuthToken>,
            db_user: Extension<DBUser>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            type ResErr = EmailChangeNewErr;

            let ServerReq::Id { id } = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected None, received: {req:?}"
                ))));
            };

            let time = app.time().await;
            // let stage =
            //     app.get_email_change_status(time, &db_user)
            //         .await?
            //         .ok_or(ServerErr::EmailChange(EmailChangeErr::InvalidStage(
            //             "email change not started".to_string(),
            //         )))?;
            let result = app
                .db
                .get_email_change(time, create_email_change_id(&id))
                .await
                .map_err(|e| match e {
                    DB404Err::NotFound => ServerErr::from(ServerErr::from(ResErr::InvalidStage(
                        format!("state not initialized, expected ReadyToConfirm"),
                    ))),
                    DB404Err::DB(_) => ServerErr::DbErr,
                })?;

            // let stage = EmailChangeStage::from(&result);
            // match stage {
            //     EmailChangeStage::ConfirmNewEmail { .. } => (),
            //     stage => {
            //         return Err(ServerErr::from(ResErr::InvalidStage(format!(
            //             "wrong stage, expected ReadyToConfirm, got {stage}"
            //         ))));
            //     }
            // }

            let result = app
                .db
                .update_email_change_complete(time, result.id.clone())
                .await
                .map_err(|_| ServerErr::DbErr)?;

            let stage = EmailChangeStage::from(&result);

            Ok(ServerRes::EmailChangeStage(stage))
        }

        pub async fn status_email_change(
            State(app): State<AppState>,
            auth_token: Extension<AuthToken>,
            db_user: Extension<DBUser>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            type ResErr = Server404Err;

            let ServerReq::Id { id } = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected None, received: {req:?}"
                ))));
            };

            let time = app.time().await;
            let result = app
                .db
                .get_email_change(time, create_email_change_id(&id))
                .await;

            let stage = match result {
                Ok(v) => EmailChangeStage::from(&v),
                Err(DB404Err::NotFound) => {
                    return Err(ResErr::NotFound.into());
                }
                Err(DB404Err::DB(_)) => {
                    return Err(ServerErr::DbErr);
                }
            };

            Ok(ServerRes::EmailChangeStage(stage))
        }
