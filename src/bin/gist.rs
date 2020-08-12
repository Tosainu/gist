use std::path::PathBuf;

use structopt::StructOpt;

use gist::error::{Error, ErrorKind, Result};

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
    #[structopt(short = "t", conflicts_with_all = &["username", "password"])]
    access_token: Option<String>,

    /// Specify user name / saved account name
    #[structopt(short)]
    username: Option<String>,

    /// Specify personal access token
    #[structopt(short, requires = "username")]
    password: Option<String>,
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

fn main() -> Result<()> {
    let args = Args::from_args();

    match args.command {
        Subcommand::Login => gist::app::login(),
        Subcommand::Upload(u) => {
            let l = select_account(args.account)?;
            gist::app::upload(&l, u.secret, u.description.as_deref(), &u.files)
        }
    }
}

fn select_account(account: Account) -> Result<gist::config::Login> {
    if let Some(token) = account.access_token {
        return Ok(gist::config::Login::OAuth(token));
    }

    if let Some(user) = account.username {
        if let Some(token) = account.password {
            return Ok(gist::config::Login::PersonalAccessToken { user, token });
        } else {
            let login = gist::config::load_config()?
                .remove(&user)
                .ok_or_else(|| Error::new(ErrorKind::AccountNotFoundInConfig { name: user }))?;
            return Ok(login);
        }
    }

    let login = gist::config::load_config()?
        .into_iter()
        .next()
        .map(|l| l.1)
        .ok_or_else(|| Error::new(ErrorKind::EmptyConfigurationFile))?;
    Ok(login)
}
