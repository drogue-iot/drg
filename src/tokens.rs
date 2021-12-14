use std::process::exit;

use crate::config::{Context, RequestBuilderExt};
use crate::util;

use anyhow::{anyhow, Result};
use reqwest::{
    blocking::{Client, Response},
    StatusCode,
};
use serde_json::Value;
use tabular::{Row, Table};

fn craft_url(config: &Context, prefix: Option<&str>) -> String {
    let prefix = match prefix {
        Some(prefix) => format!("/{}", urlencoding::encode(prefix)),
        None => String::new(),
    };

    format!("{}{}{}", &config.registry_url, util::KEYS_API_PATH, prefix)
}

pub fn get_api_keys(config: &Context) -> Result<()> {
    let client = Client::new();
    let url = craft_url(config, None);

    let res = client
        .get(&url)
        .auth(&config.token)
        .send()
        .map_err(|e| {
            log::error!("Error: {}", e);
            exit(2);
        })
        .map(|res: Response| match res.status() {
            StatusCode::OK => res,
            e => {
                log::error!("{}", e);
                util::exit_with_code(e);
            }
        });

    match res {
        Ok(res) => {
            let body: Vec<Value> =
                serde_json::from_str(&res.text().unwrap_or_else(|_| "{}".to_string()))?;

            let mut table = Table::new("{:<} | {:<}");
            table.add_row(Row::new().with_cell("TOKEN PREFIX").with_cell("AGE"));

            for token in body {
                let prefix = token["prefix"].as_str();
                let creation = token["created"].as_str();
                if let Some(prefix) = prefix {
                    table.add_row(
                        Row::new()
                            .with_cell(prefix)
                            .with_cell(util::age(creation.unwrap())?),
                    );
                }
            }
            print!("{}", table);
            Ok(())
        }
        Err(_) => Err(anyhow!("Error while requesting access tokens.")),
    }
}

pub fn create_api_key(config: &Context) -> Result<()> {
    let client = Client::new();
    let url = craft_url(config, None);

    let res = client
        .post(&url)
        .auth(&config.token)
        .send()
        .map_err(|e| {
            log::error!("Error: {}", e);
            exit(2);
        })
        .map(|res: Response| match res.status() {
            StatusCode::OK => res,
            e => {
                log::error!("{}", e);
                util::exit_with_code(e);
            }
        });
    match res {
        Ok(res) => {
            let body: Value =
                serde_json::from_str(&res.text().unwrap_or_else(|_| "{}".to_string()))?;
            let key = body["token"].as_str().unwrap();
            println!("A new Access Token was created:\n");
            println!("{}", key);
            println!("Make sure you save it, as you will not be able to display it again.");
            Ok(())
        }
        Err(_) => Err(anyhow!("Error creating a token")),
    }
}

pub fn delete_api_key(config: &Context, prefix: &str) -> Result<()> {
    let client = Client::new();
    let url = craft_url(config, Some(prefix));

    client
        .delete(&url)
        .auth(&config.token)
        .send()
        .map_err(|e| {
            log::error!("Error: {}", e);
            exit(2);
        })
        .map(|res: Response| match res.status() {
            StatusCode::NO_CONTENT => {
                println!("Access token with prefix {} deleted", prefix);
            }
            e => {
                log::error!("{}", e);
                util::exit_with_code(e);
            }
        })
}
