use oauth2::{
    AuthorizationCode,
    AuthUrl,
    ClientId,
    CsrfToken,
    PkceCodeChallenge,
    RedirectUrl,
    Scope,
    TokenUrl
};
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;

use anyhow::Result;

use tiny_http::Server;

use qstring::QString;

const CLIENT_ID: &str = "drogue";
const SERVER_PORT: u16 = 8080;
const REDIRECT_URL: &str = "http://localhost:8080";

// see https://sso.sandbox.drogue.cloud/auth/realms/drogue/.well-known/openid-configuration for those
const AUTH_URL: &str = "https://keycloak-drogue-dev.apps.wonderful.iot-playground.org/auth/realms/drogue/protocol/openid-connect/auth";
const TOKEN_URL: &str = "https://keycloak-drogue-dev.apps.wonderful.iot-playground.org/auth/realms/drogue/protocol/openid-connect/token";


// Create an OAuth2 client by specifying the client ID, authorization URL and token URL.

fn get_token() -> Result<()>{
    let client =
        BasicClient::new(
            ClientId::new(CLIENT_ID.to_string()),
            //Some(ClientSecret::new("616a4e1e-a3cb-401f-86d4-3539a3f31a9b".to_string())),
            None,
            AuthUrl::new(AUTH_URL.to_string())?,
            Some(TokenUrl::new(TOKEN_URL.to_string())?)
        )
// Set the URL the user will be redirected to after the authorization process.
            .set_redirect_url(RedirectUrl::new(REDIRECT_URL.to_string())?);

// Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

// Generate the full authorization URL.
    let (final_auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
// Set the desired scopes.
        //todo : get an offline token
//        .add_scope(Scope::new("read".to_string()))
//        .add_scope(Scope::new("write".to_string()))
// Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

// This is the URL you should redirect the user to, in order to trigger the authorization
// process.
    println!("Browse to: {}", final_auth_url);

    let bind = format!("0.0.0.0:{}", SERVER_PORT);
    //start a local server
    let server = Server::http(bind).unwrap();
    let request = server.recv()?;

    // extract code and state from the openID server request
    let querry = QString::from(request.url().trim_start_matches("/?"));
    let state = querry.get("state").unwrap();
    let code = querry.get("code").unwrap();

// Once the user has been redirected to the redirect URL, you'll have access to the
// authorization code. For security reasons, your code should verify that the `state`
// parameter returned by the server matches `csrf_state`.
    assert_eq!(csrf_token.secret().as_str(), state);

// Now you can trade it for an access token.
    let token_result = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
// Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier)
        .request(http_client);

// Unwrapping token_result will either produce a Token or a RequestTokenError.
    println!("{:?}", token_result.unwrap());
    token_result
}