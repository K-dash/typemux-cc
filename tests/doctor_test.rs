use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn doctor_exits_zero_with_human_output() {
    let mut cmd = Command::cargo_bin("typemux-cc").unwrap();
    cmd.arg("--doctor");

    cmd.assert()
        .success()
        .stdout(predicate::str::starts_with("typemux-cc v"))
        .stdout(predicate::str::contains("Configuration:"))
        .stdout(predicate::str::contains("backend"))
        .stdout(predicate::str::contains("Environment:"))
        .stdout(predicate::str::contains("System:"))
        .stdout(predicate::str::contains("OS"))
        .stdout(predicate::str::contains("Arch"));
}

#[test]
fn doctor_json_outputs_valid_json() {
    let mut cmd = Command::cargo_bin("typemux-cc").unwrap();
    cmd.arg("--doctor").arg("--json");

    let output = cmd.output().expect("failed to run");
    assert!(output.status.success());

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("output should be valid JSON");

    assert!(json.get("version").is_some());
    assert!(json.get("configuration").is_some());
    assert!(json.get("environment").is_some());
    assert!(json.get("system").is_some());
}

#[test]
fn json_without_doctor_fails() {
    let mut cmd = Command::cargo_bin("typemux-cc").unwrap();
    cmd.arg("--json");

    cmd.assert().failure();
}

#[test]
fn doctor_with_backend_flag() {
    let mut cmd = Command::cargo_bin("typemux-cc").unwrap();
    cmd.arg("--doctor").arg("--backend").arg("ty");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ty"))
        .stdout(predicate::str::contains("(cli)"));
}

#[test]
fn doctor_with_env_var_source() {
    let mut cmd = Command::cargo_bin("typemux-cc").unwrap();
    cmd.arg("--doctor").env("TYPEMUX_CC_MAX_BACKENDS", "4");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("4"))
        .stdout(predicate::str::contains("env: TYPEMUX_CC_MAX_BACKENDS"));
}
