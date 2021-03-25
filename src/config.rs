use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{env, fs::write, fs::File};

use oauth2::basic::BasicTokenResponse;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub drogue_cloud_url: String,
    pub default_app: Option<String>,
    pub token: Option<BasicTokenResponse>,
}

pub fn load_config(path: Option<&str>) -> Result<Config> {
    let path = eval_config_path(path);

    //todo verbose option
    println!("Loading config file: {}", path);

    let file = File::open(path)?;
    let config: Config = serde_json::from_reader(file)?;
    Ok(config)
}

pub fn save_config(config: Config) -> Result<()> {
    let path = eval_config_path(None);
    //todo verbose option
    println!("Saving config file: {}", path);

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
