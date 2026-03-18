use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use myx_core::{Identity, Permissions, ToolDefinition, ToolExecution};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub schema_version: u32,
    pub identity: Identity,
    #[serde(default)]
    pub base_dir: Option<String>,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOutput {
    pub kind: String,
    #[serde(default)]
    pub status_code: Option<u16>,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub stdout: Option<String>,
    #[serde(default)]
    pub stderr: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
}

pub fn load_runtime_config(path: &Path) -> Result<RuntimeConfig> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read runtime config '{}'", path.display()))?;
    let config: RuntimeConfig = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse runtime config '{}'", path.display()))?;
    Ok(config)
}

pub fn validate_runtime_config(config: &RuntimeConfig, base_dir: &Path) -> Result<()> {
    if config.tools.is_empty() {
        anyhow::bail!("runtime config contains no tools");
    }

    for tool in &config.tools {
        validate_execution(&tool.execution)?;
        validate_tool_permissions(tool, &config.permissions, base_dir)?;
    }

    Ok(())
}

pub fn execute_tool_call(
    config: &RuntimeConfig,
    base_dir: &Path,
    tool_name: &str,
    args: &Value,
) -> Result<ExecutionOutput> {
    let tool = config
        .tools
        .iter()
        .find(|t| t.name == tool_name)
        .ok_or_else(|| anyhow!("tool '{}' not found in runtime config", tool_name))?;
    execute_tool(tool, &config.permissions, base_dir, args)
}

pub fn execute_tool(
    tool: &ToolDefinition,
    permissions: &Permissions,
    base_dir: &Path,
    args: &Value,
) -> Result<ExecutionOutput> {
    validate_execution(&tool.execution)?;
    validate_tool_permissions(tool, permissions, base_dir)?;

    let empty = Map::new();
    let args_obj = args.as_object().unwrap_or(&empty);

    match &tool.execution {
        ToolExecution::Http {
            method,
            url,
            headers,
            timeout_ms,
        } => execute_http(method, url, headers, *timeout_ms, permissions, args_obj),
        ToolExecution::Subprocess {
            command,
            args,
            cwd,
            env_passthrough,
            timeout_ms,
        } => execute_subprocess(
            command,
            args,
            cwd.as_deref(),
            env_passthrough,
            *timeout_ms,
            permissions,
            base_dir,
            args_obj,
        ),
    }
}

pub fn validate_execution(exec: &ToolExecution) -> Result<()> {
    match exec {
        ToolExecution::Http { method, url, .. } => {
            if method.trim().is_empty() || url.trim().is_empty() {
                anyhow::bail!("http execution requires non-empty method and url");
            }
            Ok(())
        }
        ToolExecution::Subprocess {
            command,
            timeout_ms,
            ..
        } => {
            if command.contains(' ') {
                anyhow::bail!("subprocess command must be a single executable token");
            }
            if command.contains('/') {
                anyhow::bail!("subprocess command must be a command name, not a path");
            }
            if timeout_ms.is_none() {
                anyhow::bail!("subprocess execution requires timeout_ms");
            }
            Ok(())
        }
    }
}

fn validate_tool_permissions(
    tool: &ToolDefinition,
    permissions: &Permissions,
    base_dir: &Path,
) -> Result<()> {
    validate_subprocess_allowlists(permissions)?;

    match &tool.execution {
        ToolExecution::Http { url, .. } => {
            let host = extract_host_from_template(url).with_context(|| {
                format!(
                    "tool '{}' has invalid http url template '{}'",
                    tool.name, url
                )
            })?;
            if !contains_or_wildcard(&permissions.network, &host) {
                anyhow::bail!(
                    "tool '{}' network host '{}' not allowed by permissions.network",
                    tool.name,
                    host
                );
            }
        }
        ToolExecution::Subprocess {
            command,
            cwd,
            env_passthrough,
            timeout_ms,
            ..
        } => {
            if !contains_exact(&permissions.subprocess.allowed_commands, command) {
                anyhow::bail!(
                    "tool '{}' command '{}' is not in permissions.subprocess.allowed_commands",
                    tool.name,
                    command
                );
            }

            let resolved_cwd = resolve_cwd(base_dir, cwd.as_deref());
            if !cwd_is_allowed(
                base_dir,
                &resolved_cwd,
                &permissions.subprocess.allowed_cwds,
            ) {
                anyhow::bail!(
                    "tool '{}' cwd '{}' is not in permissions.subprocess.allowed_cwds",
                    tool.name,
                    resolved_cwd.display()
                );
            }
            if !cwd_within_filesystem_bounds(base_dir, &resolved_cwd, permissions) {
                anyhow::bail!(
                    "tool '{}' cwd '{}' is outside permissions.filesystem bounds",
                    tool.name,
                    resolved_cwd.display()
                );
            }

            for env_key in env_passthrough {
                if !contains_exact(&permissions.subprocess.allowed_env, env_key) {
                    anyhow::bail!(
                        "tool '{}' env '{}' is not in permissions.subprocess.allowed_env",
                        tool.name,
                        env_key
                    );
                }
            }

            let timeout = timeout_ms.unwrap_or_default();
            let max_timeout = permissions
                .subprocess
                .max_timeout_ms
                .ok_or_else(|| anyhow!("permissions.subprocess.max_timeout_ms is required"))?;
            if timeout > max_timeout {
                anyhow::bail!(
                    "tool '{}' timeout_ms {} exceeds permissions.subprocess.max_timeout_ms {}",
                    tool.name,
                    timeout,
                    max_timeout
                );
            }
        }
    }

    Ok(())
}

fn execute_http(
    method: &str,
    url_template: &str,
    headers: &std::collections::BTreeMap<String, String>,
    timeout_ms: Option<u64>,
    permissions: &Permissions,
    args: &Map<String, Value>,
) -> Result<ExecutionOutput> {
    let rendered_url = render_template(url_template, args);
    let parsed = url::Url::parse(&rendered_url)
        .with_context(|| format!("invalid url '{}'", rendered_url))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow!("http url '{}' has no host", rendered_url))?;
    if !contains_or_wildcard(&permissions.network, host) {
        anyhow::bail!("network host '{}' not allowed by permissions.network", host);
    }

    let timeout = Duration::from_millis(timeout_ms.unwrap_or(10_000));
    let config = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(timeout))
        .build();
    let agent: ureq::Agent = config.into();

    let rendered_headers = headers
        .iter()
        .map(|(k, v)| (k.clone(), render_template(v, args)))
        .collect::<Vec<_>>();

    let method_upper = method.trim().to_ascii_uppercase();
    let request_body = Value::Object(args.clone()).to_string();

    let mut response = match method_upper.as_str() {
        "GET" => {
            let mut req = agent.get(&rendered_url);
            for (k, v) in &rendered_headers {
                req = req.header(k, v);
            }
            req.call()
        }
        "DELETE" => {
            let mut req = agent.delete(&rendered_url);
            for (k, v) in &rendered_headers {
                req = req.header(k, v);
            }
            req.call()
        }
        "HEAD" => {
            let mut req = agent.head(&rendered_url);
            for (k, v) in &rendered_headers {
                req = req.header(k, v);
            }
            req.call()
        }
        "OPTIONS" => {
            let mut req = agent.options(&rendered_url);
            for (k, v) in &rendered_headers {
                req = req.header(k, v);
            }
            req.call()
        }
        "POST" => {
            let mut req = agent
                .post(&rendered_url)
                .header("content-type", "application/json");
            for (k, v) in &rendered_headers {
                req = req.header(k, v);
            }
            req.send(&request_body)
        }
        "PUT" => {
            let mut req = agent
                .put(&rendered_url)
                .header("content-type", "application/json");
            for (k, v) in &rendered_headers {
                req = req.header(k, v);
            }
            req.send(&request_body)
        }
        "PATCH" => {
            let mut req = agent
                .patch(&rendered_url)
                .header("content-type", "application/json");
            for (k, v) in &rendered_headers {
                req = req.header(k, v);
            }
            req.send(&request_body)
        }
        _ => {
            anyhow::bail!("unsupported http method '{}'", method);
        }
    }
    .with_context(|| {
        format!(
            "http execution failed for '{} {}'",
            method_upper, rendered_url
        )
    })?;

    let status_code = response.status().as_u16();
    let body = response.body_mut().read_to_string().unwrap_or_default();

    Ok(ExecutionOutput {
        kind: "http".to_string(),
        status_code: Some(status_code),
        exit_code: None,
        stdout: None,
        stderr: None,
        body: Some(body),
    })
}

#[allow(clippy::too_many_arguments)]
fn execute_subprocess(
    command: &str,
    args_template: &[String],
    cwd: Option<&str>,
    env_passthrough: &[String],
    timeout_ms: Option<u64>,
    permissions: &Permissions,
    base_dir: &Path,
    args: &Map<String, Value>,
) -> Result<ExecutionOutput> {
    if !contains_exact(&permissions.subprocess.allowed_commands, command) {
        anyhow::bail!(
            "subprocess command '{}' not allowed by permissions.subprocess.allowed_commands",
            command
        );
    }

    let resolved_cwd = resolve_cwd(base_dir, cwd);
    if !cwd_is_allowed(
        base_dir,
        &resolved_cwd,
        &permissions.subprocess.allowed_cwds,
    ) {
        anyhow::bail!(
            "subprocess cwd '{}' not allowed by permissions.subprocess.allowed_cwds",
            resolved_cwd.display()
        );
    }
    if !cwd_within_filesystem_bounds(base_dir, &resolved_cwd, permissions) {
        anyhow::bail!(
            "subprocess cwd '{}' is outside permissions.filesystem bounds",
            resolved_cwd.display()
        );
    }

    for env_key in env_passthrough {
        if !contains_exact(&permissions.subprocess.allowed_env, env_key) {
            anyhow::bail!(
                "subprocess env '{}' not allowed by permissions.subprocess.allowed_env",
                env_key
            );
        }
    }

    let timeout_value =
        timeout_ms.ok_or_else(|| anyhow!("subprocess execution requires timeout_ms"))?;
    let max_timeout = permissions
        .subprocess
        .max_timeout_ms
        .ok_or_else(|| anyhow!("permissions.subprocess.max_timeout_ms is required"))?;
    if timeout_value > max_timeout {
        anyhow::bail!(
            "subprocess timeout_ms {} exceeds permissions.subprocess.max_timeout_ms {}",
            timeout_value,
            max_timeout
        );
    }

    let rendered_args = args_template
        .iter()
        .map(|a| render_template(a, args))
        .collect::<Vec<_>>();

    let mut process = Command::new(command);
    process
        .args(rendered_args)
        .current_dir(&resolved_cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_clear();

    if let Ok(path) = std::env::var("PATH") {
        process.env("PATH", path);
    }
    for env_key in env_passthrough {
        if let Ok(value) = std::env::var(env_key) {
            process.env(env_key, value);
        }
    }

    let mut child = process.spawn().with_context(|| {
        format!(
            "failed to spawn subprocess '{}' in cwd '{}'",
            command,
            resolved_cwd.display()
        )
    })?;

    let timeout = Duration::from_millis(timeout_value);
    let started = Instant::now();

    loop {
        match child
            .try_wait()
            .context("failed polling subprocess status")?
        {
            Some(status) => {
                let (stdout, stderr) = read_child_pipes(&mut child)?;
                return Ok(ExecutionOutput {
                    kind: "subprocess".to_string(),
                    status_code: None,
                    exit_code: status.code(),
                    stdout: Some(stdout),
                    stderr: Some(stderr),
                    body: None,
                });
            }
            None => {
                if started.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = read_child_pipes(&mut child);
                    anyhow::bail!(
                        "subprocess timed out after {}ms for command '{}'",
                        timeout_value,
                        command
                    );
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

fn read_child_pipes(child: &mut Child) -> Result<(String, String)> {
    let mut stdout = String::new();
    if let Some(mut out) = child.stdout.take() {
        out.read_to_string(&mut stdout)
            .context("failed reading subprocess stdout")?;
    }

    let mut stderr = String::new();
    if let Some(mut err) = child.stderr.take() {
        err.read_to_string(&mut stderr)
            .context("failed reading subprocess stderr")?;
    }

    Ok((stdout, stderr))
}

fn render_template(template: &str, args: &Map<String, Value>) -> String {
    let mut out = template.to_string();
    for (k, v) in args {
        let placeholder = format!("{{{{{k}}}}}");
        let value = match v {
            Value::String(s) => s.clone(),
            _ => v.to_string(),
        };
        out = out.replace(&placeholder, &value);
    }
    out
}

fn contains_or_wildcard(allowed: &[String], value: &str) -> bool {
    allowed.iter().any(|a| a == "*" || a == value)
}

fn contains_exact(allowed: &[String], value: &str) -> bool {
    allowed.iter().any(|a| a == value)
}

fn contains_wildcard(allowed: &[String]) -> bool {
    allowed.iter().any(|a| a == "*")
}

fn validate_subprocess_allowlists(permissions: &Permissions) -> Result<()> {
    if contains_wildcard(&permissions.subprocess.allowed_commands) {
        anyhow::bail!(
            "permissions.subprocess.allowed_commands must use exact entries; wildcard '*' is not allowed in MVP"
        );
    }
    if contains_wildcard(&permissions.subprocess.allowed_cwds) {
        anyhow::bail!(
            "permissions.subprocess.allowed_cwds must use exact entries; wildcard '*' is not allowed in MVP"
        );
    }
    if contains_wildcard(&permissions.subprocess.allowed_env) {
        anyhow::bail!(
            "permissions.subprocess.allowed_env must use exact entries; wildcard '*' is not allowed in MVP"
        );
    }
    Ok(())
}

fn extract_host_from_template(url_template: &str) -> Result<String> {
    let parsed = match url::Url::parse(url_template) {
        Ok(url) => url,
        Err(_) => {
            let sanitized = sanitize_template_placeholders(url_template);
            url::Url::parse(&sanitized)?
        }
    };
    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow!("url '{}' has no host", url_template))?;
    Ok(host.to_string())
}

fn sanitize_template_placeholders(input: &str) -> String {
    let mut out = String::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'}' && bytes[i + 1] == b'}') {
                i += 1;
            }
            if i + 1 < bytes.len() {
                i += 2;
            }
            out.push('x');
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

fn resolve_cwd(base_dir: &Path, cwd: Option<&str>) -> PathBuf {
    match cwd {
        Some(path) => {
            let p = Path::new(path);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                base_dir.join(p)
            }
        }
        None => base_dir.to_path_buf(),
    }
}

fn cwd_is_allowed(base_dir: &Path, requested: &Path, allowed_cwds: &[String]) -> bool {
    let requested_norm = normalize_path(requested);
    allowed_cwds.iter().any(|entry| {
        let allow_path = resolve_cwd(base_dir, Some(entry));
        normalize_path(&allow_path) == requested_norm
    })
}

fn cwd_within_filesystem_bounds(
    base_dir: &Path,
    requested: &Path,
    permissions: &Permissions,
) -> bool {
    path_is_within_allowlist(base_dir, requested, &permissions.filesystem.read)
        || path_is_within_allowlist(base_dir, requested, &permissions.filesystem.write)
}

fn path_is_within_allowlist(base_dir: &Path, requested: &Path, allowlist: &[String]) -> bool {
    if allowlist.iter().any(|a| a == "*") {
        return true;
    }

    let requested_norm = normalize_path(requested);
    allowlist.iter().any(|entry| {
        let allowed_path = resolve_cwd(base_dir, Some(entry));
        let allowed_norm = normalize_path(&allowed_path);
        requested_norm == allowed_norm || requested_norm.starts_with(&allowed_norm)
    })
}

fn normalize_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use myx_core::{FilesystemPermissions, SubprocessPermissions, ToolClass};
    use std::collections::BTreeMap;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use tempfile::TempDir;

    fn test_permissions() -> Permissions {
        Permissions {
            network: vec!["127.0.0.1".to_string()],
            secrets: Vec::new(),
            filesystem: FilesystemPermissions {
                read: vec![".".to_string()],
                write: vec![".".to_string()],
            },
            subprocess: SubprocessPermissions {
                allowed_commands: vec!["echo".to_string()],
                allowed_cwds: vec![".".to_string()],
                allowed_env: vec!["HOME".to_string()],
                max_timeout_ms: Some(2_000),
            },
        }
    }

    fn subprocess_tool() -> ToolDefinition {
        ToolDefinition {
            name: "echo_tool".to_string(),
            description: "Echo value".to_string(),
            parameters: serde_json::json!({"type":"object"}),
            tool_class: ToolClass::LocalProcess,
            execution: ToolExecution::Subprocess {
                command: "echo".to_string(),
                args: vec!["{{message}}".to_string()],
                cwd: Some(".".to_string()),
                env_passthrough: vec!["HOME".to_string()],
                timeout_ms: Some(1000),
            },
        }
    }

    #[test]
    fn subprocess_executes_with_allowlisted_policy() {
        let tmp = TempDir::new().expect("tempdir");
        let output = execute_tool(
            &subprocess_tool(),
            &test_permissions(),
            tmp.path(),
            &serde_json::json!({"message":"hello"}),
        )
        .expect("execute subprocess");
        assert_eq!(output.kind, "subprocess");
        assert_eq!(output.exit_code, Some(0));
        assert!(output.stdout.unwrap_or_default().contains("hello"));
    }

    #[test]
    fn subprocess_rejects_unallowlisted_command() {
        let tmp = TempDir::new().expect("tempdir");
        let mut permissions = test_permissions();
        permissions.subprocess.allowed_commands = vec!["git".to_string()];

        let err = execute_tool(
            &subprocess_tool(),
            &permissions,
            tmp.path(),
            &serde_json::json!({"message":"hello"}),
        )
        .expect_err("expected allowlist failure");
        assert!(err.to_string().contains("allowed_commands"));
    }

    #[test]
    fn subprocess_rejects_cwd_outside_filesystem_bounds() {
        let tmp = TempDir::new().expect("tempdir");
        let mut permissions = test_permissions();
        permissions.filesystem.read = vec!["./allowed".to_string()];
        permissions.filesystem.write = vec!["./allowed".to_string()];

        let err = execute_tool(
            &subprocess_tool(),
            &permissions,
            tmp.path(),
            &serde_json::json!({"message":"hello"}),
        )
        .expect_err("expected filesystem bounds failure");
        assert!(err.to_string().contains("filesystem bounds"));
    }

    #[test]
    fn subprocess_rejects_wildcard_command_allowlist() {
        let tmp = TempDir::new().expect("tempdir");
        let mut permissions = test_permissions();
        permissions.subprocess.allowed_commands = vec!["*".to_string()];

        let err = execute_tool(
            &subprocess_tool(),
            &permissions,
            tmp.path(),
            &serde_json::json!({"message":"hello"}),
        )
        .expect_err("expected wildcard deny");
        assert!(err.to_string().contains("exact entries"));
    }

    #[test]
    fn http_executes_against_allowlisted_host() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buffer = [0u8; 2048];
            let _ = stream.read(&mut buffer);
            let body = "ok";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write response");
        });

        let tool = ToolDefinition {
            name: "http_tool".to_string(),
            description: "test".to_string(),
            parameters: serde_json::json!({"type":"object"}),
            tool_class: ToolClass::HttpApi,
            execution: ToolExecution::Http {
                method: "GET".to_string(),
                url: format!("http://{}/status?q={{{{query}}}}", addr),
                headers: BTreeMap::new(),
                timeout_ms: Some(1000),
            },
        };

        let output = execute_tool(
            &tool,
            &test_permissions(),
            Path::new("."),
            &serde_json::json!({"query":"repo"}),
        )
        .expect("execute http");
        handle.join().expect("join server");

        assert_eq!(output.kind, "http");
        assert_eq!(output.status_code, Some(200));
        assert_eq!(output.body.unwrap_or_default(), "ok");
    }

    #[test]
    fn http_rejects_non_allowlisted_host() {
        let tool = ToolDefinition {
            name: "http_tool".to_string(),
            description: "test".to_string(),
            parameters: serde_json::json!({"type":"object"}),
            tool_class: ToolClass::HttpApi,
            execution: ToolExecution::Http {
                method: "GET".to_string(),
                url: "https://api.github.com/search/repositories?q={{query}}".to_string(),
                headers: BTreeMap::new(),
                timeout_ms: Some(1000),
            },
        };

        let permissions = Permissions {
            network: vec!["example.com".to_string()],
            ..test_permissions()
        };

        let err = execute_tool(
            &tool,
            &permissions,
            Path::new("."),
            &serde_json::json!({"query":"myx"}),
        )
        .expect_err("expected host deny");
        assert!(err.to_string().contains("not allowed"));
    }
}
