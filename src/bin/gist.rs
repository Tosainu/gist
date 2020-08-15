use std::path::{Path, PathBuf};

use structopt::StructOpt;

use gist::error::{Error, ErrorKind, Result};

#[derive(Debug, StructOpt)]
#[structopt(about = "simple GitHub Gist CLI")]
struct Args {
    /// Specify configuration file
    #[structopt(long, parse(from_os_str))]
    config: Option<PathBuf>,

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
    #[structopt(flatten)]
    account: Account,

    /// Upload the files as secret gist
    #[structopt(short)]
    secret: bool,

    /// Specify a default name of gist
    #[structopt(short, default_value = "file.txt")]
    filename: String,

    /// Add a description to gist
    #[structopt(short)]
    description: Option<String>,

    /// Specify the files to upload
    #[structopt(name = "FILES", parse(from_os_str))]
    files: Vec<PathBuf>,
}

#[derive(Debug, StructOpt)]
struct List {
    #[structopt(flatten)]
    account: Account,

    /// List the starred gists
    #[structopt(long, conflicts_with = "username")]
    starred: bool,

    /// List public gists for the specified user
    author: Option<String>,
}

#[derive(Debug, StructOpt)]
struct Delete {
    #[structopt(flatten)]
    account: Account,

    /// The ID of gist to delete
    #[structopt(required = true)]
    id: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_args();

    let path = args.config.or_else(gist::config::default_config_file);

    match args.command {
        Subcommand::Login(opt) => {
            let path = path.ok_or_else(|| Error::new(ErrorKind::ConfigDirectoryNotDetected))?;
            gist::app::login(path, &opt.client_id).await?;
        }
        Subcommand::Upload(opt) => {
            let l = select_account(path, opt.account)?;
            if opt.files.is_empty() {
                gist::app::upload_from_stdin(
                    &l,
                    opt.secret,
                    &opt.filename,
                    opt.description.as_deref(),
                )
                .await?;
            } else {
                gist::app::upload(&l, opt.secret, opt.description.as_deref(), &opt.files).await?;
            }
        }
        Subcommand::List(opt) => {
            let l = select_account(path, opt.account);
            if opt.starred {
                gist::app::list_starred(&l?).await?;
            } else {
                gist::app::list(l.ok().as_ref(), opt.author.as_deref()).await?;
            }
        }
        Subcommand::Delete(opt) => {
            let l = select_account(path, opt.account)?;
            gist::app::delete(&l, &opt.id).await?;
        }
    }

    Ok(())
}

fn select_account<P: AsRef<Path>>(
    path: Option<P>,
    account: Account,
) -> Result<gist::config::Login> {
    if let Some(token) = account.access_token {
        return Ok(gist::config::Login::OAuth(token));
    }

    if let Some(username) = account.username {
        if let Some(token) = account.password {
            return Ok(gist::config::Login::PersonalAccessToken { username, token });
        } else {
            let path = path.ok_or_else(|| Error::new(ErrorKind::ConfigDirectoryNotDetected))?;
            let login = gist::config::load_config(path)?
                .remove(&username)
                .ok_or_else(|| Error::new(ErrorKind::AccountNotFoundInConfig { name: username }))?;
            return Ok(login);
        }
    }

    let path = path.ok_or_else(|| Error::new(ErrorKind::ConfigDirectoryNotDetected))?;
    let login = gist::config::load_config(path)?
        .into_iter()
        .next()
        .map(|l| l.1)
        .ok_or_else(|| Error::new(ErrorKind::EmptyConfigurationFile))?;
    Ok(login)
}
