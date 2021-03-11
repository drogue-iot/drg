use crate::{Url, AppId, Verbs, util};
use reqwest::blocking::Client;
use serde_json::json;
use anyhow::{Context, Result};

pub fn create(url: &Url, app: &AppId, data: serde_json::Value) -> Result<()>{
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

    // Later replace with custom error type ( to help write test cases )
    util::print_result(res, format!("App {}", app), Verbs::create);
    Ok(())
}

pub fn read(url: &Url, app: &AppId) -> Result<()> {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}", url, app);

    let res = client.get(&url).send()
                    .with_context(|| format!("Can't get app "))?;
    util::print_result(res, app.to_string(), Verbs::get);
    Ok(())
}

pub fn delete(url: &Url, app: &AppId) -> Result<()> {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}", url, app);

    let res = client.delete(&url).send().with_context(|| format!("Can't delete app "))?;
    util::print_result(res, format!("App {}", app), Verbs::delete);
    Ok(())
}

