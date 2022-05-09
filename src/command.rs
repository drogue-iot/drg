use crate::config::Context;
use crate::util::DrogueError;
use crate::util::Outcome;

use drogue_client::command::v1::Client;
use serde_json::Value;

pub async fn send_command(
    config: &Context,
    app: &str,
    device: &str,
    command: &str,
    body: Value,
) -> Result<Outcome<String>, DrogueError> {
    let client = Client::new(
        reqwest::Client::new(),
        config.registry_url.clone(),
        config.token.clone(),
    );

    Ok(client
        .publish_command(app, device, command, Some(body))
        .await
        .map(|_| Outcome::SuccessWithMessage("Command accepted".to_string()))?)
}
