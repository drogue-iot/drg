use drg_test_utils::setup;
use std::env;

#[test]
/// make sure we can log in into a drogue-cloud instance using an api token
fn test_login_with_api_token() {
    setup().success();
}

#[test]
/// make sure login in into a drogue-cloud with invalid token fails
fn test_login_with_invalid_api_token_fails() {
    env::set_var("DROGUE_SANDBOX_ACCESS_KEY", "invalid");
    setup().failure();
}
