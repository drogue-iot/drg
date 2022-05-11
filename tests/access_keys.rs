use assert_cmd::Command;
use drg_test_utils::{cleanup_tokens, drg, setup_ctx, JsonOutcome};
use drogue_client::tokens::v1::{AccessToken, CreatedAccessToken};
use rstest::*;

#[fixture]
#[once]
fn context() -> String {
    setup_ctx()
}

#[rstest]
fn create_access_token(context: &String) {
    let create = drg!(context).arg("create").arg("token").assert().success();

    let output: CreatedAccessToken = serde_json::from_slice(&create.get_output().stdout).unwrap();

    assert!(!output.prefix.is_empty());
    cleanup_tokens(context);
}

#[rstest]
fn list_access_tokens(context: &String) {
    let list = drg!(context).arg("get").arg("token").assert().success();

    let output: Vec<AccessToken> = serde_json::from_slice(&list.get_output().stdout).unwrap();

    assert!(!output.is_empty());
    assert!(!output[0].prefix.is_empty());
}

#[rstest]
fn delete_access_token(context: &String) {
    let create = drg!(context).arg("create").arg("token").assert().success();

    let output: CreatedAccessToken = serde_json::from_slice(&create.get_output().stdout).unwrap();

    let prefix = output.prefix;
    assert!(!prefix.is_empty());

    let delete = drg!(context)
        .arg("delete")
        .arg("token")
        .arg(prefix)
        .assert()
        .success();

    let output: JsonOutcome = serde_json::from_slice(&delete.get_output().stdout).unwrap();

    assert!(output.is_success());
}
