use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::api;
use crate::config;
use crate::error::Result;

pub async fn upload(
    login: &config::Login,
    secret: bool,
    description: Option<&str>,
    files: &[PathBuf],
) -> Result<()> {
    let files = files
        .iter()
        .map(|p| {
            let mut buf = String::new();
            let mut f = File::open(p)?;
            f.read_to_string(&mut buf)?;

            let filename = p.file_name().unwrap().to_str().unwrap().to_string();
            Ok((filename, api::FileMetadata { content: buf }))
        })
        .collect::<io::Result<_>>()?;

    let req = api::UploadRequest {
        files,
        description: description.map(String::from),
        public: !secret,
    };

    let client = api::Client::build()?;
    let res = client.upload(&login, &req).await?;

    println!("{}", res.html_url);

    Ok(())
}

pub async fn upload_from_stdin(
    login: &config::Login,
    secret: bool,
    filename: &str,
    description: Option<&str>,
) -> Result<()> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;

    let mut files = HashMap::with_capacity(1);
    files.insert(filename.into(), api::FileMetadata { content: buf });

    let req = api::UploadRequest {
        files,
        description: description.map(String::from),
        public: !secret,
    };

    let client = api::Client::build()?;
    let res = client.upload(&login, &req).await?;

    println!("{}", res.html_url);

    Ok(())
}

pub async fn list(login: Option<&config::Login>, username: Option<&str>) -> Result<()> {
    let client = api::Client::build()?;
    let r = client.list(login, username).await?;
    list_gists(&r);
    Ok(())
}

pub async fn list_starred(login: &config::Login) -> Result<()> {
    let client = api::Client::build()?;
    let r = client.list_starred(login).await?;
    list_gists(&r);
    Ok(())
}

fn list_gists(gists: &[api::GistResponse]) {
    for g in gists.iter() {
        if let Some(d) = &g.description {
            println!("{} {}", g.html_url, d);
        } else {
            println!("{}", g.html_url);
        }
    }
}

pub async fn delete(login: &config::Login, id: &[String]) -> Result<()> {
    let client = api::Client::build()?;
    for i in id.iter() {
        client.delete(login, &i).await?;
        println!("{}", i);
    }
    println!("Success!");
    Ok(())
}

pub async fn login<P: AsRef<Path>>(path: P, client_id: &str) -> Result<()> {
    let client = api::Client::build()?;

    let vc = client.request_verification_code(client_id, "gist").await?;

    println!("open {} and enter '{}'", vc.verification_uri, vc.user_code);

    let login = client
        .request_access_token(client_id, &vc.device_code, vc.interval)
        .await?;

    let u = client.user(&login).await?;

    let mut cfg = config::load_config(path.as_ref()).unwrap_or_else(|_| config::Config::new());
    cfg.insert(u.login, login);
    config::save_config(path.as_ref(), &cfg)?;

    println!("Success!");

    Ok(())
}
