use crate::config::{Context, RequestBuilderExt};
use crate::{util, AppId, DeviceId, Verbs};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use json_value_merge::Merge;
use reqwest::blocking::Client;
use reqwest::blocking::Response;
use reqwest::{StatusCode, Url};
use serde_json::{from_str, json, Value};
use sha2::{Digest, Sha512};
use std::process::exit;
use tabular::{Row, Table};

fn craft_url(base: &Url, app_id: &str, device_id: Option<&str>) -> String {
    let device = match device_id {
        Some(dev) => format!("/{}", urlencoding::encode(dev)),
        None => String::new(),
    };
    format!(
        "{}{}/apps/{}/devices{}",
        base,
        util::REGISTRY_API_PATH,
        urlencoding::encode(app_id),
        device
    )
}

pub fn delete(
    config: &Context,
    app: AppId,
    device_id: DeviceId,
    ignore_missing: bool,
) -> Result<()> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, &app, Some(&device_id));

    client
        .delete(&url)
        .auth(&config.token)
        .send()
        .context("Can't delete device.")
        .map(|res| {
            if ignore_missing && res.status() == StatusCode::NOT_FOUND {
                exit(0);
            } else {
                util::print_result(res, format!("Device {}", device_id), Verbs::delete)
            }
        })
}

pub fn read(config: &Context, app: AppId, device_id: DeviceId) -> Result<()> {
    get(config, &app, &device_id)
        .map(|res| util::print_result(res, device_id.to_string(), Verbs::get))
}

pub fn create(
    config: &Context,
    device_id: DeviceId,
    data: serde_json::Value,
    app_id: AppId,
    file: Option<&str>,
) -> Result<()> {
    let data = if data == json!({}) {
        json!({"credentials": {}})
    } else {
        data
    };

    let body = match file {
        Some(f) => util::get_data_from_file(f)?,
        None => {
            json!({
            "metadata": {
                "name": device_id,
                "application": app_id
            },
            "spec": data
            })
        }
    };

    let client = Client::new();
    let url = craft_url(&config.registry_url, &app_id, None);

    client
        .post(&url)
        .auth(&config.token)
        .json(&body)
        .send()
        .context("Can't create device.")
        .map(|res| util::print_result(res, format!("Device {}", device_id), Verbs::create))
}

pub fn edit(config: &Context, app: AppId, device_id: DeviceId, file: Option<&str>) -> Result<()> {
    match file {
        Some(f) => {
            let data = util::get_data_from_file(f)?;

            put(config, &app, &device_id, data)
                .map(|res| util::print_result(res, format!("Device {}", device_id), Verbs::edit))
        }
        None => {
            //read device data
            let res = get(config, &app, &device_id);
            match res {
                Ok(r) => match r.status() {
                    StatusCode::OK => {
                        let body = r.text().unwrap_or_else(|_| "{}".to_string());
                        let insert = util::editor(body)?;
                        put(config, &app, &device_id, insert).map(|p| {
                            util::print_result(p, format!("Device {}", device_id), Verbs::edit)
                        })
                    }
                    e => {
                        log::error!("Error : could not retrieve device: {}", e);
                        util::exit_with_code(e)
                    }
                },
                Err(e) => {
                    log::error!("Error : could not execute request: {}", e);
                    exit(2)
                }
            }
        }
    }
}

pub fn list(config: &Context, app: AppId, labels: Option<String>) -> Result<()> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, &app, None);

    let mut req = client.get(&url).auth(&config.token);

    if let Some(labels) = labels {
        req = req.query(&[("labels", labels)]);
    }

    let res = req.send().context("Can't list devices");

    if let Ok(r) = res {
        if r.status() == StatusCode::OK {
            pretty_list(r.text()?)?;
            Ok(())
        } else {
            Err(anyhow!("List operation failed with {}", r.status()))
        }
    } else {
        Err(anyhow!("Error while requesting devices list."))
    }
}

pub fn set_gateway(
    config: &Context,
    app: AppId,
    device_id: DeviceId,
    gateway_id: DeviceId,
) -> Result<()> {
    // prepare json data to merge
    let data = json!({"spec": {
    "gatewaySelector": {
        "matchNames": [gateway_id]
    }
    }});

    set(config, app, device_id, data)
}

pub fn set_password(
    config: &Context,
    app: AppId,
    device_id: DeviceId,
    password: String,
    username: Option<&str>,
) -> Result<()> {
    let mut hasher = Sha512::new();
    hasher.update(password.as_bytes());
    let hash = &hasher.finalize()[..];

    let credential = match username {
        Some(user) => json!({"user": {"username": user, "password": {"sha512": hash}}}),
        None => json!({ "pass": {"sha512": hash} }),
    };

    // prepare json data to merge
    let data = json!({"spec": {
    "credentials": {
        "credentials": [
          credential
        ]
    }
    }});

    set(config, app, device_id, data)
}

pub fn add_alias(
    config: &Context,
    app: AppId,
    device_id: DeviceId,
    new_alias: String,
) -> Result<()> {
    // prepare json data to merge
    let data = json!({"spec": {
    "alias": [
          new_alias
        ]
    }});

    set(config, app, device_id, data)
}

// The "set" operation merges the data with what already exists on the server side
fn set(config: &Context, app: AppId, device_id: DeviceId, data: Value) -> Result<()> {
    //read device data
    let res = get(config, &app, &device_id);
    match res {
        Ok(r) => match r.status() {
            StatusCode::OK => {
                let mut body: Value =
                    serde_json::from_str(r.text().unwrap_or_else(|_| "{}".to_string()).as_str())?;
                body.merge(data);
                put(config, &app, &device_id, body)
                    .map(|p| util::print_result(p, format!("Device {}", device_id), Verbs::edit))
            }
            e => {
                log::error!("Error : could not retrieve device: {}", e);
                util::exit_with_code(e)
            }
        },
        Err(e) => {
            log::error!("Error : could not execute request: {}", e);
            exit(2)
        }
    }
}

fn get(config: &Context, app: &str, device_id: &DeviceId) -> Result<Response> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, app, Some(device_id));

    client
        .get(&url)
        .auth(&config.token)
        .send()
        .context("Can't get device.")
}

fn put(
    config: &Context,
    app: &AppId,
    device_id: &DeviceId,
    data: serde_json::Value,
) -> Result<Response> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, app, Some(device_id));

    client
        .put(&url)
        .auth(&config.token)
        .json(&data)
        .send()
        .context(format!(
            "Error while updating device data for {}",
            device_id
        ))
}

// todo drogue-client and the types would be useful for this
fn pretty_list(data: String) -> Result<()> {
    let device_array: Vec<Value> = from_str(data.as_str())?;

    let mut table = Table::new("{:<} {:<}");
    table.add_row(Row::new().with_cell("NAME").with_cell("AGE"));

    for dev in device_array {
        let name = dev["metadata"]["name"].as_str();
        let creation = dev["metadata"]["creationTimestamp"].as_str();
        if let Some(name) = name {
            table.add_row(
                Row::new()
                    .with_cell(name)
                    .with_cell(util::age(creation.unwrap())?),
            );
        }
    }

    print!("{}", table);
    Ok(())
}
