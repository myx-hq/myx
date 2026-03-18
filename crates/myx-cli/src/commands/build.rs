use std::path::PathBuf;

use anyhow::Result;
use myx_core::{assert_supported_target, load_config, load_package_bundle};
use serde_json::json;

use crate::exit::{fail, CliExit};
use crate::export::{
    build_mcp, build_openai, build_skill, loss_report_json, required_mismatch_count,
};
use crate::util::{resolve_bundle, write_json};

pub fn command_build(
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
        "mcp" => build_mcp(&out_dir, &bundle.package_dir, &bundle.profile),
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
        if target == "mcp" {
            println!("run: {}", out_dir.join("run.sh").display());
        }
        if !loss.is_empty() {
            println!(
                "loss report: {}",
                out_dir.join("loss-report.json").display()
            );
        }
    }
    Ok(())
}
