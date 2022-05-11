mod macros;
mod outcome;
pub mod util;

pub use macros::*;
pub use outcome::*;

use assert_cmd::Command;
use dotenv;
use drogue_client::tokens::v1::AccessToken;
use std::env;
use assert_cmd::assert::Assert;
use uuid::Uuid;

// todo save the context in a file in /tmp
pub fn setup(ctx: String) -> Assert {
    // load a dotenv file if it exists
    dotenv::dotenv().ok();

    let mut cmd = Command::cargo_bin("drg").unwrap();
    let cred = load_credentials();
    let url = env::var("DROGUE_SANDBOX_URL").unwrap();

    cmd
        .arg("login")
        .arg(url)
        .arg("-c")
        .arg(ctx)
        .arg("--access-token")
        .arg(cred)
        .assert()
}

pub fn setup_ctx() -> String {
    let ctx_name = Uuid::new_v4().to_string();
    setup(ctx_name.clone()).success();

    ctx_name
}

pub fn setup_no_login() {
    // load a dotenv file if it exists
    dotenv::dotenv().ok();

    let mut cmd = Command::cargo_bin("drg").unwrap();
    cmd.arg("version").assert().success();
}

/// delete all the tokens we may have created except the one we need to log the CI
pub fn cleanup_tokens(ctx: &String) {
    let dont_delete = env::var("DROGUE_SANDBOX_KEY_PREFIX").unwrap();

    let list = drg!(ctx)
        .arg("get")
        .arg("token")
        .assert();

    let output: Vec<AccessToken> = serde_json::from_slice(&list.get_output().stdout).unwrap();
    list.success();

    for access_token in output {
        if access_token.prefix != dont_delete {
            drg!(ctx)
                .arg("delete")
                .arg("token")
                .arg(access_token.prefix)
                .assert().success();
        }
    }

}

fn load_credentials() -> String {
    let username = env::var("DROGUE_SANDBOX_USERNAME").unwrap();
    let key = env::var("DROGUE_SANDBOX_ACCESS_KEY").unwrap();

    format!("{username}:{key}")
}

pub fn app_delete(ctx: &String, id: String) -> Assert {
    drg!(ctx)
        .arg("delete")
        .arg("app")
        .arg(id)
        .assert()
}

pub fn app_create(ctx: &String) -> String {
    let id = Uuid::new_v4().to_string();

    drg!(ctx)
        .arg("create")
        .arg("app")
        .arg(id.clone())
        .assert()
        .success();

    id
}

pub fn device_create(ctx: &String, app: &String) -> String {
    let id = Uuid::new_v4().to_string();

    drg!(ctx)
        .arg("create")
        .arg("device")
        .arg(id.clone())
        .arg("--app")
        .arg(app)
        .assert().success();

    id
}

pub fn device_delete(ctx: &String, app: &String, id: String) -> Assert {
    drg!(ctx)
        .arg("delete")
        .arg("device")
        .arg("--app")
        .arg(app)
        .arg(id)
        .assert()
}

pub fn set_default_app(ctx: &String, app: &String) {
    drg!(ctx)
        .arg("context")
        .arg("default-app")
        .arg(app.clone())
        .assert()
        .success();
}