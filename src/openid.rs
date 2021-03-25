use oauth2::{
    AuthorizationCode,
    AuthUrl,
    ClientId,
    CsrfToken,
    PkceCodeChallenge,
    RedirectUrl,
    Scope,
    TokenUrl,
};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::http_client;

use anyhow::Result;

use tiny_http::{Server, Response};

use qstring::QString;
use reqwest::{Url};

use crate::util;

const CLIENT_ID: &str = "drogue";
const SERVER_PORT: u16 = 8080;
const REDIRECT_URL: &str = "http://localhost:8080";

pub fn login(api_endpoint: Url) -> Result<BasicTokenResponse> {

    println!("Starting authentication process with {}", api_endpoint);

    let sso_url = util::get_sso_endpoint(api_endpoint)?;

    let (auth, token) = util::get_auth_and_tokens_endpoints(sso_url)?;

    get_token(auth, token)
}

fn get_token(auth_url: Url, token_url: Url) -> Result<BasicTokenResponse>{
    let client =
        BasicClient::new(
            ClientId::new(CLIENT_ID.to_string()),
            None,
            AuthUrl::new(auth_url.to_string())?,
            Some(TokenUrl::new(token_url.to_string())?)
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
    println!("\nTo authenticate with drogue cloud please browse to: \n{}", final_auth_url);

    let bind = format!("0.0.0.0:{}", SERVER_PORT);
    //start a local server
    let server = Server::http(bind).unwrap();
    let request = server.recv()?;

    // extract code and state from the openID server request
    let querry = QString::from(request.url().trim_start_matches("/?"));
    let state = querry.get("state").unwrap();
    let code = querry.get("code").unwrap();

    let _ = request.respond(Response::from_string("Authentication code retrieved. This browser can be closed."));

// For security reasons, verify that the `state` parameter returned by the server matches `csrf_state`.
    assert_eq!(csrf_token.secret().as_str(), state);

// Now trade it for an access token.
    let token_result = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
// Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier)
        .request(http_client);

// Unwrapping token_result will either produce a Token or a RequestTokenError.
    token_result.map_err(|_| anyhow::Error::msg("error retrieving the authentication token"))
}