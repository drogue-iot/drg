use assert_cmd::Command;
use drg_test_utils::{app_create, app_delete, setup};
use drogue_client::registry::v1::Application;
use json_value_merge::Merge;
use serde_json::{json, Value};
use std::io::Write;
use tempfile::Builder;
use uuid::Uuid;

// fixme : maybe run tests with several threads but only some of them in serial ?
// use serial_test::serial;

#[test]
fn create_and_delete_app() {
    setup().success();
    let id = app_create();

    app_delete(id);
}

#[test]
fn list_apps() {
    setup().success();
    let id = app_create();

    let list = Command::cargo_bin("drg")
        .unwrap()
        .arg("get")
        .arg("apps")
        .arg("-o")
        .arg("json")
        .assert();

    // fixme
    // this deserialization is flaky
    let output: Vec<Application> = serde_json::from_slice(&list.get_output().stdout).unwrap();
    list.success();

    assert!(!output.is_empty());
    assert_eq!(output[0].metadata.name, id);

    app_delete(id);
}

#[test]
fn read_app() {
    setup().success();
    let id = app_create();

    let get = Command::cargo_bin("drg")
        .unwrap()
        .arg("get")
        .arg("app")
        .arg(id.clone())
        .arg("-o")
        .arg("json")
        .assert();

    let output: Application = serde_json::from_slice(&get.get_output().stdout).unwrap();
    get.success();

    assert_eq!(output.metadata.name, id);

    app_delete(id);
}

#[test]
fn update_app_spec() {
    setup().success();
    let id = app_create();
    let spec = json!({"mykey": "myvalue", "numkey": 0, "boolkey": true});

    Command::cargo_bin("drg")
        .unwrap()
        .arg("edit")
        .arg("app")
        .arg(id.clone())
        .arg("-s")
        .arg(spec.to_string())
        .arg("-o")
        .arg("json")
        .assert()
        .success();

    let get = Command::cargo_bin("drg")
        .unwrap()
        .arg("get")
        .arg("app")
        .arg(id.clone())
        .arg("-o")
        .arg("json")
        .assert();

    let output: Application = serde_json::from_slice(&get.get_output().stdout).unwrap();
    get.success();

    assert_eq!(output.spec.get("mykey").unwrap(), "myvalue");
    assert_eq!(output.spec.get("numkey").unwrap(), 0);
    assert_eq!(output.spec.get("boolkey").unwrap(), true);

    app_delete(id);
}

#[test]
fn update_spec_from_file() {
    setup().success();
    let id = app_create();
    let spec = json!({"mykey": "myvalue", "numkey": 0, "boolkey": true});

    let get = Command::cargo_bin("drg")
        .unwrap()
        .arg("get")
        .arg("app")
        .arg(id.clone())
        .arg("-o")
        .arg("json")
        .assert();

    let mut output: Value = serde_json::from_slice(&get.get_output().stdout).unwrap();

    // add our spec to the app
    output.merge_in("/spec", spec);

    let file = Builder::new().tempfile().unwrap();
    // Write the serialized data to the file
    file.as_file()
        .write_all(output.to_string().as_bytes())
        .unwrap();

    Command::cargo_bin("drg")
        .unwrap()
        .arg("edit")
        .arg("app")
        .arg("--filename")
        .arg(file.path())
        .arg("-o")
        .arg("json")
        .assert()
        .success();

    let get = Command::cargo_bin("drg")
        .unwrap()
        .arg("get")
        .arg("app")
        .arg(id.clone())
        .arg("-o")
        .arg("json")
        .assert();

    let output: Application = serde_json::from_slice(&get.get_output().stdout).unwrap();
    get.success();

    assert_eq!(output.spec.get("mykey").unwrap(), "myvalue");
    assert_eq!(output.spec.get("numkey").unwrap(), 0);
    assert_eq!(output.spec.get("boolkey").unwrap(), true);

    app_delete(id);
}

#[test]
fn create_with_spec() {
    setup().success();

    let id = Uuid::new_v4().to_string();
    let spec = json!({"mykey": "myvalue", "numkey": 0, "boolkey": true});

    Command::cargo_bin("drg")
        .unwrap()
        .arg("create")
        .arg("app")
        .arg(id.clone())
        .arg("--spec")
        .arg(spec.to_string())
        .arg("-o")
        .arg("json")
        .assert()
        .success();

    let get = Command::cargo_bin("drg")
        .unwrap()
        .arg("get")
        .arg("app")
        .arg(id.clone())
        .arg("-o")
        .arg("json")
        .assert();

    let output: Application = serde_json::from_slice(&get.get_output().stdout).unwrap();
    get.success();

    assert_eq!(output.spec.get("mykey").unwrap(), "myvalue");
    assert_eq!(output.spec.get("numkey").unwrap(), 0);
    assert_eq!(output.spec.get("boolkey").unwrap(), true);

    app_delete(id);
}

#[test]
fn create_from_file() {
    setup().success();

    let id = Uuid::new_v4().to_string();
    let app = Application::new(id.clone());

    let file = Builder::new().tempfile().unwrap();
    // Write the serialized data to the file
    file.as_file()
        .write_all(serde_json::to_string(&app).unwrap().as_bytes())
        .unwrap();

    Command::cargo_bin("drg")
        .unwrap()
        .arg("create")
        .arg("app")
        .arg("--filename")
        .arg(file.path())
        .arg("-o")
        .arg("json")
        .assert()
        .success();

    Command::cargo_bin("drg")
        .unwrap()
        .arg("get")
        .arg("apps")
        .arg(id.clone())
        .arg("-o")
        .arg("json")
        .assert()
        .success();

    app_delete(id);
}

#[test]
fn add_labels() {
    setup().success();
    let id = app_create();

    Command::cargo_bin("drg")
        .unwrap()
        .arg("set")
        .arg("label")
        .arg("test-label=someValue")
        .arg("owner=tests")
        .arg("--application")
        .arg(id.clone())
        .arg("-o")
        .arg("json")
        .assert()
        .success();

    let app = Command::cargo_bin("drg")
        .unwrap()
        .arg("get")
        .arg("apps")
        .arg(id.clone())
        .arg("-o")
        .arg("json")
        .assert();

    let output: Application = serde_json::from_slice(&app.get_output().stdout).unwrap();
    app.success();

    let label = output.metadata.labels.get("test-label");
    assert!(label.is_some());
    let label = label.unwrap();
    assert_eq!(label, "someValue");

    app_delete(id);
}

#[test]
fn list_apps_with_labels() {
    setup().success();

    let id = app_create();
    let id2 = app_create();

    Command::cargo_bin("drg")
        .unwrap()
        .arg("set")
        .arg("label")
        .arg("test-label=true")
        .arg("--application")
        .arg(id.clone())
        .arg("-o")
        .arg("json")
        .assert()
        .success();

    let apps = Command::cargo_bin("drg")
        .unwrap()
        .arg("get")
        .arg("apps")
        .arg("--labels")
        .arg("test-label=true")
        .arg("-o")
        .arg("json")
        .assert();

    let output: Vec<Application> = serde_json::from_slice(&apps.get_output().stdout).unwrap();
    apps.success();

    assert_eq!(output.len(), 1);
    assert_eq!(output[0].metadata.name, id);

    app_delete(id);
    app_delete(id2);
}

// - add labels don't owerwrite existing labels

// TODO add more tests
// - update an app preserve existing spec
// - update an app spec with invalid data should fail
// - update an app with invalid data fails
// - add and read trust anchor
