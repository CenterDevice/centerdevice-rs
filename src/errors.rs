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
    #[fail(display = "failed to parse URL '{}'", _0)]
    ParseUrlFailed(String),
    #[fail(display = "failed to serialize '{}' to JSON", _0)]
    SerializeJsonFailed(String),
    #[fail(display = "filesystem failure")]
    FileSystemFailure,
    #[fail(display = "failed to create multipart form")]
    FailedToMultipart,
}

impl Clone for ErrorKind {
    fn clone(&self) -> Self {
        use self::ErrorKind::*;
        match *self {
            NotYetImplemented(ref s) => NotYetImplemented(s.clone()),
            ApiCallFailed => ApiCallFailed,
            ReadResponseFailed => ReadResponseFailed,
            ParseUrlFailed(ref s) => ParseUrlFailed(s.clone()),
            SerializeJsonFailed(ref s) => SerializeJsonFailed(s.clone()),
            FileSystemFailure => FileSystemFailure,
            FailedToMultipart => FailedToMultipart,
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
