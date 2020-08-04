use std::path::PathBuf;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gist::post(
        "Tosainu",
        "XXXXXXXX",
        false,
        "hogehoge",
        &[PathBuf::from_str("src/lib.rs").unwrap()],
    )?;

    Ok(())
}
