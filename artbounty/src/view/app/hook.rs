use std::sync::Arc;
use std::time::Duration;

use crate::get_timestamp;
use crate::view::toolbox::prelude::*;
use humantime::format_duration;
use jiff::Span;
use jiff::{
    ToSpan,
    fmt::friendly::{Designator, SpanPrinter},
};
use leptos::html;
use leptos::{prelude::*, task::spawn_local};
use leptos_router::params::Params;
use leptos_router::{NavigateOptions, hooks::use_query};
use tracing::{error, info, trace, warn};
use web_sys::{HtmlInputElement, SubmitEvent};

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
    pub general_info: Option<String>,
    pub stage_error: Option<String>,
    pub expires: Option<u128>,
}

// #[derive(Clone, Default, PartialEq)]
// pub struct ChangeEmailView {
//     pub new_email: Memo<String>,
//     pub confirm_token: Memo<String>,
//     pub email_stage: Memo<EmailChangeFormStage>,
//     pub general_info: Memo<String>,
//     pub stage_error: Memo<String>,
//     pub expires: Memo<u128>,
// }

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
        general_info: Option<String>,
        expires: u128,
    ) -> Result<String, String> {
        let err_token = String::from("missing token");
        let err_email = String::from("missing email");
        let link = match self {
            Self::CurrentSendConfirm => {
                link_settings_form_email_current_send(stage_error, general_info)
            }
            Self::CurrentClickConfirm => {
                link_settings_form_email_current_click(expires, stage_error, general_info)
            }
            Self::CurrentConfirm => link_settings_form_email_current_confirm(
                expires,
                token.ok_or(err_token)?,
                stage_error,
                general_info,
            ),
            Self::NewEnterEmail => {
                link_settings_form_email_new_send(expires, stage_error, general_info)
            }
            Self::NewClickConfirm => link_settings_form_email_new_click(
                expires,
                new_email.ok_or(err_email)?,
                stage_error,
                general_info,
            ),
            Self::NewConfirmEmail => link_settings_form_email_new_confirm(
                expires,
                new_email.ok_or(err_email)?,
                token.ok_or(err_token)?,
                stage_error,
                general_info,
            ),
            Self::FinalConfirm => link_settings_form_email_final_confirm(
                expires,
                new_email.ok_or(err_email)?,
                stage_error,
                general_info,
            ),
            Self::Completed => link_settings_form_email_completed(
                expires,
                new_email.ok_or(err_email)?,
                stage_error,
                general_info,
            ),
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
    // pub view: ChangeEmailView,
    // pub callback_btn_stage: F_BTN_STAGE,
    pub get_new_email: StoredValue<Box<dyn Fn() -> String + Sync + Send + 'static>>,
    pub check_new_email: StoredValue<Box<dyn Fn() -> bool + Sync + Send + 'static>>,
    pub get_token: StoredValue<Box<dyn Fn() -> String + Sync + Send + 'static>>,
    pub check_token: StoredValue<Box<dyn Fn() -> bool + Sync + Send + 'static>>,
    pub get_form_stage: StoredValue<Box<dyn Fn() -> EmailChangeFormStage + Sync + Send + 'static>>,
    pub check_form_stage: StoredValue<Box<dyn Fn() -> bool + Sync + Send + 'static>>,
    pub get_info: StoredValue<Box<dyn Fn() -> String + Sync + Send + 'static>>,
    pub check_info: StoredValue<Box<dyn Fn() -> bool + Sync + Send + 'static>>,
    pub get_err: StoredValue<Box<dyn Fn() -> String + Sync + Send + 'static>>,
    pub check_err: StoredValue<Box<dyn Fn() -> bool + Sync + Send + 'static>>,
    pub get_expires: StoredValue<Box<dyn Fn() -> u128 + Sync + Send + 'static>>,
    pub check_expires: StoredValue<Box<dyn Fn() -> bool + Sync + Send + 'static>>,
    pub expires_str: RwSignal<String>,
    pub get_btn_stage: StoredValue<Box<dyn Fn() -> BtnStage + Sync + Send + 'static>>,
    pub post_cancel: StoredValue<Box<dyn Fn(SubmitEvent) -> () + Sync + Send + 'static>>,
    pub post_run: StoredValue<Box<dyn Fn(SubmitEvent) + Sync + Send + 'static>>,
}

// impl EmailChange {
//     pub fn get_email<'a>(
//         &self,
//     ) -> impl Fn() -> String + Send + Sync + Clone + Copy + 'static + use<'a> {
//         let q = self.query.clone();
//         move || q.get().new_email.unwrap_or(String::from("new email"))
//     }
//     pub fn get_general_info<'a>(
//         &self,
//     ) -> impl Fn() -> String + Send + Sync + Clone + Copy + 'static + use<'a> {
//         let q = self.query.clone();
//         move || q.get().general_info.unwrap_or_default()
//     }
//     pub fn get_stage_error<'a>(
//         &self,
//     ) -> impl Fn() -> String + Send + Sync + Clone + Copy + 'static + use<'a> {
//         let q = self.query.clone();
//         move || q.get().stage_error.unwrap_or_default()
//     }
//     pub fn get_form_stage<'a>(
//         &self,
//     ) -> impl Fn() -> EmailChangeFormStage + Send + Sync + Clone + Copy + 'static + use<'a> {
//         let q = self.query.clone();
//         move || q.get().email_stage.unwrap_or_default()
//     }
//     pub fn get_btn_stage<'a>(
//         &self,
//     ) -> impl Fn() -> BtnStage + Send + Sync + Clone + Copy + 'static + use<'a> {
//         let f = self.callback_btn_stage.clone();
//         move || (f.read_value())()
//     }
//     pub fn get_run<'a>(
//         &self,
//     ) -> impl Fn(SubmitEvent) + Send + Sync + Clone + Copy + 'static + use<'a> {
//         let f = self.callback_run.clone();
//         move |e: SubmitEvent| (f.read_value())(e)
//     }
//     pub fn get_cancel<'a>(
//         &self,
//     ) -> impl Fn(SubmitEvent) + Send + Sync + Clone + Copy + 'static + use<'a> {
//         let f = self.callback_cancel.clone();
//         move |e: SubmitEvent| (f.read_value())(e)
//     }
// }

pub fn use_change_email(api: ApiWeb, input_new_email: NodeRef<html::Input>) -> EmailChange {
    // let errors = RwSignal::new(String::new());
    // let btn_stage = RwSignal::new(BtnStage::Confirm);

    // let a = Box::new(0);
    // let b = a;
    // let c = a;
    const EXPIRED_STR: &'static str = "expired";

    let time_until_expires = RwSignal::new(String::from(EXPIRED_STR));
    let query = use_query::<ParamsChangeEmail>();
    let fn_get_new_email = move || {
        query
            .with(|v| v.as_ref().ok().and_then(|v| v.new_email.clone()))
            .unwrap_or_else(|| "new email".to_string())
    };
    let fn_check_new_email = move || {
        query
            .with(|v| v.as_ref().ok().map(|v| v.new_email.is_some()))
            .unwrap_or_default()
    };
    let fn_get_confirm_token = move || {
        query
            .with(|v| v.as_ref().ok().and_then(|v| v.confirm_token.clone()))
            .unwrap_or_default()
    };
    let fn_check_confirm_token = move || {
        query
            .with(|v| v.as_ref().ok().map(|v| v.confirm_token.is_some()))
            .unwrap_or_default()
    };
    let fn_get_form_stage = move || {
        query
            .with(|v| v.as_ref().ok().and_then(|v| v.email_stage.clone()))
            .unwrap_or_default()
    };
    let fn_check_email_stage = move || {
        query
            .with(|v| v.as_ref().ok().map(|v| v.email_stage.is_some()))
            .unwrap_or_default()
    };
    let fn_get_general_info = move || {
        query
            .with(|v| v.as_ref().ok().and_then(|v| v.general_info.clone()))
            .unwrap_or_default()
    };
    let fn_check_general_info = move || {
        query
            .with(|v| v.as_ref().ok().map(|v| v.general_info.is_some()))
            .unwrap_or_default()
    };
    let fn_get_stage_err = move || {
        query
            .with(|v| v.as_ref().ok().and_then(|v| v.stage_error.clone()))
            .unwrap_or_default()
    };
    let fn_check_stage_err = move || {
        query
            .with(|v| v.as_ref().ok().map(|v| v.stage_error.is_some()))
            .unwrap_or_default()
    };
    let fn_get_expires = move || {
        query
            .with(|v| v.as_ref().ok().and_then(|v| v.expires.clone()))
            .unwrap_or_default()
    };
    let fn_check_expires = move || {
        query
            .with(|v| v.as_ref().ok().map(|v| v.expires.is_some()))
            .unwrap_or_default()
    };
    let fn_btn_stage = move || -> BtnStage {
        let stage = fn_get_form_stage();
        if time_until_expires.with(|v| v == EXPIRED_STR)
            && stage != EmailChangeFormStage::CurrentSendConfirm
        {
            return BtnStage::None;
        }
        match stage {
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
    // let view_query = Memo::new(move |_| {
    //     let v = query.get().ok().unwrap_or_default();
    //     ChangeEmailView {
    //         new_email: v.new_email.unwrap_or(String::from("new email")),
    //         confirm_token: v.confirm_token.unwrap_or_default(),
    //         email_stage: v.email_stage.unwrap_or_default(),
    //         general_info: v.general_info.unwrap_or_default(),
    //         stage_error: v.stage_error.unwrap_or_default(),
    //         expires: v.expires.unwrap_or_default(),
    //     }
    // });
    let get_query = move || query.get().ok().unwrap_or_default();
    let get_query_untracked = move || query.get_untracked().ok().unwrap_or_default();
    let get_query_email_stage = move || get_query().email_stage.unwrap_or_default();
    let get_query_email_stage_untracked =
        move || get_query_untracked().email_stage.unwrap_or_default();
    let create_err_link = move |err: String| -> String {
        let query = get_query_untracked();
        query
            .email_stage
            .unwrap_or_default()
            .link(
                query.confirm_token,
                query.new_email,
                Some(err),
                None,
                query.expires.unwrap_or_default(),
            )
            .unwrap_or_else(|err| {
                EmailChangeFormStage::CurrentSendConfirm
                    .link(
                        None,
                        None,
                        Some(err),
                        None,
                        query.expires.unwrap_or_default(),
                    )
                    .unwrap()
            })
    };
    let navigate = leptos_router::hooks::use_navigate();
    let _ = interval::new(
        move || {
            let time = get_timestamp();
            // let time = Duration::from_nanos(time as u64);
            let expires = get_query_untracked().expires.unwrap_or_default();
            // let expires = get_query_untracked().expires.unwrap_or_default();
            // let expires = Duration::from_nanos(expires as u64);
            let elapsed = expires.saturating_sub(time);
            let output = if elapsed == 0 {
                EXPIRED_STR.to_string()
            } else {
                let elapsed = Duration::from_nanos(elapsed as u64);
                format_duration(elapsed).to_string()
            };
            let _ = time_until_expires.try_set(output);

            // let printer = SpanPrinter::new().designator(Designator::HumanTime);
            // let span = Span::new().nanoseconds(time as i64);
            // let output = printer.span_to_string(&span);

            // use std::time::{SystemTime, UNIX_EPOCH};
            // let r = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            // trace!("wtf {output:?}");
            // let query = get_query_untracked();
            // query.expires.map(|time| timestamp)
        },
        Duration::from_secs(1),
    );
    Effect::new({
        let navigate = navigate.clone();
        move || {
            let navigate = navigate.clone();
            let query = get_query();
            if query.email_stage == Some(EmailChangeFormStage::CurrentSendConfirm)
                && query.general_info.is_none()
            {
                api.change_email_status().send_web(async move |result| {
                    let result = match result {
                        Ok(ServerRes::EmailChangeStage(stage)) => Ok(stage),
                        Ok(err) => {
                            error!("expected EmailChangeState, received {err:?}");
                            Err("SERVER ERROR, wrong response.".to_string())
                        }
                        Err(err) => {
                            error!("received {err:?}");
                            Err(err.to_string())
                        }
                    };
                    let link = match result {
                        Ok(stage) => stage.link(None, None),
                        Err(err) => create_err_link(format!("error getting status {err}")),
                    };
                    // let Some(link) = link else {
                    //     return;
                    // };
                    navigate(&link, NavigateOptions::default());
                });
            }
        }
    });

    let fn_cancel = {
        let navigate = navigate.clone();
        move |e: SubmitEvent| {
            e.prevent_default();
            let navigate = navigate.clone();
            api.cancel_email_change().send_web(async move |result| {
                let result = match result {
                    Ok(ServerRes::EmailChangeStage(EmailChangeStage::Complete { .. })) => {
                        Ok("Succesfully canceled".to_string())
                    }
                    Ok(err) => Err(format!("unexpected response: {err:?}, expected Ok")),
                    Err(err) => Err(format!("unexpected response: {err}")),
                };

                let link = match result {
                    Ok(msg) => link_settings_form_email_current_send(None, Some(msg)),
                    Err(msg) => create_err_link(msg),
                };

                navigate(&link, NavigateOptions::default());
            });
        }
    };
    let fn_run = {
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
                        let result = match result {
                            Ok(ServerRes::EmailChangeStage(stage)) => Ok(stage),
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
                                    Ok(ServerRes::EmailChangeStage(stage)) => Ok(stage),
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

                        let link = match result {
                            Ok(v) => v.link(None, None),
                            Err(err) => create_err_link(err),
                        };
                        // let link = result
                        //     .map(|v| v.link(None, None))
                        //     .unwrap_or_else(|err| {
                        //         EmailChangeFormStage::CurrentSendConfirm
                        //             .link(None, None, Some(err), None, 0)
                        //             .unwrap()
                        //     });
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
                let link = create_err_link(err);
                navigate(&link, NavigateOptions::default());
            }
        }
    };
    EmailChange {
        // query: view_query,
        get_new_email: StoredValue::new(Box::new(fn_get_new_email)),
        check_new_email: StoredValue::new(Box::new(fn_check_new_email)),
        get_token: StoredValue::new(Box::new(fn_get_confirm_token)),
        check_token: StoredValue::new(Box::new(fn_check_confirm_token)),
        get_form_stage: StoredValue::new(Box::new(fn_get_form_stage)),
        check_form_stage: StoredValue::new(Box::new(fn_check_email_stage)),
        get_info: StoredValue::new(Box::new(fn_get_general_info)),
        check_info: StoredValue::new(Box::new(fn_check_general_info)),
        get_err: StoredValue::new(Box::new(fn_get_stage_err)),
        check_err: StoredValue::new(Box::new(fn_check_stage_err)),
        get_expires: StoredValue::new(Box::new(fn_get_expires)),
        check_expires: StoredValue::new(Box::new(fn_check_expires)),
        expires_str: time_until_expires,
        get_btn_stage: StoredValue::new(Box::new(fn_btn_stage)),
        post_cancel: StoredValue::new(Box::new(fn_cancel)),
        post_run: StoredValue::new(Box::new(fn_run)),
    }
}
