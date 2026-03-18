use std::path::{Path, PathBuf};

use anyhow::Result;
use myx_core::{CapabilityProfile, ToolExecution};
use serde_json::json;

use crate::util::write_json;

#[derive(Debug, Clone)]
pub struct BuildIssue {
    level: String,
    category: String,
    tool: Option<String>,
    message: String,
    required_mismatch: bool,
}

impl BuildIssue {
    pub fn error_required(category: &str, tool: &str, message: impl Into<String>) -> Self {
        Self {
            level: "error".to_string(),
            category: category.to_string(),
            tool: Some(tool.to_string()),
            message: message.into(),
            required_mismatch: true,
        }
    }

    fn as_json(&self) -> serde_json::Value {
        json!({
            "level": self.level,
            "category": self.category,
            "tool": self.tool,
            "message": self.message,
            "required_mismatch": self.required_mismatch
        })
    }
}

pub fn required_mismatch_count(issues: &[BuildIssue]) -> usize {
    issues.iter().filter(|i| i.required_mismatch).count()
}

pub fn loss_report_json(target: &str, issues: &[BuildIssue]) -> serde_json::Value {
    json!({
        "target": target,
        "issues": issues.iter().map(BuildIssue::as_json).collect::<Vec<_>>(),
        "summary": {
            "total": issues.len(),
            "required_mismatches": required_mismatch_count(issues)
        }
    })
}

pub fn build_openai(out_dir: &Path, profile: &CapabilityProfile) -> Result<Vec<BuildIssue>> {
    let mut tools = profile.tools.clone();
    tools.sort_by(|a, b| a.name.cmp(&b.name));

    let mut exported = Vec::new();
    let mut loss = Vec::new();
    for tool in &tools {
        if matches!(tool.execution, ToolExecution::Subprocess { .. }) {
            loss.push(BuildIssue::error_required(
                "semantic_mismatch",
                &tool.name,
                format!(
                    "tool '{}' uses subprocess execution; target openai cannot execute subprocess tools without a runtime bridge",
                    tool.name
                ),
            ));
        }
        exported.push(json!({
            "type": "function",
            "function": {
                "name": tool.name,
                "description": tool.description,
                "parameters": tool.parameters
            },
            "x_myx": {
                "tool_class": serde_json::to_value(&tool.tool_class)?,
                "execution": serde_json::to_value(&tool.execution)?
            }
        }));
    }

    write_json(&out_dir.join("tools.json"), &json!(exported))?;
    std::fs::write(
        out_dir.join("instructions.md"),
        format!(
            "{}\n\n{}",
            profile.instructions.system, profile.instructions.usage
        ),
    )?;
    Ok(loss)
}

pub fn build_skill(out_dir: &Path, profile: &CapabilityProfile) -> Result<Vec<BuildIssue>> {
    let mut tools = profile.tools.clone();
    tools.sort_by(|a, b| a.name.cmp(&b.name));

    let mut loss = Vec::new();
    let mut lines = vec![
        "# Skill Export".to_string(),
        String::new(),
        "| Command | Description |".to_string(),
        "|---|---|".to_string(),
    ];

    for tool in &tools {
        if matches!(tool.execution, ToolExecution::Subprocess { .. }) {
            loss.push(BuildIssue::error_required(
                "semantic_mismatch",
                &tool.name,
                format!(
                    "tool '{}' uses subprocess execution; target skill exports docs and cannot preserve runnable subprocess semantics",
                    tool.name
                ),
            ));
        }
        lines.push(format!("| `{}` | {} |", tool.name, tool.description));
    }

    std::fs::write(out_dir.join("SKILL.md"), lines.join("\n"))?;
    Ok(loss)
}

fn relative_path_or_absolute(base: &Path, target: &Path) -> PathBuf {
    let base_abs = match std::fs::canonicalize(base) {
        Ok(p) => p,
        Err(_) => return target.to_path_buf(),
    };
    let target_abs = match std::fs::canonicalize(target) {
        Ok(p) => p,
        Err(_) => return target.to_path_buf(),
    };

    let base_components = base_abs.components().collect::<Vec<_>>();
    let target_components = target_abs.components().collect::<Vec<_>>();

    let mut common_len = 0usize;
    while common_len < base_components.len()
        && common_len < target_components.len()
        && base_components[common_len] == target_components[common_len]
    {
        common_len += 1;
    }

    if common_len == 0 {
        return target.to_path_buf();
    }

    let mut rel = PathBuf::new();
    for _ in common_len..base_components.len() {
        rel.push("..");
    }
    for component in target_components.iter().skip(common_len) {
        rel.push(component.as_os_str());
    }
    if rel.as_os_str().is_empty() {
        rel.push(".");
    }
    rel
}

pub fn build_mcp(
    out_dir: &Path,
    package_dir: &Path,
    profile: &CapabilityProfile,
) -> Result<Vec<BuildIssue>> {
    let mut tools = profile.tools.clone();
    tools.sort_by(|a, b| a.name.cmp(&b.name));
    let tool_values = tools
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "parameters": t.parameters,
                "tool_class": t.tool_class,
                "execution": t.execution
            })
        })
        .collect::<Vec<_>>();

    let server = json!({
        "schema_version": 1,
        "name": profile.identity.name,
        "version": profile.identity.version,
        "runtime": {
            "kind": "myx_global_executor",
            "startup": "deterministic"
        },
        "tools": tool_values
    });
    write_json(&out_dir.join("server.json"), &server)?;

    let runtime_config = json!({
        "schema_version": 1,
        "identity": profile.identity,
        "base_dir": relative_path_or_absolute(out_dir, package_dir).display().to_string(),
        "permissions": profile.permissions,
        "executor": {
            "kind": "myx_global_executor"
        },
        "tools": tools
    });
    write_json(&out_dir.join("runtime-config.json"), &runtime_config)?;

    let launch = json!({
        "command": "myx-mcp-wrapper",
        "args": ["--config", "runtime-config.json", "--protocol", "mcp"],
        "cwd": ".",
        "startup": "deterministic"
    });
    write_json(&out_dir.join("launch.json"), &launch)?;

    let run_script = r#"#!/usr/bin/env sh
set -eu
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
exec "${MYX_MCP_WRAPPER_BIN:-myx-mcp-wrapper}" --config "$SCRIPT_DIR/runtime-config.json" --protocol mcp
"#;
    let run_path = out_dir.join("run.sh");
    std::fs::write(&run_path, run_script)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&run_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&run_path, perms)?;
    }

    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use myx_core::{ToolClass, ToolDefinition};
    use tempfile::TempDir;

    fn sample_profile(tools: Vec<ToolDefinition>) -> CapabilityProfile {
        CapabilityProfile {
            schema_version: myx_core::PROFILE_SCHEMA_VERSION.to_string(),
            identity: myx_core::Identity {
                name: "github".to_string(),
                version: "1.0.0".to_string(),
                publisher: "myx".to_string(),
                license: "Apache-2.0".to_string(),
            },
            metadata: myx_core::Metadata {
                description: "sample".to_string(),
                homepage: String::new(),
                source: String::new(),
            },
            capabilities: vec!["scm".to_string()],
            instructions: myx_core::Instructions {
                system: "system".to_string(),
                usage: "usage".to_string(),
            },
            tools,
            permissions: myx_core::Permissions::default(),
            compatibility: myx_core::Compatibility {
                runtimes: vec!["openai".to_string(), "mcp".to_string(), "skill".to_string()],
                platforms: vec!["darwin".to_string()],
            },
        }
    }

    fn http_tool(name: &str) -> ToolDefinition {
        ToolDefinition {
            name: name.to_string(),
            description: format!("{name} desc"),
            parameters: json!({
                "type": "object",
                "properties": {
                    "q": {"type": "string"}
                }
            }),
            tool_class: ToolClass::HttpApi,
            execution: ToolExecution::Http {
                method: "GET".to_string(),
                url: "https://api.example.com".to_string(),
                headers: Default::default(),
                timeout_ms: Some(1000),
            },
        }
    }

    fn subprocess_tool(name: &str) -> ToolDefinition {
        ToolDefinition {
            name: name.to_string(),
            description: format!("{name} desc"),
            parameters: json!({"type": "object"}),
            tool_class: ToolClass::LocalProcess,
            execution: ToolExecution::Subprocess {
                command: "git".to_string(),
                args: vec!["status".to_string()],
                cwd: Some(".".to_string()),
                env_passthrough: vec!["HOME".to_string()],
                timeout_ms: Some(1000),
            },
        }
    }

    #[test]
    fn build_openai_writes_sorted_tools() {
        let profile = sample_profile(vec![http_tool("z_tool"), http_tool("a_tool")]);
        let tmp = TempDir::new().expect("tempdir");
        build_openai(tmp.path(), &profile).expect("build openai");

        let tools_text =
            std::fs::read_to_string(tmp.path().join("tools.json")).expect("read tools output");
        let tools: serde_json::Value = serde_json::from_str(&tools_text).expect("parse tools");
        let names = tools
            .as_array()
            .expect("array")
            .iter()
            .map(|item| {
                item["function"]["name"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string()
            })
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["a_tool".to_string(), "z_tool".to_string()]);
    }

    #[test]
    fn build_openai_reports_required_mismatch_for_subprocess() {
        let profile = sample_profile(vec![subprocess_tool("run_git")]);
        let tmp = TempDir::new().expect("tempdir");
        let issues = build_openai(tmp.path(), &profile).expect("build openai");
        assert_eq!(required_mismatch_count(&issues), 1);
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn build_mcp_writes_wrapper_assets() {
        let profile = sample_profile(vec![http_tool("get_repo")]);
        let tmp = TempDir::new().expect("tempdir");
        build_mcp(tmp.path(), tmp.path(), &profile).expect("build mcp");

        assert!(tmp.path().join("server.json").exists());
        assert!(tmp.path().join("runtime-config.json").exists());
        assert!(tmp.path().join("launch.json").exists());
        assert!(tmp.path().join("run.sh").exists());

        let runtime_config: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join("runtime-config.json"))
                .expect("read runtime config"),
        )
        .expect("parse runtime config");
        assert_eq!(runtime_config["base_dir"], ".");

        let launch: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join("launch.json")).expect("read launch"),
        )
        .expect("parse launch");
        assert_eq!(
            launch["args"],
            serde_json::json!(["--config", "runtime-config.json", "--protocol", "mcp"])
        );
    }

    #[test]
    fn loss_report_tracks_required_mismatch_count() {
        let issues = vec![
            BuildIssue::error_required("semantic_mismatch", "a", "x"),
            BuildIssue::error_required("semantic_mismatch", "b", "y"),
        ];
        let report = loss_report_json("openai", &issues);
        assert_eq!(report["summary"]["total"], 2);
        assert_eq!(report["summary"]["required_mismatches"], 2);
    }
}
