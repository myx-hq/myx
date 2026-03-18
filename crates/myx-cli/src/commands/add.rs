use std::path::PathBuf;

use anyhow::Result;
use myx_core::load_config;
use myx_lockfile::{load_lock, upsert_entry, write_lock_atomic, LockEntry};
use myx_policy::{evaluate_install_policy, Decision, PolicyErrorCode};
use serde_json::json;

use crate::exit::{fail, CliExit};
use crate::non_interactive::resolve_non_interactive_mode;
use crate::util::{parse_expected_digest, resolve_bundle};

fn map_policy_error_to_exit_code(code: PolicyErrorCode) -> i32 {
    match code {
        PolicyErrorCode::InvalidConfiguration => 3,
        PolicyErrorCode::PromptIo => 1,
    }
}

pub fn command_add(
    package: &str,
    config_override: Option<PathBuf>,
    non_interactive_flag: bool,
    json_output: bool,
) -> Result<(), CliExit> {
    let cwd = std::env::current_dir().map_err(|e| fail(1, e))?;
    let config = load_config(config_override.as_deref(), &cwd).map_err(|e| fail(1, e))?;
    let (resolved, bundle) = resolve_bundle(package, &config, &cwd).map_err(|e| fail(4, e))?;
    let non_interactive_mode =
        resolve_non_interactive_mode(non_interactive_flag).map_err(|e| fail(2, e))?;

    for tool in &bundle.profile.tools {
        myx_runtime_executor::validate_execution(&tool.execution).map_err(|e| fail(3, e))?;
    }

    let actual_digest = myx_store::compute_package_digest(&bundle.package_dir).map_err(|e| {
        fail(
            1,
            format!(
                "failed to compute package digest for '{}': {e}",
                bundle.package_dir.display()
            ),
        )
    })?;
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

    let policy_result = evaluate_install_policy(
        &config.policy,
        &bundle.profile.permissions,
        non_interactive_mode.enabled,
    )
    .map_err(|e| fail(map_policy_error_to_exit_code(e.code), e))?;
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
