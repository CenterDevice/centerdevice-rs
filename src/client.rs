pub mod auth;
pub mod download;
pub mod search;
pub mod upload;

pub use auth::{Code, CodeProvider, Token};

use crate::{CenterDevice, ClientCredentials, ErrorKind, Result};
use crate::client::search::{Search, SearchResult};
use crate::client::upload::Upload;
use crate::client::download::Download;

use failure::Fail;
use mime;
use reqwest;
use reqwest::IntoUrl;
use serde::{self, Deserialize, Serialize, Deserializer};
use serde::de::Visitor;
use std::fmt;
use std::str::FromStr;

pub struct UnauthorizedClient {
    pub(crate) base_url: String,
    pub(crate) client_credentials: ClientCredentials,
    pub(crate) http_client: reqwest::Client,
}

impl UnauthorizedClient {
    pub fn authorize_with_code_flow<T: IntoUrl + ToString + Clone, S: CodeProvider>(
        self,
        redirect_uri: T,
        code_provider: &S,
    ) -> Result<AuthorizedClient> {
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
    pub(crate) base_url: String,
    pub(crate) client_credentials: ClientCredentials,
    pub(crate) token: Token,
    pub(crate) http_client: reqwest::Client,
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

    fn search_documents(&self, search: Search) -> Result<SearchResult> {
        search::search_documents(&self, search)
    }

    fn upload_file(&self, upload: Upload) -> Result<ID> {
        upload::upload_file(&self, upload)
    }

    fn download_file(&self, download: Download) -> Result<u64> {
        download::download_file(&self, download)
    }
}

pub type ID = String;

