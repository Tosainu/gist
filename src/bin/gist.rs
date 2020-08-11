use std::path::PathBuf;

use anyhow::anyhow;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "simple GitHub Gist CLI")]
struct Args {
    #[structopt(flatten)]
    account: Account,

    #[structopt(subcommand)]
    command: Subcommand,
}

#[derive(Debug, StructOpt)]
struct Account {
    /// Specify OAuth2 access token
    #[structopt(short, conflicts_with_all = &["user-and-token", "config"])]
    access_token: Option<String>,

    /// Specify user name and personal access token
    #[structopt(
        short,
        conflicts_with_all = &["access-token", "config"],
        parse(try_from_str = parse_user_and_token),
        value_name = "user>:<token"
    )]
    user_and_token: Option<(String, String)>,

    /// Use saved token
    #[structopt(short, conflicts_with_all = &["access-token", "user-and-token"])]
    config: Option<String>,
}

#[derive(Debug, StructOpt)]
enum Subcommand {
    Login,
    Upload(Upload),
}

#[derive(Debug, StructOpt)]
struct Upload {
    #[structopt(short)]
    secret: bool,
    #[structopt(short)]
    description: Option<String>,
    #[structopt(name = "FILES", parse(from_os_str), required = true)]
    files: Vec<PathBuf>,
}

fn parse_user_and_token(s: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let pos = s
        .find(':')
        .ok_or_else(|| anyhow!("no ':' found in '{}'", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();

    match args.command {
        Subcommand::Login => login(),
        Subcommand::Upload(u) => {
            let l = select_account(args.account)?;
            upload(&l, u)
        }
    }
}

fn select_account(account: Account) -> Result<gist::config::Login, Box<dyn std::error::Error>> {
    if let Some(token) = account.access_token {
        return Ok(gist::config::Login::OAuth(token));
    }

    if let Some((user, token)) = account.user_and_token {
        return Ok(gist::config::Login::PersonalAccessToken { user, token });
    }

    let login = if let Some(key) = account.config {
        gist::config::load_config()?
            .remove(&key)
            .ok_or_else(|| anyhow!("token for '{}' not found", key))
    } else {
        gist::config::load_config()?
            .into_iter()
            .next()
            .map(|l| l.1)
            .ok_or_else(|| anyhow!("empty config file"))
    }?;

    Ok(login)
}

fn upload(login: &gist::config::Login, args: Upload) -> Result<(), Box<dyn std::error::Error>> {
    let client = gist::github::Client::build()?;
    client.upload(
        &login,
        !args.secret,
        args.description.as_deref(),
        &args.files,
    )?;

    Ok(())
}

fn login() -> Result<(), Box<dyn std::error::Error>> {
    const CLIENT_ID: &str = env!("GIST_CLI_CLIENT_ID");

    let client = gist::github::Client::build()?;

    let vc = client.request_verification_code(CLIENT_ID, "gist")?;

    println!("open {} and enter '{}'", vc.verification_uri, vc.user_code);

    let login = client.request_access_token(CLIENT_ID, &vc.device_code, vc.interval)?;

    println!("{:#?}", login);

    let u = client.user(&login)?;
    println!("{:#?}", u);

    let mut cfg = gist::config::load_config()?;
    cfg.insert(u.login, login); // TODO: check exisiting entry
    gist::config::save_config(&cfg)?;

    Ok(())
}
