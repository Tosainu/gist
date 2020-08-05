use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "simple GitHub Gist CLI")]
struct Args {
    #[structopt(short, required = true)]
    user: String,
    #[structopt(short, required = true)]
    token: String,
    #[structopt(short)]
    secret: bool,
    #[structopt(short)]
    description: Option<String>,
    #[structopt(name = "FILES", parse(from_os_str), required = true)]
    files: Vec<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();

    gist::post(
        &args.user,
        &args.token,
        !args.secret,
        args.description.as_deref(),
        &args.files,
    )?;

    Ok(())
}
