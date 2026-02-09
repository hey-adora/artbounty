
        use axum::{Extension, extract::State};
        use thiserror::Error;
        use tracing::{debug, trace};

        use crate::{
            api::{
                AuthToken, ChangeUsernameErr, ServerDesErr, ServerErr, ServerReq, ServerRes,
                app_state::AppState, verify_password,
            },
            db::{DBChangeUsernameErr, DBUser},
        };

        pub async fn change_username(
            State(app_state): State<AppState>,
            auth_token: Extension<AuthToken>,
            db_user: Extension<DBUser>,
            req: ServerReq,
        ) -> Result<ServerRes, ServerErr> {
            let ServerReq::ChangeUsername { username, password } = req else {
                return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
                    "expected ChangeUsername, received: {req:?}"
                ))));
            };
            let time = app_state.clock.now().await;
            debug!("step 1");

            verify_password(password, db_user.password.clone())
                .inspect_err(|err| trace!("passwords verification failed {err}"))
                .map_err(|_| ServerErr::ChangeUsernameErr(ChangeUsernameErr::WrongCredentials))?;
            debug!("step 2");

            let result = app_state
                .db
                .update_user_username(db_user.id.clone(), username, time)
                .await
                .map_err(|err| match err {
                    DBChangeUsernameErr::DB(err) => ServerErr::DbErr,
                    DBChangeUsernameErr::UsernameIsTaken(username) => {
                        ServerErr::ChangeUsernameErr(ChangeUsernameErr::UsernameIsTaken(username))
                    }
                    DBChangeUsernameErr::NotFound => {
                        ServerErr::ChangeUsernameErr(ChangeUsernameErr::NotFound)
                    }
                })?;

            Ok(ServerRes::User {
                username: result.username,
            })
        }
