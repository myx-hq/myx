use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use myx_core::load_config;
use myx_policy::{evaluate_install_policy, Decision};
use serde_json::{json, Value};

use crate::exit::{fail, CliExit};
use crate::non_interactive::resolve_non_interactive_mode;
use crate::util::resolve_bundle;

fn parse_run_target(target: &str) -> Result<(&str, &str), CliExit> {
    let (package, tool) = target.rsplit_once('.').ok_or_else(|| {
        fail(
            2,
            "invalid run target; expected '<package>.<tool>' (example: github.search_repositories)",
        )
    })?;
    if package.trim().is_empty() || tool.trim().is_empty() {
        return Err(fail(
            2,
            "invalid run target; package and tool must both be non-empty",
        ));
    }
    Ok((package, tool))
}

fn parse_input_json(input: Option<&str>) -> Result<Value, CliExit> {
    let value = match input {
        Some(raw) => serde_json::from_str::<Value>(raw)
            .map_err(|e| fail(3, format!("invalid --input JSON payload: {e}")))?,
        None => json!({}),
    };
    if !value.is_object() {
        return Err(fail(3, "--input payload must be a JSON object"));
    }
    Ok(value)
}

fn validate_input_against_schema(schema: &Value, input: &Value) -> Result<(), CliExit> {
    if !schema.is_object() {
        return Ok(());
    }
    if let Some(schema_type) = schema.get("type").and_then(Value::as_str) {
        if schema_type != "object" {
            return Err(fail(
                3,
                format!(
                    "tool schema type '{}' is not supported by run input validator",
                    schema_type
                ),
            ));
        }
    }
    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        let input_obj = input
            .as_object()
            .ok_or_else(|| fail(3, "--input payload must be a JSON object"))?;
        for key in required.iter().filter_map(Value::as_str) {
            if !input_obj.contains_key(key) {
                return Err(fail(
                    3,
                    format!("input validation failed: missing required field '{}'", key),
                ));
            }
        }
    }
    Ok(())
}

fn classify_runtime_error(message: &str) -> i32 {
    if message.contains("not allowed")
        || message.contains("outside permissions")
        || message.contains("permissions.")
    {
        return 6;
    }
    1
}

pub fn command_run(
    target: &str,
    input: Option<String>,
    config_override: Option<PathBuf>,
    non_interactive_flag: bool,
    json_output: bool,
) -> Result<(), CliExit> {
    let cwd = std::env::current_dir().map_err(|e| fail(1, e))?;
    let config = load_config(config_override.as_deref(), &cwd).map_err(|e| fail(1, e))?;
    let (package_spec, tool_name) = parse_run_target(target)?;
    let (_resolved, bundle) =
        resolve_bundle(package_spec, &config, &cwd).map_err(|e| fail(4, e))?;
    let non_interactive_mode =
        resolve_non_interactive_mode(non_interactive_flag).map_err(|e| fail(2, e))?;

    let policy_result = evaluate_install_policy(
        &config.policy,
        &bundle.profile.permissions,
        non_interactive_mode.enabled,
    )
    .map_err(|e| fail(1, e))?;
    if matches!(policy_result.decision, Decision::Deny) {
        if non_interactive_mode.enabled {
            return Err(fail(
                6,
                format!(
                    "{} (non-interactive mode: {})",
                    policy_result.reason, non_interactive_mode.reason
                ),
            ));
        }
        return Err(fail(6, policy_result.reason));
    }

    let tool = bundle
        .profile
        .tools
        .iter()
        .find(|t| t.name == tool_name)
        .ok_or_else(|| {
            fail(
                3,
                format!(
                    "tool '{}' not found in package '{}'",
                    tool_name, bundle.manifest.name
                ),
            )
        })?;

    myx_runtime_executor::validate_execution(&tool.execution).map_err(|e| fail(3, e))?;
    let input_value = parse_input_json(input.as_deref())?;
    validate_input_against_schema(&tool.parameters, &input_value)?;

    let start = Instant::now();
    let result = myx_runtime_executor::execute_tool(
        tool,
        &bundle.profile.permissions,
        &bundle.package_dir,
        &input_value,
    )
    .map_err(|e| {
        let msg = e.to_string();
        fail(classify_runtime_error(&msg), msg)
    })?;
    let duration_ms = start.elapsed().as_millis() as u64;

    if json_output {
        let out = json!({
            "command": "run",
            "ok": true,
            "package": {
                "name": bundle.manifest.name,
                "version": bundle.manifest.version
            },
            "tool": tool.name,
            "duration_ms": duration_ms,
            "result": result
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out).map_err(|e| fail(1, e))?
        );
        return Ok(());
    }

    println!("executed {}.{}", bundle.manifest.name, tool.name);
    println!("kind: {}", result.kind);
    if let Some(status_code) = result.status_code {
        println!("status_code: {}", status_code);
    }
    if let Some(exit_code) = result.exit_code {
        println!("exit_code: {}", exit_code);
    }
    if let Some(stdout) = &result.stdout {
        if !stdout.trim().is_empty() {
            println!("stdout:\n{}", stdout);
        }
    }
    if let Some(body) = &result.body {
        if !body.trim().is_empty() {
            println!("body:\n{}", body);
        }
    }
    println!("duration_ms: {}", duration_ms);

    Ok(())
}
