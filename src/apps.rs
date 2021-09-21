use crate::config::Context;
use crate::{trust, util, AppId, Verbs};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use oauth2::TokenResponse;
use reqwest::blocking::{Client, Response};
use reqwest::{StatusCode, Url};
use serde_json::{from_str, json, Value};
use std::process::exit;
use tabular::{Row, Table};

fn craft_url(base: &Url, app_id: Option<&str>) -> String {
    let app = match app_id {
        Some(app) => format!("/{}", app),
        None => String::new(),
    };
    format!("{}{}/apps{}", base, util::REGISTRY_API_PATH, app)
}

pub fn create(
    config: &Context,
    app: AppId,
    data: serde_json::Value,
    file: Option<&str>,
) -> Result<()> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, None);
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

pub fn delete(config: &Context, app: AppId, ignore_missing: bool) -> Result<()> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, Some(&app));

    client
        .delete(&url)
        .bearer_auth(&config.token.access_token().secret())
        .send()
        .context("Can't get app.")
        .map(|res| {
            if ignore_missing && res.status() == StatusCode::NOT_FOUND {
                exit(0);
            } else {
                util::print_result(res, format!("App {}", &app), Verbs::delete)
            }
        })
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
                        let body = r.text().unwrap_or_else(|_| "{}".to_string());
                        let insert = util::editor(body)?;

                        put(config, &app, insert)
                            .map(|p| util::print_result(p, format!("App {}", &app), Verbs::edit))
                    }
                    e => {
                        log::error!("Error : could not retrieve app: {}", e);
                        util::exit_with_code(e)
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

pub fn list(config: &Context, labels: Option<String>) -> Result<()> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, None);

    let mut req = client
        .get(&url)
        .bearer_auth(&config.token.access_token().secret());

    if let Some(labels) = labels {
        req = req.query(&[("labels", labels)]);
    }

    let res = req.send().context("Can't list apps");

    if let Ok(r) = res {
        match r.status() {
            StatusCode::OK => {
                pretty_list(r.text()?)?;
                Ok(())
            }
            e => {
                log::error!("List operation failed with {}", r.status());
                util::exit_with_code(e)
            }
        }
    } else {
        Err(anyhow!("Error while requesting app list."))
    }
}

fn get(config: &Context, app: &str) -> Result<Response> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, Some(app));
    client
        .get(&url)
        .bearer_auth(&config.token.access_token().secret())
        .send()
        .context("Can't retrieve app data.")
}

pub fn add_trust_anchor(
    config: &Context,
    app: &str,
    keyout: Option<&str>,
    key_pair_algorithm: Option<trust::SignAlgo>,
    days: Option<&str>,
    key_input: Option<rcgen::KeyPair>,
) -> Result<()> {
    let res = get(config, &app);
    match res {
        Ok(r) => match r.status() {
            StatusCode::OK => {
                let app_obj = r.text().unwrap_or_else(|_| "{}".to_string());
                let mut app_obj: Value = serde_json::from_str(&app_obj)?;
                app_obj["spec"]["trustAnchors"] =
                    trust::create_trust_anchor(app, keyout, key_pair_algorithm, days, key_input)?;

                put(config, app, app_obj)
                    .map(|p| util::print_result(p, format!("App {}", &app), Verbs::edit))
            }
            e => {
                log::error!("Error : could not retrieve app: {}", e);
                util::exit_with_code(e)
            }
        },
        Err(e) => {
            log::error!("Error : could not retrieve app: {}", e);
            exit(2);
        }
    }
}

pub fn get_trust_anchor(config: &Context, app: &str) -> Result<String> {
    let res = get(config, &app);
    match res {
        Ok(r) => match r.status() {
            StatusCode::OK => {
                let app_obj = r.text().unwrap_or_else(|_| "{}".to_string());
                let app_obj_json: Value = serde_json::from_str(&app_obj)?;
                let cert =
                    app_obj_json["spec"]["trustAnchors"]["anchors"][0]["certificate"].clone();

                if cert == Value::Null {
                    log::error!("No trust anchor found in this application.");
                    exit(1);
                }

                Ok(cert.to_string().replace("\"", ""))
            }
            e => {
                log::error!("Error : could not retrieve app: {}", e);
                util::exit_with_code(e)
            }
        },
        Err(e) => {
            log::error!("Error : could not retrieve app: {}", e);
            exit(2);
        }
    }
}

fn put(config: &Context, app: &str, data: serde_json::Value) -> Result<Response> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, Some(app));

    client
        .put(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .bearer_auth(&config.token.access_token().secret())
        .body(data.to_string())
        .send()
        .context("Can't update app data.")
}

// todo drogue-client and the types would be useful for this
fn pretty_list(data: String) -> Result<()> {
    let apps_array: Vec<Value> = from_str(data.as_str())?;

    let mut table = Table::new("{:<} {:<}");
    table.add_row(Row::new().with_cell("NAME").with_cell("AGE"));

    for app in apps_array {
        let name = app["metadata"]["name"].as_str();
        let creation = app["metadata"]["creationTimestamp"].as_str();
        if let Some(name) = name {
            table.add_row(
                Row::new()
                    .with_cell(name)
                    .with_cell(util::age(creation.unwrap())?),
            );
        }
    }

    print!("{}", table);
    Ok(())
}
