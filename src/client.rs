pub mod auth;
pub mod delete;
pub mod download;
pub mod search;
pub mod upload;

pub use auth::{Code, CodeProvider, Token};

use crate::client::download::Download;
use crate::client::search::{Search, SearchResult};
use crate::client::upload::Upload;
use crate::{CenterDevice, ClientCredentials, ErrorKind, Result, WithProgress};

use failure::Fail;
use reqwest;
use reqwest::IntoUrl;

pub struct UnauthorizedClient<'a> {
    pub(crate) base_url: &'a str,
    pub(crate) client_credentials: ClientCredentials<'a>,
    pub(crate) http_client: reqwest::Client,
}

impl<'a, 'b: 'a> UnauthorizedClient<'b> {
    pub fn authorize_with_code_flow<T: IntoUrl + ToString + Clone, S: CodeProvider>(
        self,
        redirect_uri: T,
        code_provider: &S,
    ) -> Result<AuthorizedClient<'a>> {
        let redirect_url = redirect_uri
            .clone()
            .into_url()
            .map_err(|e| e.context(ErrorKind::FailedToPrepareHttpRequest(redirect_uri.to_string())))?;

        let token =
            auth::authorization_code_flow(&self.client_credentials, self.base_url, &redirect_url, code_provider)?;

        let authorized_client = AuthorizedClient {
            base_url: self.base_url,
            client_credentials: self.client_credentials,
            token,
            http_client: self.http_client,
        };

        Ok(authorized_client)
    }
}

pub struct AuthorizedClient<'a> {
    pub(crate) base_url: &'a str,
    pub(crate) client_credentials: ClientCredentials<'a>,
    pub(crate) token: Token,
    pub(crate) http_client: reqwest::Client,
}

impl<'a> AuthorizedClient<'a> {
    pub fn token(&self) -> &Token {
        &self.token
    }
}

impl<'a> CenterDevice for AuthorizedClient<'a> {
    fn refresh_access_token(&self) -> Result<Token> {
        auth::refresh_access_token(self)
    }

    fn search_documents(&self, search: Search) -> Result<SearchResult> {
        search::search_documents(self, search)
    }

    fn upload_file(&self, upload: Upload) -> Result<ID> {
        upload::upload_file(&self, upload)
    }

    fn download_file(&self, download: Download) -> Result<u64> {
        download::download_file(self, download)
    }

    fn download_file_with_progress<T: WithProgress>(&self, download: Download, progress: &mut T) -> Result<u64> {
        download::download_file_with_progress(self, download, progress)
    }

    fn delete_documents(&self, document_ids: &[&str]) -> Result<()> {
        delete::delete_documents(self, document_ids)
    }
}

pub type ID = String;
