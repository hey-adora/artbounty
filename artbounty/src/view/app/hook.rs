use std::sync::Arc;
use std::time::Duration;

use leptos::html;
use leptos::{prelude::*, task::spawn_local};
use leptos_router::params::Params;
use leptos_router::{NavigateOptions, hooks::use_query};
use tracing::{error, info, trace, warn};
use web_sys::{HtmlInputElement, SubmitEvent};
use crate::view::toolbox::prelude::*;

use crate::api::{
    Api, ApiWeb, ApiWebTmp, EmailChangeErr, EmailChangeNewErr, EmailChangeStage,
    EmailChangeTokenErr, ServerErr, ServerRes,
};
use crate::path::{
    link_settings_form_email_completed, link_settings_form_email_current_click,
    link_settings_form_email_current_confirm, link_settings_form_email_current_send,
    link_settings_form_email_final_confirm, link_settings_form_email_new_click,
    link_settings_form_email_new_confirm, link_settings_form_email_new_send,
};
use crate::valid::auth::proccess_email;

#[derive(Params, PartialEq, Clone, Default)]
pub struct ParamsChangeEmail {
    pub new_email: Option<String>,
    pub confirm_token: Option<String>,
    pub email_stage: Option<EmailChangeFormStage>,
    pub stage_error: Option<String>,
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    PartialOrd,
    Default,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumString,
    strum::EnumIter,
    strum::Display,
)]
#[strum(serialize_all = "lowercase")]
pub enum EmailChangeFormStage {
    #[default]
    CurrentSendConfirm,
    CurrentClickConfirm,
    CurrentConfirm,
    NewEnterEmail,
    NewClickConfirm,
    NewConfirmEmail,
    FinalConfirm,
    Completed,
}

impl EmailChangeFormStage {
    pub fn link(
        &self,
        token: Option<String>,
        new_email: Option<String>,
        stage_error: Option<String>,
    ) -> Result<String, String> {
        let err_token = String::from("missing token");
        let err_email = String::from("missing email");
        let link = match self {
            Self::CurrentSendConfirm => link_settings_form_email_current_send(stage_error),
            Self::CurrentClickConfirm => link_settings_form_email_current_click(stage_error),
            Self::CurrentConfirm => {
                link_settings_form_email_current_confirm(token.ok_or(err_token)?, stage_error)
            }
            Self::NewEnterEmail => link_settings_form_email_new_send(stage_error),
            Self::NewClickConfirm => {
                link_settings_form_email_new_click(new_email.ok_or(err_email)?, stage_error)
            }
            Self::NewConfirmEmail => link_settings_form_email_new_confirm(
                new_email.ok_or(err_email)?,
                token.ok_or(err_token)?,
                stage_error,
            ),
            Self::FinalConfirm => {
                link_settings_form_email_final_confirm(new_email.ok_or(err_email)?, stage_error)
            }
            Self::Completed => {
                link_settings_form_email_completed(new_email.ok_or(err_email)?, stage_error)
            }
        };
        Ok(link)
    }
}

// impl From<EmailChangeFormStage> for EmailChangeStage {
//     fn from(value: EmailChangeFormStage) -> Self {
//         match value {
//             EmailChangeFormStage::CurrentSendConfirm | EmailChangeFormStage::CurrentClickConfirm | EmailChangeFormStage::CurrentConfirm => EmailChangeStage::ConfirmEmail,
//             EmailChangeFormStage::NewEnterEmail | EmailChangeFormStage::NewClickConfirm | EmailChangeFormStage::NewConfirmEmail  => EmailChangeStage::EnterNewEmail,
//             EmailChangeFormStage::NewEnterEmail | EmailChangeFormStage::NewClickConfirm | EmailChangeFormStage::NewConfirmEmail  => EmailChangeStage::EnterNewEmail,
//             EmailChangeFormStage::  => EmailChangeStage::EnterNewEmail,
//         }
//     }
// }

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BtnStage {
    Send,
    Resend,
    Confirm,
    None,
}

#[derive(Clone, Copy)]
pub struct EmailChange {
    pub query: Memo<ParamsChangeEmail>,
    pub callback_btn_stage: StoredValue<Box<dyn Fn() -> BtnStage + Sync + Send + 'static>>,
    pub callback_run: StoredValue<Box<dyn Fn(SubmitEvent) + Sync + Send + 'static>>,
}

impl EmailChange {
    pub fn get_email<'a>(
        &self,
    ) -> impl Fn() -> String + Send + Sync + Clone + Copy + 'static + use<'a> {
        let q = self.query.clone();
        move || q.get().new_email.unwrap_or(String::from("new email"))
    }
    pub fn get_stage_error<'a>(
        &self,
    ) -> impl Fn() -> String + Send + Sync + Clone + Copy + 'static + use<'a> {
        let q = self.query.clone();
        move || q.get().stage_error.unwrap_or_default()
    }
    pub fn get_form_stage<'a>(
        &self,
    ) -> impl Fn() -> EmailChangeFormStage + Send + Sync + Clone + Copy + 'static + use<'a> {
        let q = self.query.clone();
        move || q.get().email_stage.unwrap_or_default()
    }
    pub fn get_btn_stage<'a>(
        &self,
    ) -> impl Fn() -> BtnStage + Send + Sync + Clone + Copy + 'static + use<'a> {
        let f = self.callback_btn_stage.clone();
        move || (f.read_value())()
    }
    pub fn get_run<'a>(
        &self,
    ) -> impl Fn(SubmitEvent) + Send + Sync + Clone + Copy + 'static + use<'a> {
        let f = self.callback_run.clone();
        move |e: SubmitEvent| (f.read_value())(e)
    }
}

pub fn use_change_email(api: ApiWeb, input_new_email: NodeRef<html::Input>) -> EmailChange {
    // let errors = RwSignal::new(String::new());
    // let btn_stage = RwSignal::new(BtnStage::Confirm);
    let time_left = RwSignal::new(String::new());
    let query = use_query::<ParamsChangeEmail>();
    let view_query = Memo::new(move |_| query.get().ok().unwrap_or_default());
    let get_query = move || query.get().ok().unwrap_or_default();
    let get_query_untracked = move || query.get_untracked().ok().unwrap_or_default();
    let get_query_email_stage = move || get_query().email_stage.unwrap_or_default();
    let get_query_email_stage_untracked =
        move || get_query_untracked().email_stage.unwrap_or_default();
    let create_err_link = move |err: String| {
        let query = get_query_untracked();
        query
            .email_stage
            .unwrap_or_default()
            .link(query.confirm_token, query.new_email, Some(err))
    };
    let navigate = leptos_router::hooks::use_navigate();
    // let _ = interval::new(move || {
    //
    // }, Duration::from_secs(1));
    let on_email_change = {
        let navigate = navigate.clone();
        move |e: SubmitEvent| {
            e.prevent_default();
            let navigate = navigate.clone();
            //navigate(&link, NavigateOptions::default());
            // errors.set("".to_string());
            let handler = {
                let navigate = navigate.clone();
                move |result: Result<ServerRes, ServerErr>| {
                    let navigate = navigate.clone();
                    //
                    async move {
                        // let proccess_stage =
                        //     move |stage: Option<EmailChangeStage>,
                        //           new_email: Option<String>|
                        //           -> Result<String, String> {
                        //         trace!("recv: {stage:?}");
                        //         let Some(stage) = stage else {
                        //             error!("email change failed to initialize");
                        //             return create_err_link(
                        //                 "Email change expired/canceled, restart the proccess."
                        //                     .to_string(),
                        //             );
                        //         };
                        //         stage
                        //             .link(new_email.clone(), None)
                        //             .ok_or(String::from("failed to generate link"))
                        //     };
                        let result = match result {
                            Ok(ServerRes::EmailChangeStage { stage, new_email }) => {
                                Ok((stage, new_email))
                            }
                            Ok(err) => {
                                error!("expected EmailChangeState, received {err:?}");
                                Err("SERVER ERROR, wrong response.".to_string())
                            }
                            Err(ServerErr::EmailChange(EmailChangeErr::InvalidStage(_)))
                            | Err(ServerErr::EmailChangeNew(EmailChangeNewErr::InvalidStage(_)))
                            | Err(ServerErr::EmailChangeToken(
                                EmailChangeTokenErr::InvalidStage(_),
                            )) => {
                                let result =
                                    ApiWebTmp::new().change_email_status().send_native().await;
                                match result {
                                    Ok(ServerRes::EmailChangeStage { stage, new_email }) => {
                                        Ok((stage, new_email))
                                    }
                                    Ok(err) => {
                                        error!("expected EmailChangeState, received {err:?}");
                                        Err("SERVER ERROR, wrong response.".to_string())
                                    }
                                    Err(err) => {
                                        error!("received {err:?}");
                                        Err(err.to_string())
                                    }
                                }
                            }
                            Err(err) => {
                                error!("received {err:?}");
                                Err(err.to_string())
                            }
                        };

                        let link = result
                            .and_then(|(stage, new_email)| {
                                stage
                                    .ok_or(
                                        "Email change expired/canceled, restart the proccess."
                                            .to_string(),
                                    )
                                    .and_then(|v| v.link(new_email, None))
                            })
                            .unwrap_or_else(|err| {
                                EmailChangeFormStage::CurrentSendConfirm
                                    .link(None, None, Some(err))
                                    .unwrap()
                            });
                        navigate(&link, NavigateOptions::default());
                    }
                }
            };
            //
            let error = match get_query_email_stage_untracked() {
                EmailChangeFormStage::CurrentSendConfirm => {
                    api.send_email_change().send_web(handler.clone());
                    None
                }
                EmailChangeFormStage::CurrentClickConfirm => {
                    api.resend_email_change().send_web(handler.clone());
                    None
                }
                EmailChangeFormStage::CurrentConfirm => {
                    let confirm_token = get_query_untracked()
                        .confirm_token
                        .ok_or("missing confirm_token.".to_string());
                    match confirm_token {
                        Ok(confirm_token) => {
                            api.confirm_email_change(confirm_token)
                                .send_web(handler.clone());
                            None
                        }
                        Err(err) => Some(err),
                    }
                }
                EmailChangeFormStage::NewEnterEmail => {
                    let new_email = input_new_email
                        .get_untracked()
                        .ok_or("missing the input box.".to_string())
                        .and_then(|v| proccess_email(v.value()));

                    match new_email {
                        Ok(new_email) => {
                            api.send_email_new(new_email).send_web(handler.clone());
                            None
                        }
                        Err(err) => Some(err),
                    }
                }
                EmailChangeFormStage::NewClickConfirm => {
                    api.resend_email_new().send_web(handler.clone());
                    None
                }
                EmailChangeFormStage::NewConfirmEmail => {
                    let confirm_token = get_query_untracked()
                        .confirm_token
                        .ok_or("missing confirm_token.".to_string());
                    match confirm_token {
                        Ok(confirm_token) => {
                            api.confirm_email_new(confirm_token)
                                .send_web(handler.clone());
                            None
                        }
                        Err(err) => Some(err),
                    }
                }
                EmailChangeFormStage::FinalConfirm => {
                    api.change_email().send_web(handler.clone());
                    None
                }
                EmailChangeFormStage::Completed => None,
            };
            if let Some(err) = error {
                let link = create_err_link(err).unwrap_or_else(|err| {
                    EmailChangeFormStage::CurrentSendConfirm
                        .link(None, None, Some(err))
                        .unwrap()
                });
                navigate(&link, NavigateOptions::default());
            }
        }
    };
    let get_btn_stage = move || -> BtnStage {
        match get_query_email_stage() {
            EmailChangeFormStage::CurrentSendConfirm => BtnStage::Send,
            EmailChangeFormStage::CurrentClickConfirm => BtnStage::Resend,
            EmailChangeFormStage::CurrentConfirm => BtnStage::Confirm,
            EmailChangeFormStage::NewEnterEmail => BtnStage::Send,
            EmailChangeFormStage::NewClickConfirm => BtnStage::Resend,
            EmailChangeFormStage::NewConfirmEmail => BtnStage::Confirm,
            EmailChangeFormStage::FinalConfirm => BtnStage::Confirm,
            EmailChangeFormStage::Completed => BtnStage::None,
        }
    };
    EmailChange {
        query: view_query,
        callback_btn_stage: StoredValue::new(Box::new(get_btn_stage)),
        callback_run: StoredValue::new(Box::new(on_email_change)),
    }
}
