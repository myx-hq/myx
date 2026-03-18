use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use myx_core::{
    assert_supported_target, load_config, load_package_bundle, CapabilityProfile, PackageManifest,
    ResolvedPackage, ToolClass, ToolDefinition, ToolExecution,
};
use myx_lockfile::{load_lock, sha256_hex, upsert_entry, write_lock_atomic, LockEntry};
use myx_policy::{evaluate_install_policy, Decision};
use serde_json::json;

#[derive(Debug, Parser)]
#[command(name = "myx")]
#[command(about = "myx Rust MVP CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init {
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    Add {
        package: String,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        non_interactive: bool,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    Inspect {
        package: String,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    Build {
        #[arg(long)]
        target: String,
        #[arg(long)]
        package: Option<String>,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

#[derive(Debug)]
struct CliExit {
    code: i32,
    message: String,
}

#[derive(Debug, Clone)]
struct BuildIssue {
    level: String,
    category: String,
    tool: Option<String>,
    message: String,
    required_mismatch: bool,
}

impl BuildIssue {
    fn error_required(category: &str, tool: &str, message: impl Into<String>) -> Self {
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

impl CliExit {
    fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

fn fail(code: i32, err: impl std::fmt::Display) -> CliExit {
    CliExit::new(code, err.to_string())
}

fn parse_expected_digest(v: &str) -> String {
    v.strip_prefix("sha256:").unwrap_or(v).to_string()
}

fn resolve_bundle(
    spec: &str,
    config: &myx_core::MyxConfig,
    cwd: &Path,
) -> Result<(ResolvedPackage, myx_core::PackageBundle)> {
    let resolved = myx_resolver::resolve(spec, config, cwd)?;
    let bundle = load_package_bundle(&resolved.source)?;
    Ok((resolved, bundle))
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<()> {
    let data = serde_json::to_vec_pretty(value)?;
    std::fs::write(path, data).with_context(|| format!("failed to write '{}'", path.display()))?;
    Ok(())
}

fn required_mismatch_count(issues: &[BuildIssue]) -> usize {
    issues.iter().filter(|i| i.required_mismatch).count()
}

fn loss_report_json(target: &str, issues: &[BuildIssue]) -> serde_json::Value {
    json!({
        "target": target,
        "issues": issues.iter().map(BuildIssue::as_json).collect::<Vec<_>>(),
        "summary": {
            "total": issues.len(),
            "required_mismatches": required_mismatch_count(issues)
        }
    })
}

fn command_init(path: Option<PathBuf>, force: bool) -> Result<()> {
    let target = path.unwrap_or_else(|| PathBuf::from("."));
    std::fs::create_dir_all(&target)?;

    let manifest_path = target.join("myx.yaml");
    let profile_path = target.join("capability.json");
    if !force && (manifest_path.exists() || profile_path.exists()) {
        anyhow::bail!(
            "refusing to overwrite existing package files in '{}'; use --force",
            target.display()
        );
    }

    let package_name = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("my-capability")
        .replace(' ', "-");

    let manifest = PackageManifest {
        name: package_name.clone(),
        version: "0.1.0".to_string(),
        description: "myx capability package".to_string(),
        publisher: "local".to_string(),
        license: "Apache-2.0".to_string(),
        ir: Some("./capability.json".to_string()),
    };

    let profile = CapabilityProfile {
        schema_version: myx_core::PROFILE_SCHEMA_VERSION.to_string(),
        identity: myx_core::Identity {
            name: package_name,
            version: "0.1.0".to_string(),
            publisher: "local".to_string(),
            license: "Apache-2.0".to_string(),
        },
        metadata: myx_core::Metadata {
            description: "Scaffolded myx capability".to_string(),
            homepage: String::new(),
            source: String::new(),
        },
        capabilities: vec!["example".to_string()],
        instructions: myx_core::Instructions {
            system: "Use these tools to help the user.".to_string(),
            usage: "Call tools only when they reduce ambiguity.".to_string(),
        },
        tools: vec![ToolDefinition {
            name: "example_http_tool".to_string(),
            description: "Example HTTP action.".to_string(),
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
                headers: Default::default(),
                timeout_ms: Some(10_000),
            },
        }],
        permissions: myx_core::Permissions {
            network: vec!["api.example.com".to_string()],
            secrets: Vec::new(),
            filesystem: myx_core::FilesystemPermissions::default(),
            subprocess: myx_core::SubprocessPermissions::default(),
        },
        compatibility: myx_core::Compatibility {
            runtimes: vec!["openai".to_string(), "mcp".to_string(), "skill".to_string()],
            platforms: vec!["darwin".to_string(), "linux".to_string()],
        },
    };

    std::fs::create_dir_all(target.join("prompts"))?;
    std::fs::create_dir_all(target.join("tools"))?;
    std::fs::write(
        target.join("prompts/system.md"),
        "# System Prompt\n\nDescribe how the capability should be used.\n",
    )?;
    std::fs::write(
        target.join("tools/schema.json"),
        serde_json::to_vec_pretty(&json!({
            "type": "object",
            "properties": {
                "query": {"type":"string"}
            },
            "required": ["query"]
        }))?,
    )?;

    std::fs::write(&manifest_path, serde_yaml::to_string(&manifest)?)?;
    std::fs::write(&profile_path, serde_json::to_vec_pretty(&profile)?)?;
    println!("initialized package scaffold in {}", target.display());
    Ok(())
}

fn command_add(
    package: &str,
    config_override: Option<PathBuf>,
    non_interactive: bool,
    json_output: bool,
) -> Result<(), CliExit> {
    let cwd = std::env::current_dir().map_err(|e| fail(1, e))?;
    let config = load_config(config_override.as_deref(), &cwd).map_err(|e| fail(1, e))?;
    let (resolved, bundle) = resolve_bundle(package, &config, &cwd).map_err(|e| fail(4, e))?;

    for tool in &bundle.profile.tools {
        myx_runtime_executor::validate_execution(&tool.execution).map_err(|e| fail(3, e))?;
    }

    let profile_bytes = std::fs::read(&bundle.profile_path).map_err(|e| fail(1, e))?;
    let actual_digest = sha256_hex(&profile_bytes);
    if let Some(expected) = &resolved.expected_digest {
        if parse_expected_digest(expected) != actual_digest {
            return Err(fail(
                5,
                format!(
                    "digest mismatch for {}@{} (expected {}, got {})",
                    resolved.name, resolved.version, expected, actual_digest
                ),
            ));
        }
    }

    let policy_result =
        evaluate_install_policy(&config.policy, &bundle.profile.permissions, non_interactive)
            .map_err(|e| fail(1, e))?;
    if matches!(policy_result.decision, Decision::Deny) {
        return Err(fail(6, policy_result.reason));
    }

    let installed_path = myx_store::install_package_to_store(
        &bundle.package_dir,
        &cwd,
        &bundle.manifest.name,
        &bundle.manifest.version,
    )
    .map_err(|e| fail(1, e))?;

    let lock_path = cwd.join("myx.lock");
    let mut lock = load_lock(&lock_path).map_err(|e| fail(3, e))?;
    let permissions_snapshot =
        serde_json::to_value(&bundle.profile.permissions).map_err(|e| fail(1, e))?;
    upsert_entry(
        &mut lock,
        LockEntry {
            name: bundle.manifest.name.clone(),
            version: bundle.manifest.version.clone(),
            source: installed_path.display().to_string(),
            digest: actual_digest.clone(),
            permissions_snapshot,
        },
    );
    write_lock_atomic(&lock_path, &lock).map_err(|e| fail(1, e))?;

    if json_output {
        let out = json!({
            "command": "add",
            "ok": true,
            "package": {
                "name": bundle.manifest.name,
                "version": bundle.manifest.version,
                "digest": actual_digest
            },
            "policy": policy_result.reason
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out).map_err(|e| fail(1, e))?
        );
    } else {
        println!(
            "installed {}@{}",
            bundle.manifest.name, bundle.manifest.version
        );
        println!("store: {}", installed_path.display());
        println!("lockfile: {}", lock_path.display());
    }

    Ok(())
}

fn command_inspect(
    package: &str,
    config_override: Option<PathBuf>,
    json_output: bool,
) -> Result<(), CliExit> {
    let cwd = std::env::current_dir().map_err(|e| fail(1, e))?;
    let config = load_config(config_override.as_deref(), &cwd).map_err(|e| fail(1, e))?;
    let (_resolved, bundle) = resolve_bundle(package, &config, &cwd).map_err(|e| fail(4, e))?;

    if json_output {
        let out = json!({
            "command": "inspect",
            "ok": true,
            "identity": bundle.profile.identity,
            "tools": bundle.profile.tools,
            "permissions": bundle.profile.permissions,
            "compatibility": bundle.profile.compatibility
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out).map_err(|e| fail(1, e))?
        );
        return Ok(());
    }

    println!(
        "{}@{}",
        bundle.profile.identity.name, bundle.profile.identity.version
    );
    println!("tools: {}", bundle.profile.tools.len());
    for tool in &bundle.profile.tools {
        println!("  - {} ({:?})", tool.name, tool.tool_class);
    }
    println!(
        "network permissions: {:?}",
        bundle.profile.permissions.network
    );
    println!("secrets: {:?}", bundle.profile.permissions.secrets);
    Ok(())
}

fn build_openai(out_dir: &Path, profile: &CapabilityProfile) -> Result<Vec<BuildIssue>> {
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

fn build_skill(out_dir: &Path, profile: &CapabilityProfile) -> Result<Vec<BuildIssue>> {
    let mut tools = profile.tools.clone();
    tools.sort_by(|a, b| a.name.cmp(&b.name));

    let mut loss = Vec::new();
    let mut lines = Vec::new();
    lines.push("# Skill Export".to_string());
    lines.push(String::new());
    lines.push("| Command | Description |".to_string());
    lines.push("|---|---|".to_string());

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

fn build_mcp(out_dir: &Path, profile: &CapabilityProfile) -> Result<Vec<BuildIssue>> {
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
        "permissions": profile.permissions,
        "executor": {
            "kind": "myx_global_executor"
        },
        "tools": tools
    });
    write_json(&out_dir.join("runtime-config.json"), &runtime_config)?;

    let launch = json!({
        "command": "myx-mcp-wrapper",
        "args": ["--config", "runtime-config.json"],
        "cwd": ".",
        "startup": "deterministic"
    });
    write_json(&out_dir.join("launch.json"), &launch)?;

    let run_script = r#"#!/usr/bin/env sh
set -eu
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
exec "${MYX_MCP_WRAPPER_BIN:-myx-mcp-wrapper}" --config "$SCRIPT_DIR/runtime-config.json"
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

fn command_build(
    target: &str,
    package: Option<String>,
    config_override: Option<PathBuf>,
    json_output: bool,
) -> Result<(), CliExit> {
    assert_supported_target(target).map_err(|e| fail(2, e))?;
    let cwd = std::env::current_dir().map_err(|e| fail(1, e))?;
    let config = load_config(config_override.as_deref(), &cwd).map_err(|e| fail(1, e))?;

    let bundle = if let Some(spec) = package {
        let (_resolved, bundle) = resolve_bundle(&spec, &config, &cwd).map_err(|e| fail(4, e))?;
        bundle
    } else {
        load_package_bundle(&cwd).map_err(|e| fail(3, e))?
    };

    for tool in &bundle.profile.tools {
        myx_runtime_executor::validate_execution(&tool.execution).map_err(|e| fail(3, e))?;
    }

    let out_dir = cwd.join(".myx").join(target);
    std::fs::create_dir_all(&out_dir).map_err(|e| fail(1, e))?;

    let loss = match target {
        "openai" => build_openai(&out_dir, &bundle.profile),
        "skill" => build_skill(&out_dir, &bundle.profile),
        "mcp" => build_mcp(&out_dir, &bundle.profile),
        _ => Err(anyhow::anyhow!("unsupported target '{}'", target)),
    }
    .map_err(|e| fail(1, e))?;

    if !loss.is_empty() {
        let report_path = out_dir.join("loss-report.json");
        write_json(&report_path, &loss_report_json(target, &loss)).map_err(|e| fail(1, e))?;

        let required_mismatches = required_mismatch_count(&loss);
        if required_mismatches > 0 {
            return Err(fail(
                7,
                format!(
                    "build for target '{}' has {} required semantic mismatch(es); see {}",
                    target,
                    required_mismatches,
                    report_path.display()
                ),
            ));
        }
    }

    if json_output {
        let out = json!({
            "command": "build",
            "ok": true,
            "target": target,
            "output_dir": out_dir,
            "loss_issues": loss.len(),
            "required_mismatches": required_mismatch_count(&loss)
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out).map_err(|e| fail(1, e))?
        );
    } else {
        println!("built target '{}' to {}", target, out_dir.display());
        if !loss.is_empty() {
            println!(
                "loss report: {}",
                out_dir.join("loss-report.json").display()
            );
        }
    }
    Ok(())
}

fn run(cli: Cli) -> Result<(), CliExit> {
    match cli.command {
        Commands::Init { path, force } => command_init(path, force).map_err(|e| fail(3, e)),
        Commands::Add {
            package,
            config,
            non_interactive,
            json,
        } => command_add(&package, config, non_interactive, json),
        Commands::Inspect {
            package,
            config,
            json,
        } => command_inspect(&package, config, json),
        Commands::Build {
            target,
            package,
            config,
            json,
        } => command_build(&target, package, config, json),
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        let message = json!({
            "command": "myx",
            "ok": false,
            "timestamp": chrono_like_timestamp(),
            "error": {
                "code": err.code,
                "message": err.message
            }
        });
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&message).unwrap_or_else(|_| {
                "{\"ok\":false,\"error\":{\"code\":1,\"message\":\"failed to serialize error\"}}"
                    .to_string()
            })
        );
        std::process::exit(err.code);
    }
}

fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => format!("{}", d.as_secs()),
        Err(_) => "0".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(issues[0].level, "error");
        assert!(issues[0].required_mismatch);
    }

    #[test]
    fn build_mcp_writes_wrapper_assets() {
        let profile = sample_profile(vec![http_tool("get_repo")]);
        let tmp = TempDir::new().expect("tempdir");
        build_mcp(tmp.path(), &profile).expect("build mcp");

        assert!(tmp.path().join("server.json").exists());
        assert!(tmp.path().join("runtime-config.json").exists());
        assert!(tmp.path().join("launch.json").exists());
        assert!(tmp.path().join("run.sh").exists());
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
