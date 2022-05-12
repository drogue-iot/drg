use assert_cmd::Command;
use drg_test_utils::*;
use drogue_client::admin::v1::{Members, Role};
use rstest::*;

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
fn add_member_and_read(context: &String, app: String) {
    set_default_app(context, &app);
    // fixme : use a stable user somehow ?
    let user = "jbtrystram";

    retry_409!(
        3,
        drg!(context)
            .arg("add")
            .arg("member")
            .arg("--role")
            .arg("reader")
            // fixme : use a stable user somehow ?
            .arg(user)
    );

    let read = drg!(context).arg("get").arg("members").assert().success();

    let output: Members = serde_json::from_slice(&read.get_output().stdout).unwrap();
    assert!(!output.members.is_empty());
    let member = output.members.get(user);
    assert!(member.is_some());
    let member = member.unwrap();
    assert_eq!(member.role, Role::Reader);

    app_delete(context, app);
}

#[rstest]
fn add_and_delete_member(context: &String, app: String) {
    set_default_app(context, &app);
    // fixme : use a stable user somehow ?
    let user = "jbtrystram";

    retry_409!(
        3,
        drg!(context)
            .arg("add")
            .arg("member")
            .arg("--role")
            .arg("reader")
            // fixme : use a stable user somehow ?
            .arg(user)
    );

    let delete = drg!(context)
        .arg("delete")
        .arg("member")
        // fixme : use a stable user somehow ?
        .arg(user)
        .assert()
        .success();

    let output: JsonOutcome = serde_json::from_slice(&delete.get_output().stdout).unwrap();
    assert!(output.is_success());

    let read = drg!(context).arg("get").arg("members").assert().success();

    let output: Members = serde_json::from_slice(&read.get_output().stdout).unwrap();
    assert!(output.members.is_empty());

    app_delete(context, app);
}
