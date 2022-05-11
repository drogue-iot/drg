use assert_cmd::Command;
use drg_test_utils::util::remove_resource_version;
use drg_test_utils::*;
use drogue_client::registry::v1::Device;
use json_value_merge::Merge;
use rstest::*;
use serde_json::{json, Value};
use std::io::Write;
use tempfile::Builder;
use uuid::Uuid;

#[fixture]
#[once]
fn context() -> Ctx {
    let ctx_id = setup_ctx();
    let app = app_create(&ctx_id);
    set_default_app(&ctx_id, &app);
    Ctx {
        context_name: ctx_id,
        app,
    }
}

#[fixture]
fn device(context: &Ctx) -> String {
    device_create(&context.context_name, &context.app)
}

struct Ctx {
    context_name: String,
    app: String,
}

#[rstest]
fn create_device(context: &Ctx) {
    let id = Uuid::new_v4().to_string();

    let create = drg!(context.context_name)
        .arg("create")
        .arg("device")
        .arg(id.clone())
        .assert()
        .success();

    let output: JsonOutcome = serde_json::from_slice(&create.get_output().stdout).unwrap();
    assert!(output.is_success());

    let read = drg!(context.context_name)
        .arg("get")
        .arg("device")
        .arg(id.clone())
        .assert()
        .success();

    let output: Device = serde_json::from_slice(&read.get_output().stdout).unwrap();
    assert_eq!(output.metadata.name, id);
}

#[rstest]
fn delete_device(context: &Ctx, device: String) {
    drg!(context.context_name)
        .arg("delete")
        .arg("device")
        .arg(device.clone())
        .assert()
        .success();

    let read = drg!(context.context_name)
        .arg("get")
        .arg("device")
        .arg(device)
        .assert()
        .failure();

    let output: JsonOutcome = serde_json::from_slice(&read.get_output().stdout).unwrap();
    assert!(output.is_failure());
    assert_eq!(output.http_status, Some(404));
}

#[rstest]
fn list_devices(context: &Ctx, device: String) {
    let list = drg!(context.context_name)
        .arg("get")
        .arg("devices")
        .assert()
        .success();

    let output: Vec<Device> = serde_json::from_slice(&list.get_output().stdout).unwrap();

    assert!(!output.is_empty());

    let names: Vec<String> = output.iter().map(|d| d.metadata.name.clone()).collect();
    assert!(names.contains(&device));
}

#[rstest]
fn read_device(context: &Ctx, device: String) {
    let get = drg!(context.context_name)
        .arg("get")
        .arg("device")
        .arg(device.clone())
        .assert()
        .success();

    let output: Device = serde_json::from_slice(&get.get_output().stdout).unwrap();

    assert_eq!(output.metadata.name, device);
}

#[rstest]
fn update_device_spec(context: &Ctx, device: String) {
    let spec = json!({"mykey": "myvalue", "numkey": 0, "boolkey": true});

    drg!(context.context_name)
        .arg("edit")
        .arg("device")
        .arg(device.clone())
        .arg("-s")
        .arg(spec.to_string())
        .assert()
        .success();

    let get = drg!(context.context_name)
        .arg("get")
        .arg("device")
        .arg(device.clone())
        .assert()
        .success();

    let output: Device = serde_json::from_slice(&get.get_output().stdout).unwrap();

    assert_eq!(output.spec.get("mykey").unwrap(), "myvalue");
    assert_eq!(output.spec.get("numkey").unwrap(), 0);
    assert_eq!(output.spec.get("boolkey").unwrap(), true);
}

#[rstest]
fn update_spec_from_file(context: &Ctx, device: String) {
    let spec = json!({"mykey": "myvalue", "numkey": 0, "boolkey": true});

    let get = drg!(context.context_name)
        .arg("get")
        .arg("device")
        .arg(device.clone())
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

    drg!(context.context_name)
        .arg("edit")
        .arg("device")
        .arg("--filename")
        .arg(file.path())
        .assert()
        .success();

    let get = drg!(context.context_name)
        .arg("get")
        .arg("device")
        .arg(device.clone())
        .assert()
        .success();

    let output: Device = serde_json::from_slice(&get.get_output().stdout).unwrap();

    assert_eq!(output.spec.get("mykey").unwrap(), "myvalue");
    assert_eq!(output.spec.get("numkey").unwrap(), 0);
    assert_eq!(output.spec.get("boolkey").unwrap(), true);
}

#[rstest]
fn create_with_spec(context: &Ctx) {
    let id = Uuid::new_v4().to_string();
    let spec = json!({"mykey": "myvalue", "numkey": 0, "boolkey": true});

    drg!(context.context_name)
        .arg("create")
        .arg("device")
        .arg(id.clone())
        .arg("--spec")
        .arg(spec.to_string())
        .assert()
        .success();

    let get = drg!(context.context_name)
        .arg("get")
        .arg("device")
        .arg(id.clone())
        .assert()
        .success();

    let output: Device = serde_json::from_slice(&get.get_output().stdout).unwrap();

    assert_eq!(output.spec.get("mykey").unwrap(), "myvalue");
    assert_eq!(output.spec.get("numkey").unwrap(), 0);
    assert_eq!(output.spec.get("boolkey").unwrap(), true);
}

#[rstest]
fn create_from_file(context: &Ctx) {
    let id = Uuid::new_v4().to_string();
    let device = Device::new(&context.app, id.clone());

    let file = Builder::new().tempfile().unwrap();
    // Write the serialized data to the file
    file.as_file()
        .write_all(serde_json::to_string(&device).unwrap().as_bytes())
        .unwrap();

    drg!(context.context_name)
        .arg("create")
        .arg("device")
        .arg("--filename")
        .arg(file.path())
        .assert()
        .success();

    drg!(context.context_name)
        .arg("get")
        .arg("device")
        .arg(id.clone())
        .assert()
        .success();

    device_delete(&context.context_name, &context.app, id);
}

#[rstest]
fn add_labels(context: &Ctx, device: String) {
    drg!(context.context_name)
        .arg("set")
        .arg("label")
        .arg("test-label=someValue")
        .arg("owner=tests")
        .arg("--device")
        .arg(device.clone())
        .assert()
        .success();

    let read = drg!(context.context_name)
        .arg("get")
        .arg("device")
        .arg(device.clone())
        .assert()
        .success();

    let output: Device = serde_json::from_slice(&read.get_output().stdout).unwrap();

    let label = output.metadata.labels.get("test-label");
    assert!(label.is_some());
    let label = label.unwrap();
    assert_eq!(label, "someValue");
}

#[rstest]
fn list_devices_with_labels(context: &Ctx, device: String) {
    let dev2 = device_create(&context.context_name, &context.app);

    retry_409!(
        3,
        drg!(context.context_name)
            .arg("set")
            .arg("label")
            .arg("test-label=list")
            .arg("--device")
            .arg(device.clone())
    );

    let read = drg!(context.context_name)
        .arg("get")
        .arg("apps")
        .arg("--labels")
        .arg("test-label=list")
        .assert()
        .success();

    let output: Vec<Device> = serde_json::from_slice(&read.get_output().stdout).unwrap();

    assert!(!output.is_empty());
    for app in output {
        assert!(app.metadata.labels.get("test-label").is_some());
        assert_eq!(app.metadata.labels.get("test-label").unwrap(), "list");
        assert_ne!(app.metadata.name, dev2);
    }
}

#[rstest]
fn set_labels_dont_overwrite_existing_labels(context: &Ctx, device: String) {
    retry_409!(
        3,
        drg!(context.context_name)
            .arg("set")
            .arg("label")
            .arg("test-label=bar")
            .arg("--device")
            .arg(device.clone())
    );

    retry_409!(
        3,
        drg!(context.context_name)
            .arg("set")
            .arg("label")
            .arg("another-label=foo")
            .arg("--device")
            .arg(device.clone())
    );

    let get = drg!(context.context_name)
        .arg("get")
        .arg("device")
        .arg(device.clone())
        .assert()
        .success();

    let output: Device = serde_json::from_slice(&get.get_output().stdout).unwrap();

    assert_eq!(output.metadata.labels.len(), 2);
    assert!(output.metadata.labels.get("another-label").is_some());
    assert!(output.metadata.labels.get("test-label").is_some());
}

// TODO add more tests
