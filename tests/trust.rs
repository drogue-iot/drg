use drg_test_utils::{app_create, app_delete, device_create, drg, retry_409, setup};
use drogue_client::registry::v1::{Application, ApplicationSpecTrustAnchors};
use drogue_client::Translator;

use assert_cmd::Command;
use rstest::*;

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
fn generate_and_add_trust_anchor(app: String) {
    drg!()
        .arg("create")
        .arg("app-cert")
        .arg("--application")
        .arg(app.clone())
        .assert()
        .success();

    let read = drg!()
        .arg("get")
        .arg("app")
        .arg(app.clone())
        .assert()
        .success();

    let output: Application = serde_json::from_slice(&read.get_output().stdout).unwrap();

    let anchors = output.section::<ApplicationSpecTrustAnchors>();
    assert!(anchors.is_some());
    let anchors = anchors.unwrap().unwrap();
    assert!(!anchors.anchors.is_empty());
    assert!(!anchors.anchors[0].certificate.is_empty());

    app_delete(app.clone());
}

#[rstest]
fn create_device_certificate(app: String) {
    let device = device_create(&app);

    retry_409!(
        3,
        drg!()
            .arg("create")
            .arg("app-cert")
            .arg("--key-output")
            .arg("app_key.pem")
            .arg("--application")
            .arg(app.clone())
    );

    drg!()
        .arg("create")
        .arg("device-cert")
        .arg("--ca-key")
        .arg("app_key.pem")
        .arg("--cert_output")
        .arg("dev-cert.pem")
        .arg("--key-output")
        .arg("dev-private.pem")
        .arg("--application")
        .arg(app.clone())
        .arg(device)
        .assert()
        .success();

    app_delete(app.clone());
}
