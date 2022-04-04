use crate::config::Context;

use anyhow::Result;
use serde_json::Value;

use drogue_client::command::v1::Client;

pub async fn send_command(
    config: &Context,
    app: &str,
    device: &str,
    command: &str,
    body: Value,
) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    match client
        .publish_command(app, device, command, Some(body))
        .await
    {
        Ok(_) => {
            println!("Command accepted");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
