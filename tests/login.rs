use assert_cmd::Command;
use drg_test_utils::{setup, setup_no_login};
use std::env;

#[test]
/// make sure we can log in into a drogue-cloud instance using an api token
fn test_login_with_api_token() {
    setup();
}

#[test]
/// make sure login in into a drogue-cloud with invalid token fails
fn test_login_with_invalid_api_token_fails() {
    setup_no_login();
    let mut cmd = Command::cargo_bin("drg").unwrap();
    let url = env::var("DROGUE_SANDBOX_URL").unwrap();
    let user = env::var("DROGUE_SANDBOX_USERNAME").unwrap();

    cmd.arg("login")
        .arg(url)
        .arg("-c")
        .arg("test")
        .arg("--access-token")
        .arg(format!("{}:invalid", user))
        .assert()
        .failure();
}
