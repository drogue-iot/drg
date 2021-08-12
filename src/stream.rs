use anyhow::{anyhow, Context as AnyhowContext, Result};
use oauth2::TokenResponse;
use tungstenite::connect;
use tungstenite::http::{header, Request};

use crate::config::Context;
use crate::util;

pub fn stream_app(config: &Context, app: &str) -> Result<()> {
    let url = util::get_drogue_websocket_endpoint(config)?;
    let url = format!("{}{}", url, app);

    let bearer_header = format!("Bearer {}", &config.token.access_token().secret());

    let request = Request::builder()
        .uri(url)
        .header(header::AUTHORIZATION, bearer_header)
        .body(())?;

    log::debug!("Connecting to websocket with request : {:?}", request);
    let (mut socket, response) =
        connect(request).context("Error connecting to the Websocket endpoint:")?;
    log::debug!("HTTP response: {}", response.status());

    loop {
        let msg = socket.read_message();
        match msg {
            Ok(m) => {
                // ignore protocol messages, only show text
                if m.is_text() {
                    util::show_json(m.into_text().expect("Invalid message"));
                }
            }
            Err(e) => break Err(anyhow!(e)),
        }

        //bail!("Websocket Error")
    }
}
