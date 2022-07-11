mod certs;
mod display;
mod endpoints;
mod error;
mod operations;
mod outcome;

pub use certs::*;
pub use display::*;
pub use endpoints::*;
pub use error::*;
pub use outcome::*;

use crate::config::Config;
use crate::{AccessToken, Context, Parameters};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use chrono::{DateTime, Duration, Utc};
use clap::crate_version;
use clap::{ArgMatches, Values};
use colored_json::write_colored_json;
use drogue_client::discovery::v1::Client;
use drogue_client::registry::v1::labels::LabelSelector;
use log::LevelFilter;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs;
use std::io::stdout;
use std::io::{Read, Write};
use tempfile::Builder;
use url::Url;

pub const VERSION: &str = crate_version!();
pub const COMPATIBLE_DROGUE_VERSION: &str = "0.10.0";

pub fn show_json(payload: &Value) {
    write_colored_json(payload, &mut stdout().lock()).ok();
    println!();
}

pub fn show_json_string<S: Into<String>>(payload: S) {
    let payload = payload.into();
    match serde_json::from_str(&payload) {
        // show as JSON
        Ok(json) => {
            show_json(&json);
        }
        // fall back to plain text output
        Err(_) => println!("{}", payload),
    }
}

pub fn url_validation(url: &str) -> Result<Url> {
    Url::parse(url).or_else(|_| {
        Url::parse(&format!("https://{}", url))
            .context(format!("URL args: \'{}\' is not valid", url))
    })
}

pub fn json_parse_option(data: Option<&str>) -> Result<Option<Value>> {
    match data {
        Some(data) => {
            let json: Value = serde_json::from_str(data)
                .context(format!("Can't parse data args: \'{data}\' into json",))?;
            Ok(Some(json))
        }
        None => Ok(None),
    }
}

pub fn editor<S, T>(original: S) -> Result<T, DrogueError>
where
    S: Serialize,
    T: DeserializeOwned,
{
    let original_string = serde_yaml::to_string(&original)?;

    let file = Builder::new().suffix(".yml").tempfile()?;
    //the handler needs to be kept to reopen the file later.
    let mut file2 = file.reopen()?;

    // Write the original data to the file, but in YAML for easier editing
    file.as_file().write_all(original_string.as_bytes())?;

    edit::edit_file(file.path())
        .map_err(|err| {
            log::debug!("{}", err);
            show_json_string(
                serde_json::to_string(&original)
                    .unwrap_or_else(|_| String::from("Error serializing the resource")),
            );
            DrogueError::InvalidInput("cannot open a text editor. You can manually edit the file then retry using --filename".to_string())
        })
        .unwrap();

    // Read the data using the second handle.
    let mut buf = String::new();
    file2.read_to_string(&mut buf)?;

    if original_string == buf {
        Err(DrogueError::InvalidInput("No changes made".to_string()))
    } else {
        Ok(serde_yaml::from_str(buf.as_str()).context("Invalid YAML data.")?)
    }
}

pub async fn print_version(config: Option<&Config>, json: bool) {
    let cloud_version = match config {
        Some(cfg) => {
            let context = cfg.get_context(&None);
            match context {
                Ok(ctx) => {
                    get_drogue_services_version(&ctx.drogue_cloud_url).await.map_err(|e| {
                        log::debug!("Failed to detect server side version: {}", e);
                        format!("Error retrieving drogue-cloud version. Compatible with drogue-cloud v{}", COMPATIBLE_DROGUE_VERSION)
             })},
             Err(e) => {
                 log::debug!("Error getting context. {}", e);
                 Err("Error reading context".to_string())
             }
            }
        }
        None => Err(format!(
            "No configuration file. Compatible with v{}",
            COMPATIBLE_DROGUE_VERSION
        )),
    };

    if json {
        show_json(&match cloud_version {
            Ok(cloud) => json!({
                "drg": VERSION,
                "compatible_cloud": COMPATIBLE_DROGUE_VERSION,
                "connected_cloud": cloud
            }),
            Err(_) => json!({
                "drg": VERSION,
                "compatible_cloud": COMPATIBLE_DROGUE_VERSION
            }),
        });
    } else {
        println!("Drg Version: {}", VERSION);
        match cloud_version {
            Ok(cloud) => println!("Connected drogue-cloud service: v{}", cloud),
            Err(e) => println!("{}", e),
        }
    }
}

pub fn log_level(matches: &ArgMatches) -> LevelFilter {
    match matches.occurrences_of(Parameters::verbose.as_ref()) {
        0 => LevelFilter::Error,
        1 => {
            println!("Log level: WARN");
            LevelFilter::Warn
        }
        2 => {
            println!("Log level: INFO");
            LevelFilter::Info
        }
        _ => {
            println!("Log level: DEBUG");
            LevelFilter::Debug
        }
    }
}

// use drogue's well known endpoint to retrieve version.
async fn get_drogue_services_version(url: &Url) -> Result<String> {
    let client: Client = Client::new_anonymous(reqwest::Client::new(), url.clone());

    client
        .get_drogue_cloud_version()
        .await?
        .ok_or_else(|| anyhow!("Error retrieving drogue version"))
        .map(|v| v.version)
}

pub fn get_data_from_file<T>(path: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let contents = fs::read_to_string(path).context("Something went wrong reading the file")?;

    serde_json::from_str(contents.as_str()).context("Invalid JSON in file")
}

pub fn age_from_timestamp(time: &DateTime<Utc>) -> String {
    let age = Utc::now().naive_utc() - time.naive_utc();

    if age > Duration::days(7) {
        format!("{}d", age.num_days())
    } else if age > Duration::days(3) {
        let hours = age
            .checked_sub(&Duration::days(age.num_days()))
            .unwrap_or_else(|| Duration::hours(0));
        format!("{}d{}h", age.num_days(), hours.num_hours())
    } else if age > Duration::hours(2) {
        format!("{}h", age.num_hours())
    } else if age > Duration::minutes(2) {
        format!("{}m", age.num_minutes())
    } else {
        format!("{}s", age.num_seconds())
    }
}

pub fn process_labels(args: &Values) -> Value {
    // split the labels around the =
    let labels: HashMap<&str, &str> = args
        .clone()
        .filter_map(|l| {
            let mut s = l.split('=');
            let k = s.next();
            let v = s.next();
            k.zip(v)
        })
        .collect();

    // prepare json data to merge
    json!({"metadata": {
    "labels": labels
    }})
}

pub fn clap_values_to_labels(labels: Option<Values>) -> Option<LabelSelector> {
    if let Some(labels) = labels {
        let labels = labels.into_iter().collect::<Vec<&str>>().join(",");

        if let Ok(ls) = LabelSelector::try_from(labels.as_str()) {
            Some(ls)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn name_from_json_or_file(param: Option<String>, file: Option<&str>) -> Result<String> {
    match (param, file) {
        (Some(id), None) => Ok(id),
        (None, Some(file)) => {
            let f: Value = get_data_from_file(file)?;
            let id = f["metadata"]["name"]
                .as_str()
                .context("Misisng `name` property in device definition file")?
                .to_string();
            Ok(id)
        }
        // we must have id or file, not both, not neither.
        _ => unreachable!(),
    }
}

// Verify the access token while constructing a Context
pub async fn context_from_access_token(
    name: String,
    api: Url,
    user: &str,
    key: &str,
) -> Result<Context> {
    let token = AccessToken {
        token: key.to_string(),
        id: user.to_string(),
    };
    let mut cfg = Context::init_with_access_token(name, api.clone(), token);

    let (sso_url, registry_url) = get_drogue_endpoints(api).await?;
    let (auth_url, token_url) = get_auth_and_tokens_endpoints(sso_url).await?;

    cfg.fill_urls(auth_url, registry_url, token_url);
    // cfg doesn't need to be mut anymore
    let cfg = cfg;

    let cfg_ref = &cfg;
    // test if the token is valid
    let _ = get_drogue_endpoints_authenticated(cfg_ref).await?;

    Ok(cfg)
}
