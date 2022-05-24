use assert_cmd::Command;
use drg_test_utils::*;
use drogue_client::admin::v1::{Members, Role};
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
fn add_member_and_read(app: String) {
    // fixme : use a stable user somehow ?
    let user = "jbtrystram";

    retry_409!(
        3,
        drg!()
            .arg("add")
            .arg("member")
            .arg("--role")
            .arg("reader")
            .arg(user)
            .arg("--application")
            .arg(app.clone())
    );

    let read = drg!()
        .arg("get")
        .arg("members")
        .arg("--application")
        .arg(app.clone())
        .assert()
        .success();

    let output: Members = serde_json::from_slice(&read.get_output().stdout).unwrap();
    assert!(!output.members.is_empty());
    let member = output.members.get(user);
    assert!(member.is_some());
    let member = member.unwrap();
    assert_eq!(member.role, Role::Reader);

    app_delete(app);
}

#[rstest]
fn add_and_delete_member(app: String) {
    // fixme : use a stable user somehow ?
    let user = "jbtrystram";

    retry_409!(
        3,
        drg!()
            .arg("add")
            .arg("member")
            .arg("--role")
            .arg("reader")
            .arg(user)
            .arg("--application")
            .arg(app.clone())
    );

    retry_409!(
        3,
        drg!()
            .arg("delete")
            .arg("member")
            .arg(user)
            .arg("--application")
            .arg(app.clone())
    );

    let read = drg!()
        .arg("get")
        .arg("members")
        .arg("--application")
        .arg(app.clone())
        .assert()
        .success();

    let output: Members = serde_json::from_slice(&read.get_output().stdout).unwrap();
    assert!(output.members.is_empty());

    app_delete(app);
}
