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
        // let password_confirmation = password_confirmtion.as_ref().to_owned();

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
    

    
    // use regex::Regex;
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
            // thread_local! {
            //     static RE: LazyCell<Regex> = LazyCell::new(|| Regex::new(
            //         // r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#,
            //         r#"^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$"#
            //     ).unwrap());
            // }
            // RE.with(|re| re.is_match(self.as_ref()))
            // EmailAddress::is_valid(self.as_ref())
            // if
            let email = self.as_ref();
            let mut email_chars = email.chars().enumerate();
            // let mut state: usize = 0;
            if email_chars
                .next()
                .map(|(i, char)| !(char == '-' || char == '.' || char.is_alphanumeric()))
                .unwrap_or(true)
            {
                trace!("invalid 1");
                return false;
            }

            let mut symbol_a: usize = 0;
            while let Some((i, char)) = email_chars.next() {
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
                .map(|(i, char)| !(char == '-' || char == '.' || char.is_alphanumeric()))
                .unwrap_or(true)
            {
                trace!("invalid 3");
                return false;
            }

            let mut last_dot: usize = 0;
            while let Some((i, char)) = email_chars.next() {
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

            let mut email_chars = email.chars().skip(last_dot + 1);
            let mut count = 0;
            while let Some(char) = email_chars.next() {
                count += 1;
                if !char.is_alphanumeric() {
                    trace!("invalid 5");
                    return false;
                }
            }

            if count < 2 || count > 4 {
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
