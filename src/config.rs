use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{env, fs::write, fs::File};

use chrono::{DateTime, Utc};
use oauth2::basic::BasicTokenResponse;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub drogue_cloud_url: Url,
    pub default_app: Option<String>,
    pub auth_url: Url,
    pub token_url: Url,
    pub registry_url: Url,
    pub token_exp_date: DateTime<Utc>,
    pub token: BasicTokenResponse,
}

pub fn load_config(path: Option<&str>) -> Result<Config> {
    let path = eval_config_path(path);
    log::info!("Loading configuration file: {}", path);

    let file = File::open(path).context("Unable to open configuration file.")?;
    let config: Config = serde_json::from_reader(file).context("Invalid configuration file.")?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = eval_config_path(None);
    log::info!("Saving config file: {}", path);

    write(&path, serde_json::to_string_pretty(&config)?)
        .context(format!("Unable to write config file :{}", path))
}

// use the provided config path or `$DRGCFG` value if set
// otherwise will default to $XDG_CONFIG_HOME
// fall back to `$HOME/.config` if XDG var is not set.
// todo crossplatform support
fn eval_config_path(path: Option<&str>) -> String {
    match path {
        Some(p) => p.to_string(),
        None => env::var("DRGCFG").unwrap_or_else(|_| {
            let xdg = env::var("XDG_CONFIG_HOME")
                .unwrap_or(format!("{}/.config", env::var("HOME").unwrap()));
            format!("{}/drg_config.json", xdg)
        }),
    }
}
