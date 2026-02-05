#![recursion_limit = "512"]
// #[macro_use]
// extern crate macro_rules_attribute;

pub mod server;
// pub mod controller;
pub mod api;
#[cfg(feature = "ssr")]
pub mod db;
pub mod view;

// pub struct OrdFloat(u64);

#[cfg(feature = "ssr")]
pub fn init_test_log() {
    let _ = tracing_subscriber::fmt()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true),
        )
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        // .with_writer(file)
        .try_init();
}

pub fn get_timestamp() -> u128 {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            use std::time::{SystemTime, UNIX_EPOCH};
            let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            time.as_nanos()
        } else {
            use wasm_bindgen::JsValue;
            use web_sys::js_sys::Date;
            // let time = Date::new(&JsValue::null());
            let time = Date::new_0();
            let time = time.get_time() as u64;
            let time = time as u128 * 1000000;
            time
        }
    }
}

pub mod valid {
    use tracing::trace;

    pub mod auth {
        use super::Validator;
        use tracing::trace;

        pub fn proccess_username(username: impl AsRef<str>) -> Result<String, String> {
            let mut errors = String::new();
            let username = username.as_ref().trim();
            match username.len() {
                len if len < 3 => errors += "username must be at least 3 characters length\n",
                len if len > 32 => errors += "username must be shorter than 33 characters length\n",
                _ => {}
            }
            let mut username_chars = username.chars();
            match username_chars.next() {
                Some(c) if c.is_alphabetic() => {}
                _ => errors += "username must start with alphabetic character\n",
            }
            for c in username_chars {
                if !(c.is_alphanumeric() || c == '_') {
                    errors += "username must be alphanumeric\n";
                }
            }

            if errors.is_empty() {
                Ok(username.to_string())
            } else {
                let _ = errors.pop();
                trace!("errors {errors}");
                Err(errors)
            }
        }

        pub fn proccess_post_title<S: AsRef<str>>(title: S) -> Result<String, String> {
            let mut errors = String::new();
            let input = title.as_ref().trim().to_string();
            // if !username.is_first_char_alphabetic() {
            //     errors += "title must start with alphabetic character\n";
            // }
            // if !username.is_alphanumerc() {
            //     errors += "title must be alphanumeric\n";
            // }
            if input.is_smaller_than(1) {
                errors += "title must be at least 1 characters length\n";
            }
            if input.is_bigger_than(120) {
                errors += "title must be shorter than 121 characters length\n";
            }

            if errors.is_empty() {
                Ok(input)
            } else {
                let _ = errors.pop();
                trace!("errors {errors}");
                Err(errors)
            }
        }

        pub fn proccess_post_description<S: AsRef<str>>(description: S) -> Result<String, String> {
            let mut errors = String::new();
            let input = description.as_ref().trim().to_string();
            // if !username.is_first_char_alphabetic() {
            //     errors += "description must start with alphabetic character\n";
            // }
            // if !username.is_alphanumerc() {
            //     errors += "description must be alphanumeric\n";
            // }
            // if username.is_smaller_than(1) {
            //     errors += "description must be at least 1 characters length\n";
            // }
            if input.is_bigger_than(10240) {
                errors += "description must be shorter than 10241 characters length\n";
            }

            if errors.is_empty() {
                Ok(input)
            } else {
                let _ = errors.pop();
                trace!("errors {errors}");
                Err(errors)
            }
        }

        pub fn proccess_password<S: Into<String>>(
            password: S,
            password_confirmation: Option<S>,
        ) -> Result<String, String> {
            let mut errors = String::new();
            let password: String = password.into();

            if password.is_smaller_than(12) {
                errors += "password must be at least 12 characters long\n";
            }
            if password.is_bigger_than(128) {
                errors += "password must be shorter than 129 characters\n";
            }
            if !password.is_containing_number() {
                errors += "password must contain at least one number\n";
            }
            if !password.is_containing_symbol() {
                errors += "password must contain at least one symbol\n";
            }
            if password_confirmation
                .map(|v| v.into() as String)
                .map(|v| v != password)
                .unwrap_or_default()
            {
                errors += "password and password confirmation dont match\n";
            }

            if errors.is_empty() {
                Ok(password)
            } else {
                let _ = errors.pop();
                trace!("errors {errors}");
                Err(errors)
            }
        }

        pub fn proccess_email<S: AsRef<str>>(email: S) -> Result<String, String> {
            let mut errors = String::new();
            let email = email.as_ref().trim().to_owned();
            if email.is_empty() {
                errors += "email cannot be empty\n";
            }

            if errors.is_empty() {
                Ok(email)
            } else {
                let _ = errors.pop();
                trace!("errors {errors}");
                Err(errors)
            }
        }

        #[cfg(test)]
        mod auth_tests {
            use super::{proccess_email, proccess_password, proccess_username};
            use test_log::test;

            #[test]
            fn test_proccess_username() {
                assert!(proccess_username("hey").is_ok());
                assert!(proccess_username("hey%").is_err());
                assert!(proccess_username("he").is_err());
                assert!(proccess_username("00000000000000000000000000000000").is_err());
                assert!(proccess_username("a0000000000000000000000000000000").is_ok());
                assert!(proccess_username("a00000000000000000000000000000000").is_err());
            }

            #[test]
            fn test_proccess_password() {
                assert!(proccess_password("password", Some("password")).is_err());
                assert!(proccess_password("password123", Some("password123")).is_err());
                assert!(proccess_password("passw*rd123", Some("passw*rd123")).is_err());
                assert!(proccess_password("passw*rd1232", Some("passw*rd1231")).is_err());
                assert!(proccess_password("passw*rd1232", Some("passw*rd1232")).is_ok());
                assert!(proccess_password("passw*rd1232", None).is_ok());
            }

            #[test]
            fn test_proccess_email() {
                assert!(proccess_email("hey@hey..com").is_ok());
                // assert!(proccess_email("heyhey.com").is_err());
                assert!(proccess_email("").is_err());
                // assert!(proccess_email("hey@hey.com").is_ok());
            }
        }
    }

    pub trait Validator {
        fn is_alphanumerc(&self) -> bool;
        fn is_containing_symbol(&self) -> bool;
        fn is_containing_number(&self) -> bool;
        fn is_first_char_alphabetic(&self) -> bool;
        fn is_smaller_than(&self, size: usize) -> bool;
        fn is_bigger_than(&self, size: usize) -> bool;
        // fn is_email(&self) -> bool;
    }

    impl<S: AsRef<str>> Validator for S {
        fn is_alphanumerc(&self) -> bool {
            self.as_ref().chars().all(|c| c.is_alphanumeric())
        }
        fn is_containing_symbol(&self) -> bool {
            self.as_ref().chars().any(|c| !c.is_alphanumeric())
        }
        fn is_containing_number(&self) -> bool {
            self.as_ref().chars().any(|c| c.is_numeric())
        }
        fn is_first_char_alphabetic(&self) -> bool {
            self.as_ref()
                .chars()
                .next()
                .map(|c| c.is_alphabetic())
                .unwrap_or_default()
        }
        // fn is_email(&self) -> bool {
        //     let email = self.as_ref();
        //     let mut email_chars = email.chars().enumerate();
        //     if email_chars
        //         .next()
        //         .map(|(_i, char)| !(char == '-' || char == '.' || char.is_alphanumeric()))
        //         .unwrap_or(true)
        //     {
        //         trace!("invalid 1");
        //         return false;
        //     }
        //
        //     let mut symbol_a: usize = 0;
        //     for (i, char) in email_chars.by_ref() {
        //         if char == '@' {
        //             symbol_a = i;
        //             break;
        //         } else if char == '-' || char == '.' || char.is_alphanumeric() {
        //             continue;
        //         } else {
        //             trace!("invalid 2");
        //             return false;
        //         }
        //     }
        //     if symbol_a == 0 {
        //         trace!("invalid 2.5");
        //         return false;
        //     }
        //     if email_chars
        //         .next()
        //         .map(|(_i, char)| !(char == '-' || char == '.' || char.is_alphanumeric()))
        //         .unwrap_or(true)
        //     {
        //         trace!("invalid 3");
        //         return false;
        //     }
        //
        //     let mut last_dot: usize = 0;
        //     for (i, char) in email_chars {
        //         if char == '.' {
        //             last_dot = i;
        //         } else if char == '-' || char == '.' || char.is_alphanumeric() {
        //             continue;
        //         } else {
        //             trace!("invalid 4");
        //             return false;
        //         }
        //     }
        //
        //     if last_dot == 0 {
        //         trace!("invalid 4.5");
        //         return false;
        //     }
        //
        //     let email_chars = email.chars().skip(last_dot + 1);
        //     let mut count = 0;
        //     for char in email_chars {
        //         count += 1;
        //         if !char.is_alphanumeric() {
        //             trace!("invalid 5");
        //             return false;
        //         }
        //     }
        //
        //     if !(2..=4).contains(&count) {
        //         trace!("invalid 6: {count}");
        //         return false;
        //     }
        //
        //     true
        // }
        fn is_bigger_than(&self, size: usize) -> bool {
            self.as_ref().len() > size
        }
        fn is_smaller_than(&self, size: usize) -> bool {
            self.as_ref().len() < size
        }
    }

    #[cfg(test)]
    mod valid_tests {
        use super::Validator;
        use test_log::test;

        #[test]
        fn test_validator() {
            assert!("input".is_alphanumerc());
            assert!(!"input@".is_alphanumerc());
            assert!(!"input".is_smaller_than(5));
            assert!("input".is_smaller_than(6));
            assert!(!"input".is_bigger_than(5));
            assert!("input".is_bigger_than(4));
            // assert!("hey@hey.com".is_email());
            // assert!("hey@hey..com".is_email());
            // assert!(!"@hey.com".is_email());
            // assert!(!"heyhey.com".is_email());
            // assert!(!"h@.com".is_email());
            // assert!(!"hhey.com".is_email());
            // assert!(!"hhey@".is_email());
            // assert!(!"h@h.".is_email());
            // assert!(!"h@h.h".is_email());
            // assert!("h@h.hh".is_email());
            assert!("hey@hey..com".is_first_char_alphabetic());
            assert!(!"0ey@hey..com".is_first_char_alphabetic());
            assert!("abcd#e".is_containing_symbol());
            assert!(!"abcd4e".is_containing_symbol());
            assert!("abcd4e".is_containing_number());
            assert!(!"abcd#e".is_containing_number());
        }
    }
}

pub mod path {
    use leptos::prelude::*;
    use leptos_router::{OptionalParamSegment, ParamSegment, StaticSegment, WildcardSegment, path};

    use crate::{
        api::EmailChangeStage,
        view::app::hook::{
            use_email_change::EmailChangeFormStage,
            use_password_change::{ChangePasswordFormStage, ChangePasswordQueryFields},
            use_register::{RegQueryFields, RegStage},
            use_username_change::ChangeUsernameFormStage,
        },
    };

    pub const PATH_API: &'static str = "/api";

    // post like
    pub const PATH_API_POST_LIKE_ADD: &'static str = "/add_post_like";
    pub const PATH_API_POST_LIKE_CHECK: &'static str = "/check_post_like";
    pub const PATH_API_POST_LIKE_DELETE: &'static str = "/delete_post_like";

    // change password
    pub const PATH_API_CHANGE_PASSWORD_SEND: &'static str = "/send_change_password";
    pub const PATH_API_CHANGE_PASSWORD_CONFIRM: &'static str = "/confirm_change_password";
    //

    pub const PATH_API_REGISTER: &'static str = "/register";
    pub const PATH_API_LOGIN: &'static str = "/login";
    pub const PATH_API_LOGOUT: &'static str = "/logout";
    pub const PATH_API_USER: &'static str = "/user";
    pub const PATH_API_ACC: &'static str = "/acc";
    pub const PATH_API_INVITE_DECODE: &'static str = "/invite_decode";
    pub const PATH_API_CHANGE_USERNAME: &'static str = "/change_username";
    pub const PATH_API_CHANGE_EMAIL: &'static str = "/change_email";
    pub const PATH_API_CHANGE_EMAIL_STATUS: &'static str = "/change_email_status";
    // pub const PATH_API_CHANGE_EMAIL: &'static str = "/change_email";
    pub const PATH_API_SEND_EMAIL_INVITE: &'static str = "/send_email_invite";
    pub const PATH_API_RESEND_EMAIL_CHANGE: &'static str = "/resend_email_change";
    pub const PATH_API_RESEND_EMAIL_NEW: &'static str = "/resend_email_new";
    pub const PATH_API_SEND_EMAIL_CHANGE: &'static str = "/send_email_change";
    pub const PATH_API_SEND_EMAIL_NEW: &'static str = "/send_email_new";
    // pub const PATH_API_EMAIL_NEW: &'static str = "/email_change";
    pub const PATH_API_CANCEL_EMAIL_CHANGE: &'static str = "/cancel_email_change";
    pub const PATH_API_CONFIRM_EMAIL_CHANGE: &'static str = "/confirm_email_change";
    pub const PATH_API_CONFIRM_EMAIL_NEW: &'static str = "/confirm_email_new";
    pub const PATH_API_POST_ADD: &'static str = "/post/add";
    pub const PATH_API_POST_GET: &'static str = "/post/get";
    pub const PATH_API_POST_GET_OLDER: &'static str = "/post/get_older";
    pub const PATH_API_POST_GET_NEWER: &'static str = "/post/get_newer";
    pub const PATH_API_POST_GET_OLDER_OR_EQUAL: &'static str = "/post/get_older_or_equal";
    pub const PATH_API_POST_GET_NEWER_OR_EQUAL: &'static str = "/post/get_newer_or_equal";
    pub const PATH_API_USER_POST_GET_OLDER: &'static str = "/post/get_user_older";
    pub const PATH_API_USER_POST_GET_NEWER: &'static str = "/post/get_user_newer";
    pub const PATH_API_USER_POST_GET_OLDER_OR_EQUAL: &'static str = "/post/get_user_older_or_equal";
    pub const PATH_API_USER_POST_GET_NEWER_OR_EQUAL: &'static str = "/post/get_user_newer_or_equal";
    pub const PATH_HOME: &'static str = "/";
    pub const PATH_HOME_BS: () = path!("/");
    pub const PATH_U_USER: &'static str = "/u/:user";
    pub const PATH_LOGIN: &'static str = "/login";
    pub const PATH_LOGIN_BS: (StaticSegment<&'static str>,) = path!("/login");
    pub const PATH_REGISTER: &'static str = "/register";
    pub const PATH_UPLOAD: &'static str = "/upload";
    pub const PATH_SETTINGS: &'static str = "/settings";

    pub fn link_post_with_history(
        user: impl AsRef<str>,
        post: impl AsRef<str>,
        scroll: usize,
    ) -> String {
        format!("/u/{}/{}?s={}", user.as_ref(), post.as_ref(), scroll,)
    }

    pub fn link_home() -> String {
        "/".to_string()
    }
    pub fn link_post(user: impl AsRef<str>, post: impl AsRef<str>) -> String {
        format!("/u/{}/{}", user.as_ref(), post.as_ref(),)
    }
    pub fn link_img(hash: impl AsRef<str>, extension: impl AsRef<str>) -> String {
        format!("/file/{}.{}", hash.as_ref(), extension.as_ref())
    }

    pub fn link_user(user: impl AsRef<str>) -> String {
        format!("/u/{}", user.as_ref())
    }

    pub fn link_settings() -> String {
        PATH_SETTINGS.to_string()
    }

    pub fn link_login() -> String {
        PATH_LOGIN.to_string()
    }

    pub fn link_login_form_password_send() -> String {
        link_login_form_password(
            ChangePasswordFormStage::Send,
            None::<String>,
            None::<String>,
        )
    }

    pub fn link_login_form_password_confirm(
        email: impl Into<String>,
        confirm_key: impl Into<String>,
    ) -> String {
        link_login_form_password(
            ChangePasswordFormStage::Confirm,
            Some(email),
            Some(confirm_key),
        )
    }

    pub fn link_login_form_password(
        stage: ChangePasswordFormStage,
        email: Option<impl Into<String>>,
        confirm_key: Option<impl Into<String>>,
    ) -> String {
        format!(
            "{}{}",
            link_login(),
            query_form_password(stage, email, confirm_key),
        )
    }

    pub fn link_settings_form_email_current_send(
        old_email: impl Into<String>,
        stage_error: Option<String>,
        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::CurrentSendConfirm,
            None,
            Some(old_email.into()),
            None,
            None,
            stage_error,
            general_info,
            None,
        )
    }

    pub fn link_settings_form_email_current_click(
        email_change_id: String,
        expires: u128,
        old_email: impl Into<String>,
        stage_error: Option<String>,
        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::CurrentClickConfirm,
            Some(email_change_id),
            Some(old_email.into()),
            None,
            None,
            stage_error,
            general_info,
            Some(expires),
        )
    }

    pub fn link_settings_form_email_current_confirm(
        email_change_id: String,
        expires: u128,
        old_email: impl Into<String>,
        confirm_token: impl Into<String>,
        stage_error: Option<String>,
        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::CurrentConfirm,
            Some(email_change_id),
            Some(old_email.into()),
            None,
            Some(confirm_token.into()),
            stage_error,
            general_info,
            Some(expires),
        )
    }

    pub fn link_settings_form_email_new_send(
        email_change_id: String,
        expires: u128,
        old_email: impl Into<String>,
        stage_error: Option<String>,

        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::NewEnterEmail,
            Some(email_change_id),
            Some(old_email.into()),
            None,
            None,
            stage_error,
            general_info,
            Some(expires),
        )
    }

    pub fn link_settings_form_email_new_click(
        email_change_id: String,
        expires: u128,
        old_email: impl Into<String>,
        new_email: impl Into<String>,
        stage_error: Option<String>,
        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::NewClickConfirm,
            Some(email_change_id),
            Some(old_email.into()),
            Some(new_email.into()),
            None,
            stage_error,
            general_info,
            Some(expires),
        )
    }

    pub fn link_settings_form_email_new_confirm(
        email_change_id: String,
        expires: u128,
        old_email: impl Into<String>,
        new_email: impl Into<String>,
        confirm_token: impl Into<String>,
        stage_error: Option<String>,
        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::NewConfirmEmail,
            Some(email_change_id),
            Some(old_email.into()),
            Some(new_email.into()),
            Some(confirm_token.into()),
            stage_error,
            general_info,
            Some(expires),
        )
    }

    pub fn link_settings_form_email_final_confirm(
        email_change_id: String,
        expires: u128,
        old_email: impl Into<String>,
        new_email: impl Into<String>,
        stage_error: Option<String>,
        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::FinalConfirm,
            Some(email_change_id),
            Some(old_email.into()),
            Some(new_email.into()),
            None,
            stage_error,
            general_info,
            Some(expires),
        )
    }

    pub fn link_settings_form_email_completed(
        email_change_id: String,
        old_email: impl Into<String>,
        new_email: impl Into<String>,
        stage_error: Option<String>,
        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::Completed,
            Some(email_change_id),
            Some(old_email.into()),
            Some(new_email.into()),
            None,
            stage_error,
            general_info,
            None,
        )
    }

    // pub struct LinkSettingsFormEmailBuilder {
    //     pub stage: EmailChangeFormStage,
    //     pub new_email: Option<String>,
    //     pub confirm_token: Option<String>,
    //     pub stage_error: Option<String>,
    // }

    pub fn link_settings_form_email(
        stage: EmailChangeFormStage,
        email_change_id: Option<String>,
        old_email: Option<String>,
        new_email: Option<String>,
        confirm_token: Option<String>,
        stage_error: Option<String>,
        general_info: Option<String>,
        expires: Option<u128>,
    ) -> String {
        format!(
            "{}?email_stage={}{}{}{}{}{}{}{}{}{}{}",
            PATH_SETTINGS,
            stage.to_string(),
            match email_change_id {
                Some(v) => format!("&change_id={v}"),
                None => "".to_string(),
            },
            match old_email {
                Some(v) => format!("&old_email={v}"),
                None => "".to_string(),
            },
            match new_email {
                Some(v) => format!("&new_email={v}"),
                None => "".to_string(),
            },
            if confirm_token.is_some() {
                "&confirm_token="
            } else {
                ""
            },
            confirm_token.unwrap_or_default(),
            if stage_error.is_some() {
                "&stage_error="
            } else {
                ""
            },
            stage_error.unwrap_or_default(),
            if general_info.is_some() {
                "&general_info="
            } else {
                ""
            },
            general_info.unwrap_or_default(),
            match expires {
                Some(v) => format!("&expires={v}"),
                None => "".to_string(),
            }
        )
    }

    pub fn link_settings_form_password(
        stage: ChangePasswordFormStage,
        email: Option<impl Into<String>>,
        confirm_key: Option<impl Into<String>>,
        // old_username: Option<impl Into<String>>,
        // new_username: Option<impl Into<String>>,
        // current_email: impl AsRef<str>,
    ) -> String {
        format!(
            "{}{}",
            PATH_SETTINGS,
            query_form_password(stage, email, confirm_key),
        )
    }

    pub fn query_form_password(
        stage: ChangePasswordFormStage,
        email: Option<impl Into<String>>,
        confirm_key: Option<impl Into<String>>,
    ) -> String {
        format!(
            "?{}={}{}{}",
            ChangePasswordQueryFields::FormStage,
            stage,
            match email {
                Some(v) => format!("&{}={}", ChangePasswordQueryFields::Email, v.into()),
                None => "".to_string(),
            },
            match confirm_key {
                Some(v) => format!("&{}={}", ChangePasswordQueryFields::Token, v.into()),
                None => "".to_string(),
            },
            // match new_username {
            //     Some(v) => format!("&new_username={}", v.into()),
            //     None => "".to_string(),
            // },
        )
    }

    pub fn link_settings_form_password_confirm(
        email: impl Into<String>,
        confirm_key: impl Into<String>,
    ) -> String {
        link_settings_form_password(
            ChangePasswordFormStage::Confirm,
            Some(email),
            Some(confirm_key),
        )
    }

    pub fn link_settings_form_password_send(email: impl Into<String>) -> String {
        link_settings_form_password(ChangePasswordFormStage::Send, Some(email), None::<String>)
    }

    pub fn query_settings_form_password_send(email: impl Into<String>) -> String {
        query_form_password(ChangePasswordFormStage::Send, Some(email), None::<String>)
    }

    pub fn link_settings_form_username(
        stage: ChangeUsernameFormStage,
        old_username: Option<impl Into<String>>,
        new_username: Option<impl Into<String>>,
        // current_email: impl AsRef<str>,
    ) -> String {
        format!(
            "{}{}",
            PATH_SETTINGS,
            query_settings_form_username(stage, old_username, new_username)
        )
    }

    pub fn query_settings_form_username(
        stage: ChangeUsernameFormStage,
        old_username: Option<impl Into<String>>,
        new_username: Option<impl Into<String>>,
        // current_email: impl AsRef<str>,
    ) -> String {
        format!(
            "?form_stage={}{}{}",
            stage,
            match old_username {
                Some(v) => format!("&old_username={}", v.into()),
                None => "".to_string(),
            },
            match new_username {
                Some(v) => format!("&new_username={}", v.into()),
                None => "".to_string(),
            },
        )
    }

    pub fn link_reg_invite() -> String {
        "/register".to_string()
    }

    pub fn link_reg_check_email<Email: AsRef<str>>(email: Email) -> String {
        format!(
            "{}?{}={}&{}={}",
            PATH_REGISTER,
            RegQueryFields::Stage,
            RegStage::CheckEmail,
            RegQueryFields::Email,
            email.as_ref()
        )
    }

    pub fn link_reg_finish<Token: AsRef<str>>(token: Token, err_general: Option<String>) -> String {
        format!(
            "{}?{}={}&{}={}{}",
            PATH_REGISTER,
            RegQueryFields::Stage,
            RegStage::Reg,
            RegQueryFields::Token,
            token.as_ref(),
            match err_general {
                Some(err) => format!("&{}={err}", RegQueryFields::ErrGeneral),
                None => String::new(),
            }
        )
    }
}
// #[cfg(test)]
// mod tests {
//     use test_log::test;
//     use tracing::trace;
//
//     #[derive(
//         Clone,
//         Debug,
//         PartialEq,
//         PartialOrd,
//         Default,
//         serde::Serialize,
//         serde::Deserialize,
//         strum::EnumString,
//         strum::EnumIter,
//         strum::Display,
//     )]
//     #[strum(serialize_all = "lowercase")]
//     pub enum Foo {
//         #[default]
//         CurrentSendConfirm,
//         CurrentClickConfirm {
//             data: String,
//         },
//         CurrentConfirm,
//         NewEnterEmail,
//         NewClickConfirm,
//         NewConfirmEmail,
//         FinalConfirm,
//         Completed,
//     }
//
//     #[derive(
//         Clone, Debug, PartialEq, PartialOrd, Default, serde::Serialize, serde::Deserialize,
//     )]
//     struct Bar {
//         one: usize,
//         two: usize,
//     }
//
//     #[test]
//     fn url_test() {
//         // let a = serde_urlencoded::to_string(Foo::CurrentClickConfirm {
//         //     data: "wowza".to_string(),
//         // });
//         let foo = Foo::CurrentClickConfirm {
//             data: "wowza".to_string(),
//         };
//         let fff = serde_json::to_string(&foo).unwrap();
//         trace!("{fff}");
//         // let fff = [(
//         //     "wtf",
//         //     r#"
//         //         {
//         //           "firstName": "John",
//         //           "lastName": "Doe",
//         //           "isStudent": false,
//         //           "age": 30
//         //         }
//         //     "#,
//         // )];
//         // let meal = &[
//         //     ("bread", "baguette"),
//         //     ("cheese", "comté"),
//         //     ("meat", "ham"),
//         //     ("fat", "butter"),
//         // ];
//         // let a = serde_urlencoded::to_string(Foo::CurrentSendConfirm);
//         // let b = serde_urlencoded::to_string(Bar { one: 1, two: 2 });
//         let c = serde_urlencoded::to_string([("wtf", fff)]);
//         let d = serde_urlencoded::to_string([("huh", foo.clone())]);
//         let g = serde_qs::to_string(&[("huh", foo)]);
//
//         // trace!("{a:?}");
//         trace!("{c:?}");
//         // trace!("{c:?}");
//         trace!("{d:?}");
//         trace!("{g:?}");
//     }
// }
