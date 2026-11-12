use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn catalog_lists_apps() {
    let mut cmd = Command::cargo_bin("artcraft").unwrap();
    cmd.arg("catalog");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PhotoCraft"));
}

#[test]
fn detect_runs() {
    let mut cmd = Command::cargo_bin("artcraft").unwrap();
    cmd.arg("detect");
    cmd.assert().success();
}

#[test]
fn discover_json() {
    let mut cmd = Command::cargo_bin("artcraft").unwrap();
    cmd.args(["discover", "--json"]);
    cmd.assert().success();
}

#[test]
fn plan_runs() {
    let mut cmd = Command::cargo_bin("artcraft").unwrap();
    cmd.arg("plan");
    cmd.assert().success();
}
