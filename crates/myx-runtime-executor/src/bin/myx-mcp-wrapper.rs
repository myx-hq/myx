use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};
use myx_runtime_executor::{
    execute_tool_call, load_runtime_config, validate_runtime_config, RuntimeConfig,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ProtocolMode {
    Mcp,
    Simple,
}

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
    #[arg(long, value_enum, default_value_t = ProtocolMode::Mcp)]
    protocol: ProtocolMode,
}

#[derive(Debug, Deserialize)]
struct SimpleRpcRequest {
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
    let base_dir = config_base_dir(&cli.config, &config);
    validate_runtime_config(&config, &base_dir)?;

    if cli.healthcheck {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": true,
                "identity": config.identity,
                "tools": config.tools.len(),
                "protocol_default": "mcp",
                "protocol_modes": ["mcp", "simple"]
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

    match cli.protocol {
        ProtocolMode::Mcp => serve_mcp_loop(&config, &base_dir),
        ProtocolMode::Simple => serve_simple_loop(&config, &base_dir),
    }
}

fn config_base_dir(path: &Path, config: &RuntimeConfig) -> PathBuf {
    let config_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let requested = config.base_dir.as_deref().unwrap_or(".");
    let requested_path = Path::new(requested);
    if requested_path.is_absolute() {
        requested_path.to_path_buf()
    } else {
        config_dir.join(requested_path)
    }
}

fn serve_simple_loop(config: &myx_runtime_executor::RuntimeConfig, base_dir: &Path) -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let request = match serde_json::from_str::<SimpleRpcRequest>(trimmed) {
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

        let (response, should_exit) = handle_simple_request(&request, config, base_dir)?;
        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
        if should_exit {
            break;
        }
    }

    Ok(())
}

fn handle_simple_request(
    request: &SimpleRpcRequest,
    config: &myx_runtime_executor::RuntimeConfig,
    base_dir: &Path,
) -> Result<(Value, bool)> {
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
        "tools/call" => handle_simple_call_request(id, config, base_dir, &request.params)?,
        "health" => json!({
            "id": id,
            "ok": true,
            "result": {
                "status": "ready",
                "tool_count": config.tools.len()
            }
        }),
        "shutdown" => {
            return Ok((
                json!({
                    "id": id,
                    "ok": true,
                    "result": {"status": "bye"}
                }),
                true,
            ));
        }
        _ => json!({
            "id": id,
            "ok": false,
            "error": format!("unsupported method '{}'", request.method)
        }),
    };
    Ok((response, false))
}

fn handle_simple_call_request(
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

fn serve_mcp_loop(config: &myx_runtime_executor::RuntimeConfig, base_dir: &Path) -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());

    loop {
        let payload = match read_framed_message(&mut reader) {
            Ok(Some(payload)) => payload,
            Ok(None) => break,
            Err(err) => {
                let parse_error =
                    jsonrpc_error(Value::Null, -32700, format!("invalid MCP frame: {}", err));
                write_framed_message(&mut writer, &parse_error)?;
                continue;
            }
        };

        let (response, should_exit) = handle_mcp_payload(config, base_dir, &payload)?;
        if let Some(response) = response {
            write_framed_message(&mut writer, &response)?;
        }
        if should_exit {
            break;
        }
    }

    Ok(())
}

fn handle_mcp_payload(
    config: &myx_runtime_executor::RuntimeConfig,
    base_dir: &Path,
    payload: &[u8],
) -> Result<(Option<Value>, bool)> {
    let request = match serde_json::from_slice::<Value>(payload) {
        Ok(request) => request,
        Err(err) => {
            return Ok((
                Some(jsonrpc_error(
                    Value::Null,
                    -32700,
                    format!("parse error: {}", err),
                )),
                false,
            ));
        }
    };
    handle_mcp_request(config, base_dir, &request)
}

fn handle_mcp_request(
    config: &myx_runtime_executor::RuntimeConfig,
    base_dir: &Path,
    request: &Value,
) -> Result<(Option<Value>, bool)> {
    let request_obj = match request.as_object() {
        Some(request_obj) => request_obj,
        None => {
            return Ok((
                Some(jsonrpc_error(Value::Null, -32600, "invalid request")),
                false,
            ))
        }
    };

    let id = request_obj.get("id").cloned();
    let method = match request_obj.get("method").and_then(Value::as_str) {
        Some(method) => method,
        None => {
            return Ok((
                id.map(|id| jsonrpc_error(id, -32600, "invalid request: method is required")),
                false,
            ));
        }
    };
    let params = request_obj
        .get("params")
        .cloned()
        .unwrap_or_else(|| json!({}));

    match method {
        "initialize" => {
            let protocol_version = params
                .as_object()
                .and_then(|p| p.get("protocolVersion"))
                .and_then(Value::as_str)
                .unwrap_or("2024-11-05");
            let result = json!({
                "protocolVersion": protocol_version,
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    }
                },
                "serverInfo": {
                    "name": config.identity.name,
                    "version": config.identity.version
                }
            });
            Ok((id.map(|id| jsonrpc_success(id, result)), false))
        }
        "tools/list" => {
            let tools = config
                .tools
                .iter()
                .map(|tool| {
                    json!({
                        "name": tool.name,
                        "description": tool.description,
                        "inputSchema": tool.parameters
                    })
                })
                .collect::<Vec<_>>();
            Ok((
                id.map(|id| jsonrpc_success(id, json!({ "tools": tools }))),
                false,
            ))
        }
        "tools/call" => {
            let params_obj = match params.as_object() {
                Some(params_obj) => params_obj,
                None => {
                    return Ok((
                        id.map(|id| jsonrpc_error(id, -32602, "invalid params: expected object")),
                        false,
                    ));
                }
            };
            let name = match params_obj.get("name").and_then(Value::as_str) {
                Some(name) => name,
                None => {
                    return Ok((
                        id.map(|id| jsonrpc_error(id, -32602, "invalid params: name is required")),
                        false,
                    ));
                }
            };
            let args = params_obj
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));

            let response = match execute_tool_call(config, base_dir, name, &args) {
                Ok(result) => {
                    let structured = serde_json::to_value(result)?;
                    let text = serde_json::to_string(&structured)
                        .context("failed to serialize tools/call response")?;
                    let payload = json!({
                        "content": [
                            {
                                "type": "text",
                                "text": text
                            }
                        ],
                        "structuredContent": structured,
                        "isError": false
                    });
                    id.map(|id| jsonrpc_success(id, payload))
                }
                Err(err) => {
                    let payload = json!({
                        "content": [
                            {
                                "type": "text",
                                "text": err.to_string()
                            }
                        ],
                        "isError": true
                    });
                    id.map(|id| jsonrpc_success(id, payload))
                }
            };

            Ok((response, false))
        }
        "ping" => Ok((id.map(|id| jsonrpc_success(id, json!({}))), false)),
        "shutdown" => Ok((id.map(|id| jsonrpc_success(id, json!({}))), true)),
        "notifications/initialized" => Ok((None, false)),
        "exit" => Ok((None, true)),
        _ => Ok((
            id.map(|id| jsonrpc_error(id, -32601, format!("method not found: {}", method))),
            false,
        )),
    }
}

fn read_framed_message<R: BufRead>(reader: &mut R) -> Result<Option<Vec<u8>>> {
    let mut content_length: Option<usize> = None;
    let mut saw_any_header = false;

    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            if !saw_any_header {
                return Ok(None);
            }
            return Err(anyhow!("unexpected EOF while reading MCP headers"));
        }
        saw_any_header = true;

        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        let (name, value) = trimmed
            .split_once(':')
            .ok_or_else(|| anyhow!("invalid MCP header '{}'", trimmed))?;
        if name.eq_ignore_ascii_case("content-length") {
            content_length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .context("invalid Content-Length header")?,
            );
        }
    }

    let content_length = content_length.ok_or_else(|| anyhow!("missing Content-Length header"))?;
    let mut payload = vec![0u8; content_length];
    reader.read_exact(&mut payload)?;
    Ok(Some(payload))
}

fn write_framed_message<W: Write>(writer: &mut W, payload: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(payload).context("failed to serialize MCP response payload")?;
    write!(writer, "Content-Length: {}\r\n\r\n", bytes.len())?;
    writer.write_all(&bytes)?;
    writer.flush()?;
    Ok(())
}

fn jsonrpc_success(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn jsonrpc_error(id: Value, code: i64, message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message.into()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use myx_core::{Identity, Permissions, ToolClass, ToolDefinition, ToolExecution};
    use std::collections::BTreeMap;
    use std::io::Cursor;

    fn sample_config() -> myx_runtime_executor::RuntimeConfig {
        myx_runtime_executor::RuntimeConfig {
            schema_version: 1,
            identity: Identity {
                name: "myx-sample".to_string(),
                version: "0.1.0".to_string(),
                publisher: "myx".to_string(),
                license: "Apache-2.0".to_string(),
            },
            base_dir: Some(".".to_string()),
            permissions: Permissions::default(),
            tools: vec![ToolDefinition {
                name: "search".to_string(),
                description: "Search repositories".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"}
                    },
                    "required": ["query"]
                }),
                tool_class: ToolClass::HttpApi,
                execution: ToolExecution::Http {
                    method: "GET".to_string(),
                    url: "https://api.example.com/search?q={{query}}".to_string(),
                    headers: BTreeMap::new(),
                    timeout_ms: Some(1000),
                },
            }],
        }
    }

    #[test]
    fn simple_mode_contract_is_preserved() {
        let config = sample_config();
        let request = SimpleRpcRequest {
            id: json!(1),
            method: "tools/list".to_string(),
            params: json!({}),
        };

        let (response, should_exit) =
            handle_simple_request(&request, &config, Path::new(".")).expect("simple request");
        assert!(!should_exit);
        assert_eq!(response["ok"], true);
        assert_eq!(response["id"], 1);
        assert!(response["result"].is_array());
    }

    #[test]
    fn mcp_framing_round_trip() {
        let mut wire = Vec::new();
        let message = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "ping"
        });
        write_framed_message(&mut wire, &message).expect("write frame");

        let mut cursor = Cursor::new(wire);
        let payload = read_framed_message(&mut cursor)
            .expect("read frame")
            .expect("payload");
        let decoded: Value = serde_json::from_slice(&payload).expect("decode payload");
        assert_eq!(decoded["method"], "ping");
        assert_eq!(decoded["id"], 1);
    }

    #[test]
    fn mcp_initialize_fixture_smoke_test() {
        let config = sample_config();
        let payload = serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05"
            }
        }))
        .expect("serialize initialize request");

        let (response, should_exit) =
            handle_mcp_payload(&config, Path::new("."), &payload).expect("handle payload");
        assert!(!should_exit);
        let response = response.expect("initialize response");
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 42);
        assert_eq!(response["result"]["serverInfo"]["name"], "myx-sample");
        assert_eq!(
            response["result"]["capabilities"]["tools"]["listChanged"],
            false
        );
    }

    #[test]
    fn mcp_tools_list_uses_input_schema() {
        let config = sample_config();
        let request = json!({
            "jsonrpc": "2.0",
            "id": "req-1",
            "method": "tools/list",
            "params": {}
        });

        let (response, should_exit) =
            handle_mcp_request(&config, Path::new("."), &request).expect("handle request");
        assert!(!should_exit);
        let response = response.expect("tools/list response");
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], "req-1");
        assert_eq!(response["result"]["tools"][0]["name"], "search");
        assert!(response["result"]["tools"][0]["inputSchema"]
            .as_object()
            .is_some());
    }

    #[test]
    fn mcp_exit_notification_returns_no_payload_and_exits() {
        let config = sample_config();
        let request = json!({
            "jsonrpc": "2.0",
            "method": "exit",
            "params": {}
        });

        let (response, should_exit) =
            handle_mcp_request(&config, Path::new("."), &request).expect("handle exit");
        assert!(should_exit);
        assert!(response.is_none());
    }
}
