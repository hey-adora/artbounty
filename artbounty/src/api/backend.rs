
    use crate::api::app_state::AppState;
    use crate::api::{
        AuthToken, ChangeUsernameErr, EmailChangeErr, EmailChangeNewErr, EmailChangeStage,
        EmailChangeTokenErr, EmailToken, Server404Err, ServerAddPostErr, ServerAuthErr,
        ServerDecodeInviteErr, ServerDesErr, ServerErr, ServerErrImg, ServerErrImgMeta,
        ServerLoginErr, ServerRegistrationErr, ServerReq, ServerRes, ServerTokenErr, User,
        UserPost, UserPostFile, auth_token_get, decode_token, encode_token, hash_password,
        verify_password,
    };
    use crate::db::email_change::create_email_change_id;
    use crate::db::{AddUserErr, email_change::DBChangeEmailErr};
    use crate::db::{DB404Err, DBChangeUsernameErr, create_user_id};
    use crate::db::{DBUser, EmailIsTakenErr};
    use crate::db::{DBUserPostFile, email_change::DBEmailChange};
    use crate::path::{
        link_settings_form_email_current_confirm, link_settings_form_email_new_confirm,
    };
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
    use surrealdb::RecordId;
    use tokio::fs;
    use tracing::{debug, error, info, trace};

    pub mod auth;
    pub mod change_username;
    pub mod change_email;
    pub mod change_password;
    pub mod post_like;
    pub mod post_comment;

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
            username: db_user.username.clone(),
            email: db_user.email.clone(),
        })
    }

    pub async fn get_post(
        State(app_state): State<AppState>,
        req: ServerReq,
    ) -> Result<ServerRes, ServerErr> {
        let ServerReq::PostId { post_id } = req else {
            return Err(ServerDesErr::ServerWrongInput(format!(
                "expected GetPost, received: {req:?}"
            ))
            .into());
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

    pub async fn add_post(
        State(app): State<AppState>,
        auth_token: axum::Extension<AuthToken>,
        req: ServerReq,
    ) -> Result<ServerRes, ServerErr> {
        let ServerReq::AddPost {
            title,
            description,
            files,
        } = req
        else {
            return Err(ServerDesErr::ServerWrongInput(format!(
                "expected AddPost, received: {req:?}"
            ))
            .into());
        };
        let time = app.clock.now().await;
        let file_path = app.get_file_path().await;

        let title = proccess_post_title(title)
            .map_err(|err| ServerAddPostErr::InvalidTitle(err.to_string()))?;
        let description = proccess_post_description(description)
            .map_err(|err| ServerAddPostErr::InvalidDescription(err.to_string()))?;

        let (files, errs) = files
            .into_iter()
            .map(|v| {
                let path = v.path;
                let img_data_for_thumbnail = v.data.clone();
                let img_data_for_org = v.data;
                ImageReader::new(Cursor::new(img_data_for_thumbnail))
                    .with_guessed_format()
                    .inspect_err(|err| error!("error guesing the format {err}"))
                    .map_err(|err| ServerErrImg::ServerImgUnsupportedFormat(err.to_string()))
                    .and_then(|v| {
                        let img_format = v.format().ok_or(
                            ServerErrImg::ServerImgUnsupportedFormat("uwknown".to_string()),
                        )?;
                        v.decode()
                            .inspect_err(|err| error!("error decoding img {err}"))
                            .map_err(|err| ServerErrImg::ServerImgDecodeFailed(err.to_string()))
                            .map(|img| (img_format, img))
                    })
                    .and_then(|(img_format, img)| {
                        let width = img.width();
                        let height = img.height();
                        webp::Encoder::from_image(&img)
                            .inspect_err(|err| error!("failed to create webp encoder {err}"))
                            .map_err(|err| {
                                ServerErrImg::ServerImgWebPEncoderCreationFailed(err.to_string())
                            })
                            .and_then(|encoder| {
                                encoder
                                    .encode_simple(false, 90.0)
                                    .inspect_err(|err| {
                                        error!("failed to create webp encoder {err:?}")
                                    })
                                    .map_err(|err| {
                                        ServerErrImg::ServerImgWebPEncodingFailed(format!(
                                            "{err:?}"
                                        ))
                                    })
                            })
                            .map(|img| (img_format, (width, height), img))
                    })
                    .and_then(|(img_format, (width, height), img_data_thumbnail)| {
                        let img_format = img_format.extensions_str()[0];
                        let mut img_data_org = img_data_for_org;
                        FileExtension::from_str(img_format)
                            .map_err(|_| {
                                ServerErrImg::ServerImgUnsupportedFormat(img_format.to_string())
                            })
                            .and_then(|img_format| {
                                little_exif::metadata::Metadata::clear_metadata(
                                    &mut img_data_org,
                                    img_format,
                                )
                                .inspect_err(|err| error!("failed to read metadata {err:?}"))
                                .map_err(|err| {
                                    ServerErrImg::ServerImgMetadataReadFail(err.to_string())
                                })
                            })
                            .map(|_| {
                                (
                                    DBUserPostFile {
                                        extension: img_format.to_string(),
                                        hash: format!("{:X}", gxhash128(&img_data_org, 0)),
                                        width,
                                        height,
                                    },
                                    img_data_org,
                                    img_data_thumbnail.to_vec(),
                                )
                            })
                    })
                    .map_err(|err| ServerErrImgMeta { path, err })
            })
            .fold(
                (
                    Vec::<(DBUserPostFile, Vec<u8>, Vec<u8>)>::new(),
                    Vec::<ServerErrImgMeta>::new(),
                ),
                |(mut oks, mut errs), file| {
                    match file {
                        Ok(v) => {
                            oks.push(v);
                        }
                        Err(v) => {
                            errs.push(v);
                        }
                    }

                    (oks, errs)
                },
            );
        if !errs.is_empty() {
            return Err(ServerAddPostErr::ServerImgErr(errs).into());
        }

        let root_path = Path::new(&file_path);
        let mut output_imgs = Vec::<UserPostFile>::new();
        for file in &files {
            let file_path = root_path.join(format!("{}.{}", &file.0.hash, &file.0.extension));
            if file_path.exists() {
                trace!(
                    "file already exists {}",
                    file_path.to_str().unwrap_or("err")
                );
                output_imgs.push(file.0.clone().into());
                continue;
            }

            trace!("saving {}", file_path.to_str().unwrap_or("err"));
            (match fs::write(&file_path, &file.1).await {
                Ok(v) => Ok(v),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    fs::create_dir_all(&root_path)
                        .await
                        .inspect(|_| trace!("created img output dir {:?}", &file_path))
                        .inspect_err(|err| error!("error creating img output dir {err}"))
                        .map_err(|err| {
                            ServerAddPostErr::ServerDirCreationFailed(err.to_string())
                        })?;
                    fs::write(&file_path, &file.1).await
                }
                Err(err) => {
                    //
                    Err(err)
                }
            })
            .inspect_err(|err| error!("failed to save img to disk {err:?}"))
            .map_err(|err| ServerAddPostErr::ServerFSErr(err.to_string()))?;
            output_imgs.push(file.0.clone().into());
        }

        let post_files = files
            .into_iter()
            .map(|v| v.0)
            .collect::<Vec<DBUserPostFile>>();
        let post = app
            .db
            .add_post(
                time,
                &auth_token.username,
                &title,
                &description,
                0,
                post_files,
            )
            .await
            .inspect_err(|err| error!("failed to save images {err:?}"))
            .map_err(|_| ServerErr::DbErr)?;

        Ok(ServerRes::Post(post.into()))
    }

    // email change

    pub async fn send_email_invite(
        State(app): State<AppState>,
        req: ServerReq,
    ) -> Result<ServerRes, ServerErr> {
        let ServerReq::EmailAddress { email } = req else {
            return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                "expected AddPost, received: {req:?}"
            ))));
        };

        let time = app.time().await;
        let address = app.get_address().await;
        let (confirm_token, exp) = app.new_token(&email).await?;
        trace!("email token created: {confirm_token}");

        let email_token = app.db.add_invite(time, confirm_token, email, exp).await;
        let confirm_token = match email_token {
            Err(EmailIsTakenErr::EmailIsTaken(_)) => {
                return Ok(ServerRes::Ok);
            }
            invite => invite.map_err(|_| ServerErr::DbErr),
        }?;
        trace!("result {confirm_token:?}");

        let link = format!(
            "{}{}",
            &address,
            crate::path::link_reg_finish(&confirm_token.token_raw, None),
        );
        trace!("{link}");

        Ok(ServerRes::Ok)
    }

    // pub async fn send_email_new(
    //     State(app_state): State<AppState>,
    //     auth_token: Extension<AuthToken>,
    //     db_user: Extension<DBUser>,
    //     req: ServerReq,
    // ) -> Result<ServerRes, ServerErr> {
    //     let ServerReq::EmailAddress { email } = req else {
    //         return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
    //             "expected AddPost, received: {req:?}"
    //         ))));
    //     };
    //
    //     let time = app_state.time().await;
    //
    //     // let result = app_state
    //     //     .db
    //     //     .get_email_confirm(time, DBEmailTokenKind::RequestChangeEmail, &new_email, 1)
    //     //     .await
    //     //     .map_err(|err| match err {
    //     //         DB404Err::NotFound => ServerErr::from(RequestEmailChangeErr),
    //     //     });
    //
    //     // let email_confirm = app_state
    //     //     .db
    //     //     .get_invite_valid(
    //     //         time,
    //     //         // DBEmailTokenKind::RequestChangeEmail,
    //     //         &db_user.email,
    //     //         // 1,
    //     //     )
    //     //     // .get_email_confirm_by_token(DBEmailTokenKind::RequestChangeEmail, confirm_token)
    //     //     .await
    //     //     .map_err(move |err| match err {
    //     //         DB404Err::NotFound => ServerErr::from(RequestEmailChangeErr::ConfirmTokenNotFound),
    //     //         DB404Err::DB(_err) => ServerErr::DbErr,
    //     //     })
    //     //     .and_then(|v| {
    //     //         // rewrite this nonsense
    //     //         if v.used {
    //     //             Ok(v)
    //     //         } else {
    //     //             Err(ServerErr::from(RequestEmailChangeErr::ConfirmTokenNotUsed))
    //     //         }
    //     //     })?;
    //     //
    //     // let (confirm_token, exp) = app_state.new_confirm_token(&new_email).await?;
    //     //
    //     // let result = app_state
    //     //     .db
    //     //     .add_invite(
    //     //         time,
    //     //         DBEmailTokenKind::RequestConfirmNewEmail,
    //     //         confirm_token,
    //     //         &new_email,
    //     //         exp,
    //     //     )
    //     //     .await
    //     //     .map_err(|err| match err {
    //     //         AddInviteErr::EmailIsTaken(email) => {
    //     //             ServerErr::RequestEmailChange(RequestEmailChangeErr::EmailIsTaken(email))
    //     //         }
    //     //         _ => ServerErr::DbErr,
    //     //     });
    //     //
    //     // let result = app_state
    //     //     .db
    //     //     .update_invite_used(time, email_confirm.token_raw, 2)
    //     //     .await
    //     //     .map_err(|_| ServerErr::DbErr)?;
    //
    //     // let (confirm_token, exp) = app_state.new_confirm_token(&new_email).await?;
    //     // trace!("email token created: {confirm_token}");
    //     //
    //     // let result = app_state
    //     //     .db
    //     //     .add_email_confirm(
    //     //         time,
    //     //         DBEmailTokenKind::RequestConfirmNewEmail,
    //     //         confirm_token,
    //     //         new_email,
    //     //         exp,
    //     //     )
    //     //     .await;
    //
    //     // let email_token = app_state
    //     //     .db
    //     //     .add_email_invite_token(time, confirm_token, email, exp)
    //     //     .await;
    //     // let confirm_token = match email_token {
    //     //     Err(AddEmailInviteTokenErr::EmailIsTaken(_)) => {
    //     //         return Ok(ServerRes::Ok);
    //     //     }
    //     //     invite => invite.map_err(|_| ServerErr::ServerDbErr),
    //     // }?;
    //     // trace!("result {confirm_token:?}");
    //     //
    //     // let link = format!(
    //     //     "{}{}",
    //     //     &app_state.settings.site.address,
    //     //     crate::path::link_reg(&confirm_token.token_raw),
    //     // );
    //     // trace!("{link}");
    //
    //     Ok(ServerRes::Ok)
    // }
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
                // let response = next.run(req).await;
                //     let status = response.status();
                // if status == StatusCode::OK
                // {
                //
                //     // TODO just make new middleware for optional auth
                // let body = response.body();
                //     body.dat;
                // // let bytes = axum::body::to_bytes(body, 255).await;
                //
                // }
                // return response;
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
        // let token = jar
        //     .get(AUTHORIZATION.as_str())
        //     .ok_or(ServerAuthErr::ServerUnauthorizedNoCookie)
        //     .inspect(|v| trace!("CHECK_AUTH COOKIE: {v:?}"))
        //     .map(|v| cut_cookie(v.value(), COOKIE_PREFIX, "").to_string())?;

        trace!("CHECKING AUTH SESSION");
        let session = app.db.get_session(&token).await.map_err(|err| match err {
            DB404Err::NotFound => ServerErr::from(ServerAuthErr::ServerUnauthorizedInvalidCookie),
            _ => ServerErr::DbErr,
        })?;

        let token = match decode_token::<AuthToken>(&secret, &token, false) {
            Ok(v) => v,
            Err(err) => {
                error!("invalid token was stored {err}");
                app.db
                    .delete_session(token)
                    .await
                    .map_err(|err| ServerErr::DbErr)?;
                return Err(ServerErr::from(
                    ServerAuthErr::ServerUnauthorizedInvalidCookie,
                ));
            }
        };

        Ok((token.claims, session.user))
    }
