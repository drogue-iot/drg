use crate::{Url, AppId, Verbs, util};
use reqwest::blocking::Client;
use serde_json::json;

pub fn create(url: &Url, app: &AppId, data: serde_json::Value) {
    let client = Client::new();
    let url = format!("{}api/v1/apps", url);
    let body = json!({
        "metadata": {
            "name": app,
        },
        "spec": {
            "data": data,
        }
    });

    let res = client.post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body.to_string())
        .send();

    util::print_result(res, format!("App {}", app), Verbs::create)
}

pub fn read(url: &Url, app: &AppId) {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}", url, app);

    let res = client.get(&url).send();
    util::print_result(res, app.to_string(), Verbs::get)
}

pub fn delete(url: &Url, app: &AppId) {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}", url, app);

    let res = client.delete(&url).send();
    util::print_result(res, format!("App {}", app), Verbs::delete)
}

