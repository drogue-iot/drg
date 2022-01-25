use anyhow::{anyhow, Context as AnyhowContext, Result};
use colored_json::write_colored_json;
use serde_json::Value;
use std::io::stdout;
use tungstenite::connect;
use tungstenite::http::Request;

use crate::config::{Context, RequestBuilderExt};
use crate::util;

pub fn stream_app(
    config: &Context,
    app: &str,
    device: Option<&str>,
    mut count: usize,
) -> Result<()> {
    let url = util::get_drogue_websocket_endpoint(config)?;
    let url = format!("{}{}", url, urlencoding::encode(app));

    let request = Request::builder().uri(url).auth(&config.token).body(())?;

    log::debug!("Connecting to websocket with request : {:?}", request);
    let (mut socket, response) =
        connect(request).context("Error connecting to the Websocket endpoint:")?;
    log::debug!("HTTP response: {}", response.status());

    while count > 0 {
        let msg = socket.read_message();
        match msg {
            Ok(m) => {
                log::debug!("Message: {:?}", m);
                if m.is_binary() || m.is_text() {
                    count -= 1;
                    // ignore protocol messages, only show text
                    if m.is_text() {
                        let message = m.into_text().expect("Invalid message");
                        filter_device(message, device);
                    }
                }
            }
            Err(e) => return Err(anyhow!(e)),
        }
        //bail!("Websocket Error")
    }
    Ok(())
}

fn filter_device<S: Into<String>>(payload: S, device: Option<&str>) {
    let payload = payload.into();
    match serde_json::from_str(&payload) {
        // show as JSON
        Ok(json) => {
            if let Some(device) = device {
                let json: Value = json;
                let sender: &str = json["sender"].as_str().unwrap_or_default();
                if sender == device {
                    write_colored_json(&json, &mut stdout().lock()).ok();
                    println!();
                }
            } else {
                write_colored_json(&json, &mut stdout().lock()).ok();
                println!();
            }
        }
        // fall back to plain text output
        Err(_) => println!("{}", payload),
    }
}
