use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use anyhow::anyhow;
use structopt::StructOpt;

type Username = String;
type Config = BTreeMap<Username, gist::github::Login>;

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

fn select_account(account: Account) -> Result<gist::github::Login, Box<dyn std::error::Error>> {
    if let Some(token) = account.access_token {
        return Ok(gist::github::Login::OAuth(token));
    }

    if let Some((user, token)) = account.user_and_token {
        return Ok(gist::github::Login::PersonalAccessToken { user, token });
    }

    let login = if let Some(key) = account.config {
        load_config()?
            .remove(&key)
            .ok_or_else(|| anyhow!("token for '{}' not found", key))
    } else {
        load_config()?
            .into_iter()
            .next()
            .map(|l| l.1)
            .ok_or_else(|| anyhow!("empty config file"))
    }?;

    Ok(login)
}

fn upload(login: &gist::github::Login, args: Upload) -> Result<(), Box<dyn std::error::Error>> {
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

    let mut cfg = load_config()?;
    cfg.insert(u.login, login); // TODO: check exisiting entry
    save_config(&cfg)?;

    Ok(())
}

fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| anyhow!("couldn't find the configuration directory"))?;
    Ok(config_dir.join("gist").join("config.json"))
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let path = config_path()?;
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader)?)
}

fn save_config(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path()?;
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, cfg)?;
    Ok(())
}
