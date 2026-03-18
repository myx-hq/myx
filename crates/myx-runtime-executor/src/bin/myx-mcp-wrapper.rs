use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use myx_runtime_executor::{execute_tool_call, load_runtime_config, validate_runtime_config};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Parser)]
#[command(name = "myx-mcp-wrapper")]
#[command(about = "myx deterministic MCP wrapper runtime")]
struct Cli {
    #[arg(long, value_name = "PATH")]
    config: PathBuf,
    #[arg(long)]
    invoke: Option<String>,
    #[arg(long, value_name = "JSON")]
    args_json: Option<String>,
    #[arg(long, default_value_t = false)]
    healthcheck: bool,
}

#[derive(Debug, Deserialize)]
struct RpcRequest {
    #[serde(default)]
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

fn main() {
    if let Err(err) = run() {
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": false,
                "error": err.to_string()
            }))
            .unwrap_or_else(|_| "{\"ok\":false,\"error\":\"runtime failure\"}".to_string())
        );
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = load_runtime_config(&cli.config)?;
    let base_dir = config_base_dir(&cli.config);
    validate_runtime_config(&config, &base_dir)?;

    if cli.healthcheck {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": true,
                "identity": config.identity,
                "tools": config.tools.len()
            }))?
        );
        return Ok(());
    }

    if let Some(tool_name) = cli.invoke {
        let args = if let Some(raw) = cli.args_json {
            serde_json::from_str::<Value>(&raw).context("invalid --args-json payload")?
        } else {
            json!({})
        };
        let result = execute_tool_call(&config, &base_dir, &tool_name, &args)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": true,
                "tool": tool_name,
                "result": result
            }))?
        );
        return Ok(());
    }

    serve_loop(&config, &base_dir)
}

fn config_base_dir(path: &Path) -> PathBuf {
    path.parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn serve_loop(config: &myx_runtime_executor::RuntimeConfig, base_dir: &Path) -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request = match serde_json::from_str::<RpcRequest>(trimmed) {
            Ok(req) => req,
            Err(err) => {
                let payload = json!({
                    "id": Value::Null,
                    "ok": false,
                    "error": format!("invalid request json: {}", err)
                });
                writeln!(stdout, "{}", serde_json::to_string(&payload)?)?;
                stdout.flush()?;
                continue;
            }
        };

        let id = request.id.clone();
        let response = match request.method.as_str() {
            "initialize" => json!({
                "id": id,
                "ok": true,
                "result": {
                    "identity": config.identity,
                    "capabilities": {
                        "tools": true
                    }
                }
            }),
            "tools/list" => json!({
                "id": id,
                "ok": true,
                "result": config.tools.iter().map(|tool| {
                    json!({
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": tool.parameters,
                        "tool_class": tool.tool_class
                    })
                }).collect::<Vec<_>>()
            }),
            "tools/call" => handle_call_request(id, config, base_dir, &request.params)?,
            "health" => json!({
                "id": id,
                "ok": true,
                "result": {
                    "status": "ready",
                    "tool_count": config.tools.len()
                }
            }),
            "shutdown" => {
                let payload = json!({
                    "id": id,
                    "ok": true,
                    "result": {"status": "bye"}
                });
                writeln!(stdout, "{}", serde_json::to_string(&payload)?)?;
                stdout.flush()?;
                break;
            }
            _ => json!({
                "id": id,
                "ok": false,
                "error": format!("unsupported method '{}'", request.method)
            }),
        };

        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }

    Ok(())
}

fn handle_call_request(
    id: Value,
    config: &myx_runtime_executor::RuntimeConfig,
    base_dir: &Path,
    params: &Value,
) -> Result<Value> {
    let params_obj = params
        .as_object()
        .ok_or_else(|| anyhow!("tools/call params must be an object"))?;
    let name = params_obj
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("tools/call params.name is required"))?;
    let args = params_obj
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    match execute_tool_call(config, base_dir, name, &args) {
        Ok(result) => Ok(json!({
            "id": id,
            "ok": true,
            "result": result
        })),
        Err(err) => Ok(json!({
            "id": id,
            "ok": false,
            "error": err.to_string()
        })),
    }
}
