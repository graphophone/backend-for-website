use std::{fs, time::Duration};
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub services: ServicesConfig,
    pub auth: AuthConfig,
}

impl Config {
    pub fn new(filename: &str) -> Result<Self> {
        let conf_str = fs::read_to_string(filename)?;
        let conf = serde_yaml::from_str(&conf_str)?;
        Ok(conf)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServicesConfig {
    pub identity: String,
    pub music_catalog: String,
    pub music_storage: String,
    pub social: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthConfig {
    pub access_token_expiration_time: Duration,
    pub refresh_token_expiration_time: Duration,
}