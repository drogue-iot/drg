use crate::{Url, AppId, Verbs, DeviceId, util};
use reqwest::blocking::Client;
use serde_json::json;
use anyhow::{Context, Result};

pub fn delete(url: &Url, app: &AppId, device_id: &DeviceId) -> Result<()> {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}/devices/{}", url, app, device_id);

    let res = client.delete(&url).send().with_context(|| format!("Can't delete device "))?;
    util::print_result(res, format!("Device {}", device_id), Verbs::delete);
    Ok(())
}

pub fn read(url: &Url, app: &AppId, device_id: &DeviceId) -> Result<()> {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}/devices/{}", url, app, device_id);

    let res = client.get(&url).send().with_context(|| format!("Can't get device "))?;
    util::print_result(res, device_id.to_string(), Verbs::get);
    Ok(())
}

pub fn create(url: &Url, id: &DeviceId, data: serde_json::Value, app_id: &AppId) -> Result<()> {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}/devices", url, app_id);
    println!("{}", url);
    let body = json!({
        "metadata": {
            "name": id,
            "application": app_id
        },
        "spec": data
    });
    let res = client.post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body.to_string())
        .send().with_context(|| format!("Can't create device "))?;
    util::print_result(res, format!("Device {}", id), Verbs::create);
    Ok(())
}