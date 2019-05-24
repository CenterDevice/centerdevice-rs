use crate::{ClientCredentials, ErrorKind, Result};
use crate::client::AuthorizedClient;

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