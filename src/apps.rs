use crate::{AppId, Verbs, util};
use reqwest::blocking::{Client, Response};
use reqwest::{StatusCode, Url};
use serde_json::json;
use anyhow::{Context, Result};

pub fn create(url: &Url, app: &AppId, data: serde_json::Value) -> Result<()> {
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
        .send().with_context(|| format!("Can't create app "))?;

    util::print_result(res, format!("App {}", app), Verbs::create);
    Ok(())
}

pub fn read(url: &Url, app: &AppId) -> Result<()> {
    let res = get(url, app)?;
    util::print_result(res, app.to_string(), Verbs::get);
    Ok(())
}

pub fn delete(url: &Url, app: &AppId) -> Result<()> {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}", url, app);

    let res = client.delete(&url).send().with_context(|| format!("Can't get app "))?;
    util::print_result(res, format!("App {}", app), Verbs::delete);
    Ok(())
}

pub fn edit(url: &Url, app: &AppId) {
    //read app data
    let res = get(url, app);
    match res {
        Ok(r) => {
            match r.status() {
                StatusCode::OK => {
                    let body = r.text().unwrap_or("{}".to_string());
                    let insert = util::editor(body).unwrap();
                    util::print_result(put(url, app, insert).unwrap(), format!("App {}", app), Verbs::edit)
                },
                e => println!("Error : could not retrieve app: {}", e)
            }
        }, Err(e) => println!("Error : could not retrieve app: {}", e)
    }
}


fn get(url: &Url, app: &AppId) -> Result<Response> {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}", url, app);
    client.get(&url).send().with_context(|| format!("Can't get app "))
}

fn put(url: &Url, app: &AppId, data: serde_json::Value) -> Result<Response, reqwest::Error> {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}", url, app);

    client.put(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(data.to_string())
        .send()
}