pub mod auth;
pub mod search;

pub use auth::{Code, CodeProvider, Token};

use crate::{CenterDevice, ClientCredentials, ErrorKind, Result};
use crate::client::search::{Search, SearchResult};

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
}

pub type ID = String;

#[derive(Debug, Deserialize)]
pub struct Document {
    author: ID,
    // collections
    comments: usize,
    #[serde(rename = "document-date")]
    document_date: String,
    // extended-metadata
    filename: String,
    hash: String,
    id: ID,
    #[serde(rename = "mimetype", deserialize_with = "deserialize_mime_type")]
    mime_type: mime::Mime,
    owner: ID,
    pages: usize,
    // representations
    score: Option<f64>,
    title: String,
    #[serde(rename = "upload-date")]
    upload_date: String,
    uploader: ID,
    // users
    version: usize,
    #[serde(rename = "version-date")]
    version_date: String,
}

fn deserialize_mime_type<'de, D>(deserializer: D) -> ::std::result::Result<mime::Mime, D::Error>
    where
        D: Deserializer<'de>,
{
    struct MechanismVisitor;

    impl<'a> Visitor<'a> for MechanismVisitor {
        type Value = mime::Mime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string with valid mime type")
        }

        fn visit_str<E>(self, s: &str) -> ::std::result::Result<Self::Value, E> where E: serde::de::Error {
            mime::Mime::from_str(s)
                .map_err(|_| serde::de::Error::custom("invalid mime type"))
        }
    }

    deserializer.deserialize_string(MechanismVisitor)
}

#[cfg(test)]
mod test {
    use super::*;

    mod document {
        use super::*;

        use spectral::prelude::*;

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

