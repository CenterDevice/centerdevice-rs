use crate::client::{AuthorizedClient, ID, GeneralErrHandler};
use crate::errors::{ErrorKind, Result};

use chrono::{DateTime, FixedOffset};
use failure::Fail;
use mime;
use reqwest::{Response, StatusCode};
use serde::de::Visitor;
use serde::{self, Serialize, Serializer, Deserialize, Deserializer};
use std::fmt;
use std::str::FromStr;

#[derive(PartialEq, Debug)]
pub enum NamedSearch {
    None,
    PublicCollections,
}

#[derive(Debug)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub documents: Option<Vec<Document>>,
    pub hits: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub author: ID,
    // collections
    pub comments: usize,
    #[serde(rename = "document-date", serialize_with = "serialize_rfc3339", deserialize_with = "deserialize_rfc3339")]
    pub document_date: DateTime<FixedOffset>,
    #[serde(rename = "extended-metadata")]
    pub extended_metadata: serde_json::Value,
    pub filename: String,
    pub hash: String,
    pub id: ID,
    #[serde(rename = "mimetype", serialize_with = "serialize_mime_type", deserialize_with = "deserialize_mime_type")]
    pub mime_type: mime::Mime,
    pub owner: ID,
    pub pages: Option<usize>,
    pub representations: Representations,
    pub score: Option<f64>,
    pub title: String,
    #[serde(rename = "upload-date", serialize_with = "serialize_rfc3339", deserialize_with = "deserialize_rfc3339")]
    pub upload_date: DateTime<FixedOffset>,
    pub uploader: ID,
    // users
    pub version: usize,
    #[serde(rename = "version-date", serialize_with = "serialize_rfc3339", deserialize_with = "deserialize_rfc3339")]
    pub version_date: DateTime<FixedOffset>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Representations {
    pub pdf: String,
    pub fulltext: String,
    pub jpg: String,
    pub png: String,
    pub mp4: String,
}

impl fmt::Display for Representations {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut reps = Vec::new();
        if self.pdf == "yes" { reps.push("pdf"); }
        if self.fulltext == "yes" { reps.push("fulltext"); }
        if self.jpg == "yes" { reps.push("jpg"); }
        if self.png == "yes" { reps.push("png"); }
        if self.mp4 == "yes" { reps.push("mp4"); }

        write!(f, "{:?}", reps)
    }
}

fn serialize_mime_type<S>(mime_type: &mime::Mime, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: Serializer {
    let s = mime_type.to_string();
    serializer.serialize_str(&s)
}


fn deserialize_mime_type<'de, D>(deserializer: D) -> ::std::result::Result<mime::Mime, D::Error>
    where
        D: Deserializer<'de>,
{
    struct MimeVisitor;

    impl<'a> Visitor<'a> for MimeVisitor {
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

    deserializer.deserialize_string(MimeVisitor)
}

fn serialize_rfc3339<S>(date_time: &DateTime<FixedOffset>, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: Serializer {
    let s = date_time.to_rfc3339();
    serializer.serialize_str(&s)
}

fn deserialize_rfc3339<'de, D>(deserializer: D) -> ::std::result::Result<DateTime<FixedOffset>, D::Error>
    where
        D: Deserializer<'de>,
{
    struct Iso8601Visitor;

    impl<'a> Visitor<'a> for Iso8601Visitor {
        type Value = DateTime<FixedOffset>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string with valid date time in RFC 3339 / ISO8601 format")
        }

        fn visit_str<E>(self, s: &str) -> ::std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
        {
            DateTime::parse_from_rfc3339(s).map_err(|_| serde::de::Error::custom("invalid date time"))
        }
    }

    deserializer.deserialize_string(Iso8601Visitor)
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
