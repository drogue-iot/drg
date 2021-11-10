use crate::config::Context;
use crate::util;

use anyhow::{Context as anyhowContext, Result};
use oauth2::TokenResponse;
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde_json::Value;
use urlencoding;

pub fn send_command(
    config: &Context,
    app: &str,
    device: &str,
    command: &str,
    body: Value,
) -> Result<()> {
    let client = Client::new();

    let url = format!(
        "{}{}/apps/{}/devices/{}",
        &config.registry_url,
        util::COMMAND_API_PATH,
        urlencoding::encode(app),
        urlencoding::encode(device)
    );

    client
        .post(&url)
        .bearer_auth(&config.token.access_token().secret())
        .query(&[("command", command)])
        .json(&body)
        .send()
        .context("Can't send command.")
        .map(|res| match res.status() {
            StatusCode::ACCEPTED => println!("Command {} accepted", command),
            r => util::exit_with_code(r),
        })
}
