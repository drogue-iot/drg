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

pub fn setup() -> Assert {
    // load a dotenv file if it exists
    dotenv::dotenv().ok();

    let mut cmd = Command::cargo_bin("drg").unwrap();
    let cred = load_credentials();
    let url = env::var("DROGUE_SANDBOX_URL").unwrap();

    cmd
        .arg("login")
        .arg(url)
        .arg("-c")
        .arg("test")
        .arg("--access-token")
        .arg(cred)
        .assert()
}

pub fn setup_no_login() {
    // load a dotenv file if it exists
    dotenv::dotenv().ok();

    let mut cmd = Command::cargo_bin("drg").unwrap();
    cmd.arg("version").assert().success();
}

/// delete all the tokens we may have created except the one we need to log the CI
pub fn cleanup_tokens() {
    let dont_delete = env::var("DROGUE_SANDBOX_KEY_PREFIX").unwrap();

    let list = Command::cargo_bin("drg")
        .unwrap()
        .arg("get")
        .arg("token")
        .arg("-o")
        .arg("json")
        .assert();

    let output: Vec<AccessToken> = serde_json::from_slice(&list.get_output().stdout).unwrap();
    list.success();

    for access_token in output {
        if access_token.prefix != dont_delete {
            Command::cargo_bin("drg")
                .unwrap()
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

pub fn app_delete(id: String) -> Assert {
    Command::cargo_bin("drg").unwrap()
        .arg("delete")
        .arg("app")
        .arg(id)
        .assert()
}

pub fn app_create() -> String {
    let id = Uuid::new_v4().to_string();

    Command::cargo_bin("drg").unwrap()
        .arg("create")
        .arg("app")
        .arg(id.clone())
        .assert()
        .success();
    id
}
