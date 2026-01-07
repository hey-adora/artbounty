use leptos::Params;
use leptos::tachys::reactive_graph::bind::GetValue;
use leptos::{html, prelude::*};
use leptos_router::NavigateOptions;
use leptos_router::hooks::{query_signal, use_query, use_query_map};
use leptos_router::params::{Params, ParamsError};
use web_sys::SubmitEvent;

use crate::api::{Api, ApiWeb, ServerErr, ServerRegistrationErr, ServerRes};
use crate::path::{self, link_reg_finish, link_reg_invite, link_user};
use crate::valid::auth::{proccess_email, proccess_password, proccess_username};
use crate::view::app::components::nav::Nav;
use crate::view::app::{Acc, GlobalState};
use crate::view::toolbox::leptos_helpers::ToQueryField;
use crate::view::toolbox::prelude::*;
use tracing::{error, trace};

// #[derive(Params, PartialEq, Clone)]
// pub struct RegParams {
//     pub err_general: Option<String>,
//     pub err_username: Option<String>,
//     pub err_token: Option<String>,
//     pub err_password: Option<String>,
//     pub token: Option<String>,
//     pub email: Option<String>,
//     pub kind: Option<RegStage>,
// }

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    PartialOrd,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum RegStage {
    #[default]
    None,
    CheckEmail,
    Reg,
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    PartialOrd,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum RegQueryFields {
    #[default]
    None,
    ErrGeneral,
    ErrUsername,
    ErrToken,
    ErrPassword,
    Stage,
    Token,
    Email,
}



#[derive(Clone, Copy)]
pub struct Register {
    pub err_general: RwQuery<String>,
    pub err_username: RwQuery<String>,
    pub err_token: RwQuery<String>,
    pub err_password: RwQuery<String>,
    pub stage: RwQuery<RegStage>,
    pub email: RwQuery<String>,
    pub token: RwQuery<String>,
    pub token_decoded: LocalResource<String>,
    pub on_reg: StoredValue<Box<dyn Fn(SubmitEvent) + Sync + Send + 'static>>,
    pub on_invite: StoredValue<Box<dyn Fn(SubmitEvent) + Sync + Send + 'static>>,
}

pub fn use_register(
    api: ApiWeb,
    input_username: NodeRef<html::Input>,
    input_email: NodeRef<html::Input>,
    input_password: NodeRef<html::Input>,
    input_password_confirmatoin: NodeRef<html::Input>,
) -> Register {
    let global_state = expect_context::<GlobalState>();
    // let query = use_query::<RegParams>();

    let navigate = leptos_router::hooks::use_navigate();

    let err_general = RwQuery::<String>::new(RegQueryFields::ErrGeneral.to_string());
    let err_username = RwQuery::<String>::new(RegQueryFields::ErrUsername.to_string());
    let err_token = RwQuery::<String>::new(RegQueryFields::ErrToken.to_string());
    let err_password = RwQuery::<String>::new(RegQueryFields::ErrPassword.to_string());
    let stage = RwQuery::<RegStage>::new(RegQueryFields::Stage.to_string());
    let token = RwQuery::<String>::new(RegQueryFields::Token.to_string());
    let email = RwQuery::<String>::new(RegQueryFields::Email.to_string());

    // let email = RwQuery::<String>::new("email");
    let token_decoded = LocalResource::new(move || async move {
        let token = token.get();
        if token.is_empty() {
            return String::new();
        }
        let result = api.decode_invite(token).send_native().await;

        match result {
            Ok(ServerRes::InviteToken(token)) => token.email,
            Ok(res) => {
                format!("error, expected InviteToken, received: {res:?}")
            }
            Err(err) => err.to_string(),
        }
    });

    let on_invite = {
        let navigate = navigate.clone();
        move |e: SubmitEvent| {
            e.prevent_default();
            let navigate = navigate.clone();

            let Some(email_field) = input_email.get_untracked() else {
                return;
            };

            let email_value = email_field.value();
            let email_value = match proccess_email(&email_value) {
                Ok(email) => {
                    err_general.clear();
                    Some(email)
                }
                Err(err) => {
                    error!("on_invite email \"{email_value}\" error: {err}");
                    err_general.set(err);
                    None
                }
            };

            let Some(email_value) = email_value else {
                return;
            };
            let email_value_clone = email_value.clone();

            api.send_email_invite(email_value_clone)
                .send_web(move |result| {
                    let email = email_value.clone();
                    let navigate = navigate.clone();

                    async move {
                        match result {
                            Ok(ServerRes::Ok) => {
                                // let result = api.profile().send_native().await;
                                // invite_completed.set(email.clone());
                                navigate(
                                    &path::link_reg_check_email(email),
                                    NavigateOptions {
                                        ..Default::default()
                                    },
                                );
                            }
                            Ok(res) => {
                                error!("expected Ok, received {res:?}");
                                err_general.set(format!("expected Ok, received {res:?}"));
                            }

                            Err(err) => {
                                error!("get invite err: {err}");
                                err_general.set(err.to_string());
                            }
                        }
                    }
                });
        }
    };

    let on_register = move |e: SubmitEvent| {
        e.prevent_default();
        let (Some(username), Some(password), Some(password_confirmation)) = (
            input_username.get_untracked(),
            // register_email.get(),
            input_password.get_untracked(),
            input_password_confirmatoin.get_untracked(),
        ) else {
            return;
        };

        let username_value = username.value();
        let username_value = match proccess_username(username_value) {
            Ok(v) => {
                err_username.clear();
                Some(v)
            }
            Err(err) => {
                let err = format!("on_register username input error: {err}");
                error!(err);
                err_username.set(err);
                None
            }
        };
        // let username = proccess_username();
        // err_username.set(username.clone().err().unwrap_or_default());
        // let email = proccess_email(email.value());
        // let password = proccess_password(password.value(), Some(password_confirmation.value()));

        // register_email_err.set(email.clone().err().unwrap_or_default());

        // err_password.set(password.clone().err().unwrap_or_default());

        let password_value = password.value();
        let password_confirmation_value = password_confirmation.value();
        let password_value =
            match proccess_password(password_value, Some(password_confirmation_value)) {
                Ok(v) => {
                    err_password.clear();
                    Some(v)
                }
                Err(err) => {
                    error!("on_register password input error: {err}");
                    err_password.set(err);
                    None
                }
            };

        if !token.is_some_untracked() {
            err_general.set(String::from("token is missing from; invalid link"));
            return;
        } else {
            err_general.clear();
        }

        let (Some(username), Some(password)) = (username_value, password_value) else {
            return;
        };

        api.register(username, token.get_untracked(), password)
            .send_web(move |result| {
                // let navigate = navigate.clone();
                async move {
                    let err: Result<(), String> = match result {
                        Ok(ServerRes::Ok) => {
                            let res = global_state.update_auth_now().await;
                            match res {
                                Ok(ServerRes::User { username }) => {
                                    let result = global_state.update_auth_now().await;
                                    match result {
                                        Ok(S) => Ok(()),
                                        Err(err) => Err(err.to_string()),
                                    }
                                }
                                res => Err(format!("expected User, received {res:?}")),
                            }
                        }
                        Ok(res) => Err(format!("error, expected OK, received: {res:?}")),
                        Err(ServerErr::RegistrationErr(ServerRegistrationErr::TokenExpired)) => {
                            Err("This invite link is already expired.".to_string())
                        }
                        Err(ServerErr::RegistrationErr(ServerRegistrationErr::TokenUsed)) => {
                            Err("This invite link was already used.".to_string())
                        }
                        Err(ServerErr::RegistrationErr(ServerRegistrationErr::TokenNotFound)) => {
                            Err("This invite link is invalid.".to_string())
                        }
                        Err(err) => Err(err.to_string()),
                    };
                    if let Err(err) = err {
                        error!(err);
                        err_general.set(err);
                    }
                }
            });
    };

    Register {
        err_general,
        err_username,
        err_token,
        err_password,
        email,
        stage,
        token,
        token_decoded,
        on_invite: StoredValue::new(Box::new(on_invite)),
        on_reg: StoredValue::new(Box::new(on_register)),
    }
}

pub fn build_query_getter<QueryInput, MapFnOutput, MapFn>(
    query: Memo<Result<QueryInput, ParamsError>>,
    f: MapFn,
) -> impl Fn() -> MapFnOutput
where
    QueryInput: Params + Sync + Send + Clone + 'static,
    MapFnOutput: Sync + Send + Default + 'static,
    MapFn: Fn(&QueryInput) -> Option<MapFnOutput> + Clone,
{
    let fn_get_token = move || {
        let f = f.clone();
        query.with(|v| v.as_ref().ok().and_then(f).unwrap_or_default())
    };

    fn_get_token
}
