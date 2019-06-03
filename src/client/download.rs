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
use std::path::{Path, PathBuf};
use std::borrow::Cow;
use reqwest::{Body, header, StatusCode, Response};
use std::io::{Read, BufWriter};
use std::fmt::Write;
use crate::client::upload::internal::DocumentMetadata;
use mime_multipart::{Node, Part, FilePart, write_multipart};
use std::ffi::OsStr;


pub struct Download<'a> {
    document_id: ID,
    dir: &'a Path,
    filename: Option<&'a Path>,

}

impl<'a> Download<'a> {
    pub fn new(document_id: ID, dir: &'a Path) -> Download<'a> {
        Download {
            document_id,
            dir,
            filename: None,
        }
    }

    pub fn filename(self, filename: &'a Path) -> Download<'a> {
        Download {
            filename: Some(filename),
            ..self
        }
    }
}


pub fn download_file(authorized_client: &AuthorizedClient, download: Download) -> Result<u64> {
    let url = format!("https://api.{}/v2/document/{}", authorized_client.base_url, download.document_id);

    let mut response = authorized_client.http_client
        .get(&url)
        .bearer_auth(&authorized_client.token.access_token)
        .send()
        .map_err(|e| e.context(ErrorKind::ApiCallFailed))?;

    if response.status() != StatusCode::OK {
        let status = response.status();
        let body = response.text().unwrap_or_else(|_| "Failed to read body".to_string());
        return Err(Error::from(ErrorKind::ApiCallError(status, body)))
    }

    let content_length = get_content_length(&response)?;
    let filename = if let Some(f_path) = download.filename {
        PathBuf::from(f_path)
    } else {
        let f_content_disposition= get_filename(&response)?;
        PathBuf::from(f_content_disposition)
    };
    println!("Filename: {:#?}", filename);

    let mut file_path = PathBuf::from(&download.dir);
    file_path.push(filename);

    let file = File::create(file_path.as_path())
        .map_err(|e| e.context(ErrorKind::ReadResponseFailed))?;
    let mut writer = BufWriter::new(file);
    let len = response.copy_to(&mut writer)
        .map_err(|e| e.context(ErrorKind::ReadResponseFailed))?;
    assert_eq!(content_length, len);

    Ok(len)
}

fn get_filename(response: &Response) -> Result<String> {
    // TODO: Upgrade to another version of mime_multifrom or replace because it uses hyper 0.10 headers and mime 0.2
    use hyper::header::{ContentDisposition, DispositionParam, Header};
    use std::str;

    let header: Vec<_> = response.headers().get(header::CONTENT_DISPOSITION)
        .ok_or(ErrorKind::FailedToGetFilename)?
        .as_bytes()
        .to_vec();
    let content_disposition: ContentDisposition = ContentDisposition::parse_header(&[header])
        .map_err(|e| e.context(ErrorKind::FailedToGetFilename))?;

    let mut filename = None;
    for cp in &content_disposition.parameters {
        if let DispositionParam::Filename(_, _, ref f) = *cp {
            let decoded = str::from_utf8(f)
                .map_err(|e| e.context(ErrorKind::FailedToGetFilename))?;
            filename = Some(decoded);
            break;
        }
    }
    filename
        .ok_or(Error::from(ErrorKind::FailedToGetFilename))
        .map(|x| x.to_string())
}

fn get_content_length(response: &Response) -> Result<u64> {
    let content_length = response.headers().get(header::CONTENT_LENGTH)
        .ok_or(ErrorKind::FailedToGetContentLength)?
        .to_str()
        .map_err(|e| e.context(ErrorKind::FailedToGetContentLength))?
        .parse::<u64>()
        .map_err(|e| e.context(ErrorKind::FailedToGetContentLength))?;
    Ok(content_length)
}
