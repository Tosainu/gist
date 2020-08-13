use std::path::PathBuf;

use crate::api;
use crate::config;
use crate::error::Result;

pub fn upload(
    login: &config::Login,
    secret: bool,
    description: Option<&str>,
    files: &[PathBuf],
) -> Result<()> {
    let client = api::Client::build()?;
    let res = client.upload(&login, !secret, description, files)?;

    println!("{}", res.html_url);

    Ok(())
}

pub fn list(login: Option<&config::Login>, username: Option<&str>) -> Result<()> {
    let client = api::Client::build()?;
    let r = client.list(login, username)?;
    list_gists(&r);
    Ok(())
}

pub fn list_starred(login: &config::Login) -> Result<()> {
    let client = api::Client::build()?;
    let r = client.list_starred(login)?;
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

pub fn login(client_id: &str) -> Result<()> {
    let client = api::Client::build()?;

    let vc = client.request_verification_code(client_id, "gist")?;

    println!("open {} and enter '{}'", vc.verification_uri, vc.user_code);

    let login = client.request_access_token(client_id, &vc.device_code, vc.interval)?;

    println!("{:#?}", login);

    let u = client.user(&login)?;
    println!("{:#?}", u);

    let mut cfg = config::load_config()?;
    cfg.insert(u.login, login); // TODO: check exisiting entry
    config::save_config(&cfg)?;

    Ok(())
}
