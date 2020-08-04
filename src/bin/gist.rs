use std::path::PathBuf;
use std::str::FromStr;

fn main() {
    gist::post(
        "Tosainu",
        "XXXXXXXX",
        false,
        "hogehoge",
        &[PathBuf::from_str("src/lib.rs").unwrap()],
    );
}
