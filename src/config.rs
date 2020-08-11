use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use anyhow::anyhow;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Login {
    #[serde(rename = "oauth")]
    OAuth(String),
    #[serde(rename = "personal_access_token")]
    PersonalAccessToken { user: String, token: String },
}

type Username = String;
pub type Config = BTreeMap<Username, Login>;

pub fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| anyhow!("couldn't find the configuration directory"))?;
    Ok(config_dir.join("gist").join("config.json"))
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let path = config_path()?;
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader)?)
}

pub fn save_config(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path()?;
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, cfg)?;
    Ok(())
}
