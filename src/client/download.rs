use crate::client::{self, AuthorizedClient, ID};
use crate::errors::{ErrorKind, Error, Result};

use failure::Fail;
use hex;
use mime::*;
use ring;
use serde::{self, Deserialize, Serialize, Deserializer};
use serde::de::Visitor;
use std::{fmt, io};
use std::str::FromStr;
use std::fs::File;
use std::path::Path;
use std::borrow::Cow;
use reqwest::{Body, header};
use std::io::Read;
use std::fmt::Write;
use crate::client::upload::internal::DocumentMetadata;
use mime_multipart::{Node, Part, FilePart, write_multipart};
use std::ffi::OsStr;


pub struct Download<'a> {
    document_id: ID,
    dir: &'a Path,
    filename: Option<OsStr>,

}

impl<'a> Download<'a> {
    pub fn new(document_id: ID, dir: &'a Path) -> Download<'a> {
        Download {
            document_id,
            dir,
            filename: None,
        };
    }

    pub fn filename(self, filename: OsStr) -> Download<'a> {
        Download {
            filename: Some(filename),
            ..self
        }
    }
}


pub fn download_file(authorized_client: &AuthorizedClient, download: Download) -> Result<()> {
    let url = format!("https://api.{}/v2/document/{}", authorized_client.base_url, download.document_id);

    let result: Id = authorized_client.http_client
        .get(&url)
        .bearer_auth(&authorized_client.token.access_token)
        .header(header::CONTENT_TYPE, content_type.to_string().as_bytes())
        .header(header::ACCEPT, mime!(Application / Json; Charset = Utf8).to_string().as_bytes())
        .body(body)
        .send()
        .map_err(|e| e.context(ErrorKind::ApiCallFailed))?
        .json()
        .map_err(|e| e.context(ErrorKind::ReadResponseFailed))?;

    Ok()
}

fn create_multipart(metadata: &DocumentMetadata, upload: &Upload) -> Result<Vec<Node>> {
    // TODO: Upgrade to another version of mime_multifrom or replace because it uses hyper 0.10 headers and mime 0.2
    use hyper::header::{ContentType, Headers, ContentDisposition, DispositionType, DispositionParam};

    let mut nodes: Vec<Node> = Vec::with_capacity(2);

    let json_bytes = serde_json::to_string(metadata)
        .map_err(|e| e.context(ErrorKind::SerializeJsonFailed("doc-metadata".to_string())))?
        .into_bytes();

    let mut h = Headers::new();
    h.set(ContentType(mime!(Application / Json)));
    h.set(ContentDisposition {
        disposition: DispositionType::Ext("form-data".to_string()),
        parameters: vec![DispositionParam::Ext("name".to_string(), "metadata".to_string())],
    });
    nodes.push(Node::Part(Part {
        headers: h,
        body: json_bytes,
    }));

    let mut h = Headers::new();
    h.set(ContentType(upload.mime_type.clone()));
    h.set(ContentDisposition {
        disposition: DispositionType::Ext("form-data".to_string()),
        parameters: vec![DispositionParam::Ext("name".to_string(), "document".to_string()),
                         DispositionParam::Ext("filename".to_string(), upload.filename.to_string())],
    });
    nodes.push(Node::File(FilePart::new(h, upload.path)));

    Ok(nodes)
}

// CenterDevice / Jersey does not accept special characters in boundary; thus, we build it ourselves.
fn generate_boundary(seed: &[u8]) -> String {
    let sha = ring::digest::digest(&ring::digest::SHA256, seed);
    let sha_str = hex::encode(sha.as_ref());
    format!("Boundary_{}", sha_str)
}