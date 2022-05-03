use assert_cmd::Command;
use drg_test_utils::{cleanup_tokens, setup, JsonOutcome};
use drogue_client::tokens::v1::{AccessToken, CreatedAccessToken};

#[test]
fn create_access_token() {
    setup().success();
    let mut cmd = Command::cargo_bin("drg").unwrap();

    let create = cmd
        .arg("create")
        .arg("token")
        .arg("-o")
        .arg("json")
        .assert();

    let output: CreatedAccessToken = serde_json::from_slice(&create.get_output().stdout).unwrap();
    create.success();

    assert!(!output.prefix.is_empty());
    cleanup_tokens();
}

#[test]
fn list_access_tokens() {
    setup().success();

    let list = Command::cargo_bin("drg")
        .unwrap()
        .arg("get")
        .arg("token")
        .arg("-o")
        .arg("json")
        .assert();

    // fixme
    // this deserialization is flaky
    let output: Vec<AccessToken> = serde_json::from_slice(&list.get_output().stdout).unwrap();
    list.success();

    assert!(!output.is_empty());
    assert!(!output[0].prefix.is_empty());
}

#[test]
fn delete_access_tokens() {
    setup().success();

    let create = Command::cargo_bin("drg")
        .unwrap()
        .arg("create")
        .arg("token")
        .arg("-o")
        .arg("json")
        .assert();

    let output: CreatedAccessToken = serde_json::from_slice(&create.get_output().stdout).unwrap();
    create.success();

    let prefix = output.prefix;
    assert!(!prefix.is_empty());

    let delete = Command::cargo_bin("drg")
        .unwrap()
        .arg("delete")
        .arg("token")
        .arg(prefix)
        .arg("-o")
        .arg("json")
        .assert();

    let output: JsonOutcome = serde_json::from_slice(&delete.get_output().stdout).unwrap();

    assert!(output.is_success());
    delete.success();
}
