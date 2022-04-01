use crate::config::{Context, RequestBuilderExt};
use crate::util;

use anyhow::{Context as anyhowContext, Result};
use reqwest::Client;
use reqwest::StatusCode;
use serde_json::Value;

pub async fn send_command(
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
        .auth(&config.token)
        .query(&[("command", command)])
        .json(&body)
        .send()
        .await
        .context("Can't send command.")
        .map(|res| match res.status() {
            StatusCode::ACCEPTED => println!("Command {} accepted", command),
            r => util::exit_with_code(r),
        })
}
