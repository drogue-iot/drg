use anyhow::{anyhow, Context as AnyhowContext, Result};
use tungstenite::connect;
use tungstenite::http::Request;

use crate::config::{Context, RequestBuilderExt};
use crate::util;

pub fn stream_app(config: &Context, app: &str, mut count: usize) -> Result<()> {
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
                        util::show_json(m.into_text().expect("Invalid message"));
                    }
                }
            }
            Err(e) => return Err(anyhow!(e)),
        }
        //bail!("Websocket Error")
    }
    Ok(())
}
