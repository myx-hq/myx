use std::path::PathBuf;

use anyhow::Result;
use myx_core::load_config;
use serde_json::json;

use crate::exit::{fail, CliExit};
use crate::util::resolve_bundle;

pub fn command_inspect(
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
