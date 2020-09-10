use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, ErrorKind, Result};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Login {
    #[serde(rename = "oauth")]
    OAuth(String),
    #[serde(rename = "personal_access_token")]
    PersonalAccessToken { username: String, token: String },
}

pub fn default_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("gist"))
}

pub fn default_config_file() -> Option<PathBuf> {
    default_config_dir().map(|p| p.join("config.json"))
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Login> {
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);
    match serde_json::from_reader(reader) {
        Ok(config) => Ok(config),
        Err(error) => Err(Error::new(ErrorKind::InvalidConfigFormat {
            path: path.as_ref().to_path_buf(),
            error,
        })),
    }
}

pub fn save_config<P: AsRef<Path>>(path: P, cfg: &Login) -> Result<()> {
    let dir = path.as_ref().parent().unwrap();
    if !dir.exists() {
        DirBuilder::new().recursive(true).create(dir)?;
    }
    let file = File::create(path.as_ref())?;
    let writer = BufWriter::new(file);
    match serde_json::to_writer_pretty(writer, cfg) {
        Ok(()) => Ok(()),
        Err(error) => Err(Error::new(ErrorKind::SaveConfigFailure {
            path: path.as_ref().to_path_buf(),
            error,
        })),
    }
}
