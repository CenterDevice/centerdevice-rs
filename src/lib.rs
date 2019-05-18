pub mod errors;
pub mod client;
pub mod old;

pub use crate::client::auth::Token;

use crate::errors::{ErrorKind, Result};
use crate::client::{UnauthorizedClient, AuthorizedClient};
use crate::client::search::{Search, SearchResult};

use serde::Serialize;

pub trait CenterDevice {
    fn refresh_access_token(&self) -> Result<Token>;
    fn search_documents(&self, search: Search) -> Result<SearchResult>;
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

    pub fn with_token(base_url: String, client_credentials: ClientCredentials, token: Token) -> AuthorizedClient {
        AuthorizedClient {
            base_url,
            client_credentials,
            token,
            http_client: reqwest::Client::new(),
        }
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
