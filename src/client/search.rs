use crate::client::{AuthorizedClient, Document};
use crate::errors::{ErrorKind, Error, Result};

use serde::{Deserialize, Serialize};
use failure::Fail;
use reqwest::header;

#[derive(PartialEq, Debug)]
pub enum NamedSearch {
    None,
    PublicCollections,
}

pub struct Search<'a> {
    filenames: Option<Vec<&'a str>>,
    tags: Option<Vec<&'a str>>,
    fulltext: Option<&'a str>,
    named_search: NamedSearch,
}

impl<'a> Search<'a> {
    pub fn new() -> Search<'a> {
        Search {
            filenames: None,
            tags: None,
            fulltext: None,
            named_search: NamedSearch::None,
        }
    }

    pub fn filenames(self, filenames: Vec<&'a str>) -> Search<'a> {
        Search {
            filenames: Some(filenames),
            ..self
        }
    }

    pub fn tags(self, tags: Vec<&'a str>) -> Search<'a> {
        Search {
            tags: Some(tags),
            ..self
        }
    }

    pub fn fulltext(self, fulltext: &'a str) -> Search<'a> {
        Search {
            fulltext: Some(fulltext),
            ..self
        }
    }

    pub fn named_searches(self, named_search: NamedSearch) -> Search<'a> {
        Search {
            named_search,
            ..self
        }
    }
}

pub(crate) mod internal {
    use serde::Serialize;

    #[derive(Serialize, Debug)]
    pub struct Search<'a> {
        action: &'a str,
        params: Params<'a>,
    }

    #[derive(Serialize, Debug)]
    struct Params<'a> {
        query: Query<'a>,
        filter: Filter<'a>,
        #[serde(skip_serializing_if = "Option::is_none")]
        named: Option<Vec<Named<'a>>>,
    }

    #[derive(Serialize, Debug)]
    struct Query<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<&'a str>,
    }

    #[derive(Serialize, Debug)]
    struct Filter<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        filenames: Option<Vec<&'a str>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tags: Option<Vec<&'a str>>,
    }

    #[derive(Serialize, Debug)]
    struct Named<'a> {
        name: &'a str,
        params: Include,
    }

    #[derive(Serialize, Debug)]
    struct Include {
        include: bool
    }

    impl<'a> Search<'a> {
        pub fn from_search(s: super::Search<'a>) -> Self {

            let named: Option<Vec<Named>> = match s.named_search {
                super::NamedSearch::None => None,
                super::NamedSearch::PublicCollections => {
                    let include = Include { include: true };
                    let named = vec![Named { name: "public-collections", params: include }];
                    Some(named)
                }
            };

            let filter = Filter { filenames: s.filenames, tags: s.tags };
            let query = Query { text: s.fulltext };
            let params = Params { query, filter, named };

            Search { action: "search", params }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchResult {
    documents: Vec<Document>,
    hits: usize,
}

pub fn search_documents(authorized_client: &AuthorizedClient, search: Search) -> Result<SearchResult> {
    let url = format!("https://api.{}/v2/documents", authorized_client.base_url);

    let internal_search = internal::Search::from_search(search);

    let result: SearchResult = authorized_client.http_client
        .post(&url)
        .bearer_auth(&authorized_client.token.access_token)
        .json(&internal_search)
        .send()
        .map_err(|e| e.context(ErrorKind::ApiCallFailed))?
        .json()
        .map_err(|e| e.context(ErrorKind::ReadResponseFailed))?;

    Ok(result)
}
