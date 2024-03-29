use crate::config::Context;
use crate::util::url_validation;

use crate::DrogueError;
use anyhow::{anyhow, Context as AnyhowContext};
use drogue_client::discovery::v1::{Client, Endpoints};
use serde_json::Value;
use serde_json::Value::String as serde_string;
use tabular::{Row, Table};
use url::Url;

// use drogue's well known endpoint to retrieve endpoints.
pub async fn get_drogue_endpoints(url: Url) -> anyhow::Result<(Url, Url)> {
    let client: Client = Client::new_anonymous(reqwest::Client::new(), url);

    let endpoints = client
        .get_public_endpoints()
        .await?
        .ok_or_else(|| anyhow!("Error fetching drogue-cloud endpoints."))?;

    let (registry, sso) = endpoints
        .registry
        .zip(endpoints.issuer_url)
        .ok_or_else(|| anyhow!("Missing SSO information from drogue-cloud endpoints"))?;

    // Url::join remove the last segment if there is no trailing slash so we append it there
    let registry = format!("{}/", registry.url);
    let sso = format!("{sso}/");

    Ok((Url::parse(sso.as_str())?, Url::parse(registry.as_str())?))
}

pub async fn get_drogue_endpoints_authenticated(
    context: &Context,
) -> Result<Endpoints, DrogueError> {
    let client = Client::new_authenticated(
        reqwest::Client::new(),
        context.drogue_cloud_url.clone(),
        context.token.clone(),
    );

    match client.get_authenticated_endpoints().await {
        Ok(Some(endpoints)) => Ok(endpoints),
        Ok(None) => Err(DrogueError::UnexpectedClient(anyhow!(
            "Cannot retreive Drogue cloud endpoints list"
        ))),
        Err(e) => Err(DrogueError::from(e)),
    }
}

pub async fn get_drogue_console_endpoint(context: &Context) -> anyhow::Result<Url> {
    let endpoints = get_drogue_endpoints_authenticated(context).await?;
    let console = endpoints
        .console
        .ok_or_else(|| anyhow!("No `console` service in drogue endpoints list"))?;

    Url::parse(console.as_str()).context("Cannot parse console url to a valid url")
    //url_validation(ws)
}

pub async fn get_drogue_websocket_endpoint(context: &Context) -> anyhow::Result<Url> {
    let endpoints = get_drogue_endpoints_authenticated(context).await?;
    let ws = endpoints
        .websocket_integration
        .ok_or_else(|| anyhow!("No `console` service in drogue endpoints list"))?;

    Url::parse(ws.url.as_str()).context("Cannot parse console url to a valid url")
}

// use keycloak's well known endpoint to retrieve endpoints.
// http://keycloakhost:keycloakport/auth/realms/{realm}/.well-known/openid-configuration
pub async fn get_auth_and_tokens_endpoints(issuer_url: Url) -> anyhow::Result<(Url, Url)> {
    let client = reqwest::Client::new();

    let url = issuer_url.join(".well-known/openid-configuration")?;
    let res = client
        .get(url)
        .send()
        .await
        .context("Can't retrieve openid-connect endpoints details")?;

    let endpoints: Value = res
        .json()
        .await
        .context("Cannot deserialize openid-connect endpoints details")?;

    let (auth, token) = endpoints["authorization_endpoint"]
        .as_str()
        .map(url_validation)
        .zip(endpoints["token_endpoint"].as_str().map(url_validation))
        .context("Missing `authorization_endpoint` or `token_endpoint` in drogue openid-connect configuration")?;

    Ok((auth?, token?))
}

// pub async fn print_endpoints(context: &Context) -> Result<Outcome<Endpoints>, DrogueError> {
//     get_drogue_endpoints_authenticated(context).await
//         .map(|e| Outcome::SuccessWithJsonData(e))
//
// }

fn deserialize_endpoint(details: &Value) -> (Option<String>, String) {
    let (host, port) = match details {
        serde_string(s) => (Some(s.clone()), None),
        Value::Object(v) => (
            v.get("url")
                .or_else(|| v.get("host"))
                .map(|h| h.as_str().unwrap().to_string()),
            v.get("port").map(|s| s.as_i64().unwrap()),
        ),
        _ => (None, None),
    };

    let port = port.map_or("".to_string(), |p| format!(":{}", p));
    (host, port)
}

pub fn endpoints_pretty_print(endpoints: &Endpoints, service: Option<&str>) {
    let endpoints = serde_json::to_value(endpoints).unwrap();
    let endpoints_map = endpoints.as_object().unwrap();

    if let Some(service) = service {
        let details = endpoints_map.get(service);
        if let Some(details) = details {
            let (host, port) = deserialize_endpoint(details);
            println!("{}{}", host.unwrap(), port);
        } else {
            println!(
                "Service {} not found in drogue cloud available endpoints",
                service
            );
        }
    } else {
        let mut table = Table::new("{:<} {:<}");
        table.add_row(Row::new().with_cell("NAME").with_cell("URL"));

        for (name, details) in endpoints_map {
            let (host, port) = deserialize_endpoint(details);
            host.map(|h| {
                table.add_row(
                    Row::new()
                        .with_cell(name)
                        .with_cell(format!("{}{}", h, port)),
                )
            });
        }
        print!("{}", table);
    }
}
