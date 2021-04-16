use crate::config::Context;
use crate::{util, AppId, Verbs};
use anyhow::{Context as AnyhowContext, Result};
use oauth2::TokenResponse;
use reqwest::blocking::{Client, Response};
use reqwest::{StatusCode, Url};
use serde_json::json;
use std::process::exit;

fn craft_url(base: &Url, app_id: &AppId) -> String {
    format!("{}api/v1/apps/{}", base, app_id)
}

pub fn create(config: &Context, app: AppId, data: serde_json::Value, file: Option<&str>) -> Result<()> {
    let client = Client::new();
    let url = format!("{}api/v1/apps", &config.registry_url);
    let body = match file {
        Some(f) => util::get_data_from_file(f)?,
        None => {
            json!({
        "metadata": {
            "name": app,
        },
        "spec": data,
        })
        }
    };

    client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body.to_string())
        .bearer_auth(&config.token.access_token().secret())
        .send()
        .context("Can't create app.")
        .map(|res| util::print_result(res, format!("App {}", app), Verbs::create))
}

pub fn read(config: &Context, app: AppId) -> Result<()> {
    get(config, &app).map(|res| util::print_result(res, app.to_string(), Verbs::get))
}

pub fn delete(config: &Context, app: AppId) -> Result<()> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, &app);

    client
        .delete(&url)
        .bearer_auth(&config.token.access_token().secret())
        .send()
        .context("Can't get app.")
        .map(|res| util::print_result(res, format!("App {}", &app), Verbs::delete))
}

pub fn edit(config: &Context, app: AppId, file: Option<&str>) -> Result<()> {
    match file {
        Some(f) => {
            let data = util::get_data_from_file(f)?;

            put(&config, &app, data)
                .map(|res| util::print_result(res, format!("App {}", &app), Verbs::edit))
        }
        None => {
            //read app data
            let res = get(config, &app);
            match res {
                Ok(r) => match r.status() {
                    StatusCode::OK => {
                        let body = r.text().unwrap_or("{}".to_string());
                        let insert = util::editor(body)?;

                        put(config, &app, insert)
                            .map(|p| util::print_result(p, format!("App {}", &app), Verbs::edit))
                    }
                    e => {
                        log::error!("Error : could not retrieve app: {}", e);
                        exit(2);
                    }
                },
                Err(e) => {
                    log::error!("Error : could not retrieve app: {}", e);
                    exit(2);
                }
            }
        }
    }
}

fn get(config: &Context, app: &AppId) -> Result<Response> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, app);
    client
        .get(&url)
        .bearer_auth(&config.token.access_token().secret())
        .send()
        .context("Can't retrieve app data.")
}

fn put(config: &Context, app: &AppId, data: serde_json::Value) -> Result<Response> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, app);

    client
        .put(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .bearer_auth(&config.token.access_token().secret())
        .body(data.to_string())
        .send()
        .context("Can't update app data.")
}
