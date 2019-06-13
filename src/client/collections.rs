use crate::client::{AuthorizedClient, GeneralErrHandler};
use crate::errors::{ErrorKind, Result};

use failure::Fail;
use reqwest::{Response, StatusCode};
use serde::{self, Serialize, Deserialize};
use std::string::ToString;


#[derive(Serialize, Debug)]
pub struct CollectionsQuery<'a> {
    include_public: bool,
    name: Option<&'a str>,
    ids: Option<Vec<&'a str>>,
}

impl<'a> CollectionsQuery<'a> {
    pub fn new() -> Self {
        CollectionsQuery {
            include_public: false,
            name: None,
            ids: None,
        }
    }

    pub fn include_public(self) -> CollectionsQuery<'a> {
        CollectionsQuery {
            include_public: true,
            ..self
        }
    }

    pub fn name(self, name: &'a str) -> CollectionsQuery<'a> {
        CollectionsQuery {
            name: Some(name),
            ..self
        }
    }

    pub fn ids(self, ids: Vec<&'a str>) -> CollectionsQuery<'a> {
        CollectionsQuery {
            ids: Some(ids),
            ..self
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CollectionsResult {
    collections: Vec<Collection>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Collection {
    pub id: String,
    pub public: bool,
    pub name: String,
}

pub fn search_collections(authorized_client: &AuthorizedClient, collection_query: CollectionsQuery) -> Result<CollectionsResult> {
    let url = format!("https://api.{}/v2/collections", authorized_client.base_url);

    let ids_str; // Borrow checker
    let mut params = Vec::new();
    if collection_query.include_public {
        params.push(("include-public", "true"));
    }
    if let Some(name) = collection_query.name {
        params.push(("name", name));
    }
    if let Some(ids) = collection_query.ids {
        ids_str = ids.as_slice().join(",");
        params.push(("ids", &ids_str));
    }

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
