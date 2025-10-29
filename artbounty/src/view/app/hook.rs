use std::sync::Arc;

use leptos::html;
use leptos::{prelude::*, task::spawn_local};
use leptos_router::params::Params;
use leptos_router::{NavigateOptions, hooks::use_query};
use tracing::{error, info, trace, warn};
use web_sys::{HtmlInputElement, SubmitEvent};

use crate::api::{Api, ApiWeb, ServerErr, ServerRes};
use crate::valid::auth::proccess_email;

#[derive(Params, PartialEq, Clone, Default)]
pub struct ParamsChangeEmail {
    pub new_email: Option<String>,
    pub confirm_token: Option<String>,
    pub email_stage: Option<EmailChangeFormStage>,
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

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BtnStage {
    Send,
    Resend,
    Confirm,
    None,
}

#[derive(Clone, Copy)]
pub struct EmailChange {
    pub errors: RwSignal<String>,
    pub query: Memo<ParamsChangeEmail>,
    pub callback_btn_stage: StoredValue<Box<dyn Fn() -> BtnStage + Sync + Send + 'static>>,
    pub callback_run: StoredValue<Box<dyn Fn(SubmitEvent) + Sync + Send + 'static>>,
}

impl EmailChange {
    pub fn get_email<'a>(&self) -> impl Fn() -> String + Send + Sync + Clone + Copy + 'static + use<'a> {
        let q = self.query.clone();
        move || q.get().new_email.unwrap_or(String::from("new email"))
    }
    pub fn get_form_stage<'a>(&self) -> impl Fn() -> EmailChangeFormStage + Send + Sync + Clone + Copy + 'static + use<'a> {
        let q = self.query.clone();
        move || q.get().email_stage.unwrap_or_default()
    }
    pub fn get_btn_stage<'a>(&self) -> impl Fn() -> BtnStage + Send + Sync + Clone + Copy + 'static + use<'a> {
        let f = self.callback_btn_stage.clone();
        move || (f.read_value())()
    }
    pub fn get_run<'a>(&self) -> impl Fn(SubmitEvent) + Send + Sync + Clone + Copy + 'static + use<'a> {
        let f = self.callback_run.clone();
        move |e: SubmitEvent| (f.read_value())(e)
    }
}

pub fn use_change_email(api: ApiWeb, input_new_email: NodeRef<html::Input>) -> EmailChange {
    let errors = RwSignal::new(String::new());
    // let btn_stage = RwSignal::new(BtnStage::Confirm);
    let query = use_query::<ParamsChangeEmail>();
    let view_query = Memo::new(move |_| query.get().ok().unwrap_or_default());
    let get_query = move || query.get().ok().unwrap_or_default();
    let get_query_untracked = move || query.get_untracked().ok().unwrap_or_default();
    let get_query_email_stage = move || get_query().email_stage.unwrap_or_default();
    let get_query_email_stage_untracked = move || get_query_untracked().email_stage.unwrap_or_default();
    let navigate = leptos_router::hooks::use_navigate();
    let on_email_change = {
        let navigate = navigate.clone();
        move |e: SubmitEvent| {
            e.prevent_default();
            let navigate = navigate.clone();
            let handler = move |result: Result<ServerRes, ServerErr>| {
                let navigate = navigate.clone();
                //
                async move {
                    match result {
                        Ok(ServerRes::EmailChangeStage { stage, new_email }) => {
                            trace!("recv: {stage:?}");
                            let Some(stage) = stage else {
                                error!("email change failed to initialize");
                                let _ = errors.try_set("email change failed to initialize.".to_string());
                                return;
                            };
                            let Some(link) = stage.link(new_email.clone()) else {
                                let _ = errors.try_set(format!("received broken response {stage:?} {new_email:?}."));
                                return;
                            };

                            trace!("link generated {link}");
                            // if let Some(new_email) = new_email {
                            //     change_email_new_email.try_set(new_email);
                            // }

                            navigate(&link, NavigateOptions::default());
                        }
                        Ok(err) => {
                            error!("expected EmailChangeState, received {err:?}");
                            let _ = errors.try_set("SERVER ERROR, wrong response.".to_string());
                        }
                        Err(err) => {
                            error!("received {err:?}");
                            let _ = errors.try_set(err.to_string());
                        }
                    }
                }
            };
            //
            match get_query_email_stage_untracked() {
                EmailChangeFormStage::CurrentSendConfirm => {
                    api.send_email_change().send_web(handler.clone());
                }
                EmailChangeFormStage::CurrentClickConfirm => {
                    // api.r
                }
                EmailChangeFormStage::CurrentConfirm => {
                    let params = get_query_untracked();
                    let Some(confirm_token) = params.confirm_token else {
                        error!("missing confirm_token from url query");
                        let _ = errors.try_set("missing confirm_token.".to_string());
                        return;
                    };
                    api.confirm_email_change(confirm_token)
                        .send_web(handler.clone());
                }
                EmailChangeFormStage::NewEnterEmail | EmailChangeFormStage::NewClickConfirm => {
                    // let params = get_query_untracked();
                    let Some(new_email) =
                        input_new_email.get_untracked() as Option<HtmlInputElement>
                    else {
                        error!("missing input to enter the new email");
                        let _ = errors.try_set("missing the input box.".to_string());
                        return;
                    };

                    let new_email = proccess_email(new_email.value());

                    let _ = errors.try_set(new_email.clone().err().unwrap_or_default());

                    let Ok(new_email) = new_email else {
                        return;
                    };

                    api.send_email_new(new_email).send_web(handler.clone());
                }
                // EmailChangeFormStage::NewClickConfirm => {
                //     //
                // }
                EmailChangeFormStage::NewConfirmEmail => {
                    let params = get_query_untracked();
                    let Some(confirm_token) = params.confirm_token else {
                        error!("missing confirm_token from url query");
                        let _ = errors.try_set("missing confirm_token.".to_string());
                        return;
                    };
                    api.confirm_email_new(confirm_token)
                        .send_web(handler.clone());
                }
                EmailChangeFormStage::FinalConfirm => {
                    api.change_email().send_web(handler.clone());
                }
                EmailChangeFormStage::Completed => {
                    //
                }
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
        errors,
        query: view_query,
        callback_btn_stage: StoredValue::new(Box::new(get_btn_stage)),
        callback_run: StoredValue::new(Box::new(on_email_change)),
    }
}
