use crate::config::{Config, Context};
use crate::Verbs;
use crate::Other_flags;
use anyhow::{Context as AnyhowContext, Result};
use chrono::{Duration, Utc};
use clap::crate_version;
use clap::ArgMatches;
use colored_json::write_colored_json;
use log::LevelFilter;
use oauth2::TokenResponse;
use reqwest::blocking::{Client, Response};
use reqwest::StatusCode;
use serde_json::{from_str, Value};
use std::fs;
use std::io::stdout;
use std::io::{Read, Write};
use std::process::exit;
use tempfile::Builder;
use url::Url;

pub const VERSION: &str = crate_version!();
pub const COMPATIBLE_DROGUE_VERSION: &str = "0.5.0";
pub const API_PATH: &str = "api/registry/v1alpha1";

pub fn print_result(r: Response, resource_name: String, op: Verbs) {
    match op {
        Verbs::create => match r.status() {
            StatusCode::CREATED => println!("{} created.", resource_name),
            r => exit_with_code(r),
        },
        Verbs::delete => match r.status() {
            StatusCode::NO_CONTENT => println!("{} deleted.", resource_name),
            r => exit_with_code(r),
        },
        Verbs::get => match r.status() {
            StatusCode::OK => show_json(r.text().expect("Empty response")),
            r => exit_with_code(r),
        },
        Verbs::edit | Verbs::set => match r.status() {
            StatusCode::NO_CONTENT => println!("{} updated.", resource_name),
            r => exit_with_code(r),
        },
    }
}

pub fn show_json<S: Into<String>>(payload: S) {
    let payload = payload.into();
    match serde_json::from_str(&payload) {
        // show as JSON
        Ok(json) => {
            write_colored_json(&json, &mut stdout().lock()).ok();
            println!();
        }
        // fall back to plain text output
        Err(_) => println!("{}", payload),
    }
}

pub fn exit_with_code(r: reqwest::StatusCode) -> ! {
    log::error!("Error : {}", r);
    if r.as_u16() == 403 {
        exit(4)
    }
    exit(2)
}

pub fn url_validation(url: &str) -> Result<Url> {
    Url::parse(url).or_else(|_| {
        Url::parse(&format!("https://{}", url))
            .context(format!("URL args: \'{}\' is not valid", url))
    })
}

pub fn json_parse(data: Option<&str>) -> Result<Value> {
    from_str(data.unwrap_or("{}")).context(format!(
        "Can't parse data args: \'{}\' into json",
        data.unwrap_or("")
    ))
}

pub fn editor(original: String) -> Result<Value> {
    let data: Value = serde_json::from_str(original.as_str())?;

    let file = Builder::new().suffix(".yml").tempfile()?;
    //the handler needs to be kept to reopen the file later.
    let mut file2 = file.reopen()?;

    // Write the original data to the file, but in YAML for easier editing
    file.as_file()
        .write_all(serde_yaml::to_string(&data)?.as_bytes())?;

    edit::edit_file(file.path())
        .map_err(|err| {
            log::debug!("{}", err);
            log::error!(
                "Error opening a text editor, please try using --filename with the following json"
            );
            show_json(&original);
            exit(1);
        })
        .unwrap();

    // Read the data using the second handle.
    let mut buf = String::new();
    file2.read_to_string(&mut buf)?;

    let new_data: Value = serde_yaml::from_str(buf.as_str()).context("Invalid YAML data.")?;
    if data == new_data {
        println!("Edit cancelled, no changes made.");
        exit(2);
    } else {
        Ok(new_data)
    }
}

pub fn print_version(config: &Result<Config>) {
    println!("Drg Version: {}", VERSION);

    match config {
        Ok(cfg) => {
            let context = cfg.get_context(&None);
            match context {
                Ok(ctx) => match get_drogue_services_version(&ctx.drogue_cloud_url) {
                    Ok(cloud_version) => {
                        println!("Connected drogue-cloud service: v{}", cloud_version);
                    }
                    Err(err) => {
                        log::debug!("Failed to detect server side version: {}", err);
                    }
                },
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
        Err(e) => {
            println!(
                "Invalid configuration file. Compatible with v{}",
                COMPATIBLE_DROGUE_VERSION
            );
            log::info!("{}", e)
        }
    }

    exit(0);
}

// use drogue's well known endpoint to retrieve endpoints.
pub fn get_drogue_services_endpoints(url: Url) -> Result<(Url, Url)> {
    let client = Client::new();

    let url = url.join(".well-known/drogue-endpoints")?;

    let res = client
        .get(url)
        .send()
        .context("Can't retrieve drogue endpoints details")?;

    let endpoints: Value = res
        .json()
        .context("Cannot deserialize drogue endpoints details")?;

    let sso = endpoints["issuer_url"]
        .as_str()
        .context("Missing `issuer_url` in drogue endpoint details")?;
    let registry = endpoints["registry"]["url"]
        .as_str()
        .context("Missing `registry` in drogue endpoint details")?;

    // a trailing / is needed to append the rest of the path.
    Ok((
        url_validation(format!("{}/", sso).as_str())?,
        url_validation(format!("{}/", registry).as_str())?,
    ))
}

pub fn get_drogue_websocket_endpoint(context: &Context) -> Result<Url> {
    let client = Client::new();
    let url = format!("{}api/console/v1alpha1/info", &context.registry_url);
    let res = client
        .get(url)
        .bearer_auth(&context.token.access_token().secret())
        .send()
        .context("Can't retrieve drogue services details")?;

    let endpoints: Value = res
        .json()
        .context("Cannot deserialize drogue endpoints details")?;

    let ws = endpoints["websocket_integration"]["url"]
        .as_str()
        .context("No `websocket_integration` service in drogue endpoints list")?;

    Ok(url_validation(format!("{}", ws).as_str())?)
}

// use keycloak's well known endpoint to retrieve endpoints.
// http://keycloakhost:keycloakport/auth/realms/{realm}/.well-known/openid-configuration
pub fn get_auth_and_tokens_endpoints(issuer_url: Url) -> Result<(Url, Url)> {
    let client = Client::new();

    let url = issuer_url.join(".well-known/openid-configuration")?;
    let res = client
        .get(url)
        .send()
        .context("Can't retrieve openid-connect endpoints details")?;

    let endpoints: Value = res
        .json()
        .context("Cannot deserialize openid-connect endpoints details")?;

    let auth = endpoints["authorization_endpoint"]
        .as_str()
        .context("Missing `authorization_endpoint` in drogue openid-connect configuration")?;
    let auth_endpoint = url_validation(auth);
    let token = endpoints["token_endpoint"]
        .as_str()
        .context("Missing `token_endpoint` in drogue openid-connect configuration")?;
    let token_endpoint = url_validation(token);

    Ok((auth_endpoint?, token_endpoint?))
}

pub fn log_level(matches: &ArgMatches) -> LevelFilter {
    match matches.occurrences_of(Other_flags::verbose) {
        0 => LevelFilter::Error,
        1 =>  {
            println!("Log level: WARN");
            LevelFilter::Warn
        },
        2 => {
            println!("Log level: INFO");
            LevelFilter::Info
        }
        _ => {
            println!("Log level: DEBUG");
            LevelFilter::Debug
        },
    }
}

// use drogue's well known endpoint to retrieve version.
fn get_drogue_services_version(url: &Url) -> Result<String> {
    let client = Client::new();

    let url = url.join(".well-known/drogue-version")?;

    let res = client
        .get(url)
        .send()
        .context("Can't retrieve drogue version")?;

    let payload: Value = res
        .json()
        .context("Cannot deserialize drogue version payload")?;

    let version = payload["version"]
        .as_str()
        .context("Missing `version` in drogue version payload")?;

    Ok(version.to_string())
}

pub fn get_data_from_file(path: &str) -> Result<Value> {
    let contents = fs::read_to_string(path).context("Something went wrong reading the file")?;

    serde_json::from_str(contents.as_str()).context("Invalid JSON in file")
}

pub fn age(str_timestamp: &str) -> Result<String> {
    let time = chrono::DateTime::parse_from_rfc3339(str_timestamp)?;
    let age = Utc::now().naive_utc() - time.naive_utc();

    if age > Duration::days(7) {
        Ok(format!("{}d", age.num_days()))
    } else if age > Duration::days(3) {
        let hours = age
            .checked_sub(&Duration::days(age.num_days()))
            .unwrap_or_else(|| Duration::hours(0));
        Ok(format!("{}d{}h", age.num_days(), hours.num_hours()))
    } else if age > Duration::hours(2) {
        Ok(format!("{}h", age.num_hours()))
    } else if age > Duration::minutes(2) {
        Ok(format!("{}m", age.num_minutes()))
    } else {
        Ok(format!("{}s", age.num_seconds()))
    }
}
