pub use search::{SearchResult, Query};
pub use refresh_access_token::Token;

use crate::client::errors::{Error, ErrorKind, Result};
use failure::{Context, Fail};
use reqwest;

pub struct Credentials {
    client_id: String,
    client_secret: String,
    access_token: String,
}

impl Credentials {
    pub fn new(client_id: String, client_secret: String, access_token: String) -> Credentials {
        Credentials {
            client_id,
            client_secret,
            access_token,
        }
    }

    pub fn access_token(&self) -> &str {
        &self.access_token
    }
}

pub struct CenterDevice {
    base_url: String,
    credentials: Credentials,
    http_client: reqwest::Client,
}

impl CenterDevice {
    pub fn new(base_url: String, credentials: Credentials) -> CenterDevice {
        CenterDevice {
            base_url,
            credentials,
            http_client: reqwest::Client::new(),
        }
    }

    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }

    pub fn refresh_access_token(&mut self, refresh_token: &str) -> Result<Token> {
        refresh_access_token::refresh_access_token(self, refresh_token)
    }

    pub fn search(&self, query: Query) -> Result<SearchResult> {
        Err(ErrorKind::NotYetImplemented("search".to_string()).into())
    }
}

mod search {
    pub struct Query {}
    pub struct SearchResult {}
}

mod refresh_access_token {
    use crate::client::CenterDevice;
    use crate::client::errors::{Error, ErrorKind, Result};

    use failure::{Context, Fail};
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Token{
        token_type: String,
        access_token: String,
        expires_in: u32,
        refresh_token: String,
    }

    pub fn refresh_access_token(centerdevice: &CenterDevice, refresh_token: &str) -> Result<Token> {
        let url = format!("https://auth.{}/token", centerdevice.base_url);
        let params = [("grant_type", "refresh_token"), ("refresh_token", refresh_token)];

        let mut token= centerdevice.http_client
            .post(&url)
            .basic_auth(&centerdevice.credentials.client_id, Some(&centerdevice.credentials.client_secret))
            .form(&params)
            .send()
            .map_err(|e| e.context(ErrorKind::ApiCallFailed))?
            .json()
            .map_err(|e| e.context(ErrorKind::ReadResponseFailed))?;

        Ok(token)
    }
}

pub mod errors {
    use failure::{Backtrace, Context, Fail};
    use std::fmt;

    /// The error kind for errors that get returned in the crate
    #[derive(Eq, PartialEq, Debug, Fail)]
    pub enum ErrorKind {
        #[fail(display = "function '{}' is not yet implemeted", _0)]
        NotYetImplemented(String),
        #[fail(display = "API call failed")]
        ApiCallFailed,
        #[fail(display = "failed to read API call response")]
        ReadResponseFailed,
    }

    impl Clone for ErrorKind {
        fn clone(&self) -> Self {
            use self::ErrorKind::*;
            match *self {
                NotYetImplemented(ref name) => NotYetImplemented(name.clone()),
                ApiCallFailed => ApiCallFailed,
                ReadResponseFailed => ReadResponseFailed,
            }
        }
    }

    /// The error type for errors that get returned in the lookup module
    #[derive(Debug)]
    pub struct Error {
        pub(crate) inner: Context<ErrorKind>,
    }

    impl Error {
        /// Get the kind of the error
        pub fn kind(&self) -> &ErrorKind {
            self.inner.get_context()
        }
    }

    impl Clone for Error {
        fn clone(&self) -> Self {
            Error {
                inner: Context::new(self.inner.get_context().clone()),
            }
        }
    }

    impl Fail for Error {
        fn cause(&self) -> Option<&Fail> {
            self.inner.cause()
        }

        fn backtrace(&self) -> Option<&Backtrace> {
            self.inner.backtrace()
        }
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Display::fmt(&self.inner, f)
        }
    }

    impl From<ErrorKind> for Error {
        fn from(kind: ErrorKind) -> Error {
            Error {
                inner: Context::new(kind),
            }
        }
    }

    impl From<Context<ErrorKind>> for Error {
        fn from(inner: Context<ErrorKind>) -> Error {
            Error { inner }
        }
    }

    pub type Result<T> = ::std::result::Result<T, Error>;

}