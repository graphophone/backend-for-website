use std::fs;
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub services: ServicesConfig,
}

impl Config {
    pub fn new(filename: &str) -> Result<Self> {
        let conf_str = fs::read_to_string(filename)?;
        let conf = toml::from_str(&conf_str)?;
        Ok(conf)
    }
}

#[derive(Debug, Deserialize)]
pub struct ServicesConfig {
    pub identity: String,
    pub music_catalog: String,
    pub music_storage: String,
    pub social: String,
}