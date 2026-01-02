use leptos::Params;
use leptos::tachys::reactive_graph::bind::GetValue;
use leptos::{html, prelude::*};
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_query;
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

#[derive(Params, PartialEq, Clone)]
pub struct RegParams {
    pub err_general: Option<String>,
    pub err_username: Option<String>,
    pub err_password: Option<String>,
    pub token: Option<String>,
    pub email: Option<String>,
    pub kind: Option<RegStage>,
}

#[derive(Debug, Default, Clone, PartialEq, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum RegStage {
    #[default]
    None,
    CheckEmail,
    Reg,
}

#[derive(Clone, Copy)]
pub struct Register {
    pub field_reg_stage: QueryField<RegStage>,
    pub field_err_general: QueryField<String>,
    pub field_err_username: QueryField<String>,
    pub field_err_password: QueryField<String>,
    pub field_token: QueryField<String>,
    pub field_email: QueryField<String>,
}

pub fn use_register(
    api: ApiWeb,
    input_username: NodeRef<html::Input>,
    input_password: NodeRef<html::Input>,
    input_password_confirmatoin: NodeRef<html::Input>,
) -> Register {
    let query = use_query::<RegParams>();

    let field_reg_stage = query.to_query_field(|v| v.kind.as_ref());
    let field_err_general = query.to_query_field(|v| v.err_general.as_ref());
    let field_err_username = query.to_query_field(|v| v.err_username.as_ref());
    let field_err_password = query.to_query_field(|v| v.err_password.as_ref());
    let field_token = query.to_query_field(|v| v.token.as_ref());
    let field_email = query.to_query_field(|v| v.email.as_ref());

    // let create_err_link = move || -> String {
    //     match (field_reg_stage.get_untracked(), field_reg_stage.get_untracked()) {
    //         (RegStage::Reg) => link_reg_invite()
    //
    //         _ =>link_reg_invite()
    //     }
    //
    //
    //
    //
    // };
    //
    // let on_register = move |e: SubmitEvent| {
    //     e.prevent_default();
    //     let (Some(username), Some(password), Some(password_confirmation)) = (
    //         input_username.get_untracked(),
    //         // register_email.get(),
    //         input_password.get_untracked(),
    //         input_password_confirmatoin.get_untracked(),
    //     ) else {
    //         return;
    //     };
    //
    //     let username = proccess_username(username.value());
    //     // let email = proccess_email(email.value());
    //     let password = proccess_password(password.value(), Some(password_confirmation.value()));
    //     let token = get_query_token();
    //
    //     register_username_err.set(username.clone().err().unwrap_or_default());
    //     // register_email_err.set(email.clone().err().unwrap_or_default());
    //     register_password_err.set(password.clone().err().unwrap_or_default());
    //     register_general_err.set(if token.is_some() {
    //         String::new()
    //     } else {
    //         String::from("token is missing from; invalid link")
    //     });
    //
    //     let (Ok(username), Ok(password), Some(invite_token)) = (username, password, token) else {
    //         return;
    //     };
    //
    //     api.register(username, invite_token, password)
    //         .send_web(move |result| {
    //             // let navigate = navigate.clone();
    //             async move {
    //                 let err: Result<(), String> = match result {
    //                     Ok(ServerRes::Ok) => {
    //                         let res = global_state.update_auth_now().await;
    //                         match res {
    //                             Ok(ServerRes::User { username }) => {
    //                                 let result = global_state.update_auth_now().await;
    //                                 match result {
    //                                     Ok(S) => Ok(()),
    //                                     Err(err) => Err(err.to_string()),
    //                                 }
    //                             }
    //                             res => Err(format!("expected User, received {res:?}")),
    //                         }
    //                     }
    //                     Ok(res) => Err(format!("error, expected OK, received: {res:?}")),
    //                     Err(ServerErr::RegistrationErr(ServerRegistrationErr::TokenExpired)) => {
    //                         Err("This invite link is already expired.".to_string())
    //                     }
    //                     Err(ServerErr::RegistrationErr(ServerRegistrationErr::TokenUsed)) => {
    //                         Err("This invite link was already used.".to_string())
    //                     }
    //                     Err(ServerErr::RegistrationErr(ServerRegistrationErr::TokenNotFound)) => {
    //                         Err("This invite link is invalid.".to_string())
    //                     }
    //                     Err(err) => Err(err.to_string()),
    //                 };
    //                 if let Err(err) = err {
    //                     error!(err);
    //                     register_general_err.set(err);
    //                 }
    //             }
    //         });
    // };

    Register {
        field_reg_stage,
        field_err_general,
        field_err_username,
        field_err_password,
        field_token,
        field_email,
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
