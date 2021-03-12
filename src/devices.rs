use crate::{Url, AppId, Verbs, DeviceId, util};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde_json::json;
use reqwest::blocking::Response;
use anyhow::{Result, Context};

const API_BASE: &str = "api/v1/apps";

pub fn delete(url: &Url, app: &AppId, device_id: &DeviceId) -> Result<()> {
    let client = Client::new();
    let url = format!("{}{}/{}/devices/{}", url, API_BASE, app, device_id);

    let res = client.delete(&url).send().with_context(|| format!("Can't delete device "))?;
    util::print_result(res, format!("Device {}", device_id), Verbs::delete);
    Ok(())
}

pub fn read(url: &Url, app: &AppId, device_id: &DeviceId) -> Result<()> {
    let res = get(url, app, device_id)?;
    util::print_result(res, device_id.to_string(), Verbs::get);
    Ok(())
}

pub fn create(url: &Url, id: &DeviceId, data: serde_json::Value, app_id: &AppId) -> Result<()> {
    let client = Client::new();
    let url = format!("{}{}/{}/devices", url, API_BASE, app_id);
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

pub fn edit(url: &Url, app: &AppId, device_id: &DeviceId) {
    //read device data
    let res = get(url, app, device_id);
    match res {
        Ok(r) => {
            match r.status() {
                StatusCode::OK => {
                    let body = r.text().unwrap_or("{}".to_string());
                    let insert = util::editor(body).unwrap();
                    util::print_result(put(url, app, device_id, insert).unwrap(), format!("Device {}", device_id), Verbs::edit)
                },
                e => println!("Error : could not retrieve device: {}", e)
            }
        }, Err(e) => println!("Error : could not retrieve device: {}", e)
    }
}

fn get(url: &Url, app: &AppId, device_id: &DeviceId) -> Result<Response> {
    let client = Client::new();
    let url = format!("{}{}/{}/devices/{}", url, API_BASE, app, device_id);

    client.get(&url).send().with_context(|| format!("Can't get device "))   
}

fn put(url: &Url, app: &AppId, device_id: &DeviceId, data: serde_json::Value) -> Result<Response, reqwest::Error> {
    let client = Client::new();
    let url = format!("{}{}/{}/devices/{}", url, API_BASE, app, device_id);

    client.put(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(data.to_string())
        .send()
}