pub use auth::{Code, CodeProvider, Token};
pub use search::{Query, SearchResult};

use crate::client::errors::{ErrorKind, Result};
use failure::Fail;
use reqwest;
use reqwest::IntoUrl;


pub trait CenterDevice {
    fn refresh_access_token(&self) -> Result<Token>;
}

pub struct Client {}

impl Client {
    pub fn new(base_url: String, client_credentials: ClientCredentials) -> UnauthorizedClient {
        UnauthorizedClient {
            base_url,
            client_credentials,
            http_client: reqwest::Client::new(),
        }
    }

    pub fn with_tokens(base_url: String, client_credentials: ClientCredentials, token: Token) -> AuthorizedClient {
        AuthorizedClient {
            base_url,
            client_credentials,
            token,
            http_client: reqwest::Client::new(),
        }
    }
}

pub struct UnauthorizedClient {
    base_url: String,
    client_credentials: ClientCredentials,
    http_client: reqwest::Client,
}

impl UnauthorizedClient {
    pub fn authorize_with_code_flow<T: IntoUrl + ToString + Clone, S: CodeProvider>(
        self,
        redirect_uri: T,
        code_provider: &S,
    ) -> Result<AuthorizedClient> {
        // FIXME: This allocation is unnecessary.
        let redirect_url = redirect_uri
            .clone()
            .into_url()
            .map_err(|e| e.context(ErrorKind::ParseUrlFailed(redirect_uri.to_string())))?;

        let token = auth::authorization_code_flow(&self.client_credentials, &self.base_url, &redirect_url, code_provider)?;

        let authorized_client = AuthorizedClient {
            base_url: self.base_url,
            client_credentials: self.client_credentials,
            token,
            http_client: self.http_client,
        };

        Ok(authorized_client)
    }
}

pub struct AuthorizedClient {
    base_url: String,
    client_credentials: ClientCredentials,
    token: Token,
    http_client: reqwest::Client,
}

impl AuthorizedClient {
    pub fn token(&self) -> &Token {
        &self.token
    }
}

impl CenterDevice for AuthorizedClient {
    fn refresh_access_token(&self) -> Result<Token> {
        auth::refresh_access_token(self)
    }
}

#[derive(Debug)]
pub struct ClientCredentials {
    client_id: String,
    client_secret: String,
}

impl ClientCredentials {
    pub fn new(client_id: String, client_secret: String) -> ClientCredentials {
        ClientCredentials {
            client_id,
            client_secret,
        }
    }
}

mod auth {
    use crate::client::errors::{ErrorKind, Result};
    use crate::client::{ClientCredentials, AuthorizedClient};

    use failure::Fail;
    use reqwest::{IntoUrl, Url};
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Token {
        pub(crate) token_type: Option<String>,
        pub(crate) access_token: String,
        pub(crate) expires_in: Option<u32>,
        pub(crate) refresh_token: String,
    }

    impl Token {
        pub fn new(access_token: String, refresh_token: String) -> Token {
            Token {
                token_type: None,
                access_token,
                expires_in: None,
                refresh_token,
            }
        }

        pub fn token_type(&self) -> Option<&str> {
            self.token_type.as_ref().map(String::as_ref)
        }

        pub fn access_token(&self) -> &str {
            &self.access_token
        }

        pub fn expires_in(&self) -> Option<u32> {
            self.expires_in
        }

        pub fn refresh_token(&self) -> &str {
            self.refresh_token.as_ref()
        }
    }

    pub trait CodeProvider {
        fn get_code<T: IntoUrl>(&self, auth_url: T) -> Result<Code>;
    }

    #[derive(Debug, Deserialize)]
    pub struct Code {
        code: String,
    }

    impl Code {
        pub fn new(code: String) -> Code {
            Code { code }
        }
    }

    pub fn authorization_code_flow<T: CodeProvider>(
        client_credentials: &ClientCredentials,
        base_url: &str,
        redirect_uri: &Url,
        code_provider: &T,
    ) -> Result<Token> {
        let code = get_code(client_credentials, &base_url, redirect_uri, code_provider)?;
        let token = exchange_code_for_token(&code, client_credentials, base_url, redirect_uri)?;

        Ok(token)
    }

    fn get_code<T: CodeProvider>(
        client_credentials: &ClientCredentials,
        base_url: &str,
        redirect_uri: &Url,
        code_provider: &T,
    ) -> Result<Code> {
        let auth_endpoint = format!("https://auth.{}/authorize", base_url);
        let params = [
            ("client_id", client_credentials.client_id.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("response_type", "code"),
        ];
        let auth_url = Url::parse_with_params(&auth_endpoint, &params)
            .map_err(|e| e.context(ErrorKind::ParseUrlFailed(redirect_uri.to_string())))?;

        code_provider.get_code(auth_url)
    }

    pub fn exchange_code_for_token(
        code: &Code,
        client_credentials: &ClientCredentials,
        base_url: &str,
        redirect_uri: &Url,
    ) -> Result<Token> {
        let token_endpoint = format!("https://auth.{}/token", base_url);
        let params = [
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
            ("code", code.code.as_str()),
        ];

        let http_client = reqwest::Client::new();

        let token = http_client
            .post(&token_endpoint)
            .basic_auth(&client_credentials.client_id, Some(&client_credentials.client_secret))
            .form(&params)
            .send()
            .map_err(|e| e.context(ErrorKind::ApiCallFailed))?
            .json()
            .map_err(|e| e.context(ErrorKind::ReadResponseFailed))?;

        Ok(token)
    }

    pub fn refresh_access_token(authorized_client: &AuthorizedClient) -> Result<Token> {
        let url = format!("https://auth.{}/token", authorized_client.base_url);
        let params = [("grant_type", "refresh_token"), ("refresh_token", &authorized_client.token.refresh_token)];

        let token = authorized_client
            .http_client
            .post(&url)
            .basic_auth(
                &authorized_client.client_credentials.client_id,
                Some(&authorized_client.client_credentials.client_secret),
            )
            .form(&params)
            .send()
            .map_err(|e| e.context(ErrorKind::ApiCallFailed))?
            .json()
            .map_err(|e| e.context(ErrorKind::ReadResponseFailed))?;

        Ok(token)
    }
}

mod search {
    pub struct Query {}

    pub struct SearchResult {}
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
        #[fail(display = "failed to parse URL '{}'", _0)]
        ParseUrlFailed(String),
    }

    impl Clone for ErrorKind {
        fn clone(&self) -> Self {
            use self::ErrorKind::*;
            match *self {
                NotYetImplemented(ref s) => NotYetImplemented(s.clone()),
                ApiCallFailed => ApiCallFailed,
                ReadResponseFailed => ReadResponseFailed,
                ParseUrlFailed(ref s) => ParseUrlFailed(s.clone()),
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
