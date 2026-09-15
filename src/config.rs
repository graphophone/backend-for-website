use std::{fs, time::Duration};
use anyhow::Result;
use serde::{Deserialize, Deserializer};

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
    pub auth: String,
    pub music_catalog: String,
    pub music_storage: String,
    pub social: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthConfig {
    #[serde(deserialize_with = "parse_duration")]
    pub access_token_expiration_time: Duration,
    #[serde(deserialize_with = "parse_duration")]
    pub refresh_token_expiration_time: Duration,
}

pub fn parse_duration<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
    let err = |msg| <D::Error as serde::de::Error>::custom(msg);
    let s: String = Deserialize::deserialize(deserializer)?;
    let num_part = s.trim_end_matches(|c: char| !c.is_numeric());
    let suffix: &str = &s[num_part.len()..];
    let num: u64 = num_part
        .parse()
        .map_err(|_| err("invalid number".to_string()))?;
    let ret = match suffix {
        "s" => Duration::from_secs(num),
        "ms" => Duration::from_millis(num),
        "min" => Duration::from_mins(num),
        "h" => Duration::from_hours(num),
        other => return Err(err(format!("invalid suffix {other}"))),
    };
    Ok(ret)
}