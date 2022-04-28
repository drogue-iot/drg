use crate::config::{Config, Context};
use crate::Parameters;
use anyhow::{anyhow, Context as AnyhowContext, Result};
use chrono::{DateTime, Duration, Utc};
use clap::crate_version;
use clap::{ArgMatches, Values};
use colored_json::write_colored_json;
use drogue_client::discovery::v1::Client;
use drogue_client::discovery::v1::Endpoints;
use drogue_client::openid::NoTokenProvider;
use drogue_client::registry::v1::labels::LabelSelector;
use log::LevelFilter;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value::String as serde_string;
use serde_json::{from_str, json, Value};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs;
use std::io::stdout;
use std::io::{Read, Write};
use std::process::exit;
use tabular::{Row, Table};
use tempfile::Builder;
use url::Url;

pub const VERSION: &str = crate_version!();
pub const COMPATIBLE_DROGUE_VERSION: &str = "0.9.0";

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

pub fn editor<S, T>(original: S) -> Result<T>
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
            log::error!(
                "Error opening a text editor, you can try with the --filename option. \
                Here is the original resource:"
            );
            show_json(
                serde_json::to_string(&original)
                    .unwrap_or_else(|_| String::from("Error serializing the resource")),
            );
            exit(1);
        })
        .unwrap();

    // Read the data using the second handle.
    let mut buf = String::new();
    file2.read_to_string(&mut buf)?;

    if original_string == buf {
        println!("Edit cancelled, no changes made.");
        exit(2);
    } else {
        Ok(serde_yaml::from_str(buf.as_str()).context("Invalid YAML data.")?)
    }
}

pub async fn print_version(config: &Result<Config>) {
    println!("Drg Version: {}", VERSION);

    match config {
        Ok(cfg) => {
            let context = cfg.get_context(&None);
            match context {
                Ok(ctx) => match get_drogue_services_version(&ctx.drogue_cloud_url).await {
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
pub async fn get_drogue_services_endpoints(url: Url) -> Result<(Url, Url)> {
    let client: Client<NoTokenProvider> = Client::new_anonymous(reqwest::Client::new(), url);

    let endpoints = client
        .get_public_endpoints()
        .await?
        .ok_or_else(|| anyhow!("Error fetching drogue-cloud endpoints."))?;

    let registry = endpoints
        .registry
        .ok_or_else(|| anyhow!("Missing SSO endpoint."))?
        .url;
    let sso = endpoints
        .issuer_url
        .ok_or_else(|| anyhow!("Missing SSO endpoint."))?;
    // Url::join remove the last segment if there is no trailing slash so we append it there
    let registry = format!("{registry}/");
    let sso = format!("{sso}/");

    Ok((Url::parse(sso.as_str())?, Url::parse(registry.as_str())?))
}

async fn get_drogue_endpoints_authenticated(context: &Context) -> Result<Endpoints> {
    let client = Client::new_authenticated(
        reqwest::Client::new(),
        context.drogue_cloud_url.clone(),
        context,
    );

    client
        .get_authenticated_endpoints()
        .await?
        .ok_or_else(|| anyhow!("Error fetching drogue-cloud endpoints."))
}

pub async fn get_drogue_console_endpoint(context: &Context) -> Result<Url> {
    let endpoints = get_drogue_endpoints_authenticated(context).await?;
    let console = endpoints
        .console
        .ok_or_else(|| anyhow!("No `console` service in drogue endpoints list"))?;

    Url::parse(console.as_str()).context("Cannot parse console url to a valid url")
    //url_validation(ws)
}

pub async fn get_drogue_websocket_endpoint(context: &Context) -> Result<Url> {
    let endpoints = get_drogue_endpoints_authenticated(context).await?;
    let ws = endpoints
        .websocket_integration
        .ok_or_else(|| anyhow!("No `console` service in drogue endpoints list"))?;

    Url::parse(ws.url.as_str()).context("Cannot parse console url to a valid url")
}

// use keycloak's well known endpoint to retrieve endpoints.
// http://keycloakhost:keycloakport/auth/realms/{realm}/.well-known/openid-configuration
pub async fn get_auth_and_tokens_endpoints(issuer_url: Url) -> Result<(Url, Url)> {
    let client = reqwest::Client::new();

    let url = issuer_url.join(".well-known/openid-configuration")?;
    let res = client
        .get(url)
        .send()
        .await
        .context("Can't retrieve openid-connect endpoints details")?;

    let endpoints: Value = res
        .json()
        .await
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
    let client: Client<NoTokenProvider> =
        Client::new_anonymous(reqwest::Client::new(), url.clone());

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

pub async fn print_endpoints(context: &Context, service: Option<&str>) -> Result<()> {
    let endpoints = get_drogue_endpoints_authenticated(context).await?;
    let endpoints = serde_json::to_value(endpoints)?;
    let endpoints = endpoints.as_object().unwrap();

    if let Some(service) = service {
        let details = endpoints
            .get(service)
            .ok_or_else(|| anyhow!("Service not found in endpoints list."))?;
        let (host, port) = deserialize_endpoint(details);

        println!("{}{}", host.unwrap(), port);
    } else {
        let mut table = Table::new("{:<} {:<}");
        table.add_row(Row::new().with_cell("NAME").with_cell("URL"));

        for (name, details) in endpoints {
            let (host, port) = deserialize_endpoint(details);
            host.map(|h| {
                table.add_row(
                    Row::new()
                        .with_cell(name)
                        .with_cell(format!("{}{}", h, port)),
                )
            });
        }
        print!("{}", table);
    }

    Ok(())
}

fn deserialize_endpoint(details: &Value) -> (Option<String>, String) {
    let (host, port) = match details {
        serde_string(s) => (Some(s.clone()), None),
        Value::Object(v) => (
            v.get("url")
                .or_else(|| v.get("host"))
                .map(|h| h.as_str().unwrap().to_string()),
            v.get("port").map(|s| s.as_i64().unwrap()),
        ),
        _ => (None, None),
    };

    let port = port.map_or("".to_string(), |p| format!(":{}", p));
    (host, port)
}

pub fn process_labels(args: &Values) -> Value {
    // split the labels around the =
    let labels: HashMap<&str, &str> = args
        .clone()
        .map(|l| {
            let mut s = l.split('=');
            let k = s.next();
            let v = s.next();
            k.zip(v)
        })
        .flatten()
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
