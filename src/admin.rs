use std::process::exit;

use crate::config::Context;
use crate::util;

use anyhow::{Context as anyhowContext, Result};
use oauth2::TokenResponse;
use reqwest::{
    blocking::{Client, Response},
    StatusCode,
};
use serde_json::Value;
use strum_macros::{AsRefStr, EnumString};
use tabular::{Row, Table};

#[derive(AsRefStr, EnumString)]
#[allow(non_camel_case_types)]
pub enum Roles {
    admin,
    manager,
    reader,
}

fn craft_url(config: &Context, app: &str) -> String {
    format!(
        "{}{}/apps/{}/members",
        &config.registry_url,
        util::ADMIN_API_PATH,
        app
    )
}

fn member_get(config: &Context, app: &str) -> Result<Response> {
    let client = Client::new();
    let url = craft_url(config, app);

    client
        .get(&url)
        .bearer_auth(&config.token.access_token().secret())
        .send()
        .map_err(|e| {
            log::error!("Error: {}", e);
            exit(2);
        })
        .map(|res| match res.status() {
            StatusCode::OK => res,
            e => {
                log::error!("{}", e);
                util::exit_with_code(e);
            }
        })
}

pub fn member_list(config: &Context, app: &str) -> Result<()> {
    let res = member_get(config, app)?;
    let body: Value = serde_json::from_str(&res.text().unwrap_or_else(|_| "{}".to_string()))?;

    let mut table = Table::new("{:<} | {:<}");
    table.add_row(Row::new().with_cell("User").with_cell("Role"));

    match body["members"].as_object() {
        Some(members) => {
            for i in members.keys() {
                table.add_row(
                    Row::new()
                        .with_cell(i)
                        .with_cell(members[i]["role"].to_owned()),
                );
            }
            println!("{}", table);
        }
        None => {
            println!("No members found for this application.");
        }
    };

    Ok(())
}

fn member_put(config: &Context, app: &str, data: serde_json::Value) -> Result<Response> {
    let client = Client::new();
    let url = craft_url(config, app);

    client
        .put(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .bearer_auth(&config.token.access_token().secret())
        .body(data.to_string())
        .send()
        .context("Can't update member list")
}

pub fn member_edit(config: &Context, app: &str) -> Result<()> {
    let res = member_get(config, app)?;
    let body = res.text().unwrap_or_else(|_| "{}".to_string());
    let insert = util::editor(body)?;

    member_put(config, app, insert).map(|res| describe_response(res))
}

pub fn member_add(config: &Context, app: &str, username: &str, role: Roles) -> Result<()> {
    let res = member_get(config, app)?;
    let obj = res.text().unwrap_or_else(|_| "{}".to_string());

    let mut body: Value = serde_json::from_str(&obj)?;
    body["members"][username] = serde_json::json!({"role": role.as_ref()});

    member_put(config, app, body).map(|res| describe_response(res))
}

fn describe_response(res: Response) {
    match res.status() {
        StatusCode::NO_CONTENT => {
            println!("The member list was updated.");
        }
        StatusCode::BAD_REQUEST => {
            println!("Invalid format: {}", res.text().unwrap_or_default());
        }
        StatusCode::NOT_FOUND => {
            println!("Application not found.");
        }
        StatusCode::CONFLICT => {
            println!("Conflict: The resource may have been modified on the server since we retrievied it.");
        }
        _ => {
            println!("Error: Can't update member list.")
        }
    }
}
