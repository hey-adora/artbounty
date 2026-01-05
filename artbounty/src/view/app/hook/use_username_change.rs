use crate::{
    api::{Api, ApiWeb, ChangeUsernameErr, ServerErr},
    valid::auth::proccess_username,
    view::{app::GlobalState, toolbox::prelude::*},
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
#[strum(serialize_all = "lowercase")]
pub enum ChangeUsernameBtnStage {
    #[default]
    None,
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
pub enum ChangeUsernameFormStage {
    #[default]
    None,
    Change,
    Finish,
}

#[derive(Clone, Copy)]
pub struct ChangeUsername {
    pub stage: RwQuery<ChangeUsernameFormStage>,
    pub old_username: RwQuery<String>,
    pub new_username: RwQuery<String>,
    pub err_username: RwQuery<String>,
    pub err_password: RwQuery<String>,
    pub err_general: RwQuery<String>,
    pub btn_stage: StoredValue<Box<dyn Fn() -> ChangeUsernameBtnStage + Sync + Send + 'static>>,
    pub on_change: StoredValue<Box<dyn Fn(SubmitEvent) + Sync + Send + 'static>>,
}

pub fn use_change_username(
    api: ApiWeb,
    input_username: NodeRef<html::Input>,
    input_password: NodeRef<html::Input>,
) -> ChangeUsername {
    let global_state = expect_context::<GlobalState>();

    let q_err_general = RwQuery::<String>::new("err_general");
    let q_err_username = RwQuery::<String>::new("err_username");
    let q_err_password = RwQuery::<String>::new("err_password");
    let q_old_username = RwQuery::<String>::new("old_username");
    let q_new_username = RwQuery::<String>::new("new_username");
    let q_stage = RwQuery::<ChangeUsernameFormStage>::new("form_stage");

    let on_change_username = {
        move |e: SubmitEvent| {
            e.prevent_default();
            let (Some(username), Some(password)) = (
                input_username.get_untracked() as Option<HtmlInputElement>,
                input_password.get_untracked() as Option<HtmlInputElement>,
            ) else {
                return;
            };

            q_err_general.clear();

            let username_value = username.value();
            let username_value = match proccess_username(&username_value) {
                Ok(v) => {
                    q_err_username.clear();
                    Some(v)
                }

                Err(err) => {
                    error!("on_change_username username \"{username_value}\" error: {err}");
                    q_err_username.set(err);
                    None
                }
            };

            let password_value = password.value();
            let password_value = match password_value.is_empty() {
                false => {
                    q_err_password.clear();
                    Some(password_value)
                }
                true => {
                    let err = "password cant be empty".to_string();
                    error!("on_change_username password error: {err}");
                    q_err_password.set(err);
                    None
                }
            };

            let (Some(username_value), Some(password_value)) = (username_value, password_value)
            else {
                return;
            };

            api.change_username(password_value, username_value)
                .send_web(move |result| {
                    async move {
                        match result {
                            Ok(crate::api::ServerRes::User {
                                username: new_username,
                            }) => {
                                let old_username = global_state
                                    .get_username_untracked()
                                    .unwrap_or("404".to_string());

                                global_state.change_username(new_username.clone());

                                q_stage.set(ChangeUsernameFormStage::Finish);
                                q_old_username.set(old_username);
                                q_new_username.set(new_username);

                                // navigate(
                                //     &link_settings_form_username(
                                //         UsernameChangeStage::Finish,
                                //         Some(old_username),
                                //         Some(new_username),
                                //     ),
                                //     NavigateOptions::default(),
                                // );
                                // selected_form.try_set(SelectedForm::None);
                            }
                            Ok(err) => {
                                error!("expected Post, received {err:?}");
                                let _ =
                                    q_err_general.set("SERVER ERROR, wrong response.".to_string());
                            }
                            Err(ServerErr::ChangeUsernameErr(
                                ChangeUsernameErr::UsernameIsTaken(_),
                            )) => {
                                q_err_username.set("Username is taken".to_string());
                            }
                            Err(err) => {
                                let _ = q_err_general.set(err.to_string());
                            }
                        }
                    }
                });
        }
    };

    let fn_btn_stage = move || -> ChangeUsernameBtnStage {
        match q_stage.get() {
            ChangeUsernameFormStage::None => ChangeUsernameBtnStage::None,
            ChangeUsernameFormStage::Change => ChangeUsernameBtnStage::Confirm,
            ChangeUsernameFormStage::Finish => ChangeUsernameBtnStage::None,
        }
    };

    ChangeUsername {
        old_username: q_old_username,
        new_username: q_new_username,
        err_username: q_err_username,
        err_password: q_err_password,
        err_general: q_err_general,
        stage: q_stage,
        btn_stage: StoredValue::new(Box::new(fn_btn_stage)),
        on_change: StoredValue::new(Box::new(on_change_username)),
    }
}
