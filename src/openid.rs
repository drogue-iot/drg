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

use crate::config::{save_config, Config};
use crate::util;
use chrono::{DateTime, Duration, Utc};

const CLIENT_ID: &str = "drogue";
const SERVER_PORT: u16 = 8080;
const REDIRECT_URL: &str = "http://localhost:8080";

pub fn login(api_endpoint: Url) -> Result<Config> {
    println!("Starting authentication process with {}", api_endpoint);

    let (sso_url, registry_url) = util::get_drogue_services_endpoint(api_endpoint.clone())?;
    let (auth_url, token_url) = util::get_auth_and_tokens_endpoints(sso_url)?;

    let token = get_token(auth_url.clone(), token_url.clone())?;
    let token_exp_date = calculate_tokent_expiration_date(&token)?;

    let config = Config {
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
    let client = BasicClient::new(
        ClientId::new(CLIENT_ID.to_string()),
        None,
        AuthUrl::new(auth_url.to_string())?,
        Some(TokenUrl::new(token_url.to_string())?),
    )
    // Where the user will be redirected to after the authorization process.
    .set_redirect_url(RedirectUrl::new(REDIRECT_URL.to_string())?);

    // Generate a PKCE challenge. As this is a client app a PKCE challenge this is needed to assure confidentiality.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (final_auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("offline_access".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // The URL the user should browse to, in order to trigger the authorization process.
    // todo : open a browser automagically.
    println!(
        "\nTo authenticate with drogue cloud please browse to: \n{}",
        final_auth_url
    );

    let bind = format!("0.0.0.0:{}", SERVER_PORT);
    //start a local server
    let server = Server::http(bind).unwrap();
    let request = server.recv()?;

    // extract code and state from the openID server request
    let querry = QString::from(request.url().trim_start_matches("/?"));
    let state = querry.get("state").unwrap();
    let code = querry.get("code").unwrap();

    let _ = request.respond(Response::from_string(
        "Authentication code retrieved. This browser can be closed.",
    ));

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

pub fn verify_token_validity(config: Config) -> Result<Config> {
    // 30 seconds should be enough
    if config.token_exp_date - Utc::now() > Duration::seconds(30) {
        Ok(config)
    } else {
        // todo verbose
        // println!("Token is expired or will be soon, refreshing...");
        refresh_token(config)
    }
}

fn refresh_token(mut config: Config) -> Result<Config> {
    let auth_url = AuthUrl::new(config.auth_url.to_string())?;
    let token_url = TokenUrl::new(config.token_url.to_string())?;
    let client = BasicClient::new(
        ClientId::new(CLIENT_ID.to_string()),
        None,
        auth_url,
        Some(token_url),
    );

    let new_token = client
        .exchange_refresh_token(
            config
                .token
                .refresh_token()
                .ok_or(Error::msg("Error loading refresh token from config"))?,
        )
        .request(http_client)
        .map_err(|_| Error::msg("Error when fetching a refresh token"))?;

    config.token_exp_date = calculate_tokent_expiration_date(&new_token)?;
    config.token = new_token;

    // todo verbose
    // println!("Token successfully refreshed.");
    save_config(&config)?;

    Ok(config)
}

fn calculate_tokent_expiration_date(token: &BasicTokenResponse) -> Result<DateTime<Utc>> {
    let now = Utc::now();
    let expiration = token
        .expires_in()
        .ok_or(anyhow::Error::msg("Missing expiration time on token"))?;

    now.checked_add_signed(Duration::from_std(expiration)?)
        .ok_or(anyhow::Error::msg(
            "Error calculating token expiration date",
        ))
}

pub fn print_token(config: &Config) {
    println!("{}", config.token.access_token().secret());
}