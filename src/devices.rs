use crate::config::Config;
use crate::{util, AppId, DeviceId, Verbs};
use anyhow::{Context, Result};
use oauth2::TokenResponse;
use reqwest::blocking::Client;
use reqwest::blocking::Response;
use reqwest::{StatusCode, Url};
use serde_json::json;

fn craft_url(base: &Url, app_id: &AppId, device_id: &DeviceId) -> String {
    format!("{}api/v1/apps/{}/devices/{}", base, app_id, device_id)
}

pub fn delete(config: &Config, app: &AppId, device_id: &DeviceId) -> Result<()> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, app, device_id);

    let res = client
        .delete(&url)
        .bearer_auth(&config.token.access_token().secret())
        .send()
        .context("Can't delete device.")?;
    util::print_result(res, format!("Device {}", device_id), Verbs::delete);
    Ok(())
}

pub fn read(config: &Config, app: &AppId, device_id: &DeviceId) -> Result<()> {
    let res = get(&config, app, device_id)?;
    util::print_result(res, device_id.to_string(), Verbs::get);
    Ok(())
}

pub fn create(
    config: &Config,
    device_id: &DeviceId,
    data: serde_json::Value,
    app_id: &AppId,
) -> Result<()> {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}/devices", &config.registry_url, app_id);
    let body = json!({
        "metadata": {
            "name": device_id,
            "application": app_id
        },
        "spec": data
    });
    let res = client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .bearer_auth(&config.token.access_token().secret())
        .body(body.to_string())
        .send()
        .context("Can't create device.")?;

    util::print_result(res, format!("Device {}", device_id), Verbs::create);
    Ok(())
}

pub fn edit(config: &Config, app: &AppId, device_id: &DeviceId) -> Result<()> {
    //read device data
    let res = get(&config, app, device_id);
    match res {
        Ok(r) => match r.status() {
            StatusCode::OK => {
                let body = r.text().unwrap_or("{}".to_string());
                let insert = util::editor(body)?;
                util::print_result(
                    put(&config, app, device_id, insert).unwrap(),
                    format!("Device {}", device_id),
                    Verbs::edit,
                );
            }
            e => println!("Error : could not retrieve device: {}", e),
        },
        Err(e) => println!("Error : could not retrieve device: {}", e),
    }
    Ok(())
}

fn get(config: &Config, app: &AppId, device_id: &DeviceId) -> Result<Response> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, app, device_id);

    client
        .get(&url)
        .bearer_auth(&config.token.access_token().secret())
        .send()
        .context("Can't get device.")
}

fn put(
    config: &Config,
    app: &AppId,
    device_id: &DeviceId,
    data: serde_json::Value,
) -> Result<Response> {
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
