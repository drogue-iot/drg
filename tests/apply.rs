use assert_cmd::Command;
use drg_test_utils::{app_create, app_delete, drg, setup};
use drogue_client::registry::v1::{Application, Device};
use rstest::*;
use serde_json::json;
use std::io::Write;
use tempfile::Builder;
use uuid::Uuid;

#[fixture]
#[once]
fn context() -> () {
    setup().success();
}

#[fixture]
pub fn app(_context: &()) -> String {
    app_create()
}

#[rstest]
fn create_app_std_in(_context: &()) {
    let id = Uuid::new_v4().to_string();
    let json = json!({"metadata": {"name": id}});

    let _create = drg!()
        .arg("apply")
        .arg("-f")
        .arg("-")
        .write_stdin(json.to_string())
        .assert()
        .success();

    // ignore output for now as apply does not support -o json yet
    //let output: JsonOutcome = serde_json::from_slice(&create.get_output().stdout).unwrap();
    //assert!(output.is_success());

    let read = drg!()
        .arg("get")
        .arg("app")
        .arg(id.clone())
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&read.get_output().stdout).unwrap();
    assert_eq!(output.metadata.name, id);

    app_delete(id);
}

#[rstest]
fn create_app_file(_context: &()) {
    let id = Uuid::new_v4().to_string();
    let json = json!({"metadata": {"name": id}});

    // we set a custom prefix bc otherwise filename contain a leading '.'
    let file = Builder::new().prefix("drg").tempfile().unwrap();
    // Write the serialized data to the file
    file.as_file()
        .write_all(json.to_string().as_bytes())
        .unwrap();

    let _create = drg!()
        .arg("apply")
        .arg("-f")
        .arg(file.path())
        .assert()
        .success();

    // ignore output for now as apply does not support -o json yet
    //let output: JsonOutcome = serde_json::from_slice(&create.get_output().stdout).unwrap();
    //assert!(output.is_success());

    let read = drg!()
        .arg("get")
        .arg("app")
        .arg(id.clone())
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&read.get_output().stdout).unwrap();
    assert_eq!(output.metadata.name, id);

    app_delete(id);
}

#[rstest]
fn update_app_file(app: String) {
    let json = json!({"metadata": {"name": app, "labels": {"origin": "integration-test"}}});

    let file = Builder::new().prefix("drg").tempfile().unwrap();
    // Write the serialized data to the file
    file.as_file()
        .write_all(json.to_string().as_bytes())
        .unwrap();

    let _create = drg!()
        .arg("apply")
        .arg("-f")
        .arg(file.path())
        .assert()
        .success();

    // ignore output for now as apply does not support -o json yet
    //let output: JsonOutcome = serde_json::from_slice(&create.get_output().stdout).unwrap();
    //assert!(output.is_success());

    let read = drg!()
        .arg("get")
        .arg("app")
        .arg(app.clone())
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&read.get_output().stdout).unwrap();
    assert_eq!(output.metadata.name, app);
    let labels = output.metadata.labels;
    assert_eq!(labels.len(), 1);
    let label = labels.get("origin");
    assert!(label.is_some());
    assert_eq!(label.unwrap(), "integration-test");

    app_delete(app);
}

#[rstest]
fn update_app_stdin(app: String) {
    let json = json!({"metadata": {"name": app, "labels": {"origin": "integration-test"}}});

    let _create = drg!()
        .arg("apply")
        .arg("-f")
        .arg("-")
        .write_stdin(json.to_string())
        .assert()
        .success();

    // ignore output for now as apply does not support -o json yet
    //let output: JsonOutcome = serde_json::from_slice(&create.get_output().stdout).unwrap();
    //assert!(output.is_success());

    let read = drg!()
        .arg("get")
        .arg("app")
        .arg(app.clone())
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&read.get_output().stdout).unwrap();
    assert_eq!(output.metadata.name, app);
    let labels = output.metadata.labels;
    assert_eq!(labels.len(), 1);
    let label = labels.get("origin");
    assert!(label.is_some());
    assert_eq!(label.unwrap(), "integration-test");

    app_delete(app);
}

#[rstest]
fn create_device_stdin(app: String) {
    let id = Uuid::new_v4().to_string();
    let json = json!({"metadata": {"name": id, "application": app, "labels": {"origin": "integration-test"}}});

    let _create = drg!()
        .arg("apply")
        .arg("-f")
        .arg("-")
        .write_stdin(json.to_string())
        .assert()
        .success();

    // ignore output for now as apply does not support -o json yet
    //let output: JsonOutcome = serde_json::from_slice(&create.get_output().stdout).unwrap();
    //assert!(output.is_success());

    let read = drg!()
        .arg("get")
        .arg("device")
        .arg(id.clone())
        .arg("--application")
        .arg(app.clone())
        .assert()
        .success();

    let output: Device = serde_json::from_slice(&read.get_output().stdout).unwrap();
    assert_eq!(output.metadata.name, id);
    let labels = output.metadata.labels;
    assert_eq!(labels.len(), 1);
    let label = labels.get("origin");
    assert!(label.is_some());
    assert_eq!(label.unwrap(), "integration-test");

    app_delete(app);
}
