use std::path::PathBuf;

use crate::api;
use crate::config;

pub fn upload(
    login: &config::Login,
    secret: bool,
    description: Option<&str>,
    files: &Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = api::Client::build()?;
    let res = client.upload(&login, !secret, description, files)?;

    println!("{}", res.html_url);

    Ok(())
}

pub fn login() -> Result<(), Box<dyn std::error::Error>> {
    const CLIENT_ID: &str = env!("GIST_CLI_CLIENT_ID");

    let client = api::Client::build()?;

    let vc = client.request_verification_code(CLIENT_ID, "gist")?;

    println!("open {} and enter '{}'", vc.verification_uri, vc.user_code);

    let login = client.request_access_token(CLIENT_ID, &vc.device_code, vc.interval)?;

    println!("{:#?}", login);

    let u = client.user(&login)?;
    println!("{:#?}", u);

    let mut cfg = config::load_config()?;
    cfg.insert(u.login, login); // TODO: check exisiting entry
    config::save_config(&cfg)?;

    Ok(())
}
