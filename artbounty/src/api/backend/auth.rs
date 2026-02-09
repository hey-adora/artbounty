
        use crate::api::app_state::AppState;
        use crate::api::{
            AuthToken, ChangeUsernameErr, EmailChangeErr, EmailChangeNewErr, EmailChangeStage,
            EmailChangeTokenErr, EmailToken, Server404Err, ServerAddPostErr, ServerAuthErr,
            ServerDecodeInviteErr, ServerDesErr, ServerErr, ServerErrImg, ServerErrImgMeta,
            ServerLoginErr, ServerRegistrationErr, ServerReq, ServerRes, ServerTokenErr, User,
            UserPost, UserPostFile, auth_token_get, decode_token, encode_token, hash_password,
            verify_password,
        };
        use crate::db::AddUserErr;
        use crate::db::DB404Err;
        use crate::valid::auth::{proccess_password, proccess_username};
        use axum::extract::State;
        // use axum_extra::extract::CookieJar;
        use http::header::COOKIE;
        use tracing::{debug, error, info, trace};

        pub async fn register(
            State(app_state): State<AppState>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            let ServerReq::Register {
                username,
                invite_token,
                password,
            } = req
            else {
                return Err(ServerDesErr::ServerWrongInput(format!(
                    "expected Register, received: {req:?}"
                ))
                .into());
            };
            let time_ns = app_state.clock.now().await;
            let secret = app_state.get_secret().await;

            let invite_token_decoded = app_state
                .db
                .get_invite_any_by_token(
                    // DBEmailTokenKind::RequestConfirmRegistrationEmail,
                    &invite_token,
                )
                .await
                .map_err(|err| match err {
                    DB404Err::DB(_) => ServerErr::DbErr,
                    DB404Err::NotFound => ServerRegistrationErr::TokenNotFound.into(),
                })
                .and_then(|invite| {
                    if invite.expires < time_ns {
                        return Err(ServerRegistrationErr::TokenExpired.into());
                    }
                    if invite.used {
                        return Err(ServerRegistrationErr::TokenUsed.into());
                    }
                    decode_token::<EmailToken>(&secret, &invite_token, false)
                        .map_err(|err| ServerRegistrationErr::ServerJWT(err.to_string()).into())
                })
                .inspect_err(|err| error!("failed to run use_invite {err}"))?;

            let email = invite_token_decoded.claims.email;
            let username = proccess_username(username);
            let password = proccess_password(password, None)
                .and_then(|pss| hash_password(pss).map_err(|_| "hasher error".to_string()));

            let (Ok(username), Ok(password)) = (&username, &password) else {
                return Err(ServerErr::from(
                    ServerRegistrationErr::ServerRegistrationInvalidInput {
                        username: username.err(),
                        email: None,
                        password: password.err(),
                    },
                ));
            };

            let user = app_state
                .db
                .add_user(time_ns, username, email, password)
                .await
                .map_err(|err| match err {
                    AddUserErr::EmailIsTaken(_) => {
                        ServerRegistrationErr::ServerRegistrationInvalidInput {
                            username: None,
                            email: Some("email is taken".to_string()),
                            password: None,
                        }
                        .into()
                    }
                    AddUserErr::UsernameIsTaken(_) => {
                        ServerRegistrationErr::ServerRegistrationInvalidInput {
                            username: Some("username is taken".to_string()),
                            email: None,
                            password: None,
                        }
                        .into()
                    }
                    _ => ServerErr::DbErr,
                })?;

            let result = app_state
                .db
                .update_invite_used(time_ns, &invite_token)
                .await
                .inspect_err(|err| error!("failed to run use_invite {err}"))
                .map_err(|err| ServerErr::DbErr)?;

            let token = encode_token(&secret, AuthToken::new(username, time_ns))
                .inspect_err(|err| error!("jwt exploded {err}"))
                .map_err(|_| ServerRegistrationErr::ServerCreateCookieErr)?;

            // let (token, cookie) = create_cookie(&app_state.settings.auth.secret, &user.username, time)
            //     .map_err(|_| ServerRegistrationErr::ServerCreateCookieErr)?;

            let _session = app_state
                .db
                .add_session(time_ns, token.clone(), &user.username)
                .await
                .map_err(|err| ServerErr::DbErr)?;

            Ok(ServerRes::SetAuthCookie { token })
        }

        pub async fn login(
            State(app): State<AppState>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            let ServerReq::Login { email, password } = req else {
                return Err(ServerDesErr::ServerWrongInput(format!(
                    "expected Login, received: {req:?}"
                ))
                .into());
            };
            let time = app.clock.now().await;
            let time_ns = time;
            let secret = app.get_secret().await;

            let user = app
                .db
                .get_user_by_email(email)
                .await
                .inspect_err(|err| trace!("user not found - {err}"))
                .map_err(|_| ServerErr::LoginErr(ServerLoginErr::WrongCredentials))?;

            verify_password(password, user.password)
                .inspect_err(|err| trace!("passwords verification failed {err}"))
                .map_err(|_| ServerErr::LoginErr(ServerLoginErr::WrongCredentials))?;

            let token = encode_token(&secret, AuthToken::new(&user.username, time))
                .inspect_err(|err| error!("jwt exploded {err}"))
                .map_err(|_| ServerRegistrationErr::ServerCreateCookieErr)?;

            // let (token, cookie) = create_cookie(&app_state.settings.auth.secret, &user.username, time)
            //     .map_err(|err| {
            //         ServerErr::ServerLoginErr(ServerLoginErr::ServerCreateCookieErr(err.to_string()))
            //     })?;

            let _session = app
                .db
                .add_session(time_ns, token.clone(), &user.username)
                .await
                .map_err(|err| ServerErr::DbErr)?;

            Ok(ServerRes::SetAuthCookie { token })
        }

        pub async fn logout(
            State(app_state): State<AppState>,
            mut parts: http::request::Parts,
            // jar: CookieJar,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            let ServerReq::None = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected None, received: {req:?}"
                ))));
            };

            let token = auth_token_get(&mut parts.headers, COOKIE).ok_or(ServerErr::AuthErr(
                ServerAuthErr::ServerUnauthorizedNoCookie,
            ))?;
            // {
            //     let r = parts.headers;
            //     let r2 = jar.get(AUTHORIZATION.as_str());
            //     trace!("headers comparison {r:?}");
            // }
            // trace!("headers comparison 1111 {jar:?} 222222 {headers:?}");
            // let token = auth_token_get(&mut parts.headers);
            // let token = jar
            //     .get(AUTHORIZATION.as_str())
            //     // .map(|v| v.value().to_string())
            //     .inspect(|v| trace!("logout token raw {v:?}"))
            //     .ok_or(ServerErr::ServerAuthErr(
            //         ServerAuthErr::ServerUnauthorizedNoCookie,
            //     ))
            //     .map(|v| cut_cookie(v.value(), COOKIE_PREFIX, "").to_string())?;

            trace!("logout token {token}");

            app_state
                .db
                .get_session(&token)
                .await
                .map_err(|_err| ServerErr::from(ServerAuthErr::ServerUnauthorizedInvalidCookie))?;

            app_state
                .db
                .delete_session(token)
                .await
                .map_err(|_err| ServerErr::from(ServerAuthErr::ServerUnauthorizedInvalidCookie))?;

            Ok(ServerRes::DeleteAuthCookie)
        }

        pub async fn decode_email_token(
            State(app): State<AppState>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            let ServerReq::ConfirmToken { token } = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected Register, received: {req:?}"
                ))));
            };
            let secret = app.get_secret().await;

            let token = decode_token::<EmailToken>(&secret, token, false)
                .map_err(|err| ServerDecodeInviteErr::JWT(err.to_string()))?;

            Ok(ServerRes::InviteToken(token.claims))
        }
