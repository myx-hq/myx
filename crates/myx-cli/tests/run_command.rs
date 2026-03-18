use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};
use tempfile::TempDir;

fn run_myx(args: &[&str], cwd: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_myx"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run myx")
}

fn write_package(dir: &Path, name: &str, version: &str, profile: &Value) -> PathBuf {
    let pkg_dir = dir.join(format!("{name}-{version}"));
    std::fs::create_dir_all(&pkg_dir).expect("create package dir");
    std::fs::write(
        pkg_dir.join("myx.yaml"),
        format!(
            "name: {name}\nversion: {version}\ndescription: test\npublisher: test\nlicense: Apache-2.0\nir: ./capability.json\n"
        ),
    )
    .expect("write manifest");
    std::fs::write(
        pkg_dir.join("capability.json"),
        serde_json::to_vec_pretty(profile).expect("serialize profile"),
    )
    .expect("write profile");
    pkg_dir
}

fn write_workspace_config(workspace: &Path, package_name: &str, version: &str, source: &Path) {
    let index_path = workspace.join("index.json");
    std::fs::write(
        &index_path,
        serde_json::to_vec_pretty(&json!({
            "packages": [
                {
                    "name": package_name,
                    "version": version,
                    "source": source.display().to_string(),
                    "digest": "sha256:test"
                }
            ]
        }))
        .expect("serialize index"),
    )
    .expect("write index");

    std::fs::write(
        workspace.join("myx.config.toml"),
        format!(
            "[index]\nsources = [\"{}\"]\n\n[policy]\nmode = \"permissive\"\n",
            index_path.display()
        ),
    )
    .expect("write config");
}

#[test]
fn run_executes_http_tool_and_returns_json() {
    let tmp = TempDir::new().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    let packages = tmp.path().join("packages");
    std::fs::create_dir_all(&workspace).expect("create workspace");
    std::fs::create_dir_all(&packages).expect("create packages");

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local listener");
    let addr = listener.local_addr().expect("addr");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = [0u8; 2048];
        let _ = stream.read(&mut buf);
        let body = "ok-http";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
    });

    let profile = json!({
        "schema_version": "1",
        "identity": {
            "name": "github",
            "version": "0.1.0",
            "publisher": "example",
            "license": "Apache-2.0"
        },
        "metadata": {"description": "test", "homepage": "", "source": ""},
        "capabilities": ["test"],
        "instructions": {"system": "s", "usage": "u"},
        "tools": [
            {
                "name": "http_ping",
                "description": "http tool",
                "parameters": {
                    "type": "object",
                    "properties": { "query": {"type":"string"} },
                    "required": ["query"]
                },
                "tool_class": "http_api",
                "execution": {
                    "kind": "http",
                    "method": "GET",
                    "url": format!("http://127.0.0.1:{}/status?q={{{{query}}}}", addr.port()),
                    "headers": {},
                    "timeout_ms": 1000
                }
            }
        ],
        "permissions": {
            "network": ["127.0.0.1"],
            "secrets": [],
            "filesystem": {"read": [], "write": []},
            "subprocess": {"allowed_commands": [], "allowed_cwds": [], "allowed_env": [], "max_timeout_ms": 10000}
        },
        "compatibility": {"runtimes": ["openai","mcp","skill"], "platforms": ["darwin"]}
    });

    let pkg_dir = write_package(&packages, "github", "0.1.0", &profile);
    write_workspace_config(&workspace, "github", "0.1.0", &pkg_dir);

    let output = run_myx(
        &[
            "run",
            "github.http_ping",
            "--input",
            "{\"query\":\"rust\"}",
            "--json",
        ],
        &workspace,
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: Value = serde_json::from_slice(&output.stdout).expect("parse output");
    assert_eq!(payload["command"], "run");
    assert_eq!(payload["ok"], true);
    assert_eq!(payload["tool"], "http_ping");
    assert_eq!(payload["result"]["kind"], "http");
    assert_eq!(payload["result"]["status_code"], 200);
    assert_eq!(payload["result"]["body"], "ok-http");

    server.join().expect("join server");
}

#[test]
fn run_executes_subprocess_tool_and_returns_json() {
    let tmp = TempDir::new().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    let packages = tmp.path().join("packages");
    std::fs::create_dir_all(&workspace).expect("create workspace");
    std::fs::create_dir_all(&packages).expect("create packages");

    let profile = json!({
        "schema_version": "1",
        "identity": {
            "name": "local",
            "version": "0.1.0",
            "publisher": "example",
            "license": "Apache-2.0"
        },
        "metadata": {"description": "test", "homepage": "", "source": ""},
        "capabilities": ["test"],
        "instructions": {"system": "s", "usage": "u"},
        "tools": [
            {
                "name": "echo_text",
                "description": "echo tool",
                "parameters": {
                    "type": "object",
                    "properties": { "text": {"type":"string"} },
                    "required": ["text"]
                },
                "tool_class": "local_process",
                "execution": {
                    "kind": "subprocess",
                    "command": "echo",
                    "args": ["{{text}}"],
                    "cwd": ".",
                    "env_passthrough": ["HOME"],
                    "timeout_ms": 1000
                }
            }
        ],
        "permissions": {
            "network": [],
            "secrets": [],
            "filesystem": {"read": ["."], "write": ["."]},
            "subprocess": {
                "allowed_commands": ["echo"],
                "allowed_cwds": ["."],
                "allowed_env": ["HOME"],
                "max_timeout_ms": 2000
            }
        },
        "compatibility": {"runtimes": ["mcp"], "platforms": ["darwin"]}
    });

    let pkg_dir = write_package(&packages, "local", "0.1.0", &profile);
    write_workspace_config(&workspace, "local", "0.1.0", &pkg_dir);

    let output = run_myx(
        &[
            "run",
            "local.echo_text",
            "--input",
            "{\"text\":\"hello\"}",
            "--json",
        ],
        &workspace,
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: Value = serde_json::from_slice(&output.stdout).expect("parse output");
    assert_eq!(payload["command"], "run");
    assert_eq!(payload["ok"], true);
    assert_eq!(payload["tool"], "echo_text");
    assert_eq!(payload["result"]["kind"], "subprocess");
    assert_eq!(payload["result"]["exit_code"], 0);
    assert!(payload["result"]["stdout"]
        .as_str()
        .unwrap_or_default()
        .contains("hello"));
}

#[test]
fn run_returns_policy_exit_code_for_network_denial() {
    let tmp = TempDir::new().expect("tempdir");
    let workspace = tmp.path().join("workspace");
    let packages = tmp.path().join("packages");
    std::fs::create_dir_all(&workspace).expect("create workspace");
    std::fs::create_dir_all(&packages).expect("create packages");

    let profile = json!({
        "schema_version": "1",
        "identity": {
            "name": "networked",
            "version": "0.1.0",
            "publisher": "example",
            "license": "Apache-2.0"
        },
        "metadata": {"description": "test", "homepage": "", "source": ""},
        "capabilities": ["test"],
        "instructions": {"system": "s", "usage": "u"},
        "tools": [
            {
                "name": "http_ping",
                "description": "http tool",
                "parameters": {
                    "type": "object",
                    "properties": { "query": {"type":"string"} },
                    "required": ["query"]
                },
                "tool_class": "http_api",
                "execution": {
                    "kind": "http",
                    "method": "GET",
                    "url": "http://127.0.0.1:9999/status?q={{query}}",
                    "headers": {},
                    "timeout_ms": 1000
                }
            }
        ],
        "permissions": {
            "network": ["example.com"],
            "secrets": [],
            "filesystem": {"read": [], "write": []},
            "subprocess": {"allowed_commands": [], "allowed_cwds": [], "allowed_env": [], "max_timeout_ms": 10000}
        },
        "compatibility": {"runtimes": ["openai","mcp","skill"], "platforms": ["darwin"]}
    });

    let pkg_dir = write_package(&packages, "networked", "0.1.0", &profile);
    write_workspace_config(&workspace, "networked", "0.1.0", &pkg_dir);

    let output = run_myx(
        &[
            "run",
            "networked.http_ping",
            "--input",
            "{\"query\":\"rust\"}",
            "--json",
        ],
        &workspace,
    );
    assert!(!output.status.success(), "expected failure");
    assert_eq!(output.status.code(), Some(6));

    let stderr_payload: Value = serde_json::from_slice(&output.stderr).expect("parse stderr json");
    assert_eq!(stderr_payload["ok"], false);
    assert_eq!(stderr_payload["error"]["code"], 6);
    assert!(stderr_payload["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("NetworkDenied"));
}
