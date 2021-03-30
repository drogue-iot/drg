use crate::config::Config;
use crate::Verbs;
use anyhow::{Context, Result};
use clap::crate_version;
use clap::ArgMatches;
use colored_json::write_colored_json;
use log::LevelFilter;
use reqwest::blocking::{Client, Response};
use reqwest::StatusCode;
use serde_json::{from_str, Value};
use std::io::stdout;
use std::process::exit;
use std::{
    env::var,
    io::{Read, Write},
    process::Command,
};
use tempfile::NamedTempFile;
use url::Url;

pub const VERSION: &str = crate_version!();
pub const COMPATIBLE_DROGUE_VERSION: &str = "0.4.0";

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
        Verbs::edit => match r.status() {
            StatusCode::NO_CONTENT => println!("{} updated.", resource_name),
            r => exit_with_code(r),
        },
    }
}

fn show_json<S: Into<String>>(payload: S) {
    let payload = payload.into();
    match serde_json::from_str(&payload) {
        // show as JSON
        Ok(json) => {
            write_colored_json(&json, &mut stdout().lock()).ok();
        }
        // fall back to plain text output
        Err(_) => println!("{}", payload),
    }
}

fn exit_with_code(r: reqwest::StatusCode) {
    log::error!("Error : {}", r);
    if r.as_u16() == 403 {
        exit(4)
    }
    exit(2)
}

// todo : assume https as the default scheme
// Or get rid of this.
pub fn url_validation(url: &str) -> Result<Url> {
    Url::parse(url).context(format!("URL args: \'{}\' is not valid", url))
}

pub fn json_parse(data: Option<&str>) -> Result<Value> {
    from_str(data.unwrap_or("{}")).context(format!(
        "Can't parse data args: \'{}\' into json",
        data.unwrap_or("")
    ))
}

pub fn editor(original: String) -> Result<Value> {
    let data: Value = serde_json::from_str(original.as_str())?;

    // todo cross platform support
    let editor = var("EDITOR").unwrap_or("vi".to_string());
    let file = NamedTempFile::new()?;
    //the handler needs to be kept to reopen the file later.
    let mut file2 = file.reopen()?;

    // Write the original data to the file.
    file.as_file()
        .write_all(serde_json::to_string_pretty(&data)?.as_bytes())?;

    Command::new(editor)
        .arg(file.path())
        .status()
        .expect("Could not open current data in editor.");

    // Read the data using the second handle.
    let mut buf = String::new();
    file2.read_to_string(&mut buf)?;

    from_str(buf.as_str()).context("Invalid JSON data.")
}

pub fn print_version(config: &Result<Config>) {
    println!("Client Version: {}", VERSION);

    match config {
        Ok(c) => {
            let cloud_version = get_drogue_services_version(&c.drogue_cloud_url).unwrap();
            println!("Connected drogue-cloud service: v{}", cloud_version);
        }
        Err(_) => {
            println!(
                "Not connected to a drogue-cloud service. Compatible with v{}",
                COMPATIBLE_DROGUE_VERSION
            );
        }
    }

    exit(0);
}

// use drogue's well known endpoint to retrieve endpoints.
pub fn get_drogue_services_endpoint(url: Url) -> Result<(Url, Url)> {
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
    match matches.occurrences_of("verbose") {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Debug,
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
