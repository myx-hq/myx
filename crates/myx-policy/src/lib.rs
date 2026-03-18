use std::io::{self, Write};

use anyhow::Result;
use myx_core::{Permissions, PolicyConfig, PolicyMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decision {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    pub decision: Decision,
    pub reason: String,
}

fn find_missing(allowed: &[String], requested: &[String]) -> Vec<String> {
    if allowed.iter().any(|a| a == "*") {
        return Vec::new();
    }
    requested
        .iter()
        .filter(|r| !allowed.contains(r))
        .cloned()
        .collect()
}

fn missing_permissions(policy: &PolicyConfig, permissions: &Permissions) -> Vec<String> {
    let mut missing = Vec::new();

    for item in find_missing(&policy.allow_network, &permissions.network) {
        missing.push(format!("network:{item}"));
    }
    for item in find_missing(&policy.allow_secrets, &permissions.secrets) {
        missing.push(format!("secret:{item}"));
    }
    for item in find_missing(&policy.allow_filesystem_read, &permissions.filesystem.read) {
        missing.push(format!("filesystem_read:{item}"));
    }
    for item in find_missing(
        &policy.allow_filesystem_write,
        &permissions.filesystem.write,
    ) {
        missing.push(format!("filesystem_write:{item}"));
    }
    for item in find_missing(
        &policy.allow_subprocess_commands,
        &permissions.subprocess.allowed_commands,
    ) {
        missing.push(format!("subprocess_command:{item}"));
    }
    for item in find_missing(
        &policy.allow_subprocess_cwds,
        &permissions.subprocess.allowed_cwds,
    ) {
        missing.push(format!("subprocess_cwd:{item}"));
    }
    for item in find_missing(
        &policy.allow_subprocess_env,
        &permissions.subprocess.allowed_env,
    ) {
        missing.push(format!("subprocess_env:{item}"));
    }

    missing
}

fn prompt_approve(missing: &[String]) -> Result<bool> {
    println!("Policy review required. Missing allowlist entries:");
    for item in missing {
        println!("  - {item}");
    }
    print!("Approve install anyway? [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let normalized = input.trim().to_ascii_lowercase();
    Ok(matches!(normalized.as_str(), "y" | "yes"))
}

pub fn evaluate_install_policy(
    policy: &PolicyConfig,
    permissions: &Permissions,
    non_interactive: bool,
) -> Result<PolicyResult> {
    if matches!(policy.mode, PolicyMode::Permissive) {
        return Ok(PolicyResult {
            decision: Decision::Allow,
            reason: "policy mode is permissive".to_string(),
        });
    }

    let missing = missing_permissions(policy, permissions);
    if missing.is_empty() {
        return Ok(PolicyResult {
            decision: Decision::Allow,
            reason: "permissions satisfied by allowlist".to_string(),
        });
    }

    if matches!(policy.mode, PolicyMode::Strict) || non_interactive {
        return Ok(PolicyResult {
            decision: Decision::Deny,
            reason: format!("permissions denied by policy: {}", missing.join(", ")),
        });
    }

    if prompt_approve(&missing)? {
        return Ok(PolicyResult {
            decision: Decision::Allow,
            reason: "approved interactively".to_string(),
        });
    }

    Ok(PolicyResult {
        decision: Decision::Deny,
        reason: "interactive approval denied".to_string(),
    })
}
