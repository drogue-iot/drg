use assert_cmd::Command;
use drg_test_utils::util::remove_resource_version;
use drg_test_utils::{app_create, app_delete, retry_409, setup};
use drogue_client::registry::v1::Application;
use json_value_merge::Merge;
use serde_json::{json, Value};
use std::io::Write;
use tempfile::Builder;
use uuid::Uuid;

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
        .assert()
        .success();

    let output: Vec<Application> = serde_json::from_slice(&list.get_output().stdout).unwrap();

    assert!(!output.is_empty());

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
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&get.get_output().stdout).unwrap();

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
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&get.get_output().stdout).unwrap();

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
        .assert()
        .success();

    let mut output: Value = serde_json::from_slice(&get.get_output().stdout).unwrap();
    // add our spec to the app
    output.merge_in("/spec", spec);
    // slice the resource version
    let output = remove_resource_version(output);

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
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&get.get_output().stdout).unwrap();

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
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&get.get_output().stdout).unwrap();

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
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&app.get_output().stdout).unwrap();

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

    retry_409!(
        3,
        Command::cargo_bin("drg")
            .unwrap()
            .arg("set")
            .arg("label")
            .arg("test-label=list")
            .arg("--application")
            .arg(id.clone())
            .arg("-o")
            .arg("json")
    );

    let apps = Command::cargo_bin("drg")
        .unwrap()
        .arg("get")
        .arg("apps")
        .arg("--labels")
        .arg("test-label=list")
        .arg("-o")
        .arg("json")
        .assert()
        .success();

    let output: Vec<Application> = serde_json::from_slice(&apps.get_output().stdout).unwrap();

    assert!(!output.is_empty());
    for app in output {
        assert!(app.metadata.labels.get("test-label").is_some());
        assert_eq!(app.metadata.labels.get("test-label").unwrap(), "list");
        assert_ne!(app.metadata.name, id2);
    }

    app_delete(id);
    app_delete(id2);
}

#[test]
fn set_labels_dont_overwrite_existing_labels() {
    setup().success();

    let id = app_create();

    retry_409!(
        3,
        Command::cargo_bin("drg")
            .unwrap()
            .arg("set")
            .arg("label")
            .arg("test-label=bar")
            .arg("--application")
            .arg(id.clone())
            .arg("-o")
            .arg("json")
    );

    retry_409!(
        3,
        Command::cargo_bin("drg")
            .unwrap()
            .arg("set")
            .arg("label")
            .arg("another-label=foo")
            .arg("--application")
            .arg(id.clone())
            .arg("-o")
            .arg("json")
    );

    let app = Command::cargo_bin("drg")
        .unwrap()
        .arg("get")
        .arg("apps")
        .arg(id.clone())
        .arg("-o")
        .arg("json")
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&app.get_output().stdout).unwrap();

    assert_eq!(output.metadata.labels.len(), 2);
    assert!(output.metadata.labels.get("another-label").is_some());
    assert!(output.metadata.labels.get("test-label").is_some());

    app_delete(id);
}

// TODO add more tests
// - update an app preserve existing spec
// - update an app spec with invalid data should fail
// - update an app with invalid data fails
// - add and read trust anchor
