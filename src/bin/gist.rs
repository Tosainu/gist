use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::anyhow;
use structopt::StructOpt;

type Username = String;
type Config = BTreeMap<Username, gist::Login>;

#[derive(Debug, StructOpt)]
#[structopt(about = "simple GitHub Gist CLI")]
struct Args {
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

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| anyhow!("couldn't find the configuration directory"))?;
    let config_file = config_dir.join("gist").join("config.json");

    let file = File::open(config_file)?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader)?)
}
