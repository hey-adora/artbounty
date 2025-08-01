pub mod fe_router {
    pub mod home {
        pub const PATH: &'static str = "";
    }

    pub mod user {
        pub const PATH: &'static str = "/u/:user";
    }

    pub mod login {
        pub const PATH: &'static str = "/login";
    }

    pub mod registration {
        pub const PATH: &'static str = "/register";

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
                PATH,
                RegKind::CheckEmail,
                email.as_ref()
            )
        }

        pub fn link_reg<Token: AsRef<str>>(token: Token) -> String {
            format!("{}?kind={}&token={}", PATH, RegKind::Reg, token.as_ref())
        }
    }
}

pub mod auth {
    use crate::valid::Validator;
    use tracing::trace;

    pub fn proccess_username<S: AsRef<str>>(username: S) -> Result<String, String> {
        let mut errors = String::new();
        let username = username.as_ref().trim().to_string();
        if !username.is_first_char_alphabetic() {
            errors += "username must start with alphabetic character\n";
        }
        if !username.is_alphanumerc() {
            errors += "username must be alphanumeric\n";
        }
        if username.is_smaller_than(3) {
            errors += "username must be at least 3 characters length\n";
        }
        if username.is_bigger_than(32) {
            errors += "username must be shorter than 33 characters length\n";
        }

        if errors.is_empty() {
            Ok(username)
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
        if !email.is_email() {
            errors += "invalid email\n";
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
        use crate::auth::{proccess_email, proccess_password, proccess_username};
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
            assert!(proccess_email("heyhey.com").is_err());
            assert!(proccess_email("").is_err());
            assert!(proccess_email("hey@hey.com").is_ok());
        }
    }
}
pub mod valid {
    use tracing::trace;

    pub trait Validator {
        fn is_alphanumerc(&self) -> bool;
        fn is_containing_symbol(&self) -> bool;
        fn is_containing_number(&self) -> bool;
        fn is_first_char_alphabetic(&self) -> bool;
        fn is_smaller_than(&self, size: usize) -> bool;
        fn is_bigger_than(&self, size: usize) -> bool;
        fn is_email(&self) -> bool;
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
        fn is_email(&self) -> bool {
            let email = self.as_ref();
            let mut email_chars = email.chars().enumerate();
            if email_chars
                .next()
                .map(|(_i, char)| !(char == '-' || char == '.' || char.is_alphanumeric()))
                .unwrap_or(true)
            {
                trace!("invalid 1");
                return false;
            }

            let mut symbol_a: usize = 0;
            for (i, char) in email_chars.by_ref() {
                if char == '@' {
                    symbol_a = i;
                    break;
                } else if char == '-' || char == '.' || char.is_alphanumeric() {
                    continue;
                } else {
                    trace!("invalid 2");
                    return false;
                }
            }
            if symbol_a == 0 {
                trace!("invalid 2.5");
                return false;
            }
            if email_chars
                .next()
                .map(|(_i, char)| !(char == '-' || char == '.' || char.is_alphanumeric()))
                .unwrap_or(true)
            {
                trace!("invalid 3");
                return false;
            }

            let mut last_dot: usize = 0;
            for (i, char) in email_chars {
                if char == '.' {
                    last_dot = i;
                } else if char == '-' || char == '.' || char.is_alphanumeric() {
                    continue;
                } else {
                    trace!("invalid 4");
                    return false;
                }
            }

            if last_dot == 0 {
                trace!("invalid 4.5");
                return false;
            }

            let email_chars = email.chars().skip(last_dot + 1);
            let mut count = 0;
            for char in email_chars {
                count += 1;
                if !char.is_alphanumeric() {
                    trace!("invalid 5");
                    return false;
                }
            }

            if !(2..=4).contains(&count) {
                trace!("invalid 6: {count}");
                return false;
            }

            true
        }
        fn is_bigger_than(&self, size: usize) -> bool {
            self.as_ref().len() > size
        }
        fn is_smaller_than(&self, size: usize) -> bool {
            self.as_ref().len() < size
        }
    }

    #[cfg(test)]
    mod valid_tests {
        use crate::valid::Validator;
        use test_log::test;

        #[test]
        fn test_validator() {
            assert!("input".is_alphanumerc());
            assert!(!"input@".is_alphanumerc());
            assert!(!"input".is_smaller_than(5));
            assert!("input".is_smaller_than(6));
            assert!(!"input".is_bigger_than(5));
            assert!("input".is_bigger_than(4));
            assert!("hey@hey.com".is_email());
            assert!("hey@hey..com".is_email());
            assert!(!"@hey.com".is_email());
            assert!(!"heyhey.com".is_email());
            assert!(!"h@.com".is_email());
            assert!(!"hhey.com".is_email());
            assert!(!"hhey@".is_email());
            assert!(!"h@h.".is_email());
            assert!(!"h@h.h".is_email());
            assert!("h@h.hh".is_email());
            assert!("hey@hey..com".is_first_char_alphabetic());
            assert!(!"0ey@hey..com".is_first_char_alphabetic());
            assert!("abcd#e".is_containing_symbol());
            assert!(!"abcd4e".is_containing_symbol());
            assert!("abcd4e".is_containing_number());
            assert!(!"abcd#e".is_containing_number());
        }
    }
}
