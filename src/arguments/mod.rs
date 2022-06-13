pub mod cli;
pub mod config;
pub mod create;
pub mod delete;
pub mod edit;
pub mod get;

use crate::Context;
use anyhow::{anyhow, Result};
use clap::ArgMatches;

pub fn get_app_id<'a>(matches: &'a ArgMatches, config: &'a Context) -> Result<String> {
    match matches.value_of("app-flag") {
        Some(a) => Ok(a.to_string()),
        None => config
            .default_app
            .as_ref()
            .map(|v| {
                log::debug!("Using default app \"{}\".", &v);
                v.to_string()
            })
            .ok_or_else(|| {
                anyhow!("Missing app argument and no default app specified in config file.")
            }),
    }
}
