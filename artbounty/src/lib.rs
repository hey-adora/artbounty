#![recursion_limit = "512"]
#[macro_use]
extern crate macro_rules_attribute;

pub mod server;
// pub mod controller;
pub mod api;
#[cfg(feature = "ssr")]
pub mod db;
pub mod view;

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
        view::app::{
            hook::EmailChangeFormStage,
            page::settings::{SelectedForm, UsernameChangeStage},
        },
    };

    pub const PATH_API: &'static str = "/api";
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

    #[derive(Debug, Clone, PartialEq, strum::EnumString, strum::Display)]
    #[strum(serialize_all = "lowercase")]
    pub enum RegKind {
        Reg,
        CheckEmail,
        // Loading,
    }
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

    pub fn link_settings_form_email_current_send(
        stage_error: Option<String>,
        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::CurrentSendConfirm,
            None,
            None,
            stage_error,
            general_info,
            None,
        )
    }

    pub fn link_settings_form_email_current_click(
        expires: u128,
        stage_error: Option<String>,
        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::CurrentClickConfirm,
            None,
            None,
            stage_error,
            general_info,
            Some(expires),
        )
    }

    pub fn link_settings_form_email_current_confirm(
        expires: u128,
        confirm_token: impl Into<String>,
        stage_error: Option<String>,
        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::CurrentConfirm,
            None,
            Some(confirm_token.into()),
            stage_error,
            general_info,
            Some(expires),
        )
    }

    pub fn link_settings_form_email_new_send(
        expires: u128,
        stage_error: Option<String>,

        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::NewEnterEmail,
            None,
            None,
            stage_error,
            general_info,
            Some(expires),
        )
    }

    pub fn link_settings_form_email_new_click(
        expires: u128,
        new_email: impl Into<String>,
        stage_error: Option<String>,
        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::NewClickConfirm,
            Some(new_email.into()),
            None,
            stage_error,
            general_info,
            Some(expires),
        )
    }

    pub fn link_settings_form_email_new_confirm(
        expires: u128,
        new_email: impl Into<String>,
        confirm_token: impl Into<String>,
        stage_error: Option<String>,
        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::NewConfirmEmail,
            Some(new_email.into()),
            Some(confirm_token.into()),
            stage_error,
            general_info,
            Some(expires),
        )
    }

    pub fn link_settings_form_email_final_confirm(
        expires: u128,
        new_email: impl Into<String>,
        stage_error: Option<String>,
        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::FinalConfirm,
            Some(new_email.into()),
            None,
            stage_error,
            general_info,
            Some(expires),
        )
    }

    pub fn link_settings_form_email_completed(
        new_email: impl Into<String>,
        stage_error: Option<String>,
        general_info: Option<String>,
    ) -> String {
        link_settings_form_email(
            EmailChangeFormStage::Completed,
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
        new_email: Option<String>,
        confirm_token: Option<String>,
        stage_error: Option<String>,
        general_info: Option<String>,
        expires: Option<u128>,
    ) -> String {
        format!(
            "{}?selected_form={}&email_stage={}{}{}{}{}{}{}{}{}{}",
            PATH_SETTINGS,
            SelectedForm::ChangeEmail,
            stage.to_string(),
            if new_email.is_some() {
                "&new_email="
            } else {
                ""
            },
            new_email.unwrap_or_default(),
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
                None => "".to_string()
            }
        )
    }

    pub fn link_settings_form_username(
        stage: UsernameChangeStage,
        old_username: Option<impl Into<String>>,
        new_username: Option<impl Into<String>>,
        // current_email: impl AsRef<str>,
    ) -> String {
        format!(
            "{}?selected_form={}{}{}{}{}",
            PATH_SETTINGS,
            SelectedForm::UsernameChanged,
            if old_username.is_some() {
                "&old_username="
            } else {
                ""
            },
            old_username.map(|v| v.into()).unwrap_or_default(),
            if new_username.is_some() {
                "&new_username="
            } else {
                ""
            },
            new_username.map(|v| v.into()).unwrap_or_default(),
        )
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
