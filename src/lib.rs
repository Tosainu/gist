use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct GistRequest {
    files: HashMap<String, FileMetadata>,
    description: String,
    public: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct FileMetadata {
    // TODO: support binary file
    content: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct GistResponse {
    id: String,
    html_url: String,
    git_pull_url: String,
    git_push_url: String,
}

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("reqwest"));
    headers
}

pub fn post(
    user: &str,
    token: &str,
    public: bool,
    description: &str,
    paths: &[PathBuf],
) -> Result<(), Box<dyn std::error::Error>> {
    let files = paths
        .iter()
        .map(|p| {
            let mut buf = String::new();
            let mut f = File::open(p)?;
            f.read_to_string(&mut buf)?;

            let filename = p.file_name().unwrap().to_str().unwrap().to_string();
            Ok((filename, FileMetadata { content: buf }))
        })
        .collect::<io::Result<_>>()?;

    let req = GistRequest {
        files,
        description: String::from(description),
        public,
    };
    println!("{:#?}", req);

    println!("{}", serde_json::to_string(&req).unwrap());

    let client = reqwest::blocking::Client::new();
    let res = client
        .post("https://api.github.com/gists")
        .basic_auth(user, Some(token))
        .headers(construct_headers())
        .json(&req)
        .send()?;
    println!("{:#?}", res);
    println!("{:#?}", res.json::<GistResponse>());

    Ok(())
}
