pub mod client;
pub mod errors;

pub use crate::client::auth::Token;

use crate::client::download::Download;
use crate::client::search::{Search, SearchResult};
use crate::client::upload::Upload;
use crate::client::{AuthorizedClient, UnauthorizedClient, ID};
use crate::errors::{ErrorKind, Result};

pub trait CenterDevice {
    fn refresh_access_token(&self) -> Result<Token>;
    fn search_documents(&self, search: Search) -> Result<SearchResult>;
    fn upload_file(&self, upload: Upload) -> Result<ID>;
    fn download_file(&self, download: Download) -> Result<u64>;
    fn download_file_with_progress<T: WithProgress>(&self, download: Download, progress: &mut T) -> Result<u64>;
    fn delete_documents(&self, document_ids: &[&str]) -> Result<()>;
}

pub struct Client {}

impl Client {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<'a>(base_url: &'a str, client_credentials: ClientCredentials<'a>) -> UnauthorizedClient<'a> {
        UnauthorizedClient {
            base_url,
            client_credentials,
            http_client: reqwest::Client::new(),
        }
    }

    pub fn with_token<'a>(base_url: &'a str, client_credentials: ClientCredentials<'a>, token: Token) -> AuthorizedClient<'a> {
        AuthorizedClient {
            base_url,
            client_credentials,
            token,
            http_client: reqwest::Client::new(),
        }
    }
}

#[derive(Debug)]
pub struct ClientCredentials<'a> {
    client_id: &'a str,
    client_secret: &'a str,
}

impl<'a> ClientCredentials<'a> {
    pub fn new(client_id: &'a str, client_secret: &'a str) -> ClientCredentials<'a> {
        ClientCredentials {
            client_id,
            client_secret,
        }
    }
}

pub trait WithProgress {
    fn setup(&mut self, size: usize);
    fn progress(&mut self, amount: usize);
    fn finish(&self);
}
