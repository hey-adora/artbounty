pub mod server;
pub mod controller;
pub mod view;
#[cfg(feature = "ssr")]
pub mod db;

pub mod path {
    use leptos::prelude::*;
    use leptos_router::{OptionalParamSegment, ParamSegment, StaticSegment, WildcardSegment, path};

    pub const PATH_API: &'static str = "/api";
    pub const PATH_API_LOGOUT: &'static str = "/logout";
    pub const PATH_API_USER: &'static str = "/user";
    pub const PATH_API_PROFILE: &'static str = "/profile";
    pub const PATH_API_INVITE_DECODE: &'static str = "/invite_decode";
    pub const PATH_API_INVITE: &'static str = "/invite";
    pub const PATH_API_REGISTER: &'static str = "/register";
    pub const PATH_API_LOGIN: &'static str = "/login";
    pub const PATH_API_POST_ADD: &'static str = "/post/add";
    pub const PATH_API_POST_GET_AFTER: &'static str = "/post/get_after";
    pub const PATH_HOME: &'static str = "/";
    pub const PATH_HOME_BS: () = path!("/");
    pub const PATH_U_USER: &'static str = "/u/:user";
    pub const PATH_LOGIN: &'static str = "/login";
    pub const PATH_LOGIN_BS: (StaticSegment<&'static str>,) = path!("/login");
    pub const PATH_REGISTER: &'static str = "/register";

    #[derive(Debug, Clone, PartialEq, strum::EnumString, strum::Display)]
    #[strum(serialize_all = "lowercase")]
    pub enum RegKind {
        Reg,
        CheckEmail,
        // Loading,
    }

    pub fn link_check_email<Email: AsRef<str>>(email: Email) -> String {
        format!(
            "{}?kind={}&email={}",
            PATH_REGISTER,
            RegKind::CheckEmail,
            email.as_ref()
        )
    }

    pub fn link_reg<Token: AsRef<str>>(token: Token) -> String {
        format!(
            "{}?kind={}&token={}",
            PATH_REGISTER,
            RegKind::Reg,
            token.as_ref()
        )
    }
}
