use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{env::var, fs::File};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub drogue_cloud_url: String,
    pub default_app: Option<String>,
}

pub fn load_config_file(path: Option<&str>) -> Result<Config> {
    let path = match path {
        Some(p) => p.to_string(),
        None => var("DRGCFG").unwrap_or(format!("{}/.drgconfig.json", var("HOME")?)),
    };
    //todo verbose option
    println!("Loading config file: {}", path);

    let file = File::open(path)?;
    let config: Config = serde_json::from_reader(file)?;
    Ok(config)
}
