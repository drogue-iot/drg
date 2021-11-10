use std::process::exit;

use crate::config::Context;
use crate::util;

use anyhow::{Context as anyhowContext, Result};
use oauth2::TokenResponse;
use reqwest::{
    blocking::{Client, Response},
    StatusCode,
};
use serde_json::{json, Value};
use strum_macros::{AsRefStr, EnumString};
use tabular::{Row, Table};

#[derive(AsRefStr, EnumString)]
#[allow(non_camel_case_types)]
pub enum Roles {
    admin,
    manager,
    reader,
}

#[derive(AsRefStr, EnumString)]
enum ApiOp {
    #[strum(to_string = "transfer-ownership")]
    TransferOwnership,
    #[strum(to_string = "accept-ownership")]
    AcceptOwnerShip,
    #[strum(to_string = "members")]
    Members,
}

fn craft_url(config: &Context, app: &str, end: &ApiOp) -> String {
    format!(
        "{}{}/apps/{}/{}",
        &config.registry_url,
        util::ADMIN_API_PATH,
        app,
        end.as_ref()
    )
}

fn member_get(config: &Context, app: &str) -> Result<Response> {
    let client = Client::new();
    let url = craft_url(config, app, &ApiOp::Members);

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
    let url = craft_url(config, app, &ApiOp::Members);

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

pub fn transfer_app(config: &Context, app: &str, username: &str) -> Result<()> {
    let client = Client::new();
    let url = craft_url(config, app, &ApiOp::TransferOwnership);

    let body = json!({ "newUser": username });

    client
        .put(&url)
        .bearer_auth(&config.token.access_token().secret())
        .json(&body)
        .send()
        .map_err(|e| {
            log::error!("Error: {}", e);
            exit(2);
        })
        .map(|res| match res.status() {
            StatusCode::ACCEPTED => {
                println!("Application transfer initated");
                println!(
                    "The new user can accept the transfer with \"drg admin transfer accept {}\"",
                    app
                );
                if let Ok(console) = util::get_drogue_console_endpoint(&config) {
                    println!("Alternatively you can share this link with the new owner :");
                    println!("{}transfer/{}", console.as_str(), app);
                }
            }
            e => {
                log::error!("{}", e);
                util::exit_with_code(e);
            }
        })
}

pub fn cancel_transfer(config: &Context, app: &str) -> Result<()> {
    let client = Client::new();
    let url = craft_url(config, app, &ApiOp::TransferOwnership);

    client
        .delete(&url)
        .bearer_auth(&config.token.access_token().secret())
        .send()
        .map_err(|e| {
            log::error!("Error: {}", e);
            exit(2);
        })
        .map(|res| match res.status() {
            StatusCode::NO_CONTENT => {
                println!("Application transfer canceled");
            }
            e => {
                log::error!("{}", e);
                util::exit_with_code(e);
            }
        })
}

pub fn accept_transfer(config: &Context, app: &str) -> Result<()> {
    let client = Client::new();
    let url = craft_url(config, app, &ApiOp::AcceptOwnerShip);

    client
        .put(&url)
        .bearer_auth(&config.token.access_token().secret())
        .send()
        .map_err(|e| {
            log::error!("Error: {}", e);
            exit(2);
        })
        .map(|res| match res.status() {
            StatusCode::NO_CONTENT => {
                println!("Application transfer completed.");
                println!("You are now the owner of application {}", app);
            }
            e => {
                log::error!("{}", e);
                util::exit_with_code(e);
            }
        })
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
