use crate::{
    api::{
        Api, ApiWeb, ChangePasswordErr, ChangeUsernameErr, PasswordChangeStage, ServerErr,
        ServerRes,
    },
    valid::auth::{proccess_email, proccess_password, proccess_username},
    view::{
        app::{GlobalState, hook::use_email_change::BtnStage},
        toolbox::prelude::*,
    },
};
use leptos::{html, prelude::*};
use tracing::{debug, error, info, trace};
use web_sys::{HtmlInputElement, SubmitEvent};

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
#[strum(serialize_all = "PascalCase")]
pub enum ChangePasswordBtnStage {
    #[default]
    None,
    Send,
    ReSend,
    Confirm,
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
pub enum ChangePasswordFormStage {
    #[default]
    None,
    Send,
    Check,
    Confirm,
    Finish,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    strum::EnumIs,
)]
#[strum(serialize_all = "lowercase")]
pub enum ChangePasswordQueryFields {
    ErrGeneral,
    ErrPassword,
    FormStage,
    Token,
    Email,
}

#[derive(Clone, Copy)]
pub struct ChangePassword {
    pub err_general: RwQuery<String>,
    pub email: RwQuery<String>,
    pub form_stage: RwQuery<ChangePasswordFormStage>,
    pub btn_stage: StoredValue<Box<dyn Fn() -> ChangePasswordBtnStage + Sync + Send + 'static>>,
    pub on_change: StoredValue<Box<dyn Fn(SubmitEvent) + Sync + Send + 'static>>,
    // pub token: RwQuery<String>,
}

pub fn use_password_change(
    api: ApiWeb,
    input_email: NodeRef<html::Input>,
    input_password: NodeRef<html::Input>,
    input_password_confirmatoin: NodeRef<html::Input>,
) -> ChangePassword {
    let global_state = expect_context::<GlobalState>();

    let err_general = RwQuery::<String>::new(ChangePasswordQueryFields::ErrGeneral.to_string());
    let err_password = RwQuery::<String>::new(ChangePasswordQueryFields::ErrPassword.to_string());
    let q_token = RwQuery::<String>::new(ChangePasswordQueryFields::Token.to_string());
    let q_stage =
        RwQuery::<ChangePasswordFormStage>::new(ChangePasswordQueryFields::FormStage.to_string());
    let q_email = RwQuery::<String>::new(ChangePasswordQueryFields::Email.to_string());

    let fn_btn_stage = move || match q_stage.get_or_default() {
        ChangePasswordFormStage::None => ChangePasswordBtnStage::None,
        ChangePasswordFormStage::Send => ChangePasswordBtnStage::Send,
        ChangePasswordFormStage::Check => ChangePasswordBtnStage::ReSend,
        ChangePasswordFormStage::Confirm => ChangePasswordBtnStage::Confirm,
        ChangePasswordFormStage::Finish => ChangePasswordBtnStage::None,
    };

    let fn_on_change = move |e: SubmitEvent| {
        e.prevent_default();

        let stage = q_stage.get_or_default_untracked();
        match stage {
            ChangePasswordFormStage::None | ChangePasswordFormStage::Finish => {
                //
            }
            ChangePasswordFormStage::Send | ChangePasswordFormStage::Check => {
                let (Some(email),) = (input_email.get_untracked(),) else {
                    return;
                };

                let email_value = email.value();
                let email_value = match proccess_email(email_value) {
                    Ok(v) => {
                        err_general.clear();
                        Some(v)
                    }
                    Err(err) => {
                        err_general.set(err);
                        None
                    }
                };
                let Some(email_value) = email_value else {
                    return;
                };

                api.send_change_password(email_value)
                    .send_web(async move |result| {
                        let err = match result {
                            Ok(ServerRes::Ok) => {
                                q_stage.set(ChangePasswordFormStage::Check);
                                Ok(())
                            }
                            Ok(res) => Err(format!("error, expected OK, received: {res:?}")),
                            Err(err) => Err(err.to_string()),
                        };

                        if let Err(err) = err {
                            error!(err);
                            err_general.set(err);
                        } else {
                            err_general.clear();
                        }
                    });

                // api.send_change_password(email)
                //     .send_web(move |result| async move {
                //         let err: Result<(), String> = match result {
                //             Ok(ServerRes::PasswordChangeStage(PasswordChangeStage::Confirm)) => {
                //                 q_stage.set(ChangePasswordFormStage::Confirm);
                //                 Ok(())
                //             }
                //             Ok(res) => Err(format!("error, expected OK, received: {res:?}")),
                //             Err(ServerErr::Cha) => Err("This invite link is already expired.".to_string()),
                //             Err(ServerErr::RegistrationErr(ServerRegistrationErr::TokenUsed)) => {
                //                 Err("This invite link was already used.".to_string())
                //             }
                //             Err(ServerErr::RegistrationErr(
                //                 ServerRegistrationErr::TokenNotFound,
                //             )) => Err("This invite link is invalid.".to_string()),
                //             Err(err) => Err(err.to_string()),
                //         };
                //         if let Err(err) = err {
                //             error!(err);
                //             err_general.set(err);
                //         }
                //     });
            }
            ChangePasswordFormStage::Confirm => {
                let (Some(password), Some(password_confirmation)) = (
                    input_password.get_untracked(),
                    input_password_confirmatoin.get_untracked(),
                ) else {
                    return;
                };

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

                let token = match q_token.get() {
                    Some(v) => Some(v),
                    None => {
                        err_general.set(String::from("token is missing from; invalid link"));
                        None
                    }
                };

                let (Some(password), Some(token)) = (password_value, token) else {
                    return;
                };

                api.confirm_change_password(password, token)
                    .send_web(async move |result| {
                        let err = match result {
                            Ok(ServerRes::Ok) => {
                                q_stage.set(ChangePasswordFormStage::Finish);
                                err_general.clear();
                                global_state.logout();
                            }
                            Ok(res) => {
                                err_general.set(format!("error, expected OK, received: {res:?}"));
                            }
                            Err(ServerErr::ChangePasswordErr(
                                ChangePasswordErr::InvalidPassword(err),
                            )) => {
                                err_password.set(err.to_string());
                            }
                            Err(err) => {
                                err_general.set(err.to_string());
                            }
                        };
                    });
            }
        }
    };

    ChangePassword {
        form_stage: q_stage,
        err_general: err_general,
        email: q_email,
        btn_stage: StoredValue::new(Box::new(fn_btn_stage)),
        on_change: StoredValue::new(Box::new(fn_on_change)),
    }
}
