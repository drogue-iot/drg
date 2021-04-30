use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};

use anyhow::Error;
use anyhow::Result;

use tiny_http::{Response, Server};

use qstring::QString;
use reqwest::Url;

use crate::config::{self, Context};
use crate::util;
use chrono::{DateTime, Duration, Utc};
use std::net::{Ipv4Addr, SocketAddr};

const CLIENT_ID: &str = "drogue";

pub fn login(
    api_endpoint: Url,
    refresh_token_val: Option<&str>,
    context_name: Option<config::ContextId>,
) -> Result<Context> {
    log::info!("Starting authentication process with {}", api_endpoint);

    let (sso_url, registry_url) = util::get_drogue_services_endpoint(api_endpoint.clone())?;
    let (auth_url, token_url) = util::get_auth_and_tokens_endpoints(sso_url)?;

    let token = match refresh_token_val {
        Some(refresh_token_val) => exchange_token(
            auth_url.clone(),
            token_url.clone(),
            &oauth2::RefreshToken::new(refresh_token_val.to_string()),
        )?,
        None => get_token(auth_url.clone(), token_url.clone())?,
    };

    let token_exp_date = calculate_token_expiration_date(&token)?;

    log::info!("Token successfully obtained.");
    log::debug!("{:?}", token);
    let name = context_name.unwrap_or_else(|| config::ask_config_name());
    let config = Context {
        name,
        drogue_cloud_url: api_endpoint,
        default_app: None,
        token,
        token_url,
        auth_url,
        registry_url,
        token_exp_date,
    };

    Ok(config)
}

fn get_token(auth_url: Url, token_url: Url) -> Result<BasicTokenResponse> {
    log::debug!("Using auth url : {}", auth_url);

    //start a local server
    let bind = SocketAddr::from((Ipv4Addr::LOCALHOST, 0));
    let server = Server::http(bind).unwrap();
    let port = server.server_addr().port();

    let client = BasicClient::new(
        ClientId::new(CLIENT_ID.to_string()),
        None,
        AuthUrl::new(auth_url.to_string())?,
        Some(TokenUrl::new(token_url.to_string())?),
    )
    // Where the user will be redirected to after the authorization process.
    .set_redirect_url(RedirectUrl::new(format!("http://localhost:{}", port))?);

    // Generate a PKCE challenge. As this is a client app a PKCE challenge this is needed to assure confidentiality.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (final_auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("offline_access".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // The URL the user should browse to, in order to trigger the authorization process.
    log::info!("Opening browser.");
    if webbrowser::open(final_auth_url.as_str()).is_err() {
        log::warn!("Failed to open browser.");
        println!(
            "\nTo authenticate with drogue cloud please browse to: \n{}",
            final_auth_url
        );
    }

    // get the request from the localhost webserver
    let request = server.recv()?;

    // extract code and state from the openID server request
    let querry = QString::from(request.url().trim_start_matches("/?"));
    let state = querry.get("state").unwrap();
    let code = querry.get("code").unwrap();

    let _ = request.respond(Response::from_string(
        "Authentication code retrieved. This browser can be closed.",
    ));
    log::info!("Authentication code retrieved.");
    log::debug!("Trading auth code with token using url : {}", token_url);

    // For security reasons, verify that the `state` parameter returned by the server matches `csrf_state`.
    assert_eq!(csrf_token.secret().as_str(), state);

    // Now trade it for an access token.
    let token_result = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        // Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier)
        .request(http_client);

    // Unwrapping token_result will either produce a Token or a RequestTokenError.
    token_result.map_err(|_| Error::msg("error retrieving the authentication token"))
}

pub fn verify_token_validity(context: &mut Context) -> Result<bool> {
    log::debug!("Token expires at : {}", context.token_exp_date);
    // 30 seconds should be enough
    if context.token_exp_date - Utc::now() > Duration::seconds(30) {
        Ok(false)
    } else {
        log::info!("Token is expired or will be soon, refreshing...");
        refresh_token(context)
    }
}

fn refresh_token(context: &mut Context) -> Result<bool> {
    let refresh_token_var = context
        .token
        .refresh_token()
        .ok_or_else(|| Error::msg("Error loading refresh token from config"))?;
    let new_token = exchange_token(
        context.auth_url.clone(),
        context.token_url.clone(),
        &refresh_token_var,
    )?;

    context.token_exp_date = calculate_token_expiration_date(&new_token)?;
    context.token = new_token;

    log::info!("New token will expire at {}", context.token_exp_date);
    log::info!("Token successfully refreshed.");

    Ok(true)
}

fn exchange_token(
    auth_url: Url,
    token_url: Url,
    refresh_token_val: &oauth2::RefreshToken,
) -> Result<BasicTokenResponse> {
    log::debug!("Refreshing token using url : {}", &token_url);

    let auth_url = AuthUrl::new(auth_url.to_string())?;
    let token_url = TokenUrl::new(token_url.to_string())?;

    let client = BasicClient::new(
        ClientId::new(CLIENT_ID.to_string()),
        None,
        auth_url,
        Some(token_url),
    );

    // Exchange the refresh token for access token
    client
        .exchange_refresh_token(refresh_token_val)
        .request(http_client)
        .map_err(|_| Error::msg("Invalid refresh token"))
}

fn calculate_token_expiration_date(token: &BasicTokenResponse) -> Result<DateTime<Utc>> {
    let now = Utc::now();
    let expiration = token
        .expires_in()
        .ok_or_else(|| anyhow::Error::msg("Missing expiration time on token"))?;

    now.checked_add_signed(Duration::from_std(expiration)?)
        .ok_or_else(|| anyhow::Error::msg("Error calculating token expiration date"))
}

pub fn print_token(context: &Context) {
    println!("{}", context.token.access_token().secret());
}
