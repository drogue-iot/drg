use assert_cmd::Command;
use drg_test_utils::util::remove_resource_version;
use drg_test_utils::{app_create, app_delete, drg, retry_409, setup_ctx, JsonOutcome};
use drogue_client::registry::v1::Application;
use json_value_merge::Merge;
use rstest::*;
use serde_json::{json, Value};
use std::io::Write;
use tempfile::Builder;
use uuid::Uuid;

#[fixture]
#[once]
fn context() -> String {
    setup_ctx()
}

#[fixture]
pub fn app(context: &String) -> String {
    app_create(context)
}

#[rstest]
fn create_app(context: &String) {
    let id = Uuid::new_v4().to_string();

    let create = drg!(context)
        .arg("create")
        .arg("app")
        .arg(id.clone())
        .assert()
        .success();

    let output: JsonOutcome = serde_json::from_slice(&create.get_output().stdout).unwrap();
    assert!(output.is_success());

    let read = drg!(context)
        .arg("get")
        .arg("app")
        .arg(id.clone())
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&read.get_output().stdout).unwrap();
    assert_eq!(output.metadata.name, id);

    app_delete(context, id);
}

#[rstest]
fn delete_app(context: &String, app: String) {
    drg!(context)
        .arg("delete")
        .arg("app")
        .arg(app.clone())
        .assert()
        .success();

    let read = drg!(context).arg("get").arg("app").arg(app).assert();

    match read.try_failure() {
        Ok(assert) => {
            let output: JsonOutcome = serde_json::from_slice(&assert.get_output().stdout).unwrap();
            assert!(output.is_failure());
            assert_eq!(output.http_status, Some(404));
        }
        // in some cases, the application can be retrieved if it's not deleted yet.
        Err(err) => {
            let output: Application =
                serde_json::from_slice(&err.get_assert().get_output().stdout).unwrap();
            // we check if it was marked for deletion. If so, it's all good.
            assert!(output.metadata.deletion_timestamp.is_some());
        }
    }
}

#[rstest]
fn list_apps(context: &String, app: String) {
    let list = drg!(context).arg("get").arg("apps").assert().success();

    let output: Vec<Application> = serde_json::from_slice(&list.get_output().stdout).unwrap();

    assert!(!output.is_empty());

    app_delete(context, app);
}

#[rstest]
fn read_app(context: &String, app: String) {
    let get = drg!(context)
        .arg("get")
        .arg("app")
        .arg(app.clone())
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&get.get_output().stdout).unwrap();

    assert_eq!(output.metadata.name, app);

    app_delete(context, app);
}

#[rstest]
fn update_app_spec(context: &String, app: String) {
    let spec = json!({"mykey": "myvalue", "numkey": 0, "boolkey": true});

    retry_409!(
        3,
        drg!(context)
            .arg("edit")
            .arg("app")
            .arg(app.clone())
            .arg("-s")
            .arg(spec.to_string())
    );

    let get = drg!(context)
        .arg("get")
        .arg("app")
        .arg(app.clone())
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&get.get_output().stdout).unwrap();

    assert_eq!(output.spec.get("mykey").unwrap(), "myvalue");
    assert_eq!(output.spec.get("numkey").unwrap(), 0);
    assert_eq!(output.spec.get("boolkey").unwrap(), true);

    app_delete(context, app);
}

#[rstest]
fn update_spec_from_file(context: &String, app: String) {
    let spec = json!({"mykey": "myvalue", "numkey": 0, "boolkey": true});

    let get = drg!(context)
        .arg("get")
        .arg("app")
        .arg(app.clone())
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

    drg!(context)
        .arg("edit")
        .arg("app")
        .arg("--filename")
        .arg(file.path())
        .assert()
        .success();

    let get = drg!(context)
        .arg("get")
        .arg("app")
        .arg(app.clone())
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&get.get_output().stdout).unwrap();

    assert_eq!(output.spec.get("mykey").unwrap(), "myvalue");
    assert_eq!(output.spec.get("numkey").unwrap(), 0);
    assert_eq!(output.spec.get("boolkey").unwrap(), true);

    app_delete(context, app);
}

#[rstest]
fn create_with_spec(context: &String) {
    let id = Uuid::new_v4().to_string();
    let spec = json!({"mykey": "myvalue", "numkey": 0, "boolkey": true});

    drg!(context)
        .arg("create")
        .arg("app")
        .arg(id.clone())
        .arg("--spec")
        .arg(spec.to_string())
        .assert()
        .success();

    let get = drg!(context)
        .arg("get")
        .arg("app")
        .arg(id.clone())
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&get.get_output().stdout).unwrap();

    assert_eq!(output.spec.get("mykey").unwrap(), "myvalue");
    assert_eq!(output.spec.get("numkey").unwrap(), 0);
    assert_eq!(output.spec.get("boolkey").unwrap(), true);

    app_delete(context, id);
}

#[rstest]
fn create_from_file(context: &String) {
    let id = Uuid::new_v4().to_string();
    let app = Application::new(id.clone());

    let file = Builder::new().tempfile().unwrap();
    // Write the serialized data to the file
    file.as_file()
        .write_all(serde_json::to_string(&app).unwrap().as_bytes())
        .unwrap();

    drg!(context)
        .arg("create")
        .arg("app")
        .arg("--filename")
        .arg(file.path())
        .assert()
        .success();

    drg!(context)
        .arg("get")
        .arg("apps")
        .arg(id.clone())
        .assert()
        .success();

    app_delete(context, id);
}

#[rstest]
fn add_labels(context: &String, app: String) {
    retry_409!(
        3,
        drg!(context)
            .arg("set")
            .arg("label")
            .arg("test-label=someValue")
            .arg("owner=tests")
            .arg("--application")
            .arg(app.clone())
    );

    let read = drg!(context)
        .arg("get")
        .arg("apps")
        .arg(app.clone())
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&read.get_output().stdout).unwrap();

    let label = output.metadata.labels.get("test-label");
    assert!(label.is_some());
    let label = label.unwrap();
    assert_eq!(label, "someValue");

    app_delete(context, app);
}

#[rstest]
fn list_apps_with_labels(context: &String, app: String) {
    let id2 = app_create(context);

    retry_409!(
        3,
        drg!(context)
            .arg("set")
            .arg("label")
            .arg("test-label=list")
            .arg("--application")
            .arg(app.clone())
    );

    let read = drg!(context)
        .arg("get")
        .arg("apps")
        .arg("--labels")
        .arg("test-label=list")
        .assert()
        .success();

    let output: Vec<Application> = serde_json::from_slice(&read.get_output().stdout).unwrap();

    assert!(!output.is_empty());
    for app in output {
        assert!(app.metadata.labels.get("test-label").is_some());
        assert_eq!(app.metadata.labels.get("test-label").unwrap(), "list");
        assert_ne!(app.metadata.name, id2);
    }

    app_delete(context, app);
    app_delete(context, id2);
}

#[rstest]
fn set_labels_dont_overwrite_existing_labels(context: &String, app: String) {
    retry_409!(
        3,
        drg!(context)
            .arg("set")
            .arg("label")
            .arg("test-label=bar")
            .arg("--application")
            .arg(app.clone())
    );

    retry_409!(
        3,
        drg!(context)
            .arg("set")
            .arg("label")
            .arg("another-label=foo")
            .arg("--application")
            .arg(app.clone())
    );

    let get = drg!(context)
        .arg("get")
        .arg("apps")
        .arg(app.clone())
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&get.get_output().stdout).unwrap();

    assert_eq!(output.metadata.labels.len(), 2);
    assert!(output.metadata.labels.get("another-label").is_some());
    assert!(output.metadata.labels.get("test-label").is_some());

    app_delete(context, app);
}

// TODO add more tests
// - update an app preserve existing spec
// - update an app spec with invalid data should fail
// - update an app with invalid data fails
// - add and read trust anchor
