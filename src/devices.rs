use crate::{Url, AppId, Verbs, DeviceId, util};
use reqwest::blocking::Client;
use serde_json::json;

pub fn delete(url: &Url, app: &AppId, device_id: &DeviceId) {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}/devices/{}", url, app, device_id);

    let res = client.delete(&url).send();
    util::print_result(res, format!("Device {}", device_id), Verbs::delete)
}

pub fn read(url: &Url, app: &AppId, device_id: &DeviceId) {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}/devices/{}", url, app, device_id);

    let res = client.get(&url).send();
    util::print_result(res, device_id.to_string(), Verbs::get)
}

pub fn create(url: &Url, id: &DeviceId, data: serde_json::Value, app_id: &AppId) {
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
        .send();

    util::print_result(res, format!("Device {}", id), Verbs::create)
}