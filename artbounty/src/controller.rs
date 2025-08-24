pub mod auth;
pub mod post;

pub mod valid {
    use tracing::trace;

    pub mod auth {
        use super::Validator;
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
                assert!(proccess_email("heyhey.com").is_err());
                assert!(proccess_email("").is_err());
                assert!(proccess_email("hey@hey.com").is_ok());
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
pub mod encode {
    use bytecheck::CheckBytes;
    use http::{HeaderMap, StatusCode};
    use leptos::prelude::location;
    use reqwest::RequestBuilder;

    use crate::path::PATH_API;
    use rkyv::{
        Archive, Deserialize,
        api::high::{HighDeserializer, HighSerializer, HighValidator},
        rancor::Strategy,
        result::ArchivedResult,
        ser::{allocator::ArenaHandle, sharing::Share},
        util::AlignedVec,
    };
    use thiserror::Error;
    use tracing::{debug, error, trace};

    #[cfg(feature = "ssr")]
    pub async fn decode_multipart<ClientInput, ServerErr>(
        mut multipart: axum::extract::Multipart,
    ) -> Result<ClientInput, ResErr<ServerErr>>
    where
        ServerErr: std::error::Error + 'static,
        ClientInput: Archive,
        ClientInput::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ClientInput, HighDeserializer<rkyv::rancor::Error>>,
    {
        let mut bytes = bytes::Bytes::new();
        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|_| ResErr::ServerDecodeErr(ServerDecodeErr::NextFieldFailed))?
        {
            trace!("2");
            if field.name().map(|name| name == "data").unwrap_or_default() {
                trace!("3");
                bytes = field
                    .bytes()
                    .await
                    .map_err(|_| ResErr::ServerDecodeErr(ServerDecodeErr::FieldToBytesFailed))?;
            }
        }

        trace!("4");
        let archived = rkyv::access::<ClientInput::Archived, rkyv::rancor::Error>(&bytes)
            .map_err(|_| ResErr::ServerDecodeErr(ServerDecodeErr::RkyvAccessErr))?;
        trace!("5");
        let client_input = rkyv::deserialize::<ClientInput, rkyv::rancor::Error>(archived)
            .map_err(|_| ResErr::ServerDecodeErr(ServerDecodeErr::RkyvErr))?;
        trace!("6");

        Ok(client_input)
    }

    #[cfg(feature = "ssr")]
    pub fn encode_server_output<ServerOutput, ServerErr>(
        response: Result<ServerOutput, ResErr<ServerErr>>,
    ) -> axum::response::Response
    where
        ServerOutput: for<'a> rkyv::Serialize<
                Strategy<
                    rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                    bytecheck::rancor::Error,
                >,
            > + std::fmt::Debug,
        ServerErr: for<'a> rkyv::Serialize<
                Strategy<
                    rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                    bytecheck::rancor::Error,
                >,
            > + Archive
            + std::error::Error
            + std::fmt::Debug
            + 'static,
        ServerErr::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ServerErr, HighDeserializer<rkyv::rancor::Error>>,
    {
        use axum::response::IntoResponse;

        trace!("ENCODING SERVER INPUT: {:?}", response);

        let result = match response {
            Ok(server_output) => {
                let body = Ok(server_output);
                trace!("encoding server output: {body:#?}");
                let body = encode_result::<ServerOutput, ServerErr>(&body);
                trace!("sending body: {body:?}");
                (axum::http::StatusCode::OK, body).into_response()
            }
            Err(ResErr::ServerErr(err)) => {
                let body = Err(ResErr::ServerErr(err));
                trace!("encoding server output: {body:#?}");
                let body = encode_result::<ServerOutput, ServerErr>(&body);
                trace!("sending body: {body:?}");
                (axum::http::StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            etc => match_output(etc),
        };

        result
    }

    #[cfg(feature = "ssr")]
    pub fn encode_server_output_custom<ServerOutput, ServerErr>(
        response: Result<ServerOutput, ResErr<ServerErr>>,
    ) -> axum::response::Response
    where
        ServerOutput: for<'a> rkyv::Serialize<
                Strategy<
                    rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                    bytecheck::rancor::Error,
                >,
            > + std::fmt::Debug
            + axum::response::IntoResponse,
        ServerErr: for<'a> rkyv::Serialize<
                Strategy<
                    rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                    bytecheck::rancor::Error,
                >,
            > + Archive
            + std::error::Error
            + axum::response::IntoResponse
            + std::fmt::Debug
            + 'static,
        ServerErr::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ServerErr, HighDeserializer<rkyv::rancor::Error>>,
    {
        use axum::response::IntoResponse;

        trace!("ENCODING SERVER INPUT: {:?}", response);

        let result = match response {
            Ok(server_output) => server_output.into_response(),
            Err(ResErr::ServerErr(err)) => err.into_response(),
            Err(ResErr::ClientErr(_)) => {
                unreachable!("client error shouldnt be send by the server")
            }
            etc => match_output(etc),
        };

        result
    }

    #[cfg(feature = "ssr")]
    pub fn match_output<ServerOutput, ServerErr>(
        response: Result<ServerOutput, ResErr<ServerErr>>,
    ) -> axum::response::Response
    where
        ServerOutput: for<'a> rkyv::Serialize<
                Strategy<
                    rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                    bytecheck::rancor::Error,
                >,
            > + std::fmt::Debug,
        ServerErr: for<'a> rkyv::Serialize<
                Strategy<
                    rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                    bytecheck::rancor::Error,
                >,
            > + Archive
            + std::error::Error
            + std::fmt::Debug
            + 'static,
        ServerErr::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ServerErr, HighDeserializer<rkyv::rancor::Error>>,
    {
        use axum::response::IntoResponse;

        trace!("ENCODING SERVER INPUT: {:?}", response);

        let result = match response {
            Ok(server_output) => {
                unreachable!("parent fn should have handeled this");
            }
            Err(ResErr::ServerErr(err)) => {
                unreachable!("parent fn should have handeled this");
            }
            Err(ResErr::ClientErr(_)) => {
                unreachable!("client error shouldnt be send by the server");
            }
            Err(ResErr::Unauthorized(ResErrUnauthorized::NoCookie)) => {
                use http::header::{AUTHORIZATION, SET_COOKIE};

                let body: Result<ServerOutput, ResErr<ServerErr>> =
                    Err(ResErr::Unauthorized(ResErrUnauthorized::NoCookie));
                trace!("encoding server output: {body:#?}");
                let body = encode_result::<ServerOutput, ServerErr>(&body);
                trace!("sending body: {body:?}");
                (axum::http::StatusCode::OK, body).into_response()
            }
            Err(ResErr::Unauthorized(ResErrUnauthorized::BadToken)) => {
                use http::header::{AUTHORIZATION, SET_COOKIE};

                let body: Result<ServerOutput, ResErr<ServerErr>> =
                    Err(ResErr::Unauthorized(ResErrUnauthorized::BadToken));
                trace!("encoding server output: {body:#?}");
                let body = encode_result::<ServerOutput, ServerErr>(&body);
                trace!("sending body: {body:?}");
                // let jar = jar.add(Cookie::new(
                //     AUTHORIZATION.as_str(),
                //     "Bearer=DELETED; Secure; HttpOnly; expires=Thu, 01 Jan 1970 00:00:00 GMT",
                // ));
                let headers = axum::response::AppendHeaders([(
                    SET_COOKIE,
                    "authorization=Bearer%3DDELETED%3B%20Secure%3B%20HttpOnly%3B%20expires%3DThu%2C%2001%20Jan%201970%2000%3A00%3A00%20GMT",
                )]);
                (axum::http::StatusCode::UNAUTHORIZED, headers, body).into_response()
            }
            Err(err) => {
                let body: Result<ServerOutput, ResErr<ServerErr>> = Err(err);
                trace!("encoding server output: {body:#?}");
                let body = encode_result::<ServerOutput, ServerErr>(&body);
                trace!("sending body: {body:?}");
                (axum::http::StatusCode::BAD_REQUEST, body).into_response()
            }
        };

        result
    }

    pub fn encode_result<ServerOutput, ServerErr>(
        result: &Result<ServerOutput, ResErr<ServerErr>>,
    ) -> Vec<u8>
    where
        ServerOutput: for<'a> rkyv::Serialize<
                Strategy<
                    rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                    bytecheck::rancor::Error,
                >,
            >,
        ServerErr: for<'a> rkyv::Serialize<
                Strategy<
                    rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                    bytecheck::rancor::Error,
                >,
            > + Archive
            + std::error::Error
            + 'static,
    {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(result)
            .unwrap()
            .to_vec();

        bytes
    }

    pub async fn send_web<ServerOutput, ServerErr>(
        path: impl AsRef<str>,
        input: &impl for<'a> rkyv::Serialize<
            Strategy<
                rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                bytecheck::rancor::Error,
            >,
        >,
    ) -> Result<ServerOutput, ResErr<ServerErr>>
    where
        ServerOutput: Archive + std::fmt::Debug,
        ServerOutput::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ServerOutput, HighDeserializer<rkyv::rancor::Error>>,
        ServerErr: Archive + std::error::Error + std::fmt::Debug + 'static,
        ServerErr::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ServerErr, HighDeserializer<rkyv::rancor::Error>>,
    {
        send_native(location().origin().unwrap(), path, None::<&str>, input).await
    }

    pub async fn send_native<ServerOutput, ServerErr>(
        origin: impl AsRef<str>,
        path: impl AsRef<str>,
        token: Option<impl AsRef<str>>,
        input: &impl for<'a> rkyv::Serialize<
            Strategy<
                rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                bytecheck::rancor::Error,
            >,
        >,
    ) -> Result<ServerOutput, ResErr<ServerErr>>
    where
        ServerOutput: Archive + std::fmt::Debug,
        ServerOutput::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ServerOutput, HighDeserializer<rkyv::rancor::Error>>,
        ServerErr: Archive + std::error::Error + std::fmt::Debug + 'static,
        ServerErr::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ServerErr, HighDeserializer<rkyv::rancor::Error>>,
    {
        let origin = origin.as_ref();
        let path = path.as_ref();
        let mut builder = reqwest::Client::new().post(format!("{origin}{PATH_API}{path}"));
        if let Some(token) = token {
            builder = builder.header(
                http::header::COOKIE,
                format!("authorization=Bearer%3D{}%3B%20Secure%3B%20HttpOnly", token.as_ref()),
            );
        }
        send_builder::<ServerOutput, ServerErr>(builder, input)
            .await
            .1
    }

    pub async fn send_builder<ServerOutput, ServerErr>(
        req_builder: RequestBuilder,
        input: &impl for<'a> rkyv::Serialize<
            Strategy<
                rkyv::ser::Serializer<AlignedVec, ArenaHandle<'a>, Share>,
                bytecheck::rancor::Error,
            >,
        >,
    ) -> (HeaderMap, Result<ServerOutput, ResErr<ServerErr>>)
    where
        ServerOutput: Archive + std::fmt::Debug,
        ServerOutput::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ServerOutput, HighDeserializer<rkyv::rancor::Error>>,
        ServerErr: Archive + std::error::Error + std::fmt::Debug + 'static,
        ServerErr::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
            + Deserialize<ServerErr, HighDeserializer<rkyv::rancor::Error>>,
    {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(input)
            .unwrap()
            .to_vec();

        let part = reqwest::multipart::Part::bytes(bytes);
        let form = reqwest::multipart::Form::new().part("data", part);
        let res = match req_builder
            .multipart(form)
            .send()
            .await
            .inspect_err(|err| error!("client err: {err}"))
            .map_err(|_| ResErr::ClientErr(ClientErr::FailedToSend))
        {
            Ok(res) => res,
            Err(err) => {
                return (HeaderMap::new(), Err(err));
            }
        };

        let headers = res.headers().clone();
        let status = res.status();
        let url = res.url().clone();
        let bytes = res.bytes().await;

        if status == StatusCode::NOT_FOUND
            && bytes.as_ref().map(|v| v.len() == 0).unwrap_or_default()
        {
            debug!("CLIENT RECV:\nstatus: {status}\nclient received headers: {headers:#?}");
            return (
                headers,
                Err(ResErr::ServerEndpointNotFoundErr(url.to_string())),
            );
        }

        let r = match
            bytes
            .inspect(|bytes| debug!("CLIENT RECV:\nstatus: {status}\nclient received: {bytes:?}\nclient received headers: {headers:#?}"))
            .inspect_err(|err| error!("client byte stream err: {err}"))
            .map_err(|_| ResErr::ClientErr(ClientErr::ByteStreamFail))
            .map(|res| res.to_vec())
            .and_then(|body| {
                let archive = rkyv::access::<
                    ArchivedResult<ServerOutput::Archived, ArchivedResErr<ServerErr>>,
                    rkyv::rancor::Error,
                >(&body)
                .map_err(|_| ResErr::ClientErr(ClientErr::from(ClientDecodeErr::RkyvAccessErr)))?;
                rkyv::deserialize::<Result<ServerOutput, ResErr<ServerErr>>, rkyv::rancor::Error>(
                    archive,
                )
                .map_err(|_| ResErr::ClientErr(ClientErr::from(ClientDecodeErr::RkyvErr)))
            }) {
            Ok(res) => res,
            Err(err) => {
                return (headers, Err(err));
            }
        };

        trace!("recv body: {r:#?}");

        (headers, r)
    }

    #[derive(
        Debug,
        Error,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        rkyv::Archive,
        rkyv::Serialize,
        rkyv::Deserialize,
    )]
    pub enum ResErr<E: std::error::Error + 'static> {
        #[error("client error: {0}")]
        ClientErr(ClientErr),

        #[error("server error: {0}")]
        ServerDecodeErr(ServerDecodeErr),

        #[error("server error: endpoint \"{0}\" not found")]
        ServerEndpointNotFoundErr(String),

        #[error("unauthorized")]
        Unauthorized(ResErrUnauthorized),

        #[error("server error: {0}")]
        ServerErr(#[from] E),
    }

    #[derive(
        Debug,
        Error,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        rkyv::Archive,
        rkyv::Serialize,
        rkyv::Deserialize,
    )]
    pub enum ResErrUnauthorized {
        #[error("unauthorized")]
        Unauthorized,

        #[error("no auth cookie")]
        NoCookie,

        #[error("jwt error")]
        BadToken,

        #[error("something is terribly wrong")]
        DbErr,
    }

    #[derive(
        Debug,
        Error,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        rkyv::Archive,
        rkyv::Serialize,
        rkyv::Deserialize,
    )]
    pub enum ClientErr {
        #[error("failed to send")]
        FailedToSend,

        #[error("invalid response")]
        ByteStreamFail,

        #[error("failed to decode response")]
        DecodeErr(#[from] ClientDecodeErr),
    }

    #[derive(
        Error,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        rkyv::Archive,
        rkyv::Serialize,
        rkyv::Deserialize,
    )]
    pub enum ServerDecodeErr {
        #[error("failed to convert data field to bytes")]
        FieldToBytesFailed,

        #[error("failed to parse multipart")]
        NextFieldFailed,

        #[error("data field is missing in multipart")]
        MissingDataField,

        #[error("rkyv failed to access")]
        RkyvAccessErr,

        #[error("rkyv failed to encode")]
        RkyvErr,
    }

    #[derive(
        Error,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        rkyv::Archive,
        rkyv::Serialize,
        rkyv::Deserialize,
    )]
    pub enum ClientDecodeErr {
        #[error("rkyv failed to access")]
        RkyvAccessErr,

        #[error("rkyv failed to encode")]
        RkyvErr,
    }
}

#[cfg(feature = "ssr")]
pub mod app_state {
    use std::{sync::Arc, time::Duration};

    use tokio::sync::Mutex;

    use crate::{
        controller::{
            clock::{Clock, get_timestamp},
            settings::Settings,
        },
        db::{self, DbEngine},
    };

    #[derive(Clone)]
    pub struct AppState {
        pub db: DbEngine,
        pub settings: Settings,
        pub clock: Clock,
    }

    impl AppState {
        pub async fn new() -> Self {
            let settings = Settings::new_from_file();
            let db = db::new_local(&settings.db.path).await;
            let f = move || async move { get_timestamp() };
            let clock = Clock::new(f);

            Self {
                db,
                settings,
                clock,
            }
        }

        pub async fn new_testng(time: Arc<Mutex<Duration>>) -> Self {
            let db = db::new_mem().await;
            let settings = Settings::new_testing();
            let f = move || {
                let time = time.clone();
                async move {
                    let t = *(time.lock().await);
                    t
                }
            };
            let clock = Clock::new(f);

            Self {
                db,
                settings,
                clock,
            }
        }
    }
}

#[cfg(feature = "ssr")]
pub mod settings {
    use config::{Config, File};

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct Settings {
        pub site: Site,
        pub auth: Auth,
        pub db: Db,
    }

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct Auth {
        pub secret: String,
    }

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct Db {
        pub path: String,
    }

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    pub struct Site {
        pub address: String,
        pub files_path: String,
    }

    impl Settings {
        pub fn new_from_file() -> Self {
            Config::builder()
                .add_source(File::with_name("artbounty"))
                .build()
                .unwrap()
                .try_deserialize()
                .unwrap()
        }

        pub fn new_testing() -> Self {
            Self {
                site: Site {
                    address: "http://localhost:3000".to_string(),
                    files_path: "../target/tmp/files".to_string(),
                },
                auth: Auth {
                    secret: "secret".to_string(),
                },
                db: Db {
                    path: "memory".to_string(),
                },
            }
        }
    }
}

#[cfg(feature = "ssr")]
pub mod clock {
    use std::{pin::Pin, sync::Arc, time::Duration};

    #[derive(Clone)]
    pub struct Clock {
        ticker: Arc<
            dyn Fn() -> Pin<Box<dyn Future<Output = Duration> + Sync + Send + 'static>>
                + Sync
                + Send
                + 'static,
        >,
    }

    impl Clock {
        pub fn new<
            F: Fn() -> Fut + Send + Sync + Clone + 'static,
            Fut: Future<Output = Duration> + Send + Sync + 'static,
        >(
            ticker: F,
        ) -> Self {
            let fut = Arc::new(move || {
                let ticker = (ticker.clone())();
                let f: Pin<Box<dyn Future<Output = Duration> + Sync + Send + 'static>> =
                    Box::pin(ticker);
                f
            });

            Self { ticker: fut }
        }

        pub async fn now(&self) -> Duration {
            let mut fut = (self.ticker)();
            let fut = fut.as_mut();
            let duration = fut.await;
            duration
        }
    }

    #[cfg(feature = "ssr")]
    pub fn get_timestamp() -> std::time::Duration {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
    }

    #[cfg(feature = "ssr")]
    pub fn get_nanos() -> u128 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }
}
