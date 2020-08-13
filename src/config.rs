use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

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

type Username = String;
pub type Config = BTreeMap<Username, Login>;

pub fn config_path() -> Result<PathBuf> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| Error::new(ErrorKind::ConfigDirectoryNotDetected))?;
    Ok(config_dir.join("gist").join("config.json"))
}

pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    match serde_json::from_reader(reader) {
        Ok(config) => Ok(config),
        Err(error) => Err(Error::new(ErrorKind::InvalidConfigFormat { path, error })),
    }
}

pub fn save_config(cfg: &Config) -> Result<()> {
    let path = config_path()?;
    let file = File::create(&path)?;
    let writer = BufWriter::new(file);
    match serde_json::to_writer_pretty(writer, cfg) {
        Ok(()) => Ok(()),
        Err(error) => Err(Error::new(ErrorKind::SaveConfigFailure { path, error })),
    }
}
