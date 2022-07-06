use anyhow::{anyhow, Context as AnyhowContext, Result};
use colored_json::write_colored_json;
use native_tls::TlsConnector;
use oauth2::TokenResponse;
use serde_json::Value;
use std::io::stdout;
use std::net::TcpStream;
use tungstenite::http::Request;
use tungstenite::{connect, Message};

use crate::config::{Context, RequestBuilderExt, Token};
use crate::{openid, util};
use drogue_client::integration::ws::v1::client::Message as Drogue_ws_message;
use tungstenite::client::IntoClientRequest;

pub async fn stream_app(
    config: &mut Context,
    app: &str,
    device: Option<&str>,
    mut count: usize,
    insecure: bool,
) -> Result<()> {
    let ws_endpoint = util::get_drogue_websocket_endpoint(config).await?;
    let url = format!("{}{}", ws_endpoint, urlencoding::encode(app));

    let mut request: Request<()> = url.into_client_request()?;
    request = request.auth(&config.token);

    log::debug!("Connecting to websocket with request : {:?}", &request);
    let (mut socket, response) = if insecure {
        log::warn!("Skipping certificate verification");
        let (connector, stream) = insecure_stream(&ws_endpoint.socket_addrs(|| None).unwrap())?;
        tungstenite::client_tls_with_config(request, stream, None, Some(connector))
            .context("Error connecting to the Websocket endpoint:")?
    } else {
        connect(request).context("Error connecting to the Websocket endpoint:")?
    };

    log::debug!("HTTP response: {}", response.status());

    while count > 0 {
        let msg = socket.read_message();
        log::debug!("Message: {:?}", msg);
        match msg {
            Ok(Message::Text(message)) => {
                count -= 1;
                filter_device(message, device);
            }
            Ok(Message::Binary(_)) => {
                count -= 1;
            }
            Ok(Message::Close(Some(cause))) => {
                // just log the message
                log::warn!(
                    "Connection closed by server (code: {}, reason: {})",
                    cause.code,
                    cause.reason
                );
                break;
            }
            Ok(_) => {
                // ignore other protocol messages, only show text and handle close
            }
            Err(e) => return Err(anyhow!(e)),
        }

        if let Some(token) = refresh_token(config).await {
            log::debug!("sending a refreshed token");
            socket.write_message(Message::Text(serde_json::to_string(
                &Drogue_ws_message::RefreshAccessToken(token),
            )?))?
        }
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

async fn refresh_token(config: &mut Context) -> Option<String> {
    match openid::verify_token_validity(config).await {
        Ok(false) => None,
        Ok(true) => match &config.token {
            Token::TokenResponse(token) => Some(token.access_token().secret().clone()),
            Token::AccessToken(_) => None,
        },
        Err(e) => {
            log::error!("Error refreshing token - {e}");
            None
        }
    }
}

fn insecure_stream(
    address: &Vec<std::net::SocketAddr>,
) -> Result<(tungstenite::Connector, TcpStream)> {
    let connector = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let stream = TcpStream::connect(address.as_slice())?;
    let connector: tungstenite::Connector = tungstenite::Connector::NativeTls(connector);

    Ok((connector, stream))
}
