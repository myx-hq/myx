use std::path::Path;
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

fn run_myx(args: &[&str], cwd: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_myx"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run myx")
}

#[test]
fn init_json_outputs_machine_readable_payload() {
    let tmp = TempDir::new().expect("tempdir");
    let pkg_dir = tmp.path().join("sample-capability");

    let output = run_myx(
        &["init", pkg_dir.to_str().expect("utf8 path"), "--json"],
        tmp.path(),
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    let payload: Value = serde_json::from_str(&stdout).expect("json output");
    assert_eq!(payload["command"], "init");
    assert_eq!(payload["ok"], true);
    assert_eq!(
        payload["manifest_path"],
        pkg_dir.join("myx.yaml").display().to_string()
    );
    assert_eq!(
        payload["profile_path"],
        pkg_dir.join("capability.json").display().to_string()
    );

    assert!(pkg_dir.join("myx.yaml").exists());
    assert!(pkg_dir.join("capability.json").exists());
}

#[test]
fn init_default_output_stays_human_readable() {
    let tmp = TempDir::new().expect("tempdir");
    let pkg_dir = tmp.path().join("human-output");

    let output = run_myx(&["init", pkg_dir.to_str().expect("utf8 path")], tmp.path());
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("initialized package scaffold in"));
    assert!(serde_json::from_str::<Value>(&stdout).is_err());
}
