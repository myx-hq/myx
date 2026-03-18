use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};
use std::net::TcpListener;
use tempfile::TempDir;

fn write_runtime_config(dir: &Path, port: u16) -> PathBuf {
    let config_path = dir.join("runtime-config.json");
    let config = json!({
        "schema_version": 1,
        "identity": {
            "name": "e2e",
            "version": "0.1.0",
            "publisher": "myx",
            "license": "Apache-2.0"
        },
        "base_dir": ".",
        "permissions": {
            "network": ["127.0.0.1"],
            "secrets": [],
            "filesystem": {
                "read": ["."],
                "write": ["."]
            },
            "subprocess": {
                "allowed_commands": ["echo"],
                "allowed_cwds": ["."],
                "allowed_env": ["HOME"],
                "max_timeout_ms": 2000
            }
        },
        "tools": [
            {
                "name": "http_ping",
                "description": "http tool for e2e",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"}
                    },
                    "required": ["query"]
                },
                "tool_class": "http_api",
                "execution": {
                    "kind": "http",
                    "method": "GET",
                    "url": format!("http://127.0.0.1:{port}/status?q={{{{query}}}}"),
                    "headers": {},
                    "timeout_ms": 1000
                }
            },
            {
                "name": "echo_local",
                "description": "subprocess tool for e2e",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "message": {"type": "string"}
                    },
                    "required": ["message"]
                },
                "tool_class": "local_process",
                "execution": {
                    "kind": "subprocess",
                    "command": "echo",
                    "args": ["{{message}}"],
                    "cwd": ".",
                    "env_passthrough": ["HOME"],
                    "timeout_ms": 1000
                }
            }
        ]
    });
    std::fs::write(
        &config_path,
        serde_json::to_vec_pretty(&config).expect("serialize config"),
    )
    .expect("write runtime config");
    config_path
}

fn run_runner(dir: &Path, config: &Path, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_myx-mcp-runner"));
    command.current_dir(dir).arg("--config").arg(config);
    for arg in args {
        command.arg(arg);
    }
    command.output().expect("run mcp runner")
}

#[test]
fn mcp_runner_healthcheck_and_tool_invokes_work_e2e() {
    let tmp = TempDir::new().expect("tempdir");
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local http listener");
    let addr = listener.local_addr().expect("listener addr");

    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept connection");
        let mut request = [0u8; 2048];
        let _ = stream.read(&mut request);
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

    let config_path = write_runtime_config(tmp.path(), addr.port());

    let health = run_runner(tmp.path(), &config_path, &["--healthcheck"]);
    assert!(
        health.status.success(),
        "healthcheck stderr: {}",
        String::from_utf8_lossy(&health.stderr)
    );
    let health_payload: Value =
        serde_json::from_slice(&health.stdout).expect("parse healthcheck json");
    assert_eq!(health_payload["ok"], true);

    let http = run_runner(
        tmp.path(),
        &config_path,
        &[
            "--invoke",
            "http_ping",
            "--args-json",
            "{\"query\":\"repo\"}",
        ],
    );
    assert!(
        http.status.success(),
        "http invoke stderr: {}",
        String::from_utf8_lossy(&http.stderr)
    );
    let http_payload: Value = serde_json::from_slice(&http.stdout).expect("parse http json");
    assert_eq!(http_payload["ok"], true);
    assert_eq!(http_payload["result"]["kind"], "http");
    assert_eq!(http_payload["result"]["status_code"], 200);
    assert_eq!(http_payload["result"]["body"], "ok-http");

    server.join().expect("join http server");

    let subprocess = run_runner(
        tmp.path(),
        &config_path,
        &[
            "--invoke",
            "echo_local",
            "--args-json",
            "{\"message\":\"hello\"}",
        ],
    );
    assert!(
        subprocess.status.success(),
        "subprocess invoke stderr: {}",
        String::from_utf8_lossy(&subprocess.stderr)
    );
    let subprocess_payload: Value =
        serde_json::from_slice(&subprocess.stdout).expect("parse subprocess json");
    assert_eq!(subprocess_payload["ok"], true);
    assert_eq!(subprocess_payload["result"]["kind"], "subprocess");
    assert!(subprocess_payload["result"]["stdout"]
        .as_str()
        .unwrap_or_default()
        .contains("hello"));
}
