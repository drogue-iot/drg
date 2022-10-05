use crate::config::Context;
use crate::handle_operation;
use crate::util::{self, DrogueError, Outcome};

use drogue_client::admin::v1::Client;
use serde::Serialize;
use url::Url;

pub async fn transfer_app(
    config: &Context,
    app: &str,
    user: &str,
) -> Result<Outcome<AppTransfer>, DrogueError> {
    let client = Client::new(
        reqwest::Client::new(),
        config.registry_url.clone(),
        config.token.clone(),
    );

    match client.initiate_app_transfer(app, user).await {
        Ok(true) => {
            let console = util::get_drogue_console_endpoint(config).await.ok();
            Ok(Outcome::SuccessWithJsonData(AppTransfer {
                console,
                app: app.to_string(),
            }))
        }
        Ok(false) => Err(DrogueError::NotFound),
        Err(e) => Err(e.into()),
    }
}

pub async fn cancel_transfer(config: &Context, app: &str) -> Result<Outcome<String>, DrogueError> {
    let client = Client::new(
        reqwest::Client::new(),
        config.registry_url.clone(),
        config.token.clone(),
    );

    handle_operation!(
        client.cancel_app_transfer(app).await,
        "Application transfer canceled"
    )
}

pub async fn accept_transfer(config: &Context, app: &str) -> Result<Outcome<String>, DrogueError> {
    let client = Client::new(
        reqwest::Client::new(),
        config.registry_url.clone(),
        config.token.clone(),
    );

    handle_operation!(
        client.accept_app_transfer(app).await,
        "Application transfer completed. \n You are now the owner of the application"
    )
}

#[derive(Serialize, Clone)]
pub struct AppTransfer {
    console: Option<Url>,
    app: String,
}

pub fn app_transfer_guide(transfer: &AppTransfer) {
    println!("Application transfer initated.");
    println!(
        "The new user can accept the transfer with \"drg admin transfer accept {}\"",
        transfer.app
    );

    if let Some(console) = &transfer.console {
        println!(
            "Alternatively you can share this link with the new owner :\n{}transfer/{}",
            console.as_str(),
            urlencoding::encode(&transfer.app)
        )
    }
}
