use crate::client::{AuthorizedClient, ID};
use crate::errors::{ErrorKind, Error, Result};

use failure::Fail;
use serde::{self, Deserialize, Serialize, Deserializer};
use reqwest::StatusCode;


#[derive(Serialize, Debug)]
struct DeleteAction<'a> {
    action: &'a str,
    params: Documents<'a>,
}

#[derive(Serialize, Debug)]
struct Documents<'a> {
    documents: &'a [&'a str],
}

impl<'a> DeleteAction<'a> {
    pub fn new(documents: &'a[&'a str]) -> Self {
        let params = Documents { documents };
        DeleteAction { action: "delete", params }
    }
}

#[derive(Deserialize, Debug)]
struct FailedDocuments {
    #[serde(rename = "failed-documents")]
    failed_documents: Vec<ID>,
}

pub fn delete_documents(authorized_client: &AuthorizedClient, document_ids: &[&str]) -> Result<()> {
    let url = format!("https://api.{}/v2/documents", authorized_client.base_url);

    let delete_action = DeleteAction::new(document_ids);

    let mut response = authorized_client.http_client
        .post(&url)
        .bearer_auth(&authorized_client.token.access_token)
        .json(&delete_action)
        .send()
        .map_err(|e| e.context(ErrorKind::ApiCallFailed))?;

    if response.status() != StatusCode::NO_CONTENT {
        let status_code = response.status();
        let body = response.text()
            .map_err(|e| e.context(ErrorKind::ReadResponseFailed))?;
        return Err(Error::from(ErrorKind::ApiCallError(status_code, body)));
    } else {
        let failed_documents = response.json::<FailedDocuments>();
        if let Ok(failed_documents) = failed_documents {
            return Err(Error::from(ErrorKind::FailedDocuments(failed_documents.failed_documents)));
        }
    }

    Ok(())
}