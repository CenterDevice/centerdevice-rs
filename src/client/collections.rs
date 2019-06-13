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

#[cfg(test)]
mod test {
    use super::*;
    use spectral::prelude::*;

    mod document {
        use super::*;

        #[test]
        fn deserialize_ok() {
            let document_json = r#"{
    "author": "Industrie- und Handelskammer Bonn/Rhein-Sieg",
    "collections": {
        "not-visible-count": 0,
        "visible": [
            "9da0ffc7-09a5-42ee-a166-05ff13a74d91"
        ]
    },
    "comments": 0,
    "document-date": "2012-12-11T14:31:57.508Z",
    "extended-metadata": {},
    "filename": "Branchenkatalog.pdf",
    "hash": "fbf2b3b1688f94c76f10adfc82f80c1d",
    "id": "0176fc13-6dfe-40db-aca7-6b7c729e3fa3",
    "mimetype": "application/pdf",
    "owner": "ded4d798-d659-4c1c-9f2d-09e02d23e604",
    "pages": 49,
    "representations": {
        "fulltext": "yes",
        "jpg": "yes",
        "mp4": "no",
        "pdf": "yes",
        "png": "no"
    },
    "score": 15.038736,
    "size": 819693,
    "title": "NACE- Klassifikation der Wirtschaftszweige 2008",
    "upload-date": "2012-12-11T14:31:57.508Z",
    "uploader": "ded4d798-d659-4c1c-9f2d-09e02d23e604",
    "users": {
        "not-visible-count": 0,
        "visible": [
            "4161b86a-9eb8-4590-af5a-6f70b4ca0efb"
        ]
    },
    "version": 1,
    "version-date": "2012-12-11T14:31:57.508Z"
}"#;

            let document: std::result::Result<Document, _> = serde_json::from_str(document_json);

            assert_that(&document).is_ok();

            println!("Document: {:#?}", document.unwrap());
        }
    }
}
