use crate::config::Context;
use crate::{util, AppId, DeviceId, Verbs};
use anyhow::{Context as AnyhowContext, Result};
use oauth2::TokenResponse;
use reqwest::blocking::Client;
use reqwest::blocking::Response;
use reqwest::{StatusCode, Url};
use serde_json::json;
use std::process::exit;

fn craft_url(base: &Url, app_id: &str, device_id: &str) -> String {
    format!("{}api/v1/apps/{}/devices/{}", base, app_id, device_id)
}

pub fn delete(config: &Context, app: AppId, device_id: DeviceId) -> Result<()> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, &app, &device_id);

    client
        .delete(&url)
        .bearer_auth(&config.token.access_token().secret())
        .send()
        .context("Can't delete device.")
        .map(|res| util::print_result(res, format!("Device {}", device_id), Verbs::delete))
}

pub fn read(config: &Context, app: AppId, device_id: DeviceId) -> Result<()> {
    get(&config, &app, &device_id)
        .map(|res| util::print_result(res, device_id.to_string(), Verbs::get))
}

pub fn create(
    config: &Context,
    device_id: DeviceId,
    data: serde_json::Value,
    app_id: AppId,
    file: Option<&str>,
) -> Result<()> {
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
    let url = format!("{}api/v1/apps/{}/devices", &config.registry_url, app_id);

    client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .bearer_auth(&config.token.access_token().secret())
        .body(body.to_string())
        .send()
        .context("Can't create device.")
        .map(|res| util::print_result(res, format!("Device {}", device_id), Verbs::create))
}

pub fn edit(config: &Context, app: AppId, device_id: DeviceId, file: Option<&str>) -> Result<()> {
    match file {
        Some(f) => {
            let data = util::get_data_from_file(f)?;

            put(&config, &app, &device_id, data)
                .map(|res| util::print_result(res, format!("Device {}", device_id), Verbs::edit))
        }
        None => {
            //read device data
            let res = get(&config, &app, &device_id);
            match res {
                Ok(r) => match r.status() {
                    StatusCode::OK => {
                        let body = r.text().unwrap_or_else(|_| "{}".to_string());
                        let insert = util::editor(body)?;
                        put(&config, &app, &device_id, insert).map(|p| {
                            util::print_result(p, format!("Device {}", device_id), Verbs::edit)
                        })
                    }
                    e => {
                        log::error!("Error : could not retrieve device: {}", e);
                        exit(2);
                    }
                },
                Err(e) => {
                    log::error!("Error : could not retrieve device: {}", e);
                    exit(2)
                }
            }
        }
    }
}

fn get(config: &Context, app: &str, device_id: &str) -> Result<Response> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, app, device_id);

    client
        .get(&url)
        .bearer_auth(&config.token.access_token().secret())
        .send()
        .context("Can't get device.")
}

fn put(config: &Context, app: &str, device_id: &str, data: serde_json::Value) -> Result<Response> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, app, device_id);
    let token = &config.token.access_token().secret();

    client
        .put(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .bearer_auth(token)
        .body(data.to_string())
        .send()
        .context(format!(
            "Error while updating device data for {}",
            device_id
        ))
}
