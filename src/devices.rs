use crate::{AppId, Verbs, DeviceId, util};
use reqwest::blocking::Client;
use reqwest::{StatusCode, Url};
use serde_json::json;
use reqwest::blocking::Response;

const API_BASE: &str = "api/v1/apps";

pub fn delete(url: &Url, app: &AppId, device_id: &DeviceId) {
    let client = Client::new();
    let url = format!("{}{}/{}/devices/{}", url, API_BASE, app, device_id);

    let res = client.delete(&url).send();
    util::print_result(res, format!("Device {}", device_id), Verbs::delete)
}

pub fn read(url: &Url, app: &AppId, device_id: &DeviceId) {
    let res = get(url, app, device_id);
    util::print_result(res, device_id.to_string(), Verbs::get)
}

pub fn create(url: &Url, id: &DeviceId, data: serde_json::Value, app_id: &AppId) {
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
        .send();

    util::print_result(res, format!("Device {}", id), Verbs::create)
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
                    util::print_result(put(url, app, device_id, insert), format!("Device {}", device_id), Verbs::edit)
                },
                e => println!("Error : could not retrieve device: {}", e)
            }
        }, Err(e) => println!("Error : could not retrieve device: {}", e)
    }
}

fn get(url: &Url, app: &AppId, device_id: &DeviceId) -> Result<Response, reqwest::Error> {
    let client = Client::new();
    let url = format!("{}{}/{}/devices/{}", url, API_BASE, app, device_id);

    client.get(&url).send()
}

fn put(url: &Url, app: &AppId, device_id: &DeviceId, data: serde_json::Value) -> Result<Response, reqwest::Error> {
    let client = Client::new();
    let url = format!("{}{}/{}/devices/{}", url, API_BASE, app, device_id);

    client.put(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(data.to_string())
        .send()
}