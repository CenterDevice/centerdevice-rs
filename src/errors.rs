use crate::ID;

use failure::{Backtrace, Context, Fail};
use reqwest::StatusCode;
use std::fmt;

/// The error kind for errors that get returned in the crate
#[derive(Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "failed to prepare HTTP request, '{}'", _0)]
    FailedToPrepareHttpRequest(String),

    #[fail(display = "HTTP request failed")]
    HttpRequestFailed,

    #[fail(display = "failed to read HTTP response, {}", _0)]
    FailedToProcessHttpResponse(String),

    #[fail(display = "API call failed with status code {}, '{}'", _0, _1)]
    ApiCallFailed(StatusCode, String),
    #[fail(display = "failed documents; ids='{:?}'", _0)]
    FailedDocuments(Vec<ID>),
}

impl Clone for ErrorKind {
    fn clone(&self) -> Self {
        use self::ErrorKind::*;
        match *self {
            HttpRequestFailed => HttpRequestFailed,
            ApiCallFailed(ref status_code, ref body) => ApiCallFailed(*status_code, body.clone()),
            FailedToProcessHttpResponse(ref s) => FailedToProcessHttpResponse(s.clone()),
            FailedToPrepareHttpRequest(ref s) => FailedToPrepareHttpRequest(s.clone()),
            FailedDocuments(ref s) => FailedDocuments(s.clone()),
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
