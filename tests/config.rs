// log in & verify the config exist
// move the file somewhere else and verify we can use the file location

use assert_cmd::Command;
use drg_test_utils::*;
use rstest::*;
use serde_json::Value;

#[fixture]
#[once]
fn context() -> () {
    setup().success();
}

#[rstest]
fn set_default_app(_context: &()) {
    let app = "some_app";
    let create = drg!()
        .arg("config")
        .arg("default-app")
        .arg(app)
        .assert()
        .success();

    let output: JsonOutcome = serde_json::from_slice(&create.get_output().stdout).unwrap();
    assert!(output.is_success());

    let read = drg!()
        .arg("config")
        .arg("show")
        .arg("--active")
        .assert()
        .success();

    let output: Value = serde_json::from_slice(&read.get_output().stdout).unwrap();
    let default_app = output.get("default_app");
    assert!(default_app.is_some());
    let default_app = default_app.unwrap().as_str();
    assert!(default_app.is_some());
    let default_app = default_app.unwrap();
    assert_eq!(default_app, app);
}
