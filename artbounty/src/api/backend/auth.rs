use surrealdb::types::ToSql;

use crate::api::app_state::AppState;
use crate::api::{
    AuthToken, ChangeUsernameErr, EmailChangeErr, EmailChangeNewErr, EmailChangeStage, EmailChangeTokenErr, Server404Err, ServerAddPostErr, ServerAuthErr, ServerDecodeInviteErr, ServerDesErr, ServerErr, ServerErrImg, ServerErrImgMeta, ServerLoginErr, ServerRegistrationErr, ServerReq, ServerRes, ServerSendInviteErr, ServerTokenErr, User, UserPost, UserPostFile, auth_token_get, hash_password, verify_password
};
use crate::db::DB404Err;
use crate::db::{AddUserErr, DBEmailIsTakenErr};
use crate::valid::auth::{proccess_email, proccess_password, proccess_username};
use axum::extract::State;
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

    let invite_token_decoded = app_state
        .db
        .get_invite_any_by_key(invite_token.clone())
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
            Ok(invite)
        })
        .inspect_err(|err| error!("failed to run use_invite {err}"))?;

    let email = invite_token_decoded.email;
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
            AddUserErr::EmailIsTaken(_) => ServerRegistrationErr::ServerRegistrationInvalidInput {
                username: None,
                email: Some("email is taken".to_string()),
                password: None,
            }
            .into(),
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
        .update_invite_used(time_ns, invite_token.clone())
        .await
        .inspect_err(|err| error!("failed to run use_invite {err}"))
        .map_err(|err| ServerErr::DbErr)?;

    let session = app_state
        .db
        .add_session(time_ns, &user.username)
        .await
        .map_err(|err| ServerErr::DbErr)?;

    let token = session.id.key.to_sql();

    Ok(ServerRes::SetAuthCookie { token })
}

pub async fn login(State(app): State<AppState>, req: ServerReq) -> Result<ServerRes, ServerErr> {
    let ServerReq::Login { email, password } = req else {
        return Err(
            ServerDesErr::ServerWrongInput(format!("expected Login, received: {req:?}")).into(),
        );
    };
    let time = app.clock.now().await;
    let time_ns = time;

    let user = app
        .db
        .get_user_by_email(email)
        .await
        .inspect_err(|err| trace!("user not found - {err}"))
        .map_err(|_| ServerErr::LoginErr(ServerLoginErr::WrongCredentials))?;

    verify_password(password, user.password)
        .inspect_err(|err| trace!("passwords verification failed {err}"))
        .map_err(|_| ServerErr::LoginErr(ServerLoginErr::WrongCredentials))?;

    let session = app
        .db
        .add_session(time_ns, &user.username)
        .await
        .map_err(|err| ServerErr::DbErr)?;

    let token = session.id.key.to_sql();

    Ok(ServerRes::SetAuthCookie { token })
}

pub async fn logout(
    State(app_state): State<AppState>,
    mut parts: http::request::Parts,
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
    type ResErr = ServerDecodeInviteErr;

    let ServerReq::ConfirmToken { token } = req else {
        return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
            "expected Register, received: {req:?}"
        ))));
    };
    let time_ns = app.clock.now().await;
    let invite_token = app
        .db
        .get_invite_any_by_key(token.clone())
        .await
        .map_err(|err| match err {
            DB404Err::DB(_) => ServerErr::DbErr,
            DB404Err::NotFound => ResErr::InviteNotFound.into(),
        })
        .and_then(|invite| {
            if invite.expires < time_ns {
                return Err(ResErr::InviteExpired.into());
            }
            if invite.used {
                return Err(ResErr::InviteUsed.into());
            }
            Ok(invite)
        })
        .inspect_err(|err| error!("failed to run use_invite {err}"))?;

    Ok(ServerRes::InviteToken {
        email: invite_token.email,
        created_at: invite_token.created_at,
        exp: invite_token.expires,
    })
}

pub async fn send_email_invite(
    State(app): State<AppState>,
    req: ServerReq,
) -> Result<ServerRes, ServerErr> {
    type ResErr = ServerSendInviteErr;

    let ServerReq::EmailAddress { email } = req else {
        return Err(ServerErr::from(ServerDesErr::ServerWrongInput(format!(
            "expected AddPost, received: {req:?}"
        ))));
    };

    let time = app.time().await;
    let address = app.get_address().await;
    let exp = app.new_exp().await;

    let email = proccess_email(email).map_err(|err| ResErr::InvalidEmail(err))?;

    let email_token = app.db.add_invite(time, email, exp).await;
    let confirm_token = match email_token {
        Err(DBEmailIsTakenErr::EmailIsTaken(_)) => {
            return Ok(ServerRes::Ok);
        }
        invite => invite.map_err(|_| ServerErr::DbErr),
    }?;
    trace!("result {confirm_token:?}");

    let link = format!(
        "{}{}",
        &address,
        crate::path::link_reg_finish(&confirm_token.id.key.to_sql(), None),
    );
    trace!("{link}");

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
    async fn api_auth_test() {
        crate::init_test_log();

        let app = ApiTestApp::new(1).await;
        let auth_token = app
            .register(0, "hey", "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();
        app
            .register_taken(0, "HEY@heyadora.com")
            .await;
        app
            .register_taken(0, " hey@heyadora.com")
            .await;
        // assert!(auth_token2.is_none());
        app.is_logged_in(0, &auth_token).await.unwrap();
        app.register_fail_expired_taken(0, 2, "hey2", "hey2@heyadora.com", "pas$word123456789")
            .await
            .unwrap();
        app.register_fail_404(0, "hey2").await.unwrap();
        app.register_fail_invalid(0, "pr", "prime@heyadora.com", "wowowowwoW12222pp")
            .await
            .unwrap();
        app.logout(0, &auth_token).await.unwrap();
        app.is_logged_out(0, &auth_token).await.unwrap();
        let auth_token = app
            .login(0, "hey@heyadora.com", "pas$word123456789")
            .await
            .unwrap();
        app.is_logged_in(0, &auth_token).await.unwrap();
    }
}
