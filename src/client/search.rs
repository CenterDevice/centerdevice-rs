use crate::client::{AuthorizedClient, ID};
use crate::errors::{Error, ErrorKind, Result};

use failure::Fail;
use mime;
use reqwest::{Response, StatusCode};
use serde::de::Visitor;
use serde::{self, Deserialize, Deserializer};
use std::fmt;
use std::str::FromStr;

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
        Search { named_search, ..self }
    }
}

impl<'a> Default for Search<'a> {
    fn default() -> Self {
        Self::new()
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
        include: bool,
    }

    impl<'a> Search<'a> {
        pub fn from_search(s: super::Search<'a>) -> Self {
            let named: Option<Vec<Named>> = match s.named_search {
                super::NamedSearch::None => None,
                super::NamedSearch::PublicCollections => {
                    let include = Include { include: true };
                    let named = vec![Named {
                        name: "public-collections",
                        params: include,
                    }];
                    Some(named)
                }
            };

            let filter = Filter {
                filenames: s.filenames,
                tags: s.tags,
            };
            let query = Query { text: s.fulltext };
            let params = Params { query, filter, named };

            Search {
                action: "search",
                params,
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchResult {
    documents: Vec<Document>,
    hits: usize,
}

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

        fn visit_str<E>(self, s: &str) -> ::std::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            mime::Mime::from_str(s).map_err(|_| serde::de::Error::custom("invalid mime type"))
        }
    }

    deserializer.deserialize_string(MechanismVisitor)
}

pub fn search_documents(authorized_client: &AuthorizedClient, search: Search) -> Result<SearchResult> {
    let url = format!("https://api.{}/v2/documents", authorized_client.base_url);

    let internal_search = internal::Search::from_search(search);

    let mut response: Response = authorized_client
        .http_client
        .post(&url)
        .bearer_auth(&authorized_client.token.access_token)
        .json(&internal_search)
        .send()
        .map_err(|e| e.context(ErrorKind::HttpRequestFailed))?;

    if response.status() != StatusCode::OK {
        let status_code = response.status();
        let body = response.text().map_err(|e| e.context(ErrorKind::HttpResponseReadFailed("reading body".to_string()))?;
        return Err(Error::from(ErrorKind::ApiCallFailed(status_code, body)));
    }

    let result = response.json().map_err(|e| e.context(ErrorKind::HttpResponseReadFailed("reading body".to_string()))?;

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
