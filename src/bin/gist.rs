use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use anyhow::anyhow;
use structopt::StructOpt;

type Username = String;
type Config = BTreeMap<Username, gist::Login>;

#[derive(Debug, StructOpt)]
#[structopt(about = "simple GitHub Gist CLI")]
enum Args {
    Login,
    Upload(Upload),
}

#[derive(Debug, StructOpt)]
struct Upload {
    #[structopt(short)]
    user: Option<String>,
    #[structopt(short, requires = "user")]
    token: Option<String>,
    #[structopt(short)]
    secret: bool,
    #[structopt(short)]
    description: Option<String>,
    #[structopt(name = "FILES", parse(from_os_str), required = true)]
    files: Vec<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();

    match args {
        Args::Login => login(),
        Args::Upload(u) => upload(u),
    }
}

fn upload(args: Upload) -> Result<(), Box<dyn std::error::Error>> {
    let login = match (args.user, args.token) {
        (Some(user), Some(token)) => Ok(gist::Login::PersonalAccessToken { user, token }),
        (Some(u), None) => load_config()?
            .remove(&u)
            .ok_or_else(|| anyhow!("token for '{}' not found", u)),
        _ => load_config()?
            .into_iter()
            .next()
            .map(|l| l.1)
            .ok_or_else(|| anyhow!("empty config file")),
    }?;

    gist::upload(
        &login,
        !args.secret,
        args.description.as_deref(),
        &args.files,
    )?;

    Ok(())
}

fn login() -> Result<(), Box<dyn std::error::Error>> {
    const CLIENT_ID: &str = env!("GIST_CLI_CLIENT_ID");

    let vc = gist::request_verification_code(CLIENT_ID, "gist")?;

    println!("open {} and enter '{}'", vc.verification_uri, vc.user_code);

    let login = gist::request_access_token(CLIENT_ID, &vc.device_code, vc.interval)?;

    println!("{:#?}", login);

    let u = gist::user(&login)?;
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
