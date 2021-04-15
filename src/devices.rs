use crate::config::Config;
use crate::util::ExitReason;
use crate::{util, AppId, DeviceId, Verbs};
use anyhow::{Context as _, Result};
use drogue_client::error::ClientError;
use drogue_client::{registry, Context};
use oauth2::TokenResponse;
use reqwest::blocking::Client;
use reqwest::Url;
use serde_json::json;

fn craft_url(base: &Url, app_id: &AppId, device_id: &DeviceId) -> String {
    format!("{}api/v1/apps/{}/devices/{}", base, app_id, device_id)
}

pub fn delete(config: &Config, app: &AppId, device_id: &DeviceId) -> Result<()> {
    let client = Client::new();
    let url = craft_url(&config.registry_url, app, device_id);

    client
        .delete(&url)
        .bearer_auth(&config.token.access_token().secret())
        .send()
        .context("Can't delete device.")
        .map(|res| util::print_result(res, format!("Device {}", device_id), Verbs::delete))
}

pub async fn read(config: &Config, app: &AppId, device_id: &DeviceId) -> Result<()> {
    Ok(util::print_resource(
        get(&config, app, device_id).await,
        "device",
        device_id,
        Some(app),
    )?)
}

pub fn create(
    config: &Config,
    device_id: &DeviceId,
    data: serde_json::Value,
    app_id: &AppId,
) -> Result<()> {
    let client = Client::new();
    let url = format!("{}api/v1/apps/{}/devices", &config.registry_url, app_id);
    let body = json!({
        "metadata": {
            "name": device_id,
            "application": app_id
        },
        "spec": data
    });

    client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .bearer_auth(&config.token.access_token().secret())
        .body(body.to_string())
        .send()
        .context("Can't create device.")
        .map(|res| util::print_result(res, format!("Device {}", device_id), Verbs::create))
}

pub async fn edit(
    config: &Config,
    app: &AppId,
    device_id: &DeviceId,
    file: Option<&str>,
) -> Result<()> {
    match file {
        Some(f) => {
            let data = util::get_data_from_file(f)?;

            let res = put(&config, app, device_id, data).await?;
            util::print_result_async(res, format!("Device {}", device_id), Verbs::edit).await;
            Ok(())
        }
        None => {
            //read device data
            let res = get(&config, app, device_id).await;
            match res {
                Ok(Some(device)) => {
                    let insert = util::editor(device)?;
                    let res = put(&config, app, device_id, insert).await?;
                    util::print_result_async(res, format!("Device {}", device_id), Verbs::edit)
                        .await;
                    Ok(())
                }
                Ok(None) => {
                    log::error!("Device not found");
                    util::exit_with(ExitReason::NotFound);
                }
                Err(e) => {
                    util::exit_with_err(e);
                }
            }
        }
    }
}

async fn get(
    config: &Config,
    app: &AppId,
    device_id: &DeviceId,
) -> std::result::Result<Option<registry::v1::Device>, ClientError<reqwest::Error>> {
    let client =
        registry::v1::Client::new(reqwest::Client::new(), config.registry_url.clone(), None);

    client
        .get_device(
            app,
            device_id,
            Context {
                provided_token: Some(config.token.access_token().secret().clone()),
            },
        )
        .await
}

async fn put(
    config: &Config,
    app: &AppId,
    device_id: &DeviceId,
    data: serde_json::Value,
) -> Result<reqwest::Response> {
    let client = reqwest::Client::new();
    let url = craft_url(&config.registry_url, app, device_id);
    let token = &config.token.access_token().secret();

    Ok(client
        .put(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .bearer_auth(token)
        .body(data.to_string())
        .send()
        .await
        .context(format!(
            "Error while updating device data for {}",
            device_id
        ))?)
}
