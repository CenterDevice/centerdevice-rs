use crate::client::{AuthorizedClient, ID, GeneralErrHandler};
use crate::errors::{ErrorKind, Result};

use failure::Fail;
use reqwest::{Response, StatusCode};
use serde::{self, Serialize, Deserialize};
use std::string::ToString;


#[derive(Debug, Serialize, Deserialize)]
pub struct UsersQuery {
    pub all: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsersResult {
    pub users: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: ID,
    #[serde(rename = "first-name")]
    pub first_name: String,
    #[serde(rename = "last-name")]
    pub last_name: String,
    pub email: String,
    pub status: UserStatus,
    pub role: UserRole,
    #[serde(rename = "technical-user")]
    pub technical_user: Option<bool>,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Invited,
    Pending,
    Active,
    Blocked,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Internal,
    External,
    Guest,
}

pub fn search_users(authorized_client: &AuthorizedClient, users_query: UsersQuery) -> Result<UsersResult> {
    let url = format!("https://api.{}/v2/users", authorized_client.base_url);
    let params = [
        ("all", &users_query.all.to_string()),
    ];

    let mut response: Response = authorized_client
        .http_client
        .get(&url)
        .form(&params)
        .bearer_auth(&authorized_client.token.access_token)
        .send()
        .map_err(|e| e.context(ErrorKind::HttpRequestFailed))?
        .general_err_handler(StatusCode::OK)?;

    let result = response.json().map_err(|e| e.context(ErrorKind::FailedToProcessHttpResponse(response.status(), "reading body".to_string())))?;

    Ok(result)
}
