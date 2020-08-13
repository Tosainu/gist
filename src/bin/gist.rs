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
    /// Login to GitHub with OAuth2 device flow
    Login(Login),
    /// Upload the files to GitHub Gist
    Upload(Upload),
    /// Browse the gists
    List(List),
    /// Delete the gists
    Delete(Delete),
}

#[derive(Debug, StructOpt)]
struct Login {
    /// Client ID of your OAuth Apps
    #[structopt(required = true)]
    client_id: String,
}

#[derive(Debug, StructOpt)]
struct Upload {
    /// Upload the files as secret gist
    #[structopt(short)]
    secret: bool,

    /// Add a description to gist
    #[structopt(short)]
    description: Option<String>,

    /// Specify the files to upload
    #[structopt(name = "FILES", parse(from_os_str), required = true)]
    files: Vec<PathBuf>,
}

#[derive(Debug, StructOpt)]
struct List {
    /// List the starred gists
    #[structopt(long, conflicts_with = "username")]
    starred: bool,

    /// Specify user name
    #[structopt(short)]
    username: Option<String>,
}

#[derive(Debug, StructOpt)]
struct Delete {
    /// The ID of gist to delete
    #[structopt(required = true)]
    id: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_args();

    match args.command {
        Subcommand::Login(opt) => gist::app::login(&opt.client_id).await?,
        Subcommand::Upload(opt) => {
            let l = select_account(args.account)?;
            gist::app::upload(&l, opt.secret, opt.description.as_deref(), &opt.files).await?;
        }
        Subcommand::List(opt) => {
            let l = select_account(args.account);
            if opt.starred {
                gist::app::list_starred(&l?).await?;
            } else {
                gist::app::list(l.ok().as_ref(), opt.username.as_deref()).await?;
            }
        }
        Subcommand::Delete(opt) => {
            let l = select_account(args.account)?;
            gist::app::delete(&l, &opt.id).await?;
        }
    }

    Ok(())
}

fn select_account(account: Account) -> Result<gist::config::Login> {
    if let Some(token) = account.access_token {
        return Ok(gist::config::Login::OAuth(token));
    }

    if let Some(username) = account.username {
        if let Some(token) = account.password {
            return Ok(gist::config::Login::PersonalAccessToken { username, token });
        } else {
            let login = gist::config::load_config()?
                .remove(&username)
                .ok_or_else(|| Error::new(ErrorKind::AccountNotFoundInConfig { name: username }))?;
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
