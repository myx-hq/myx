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

fn find_missing_exact(allowed: &[String], requested: &[String]) -> Vec<String> {
    requested
        .iter()
        .filter(|r| !allowed.contains(r))
        .cloned()
        .collect()
}

fn contains_wildcard(values: &[String]) -> bool {
    values.iter().any(|v| v == "*")
}

fn validate_subprocess_policy_allowlists(policy: &PolicyConfig) -> Result<()> {
    if contains_wildcard(&policy.allow_subprocess_commands) {
        anyhow::bail!(
            "policy.allow_subprocess_commands must use exact allowlist entries; wildcard '*' is not allowed in MVP"
        );
    }
    if contains_wildcard(&policy.allow_subprocess_cwds) {
        anyhow::bail!(
            "policy.allow_subprocess_cwds must use exact allowlist entries; wildcard '*' is not allowed in MVP"
        );
    }
    if contains_wildcard(&policy.allow_subprocess_env) {
        anyhow::bail!(
            "policy.allow_subprocess_env must use exact allowlist entries; wildcard '*' is not allowed in MVP"
        );
    }
    Ok(())
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
    for item in find_missing_exact(
        &policy.allow_subprocess_cwds,
        &permissions.subprocess.allowed_cwds,
    ) {
        missing.push(format!("subprocess_cwd:{item}"));
    }
    for item in find_missing_exact(
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
    validate_subprocess_policy_allowlists(policy)?;

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

    if matches!(policy.mode, PolicyMode::Strict) {
        return Ok(PolicyResult {
            decision: Decision::Deny,
            reason: format!(
                "permissions denied by strict allowlist policy: {}",
                missing.join(", ")
            ),
        });
    }

    if non_interactive {
        return Ok(PolicyResult {
            decision: Decision::Deny,
            reason: format!(
                "permissions require interactive approval; non-interactive mode requires explicit allowlist entries: {}",
                missing.join(", ")
            ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use myx_core::{
        FilesystemPermissions, Permissions, PolicyConfig, PolicyMode, SubprocessPermissions,
    };

    fn sample_permissions() -> Permissions {
        Permissions {
            network: vec!["api.github.com".to_string()],
            secrets: vec!["GITHUB_TOKEN".to_string()],
            filesystem: FilesystemPermissions {
                read: vec!["./repo".to_string()],
                write: vec!["./repo".to_string()],
            },
            subprocess: SubprocessPermissions {
                allowed_commands: vec!["git".to_string()],
                allowed_cwds: vec![".".to_string()],
                allowed_env: vec!["HOME".to_string()],
                max_timeout_ms: Some(2000),
            },
        }
    }

    #[test]
    fn permissive_mode_allows_missing_permissions() {
        let policy = PolicyConfig {
            mode: PolicyMode::Permissive,
            ..PolicyConfig::default()
        };
        let result =
            evaluate_install_policy(&policy, &sample_permissions(), true).expect("policy eval");
        assert!(matches!(result.decision, Decision::Allow));
    }

    #[test]
    fn strict_mode_denies_when_not_allowlisted() {
        let policy = PolicyConfig {
            mode: PolicyMode::Strict,
            ..PolicyConfig::default()
        };
        let result =
            evaluate_install_policy(&policy, &sample_permissions(), true).expect("policy eval");
        assert!(matches!(result.decision, Decision::Deny));
        assert!(result.reason.contains("strict allowlist policy"));
    }

    #[test]
    fn strict_mode_allows_when_fully_allowlisted() {
        let policy = PolicyConfig {
            mode: PolicyMode::Strict,
            allow_network: vec!["api.github.com".to_string()],
            allow_secrets: vec!["GITHUB_TOKEN".to_string()],
            allow_filesystem_read: vec!["./repo".to_string()],
            allow_filesystem_write: vec!["./repo".to_string()],
            allow_subprocess_commands: vec!["git".to_string()],
            allow_subprocess_cwds: vec![".".to_string()],
            allow_subprocess_env: vec!["HOME".to_string()],
        };
        let result =
            evaluate_install_policy(&policy, &sample_permissions(), true).expect("policy eval");
        assert!(matches!(result.decision, Decision::Allow));
    }

    #[test]
    fn review_required_non_interactive_denies_missing_permissions() {
        let policy = PolicyConfig {
            mode: PolicyMode::ReviewRequired,
            ..PolicyConfig::default()
        };
        let result =
            evaluate_install_policy(&policy, &sample_permissions(), true).expect("policy eval");
        assert!(matches!(result.decision, Decision::Deny));
        assert!(result.reason.contains("non-interactive mode"));
    }

    #[test]
    fn rejects_wildcard_subprocess_allowlist_entries() {
        let policy = PolicyConfig {
            mode: PolicyMode::ReviewRequired,
            allow_subprocess_commands: vec!["*".to_string()],
            ..PolicyConfig::default()
        };
        let err = evaluate_install_policy(&policy, &sample_permissions(), true)
            .expect_err("expected wildcard subprocess policy rejection");
        assert!(err
            .to_string()
            .contains("allow_subprocess_commands must use exact allowlist entries"));
    }
}
