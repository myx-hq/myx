use std::path::Path;

use anyhow::{Context, Result};
use myx_core::{PackageBundle, ResolvedPackage};

pub fn parse_expected_digest(v: &str) -> String {
    v.strip_prefix("sha256:").unwrap_or(v).to_string()
}

pub fn resolve_bundle(
    spec: &str,
    config: &myx_core::MyxConfig,
    cwd: &Path,
) -> Result<(ResolvedPackage, PackageBundle)> {
    let resolved = myx_resolver::resolve(spec, config, cwd)?;
    let bundle = myx_core::load_package_bundle(&resolved.source)?;
    Ok((resolved, bundle))
}

pub fn write_json(path: &Path, value: &serde_json::Value) -> Result<()> {
    let data = serde_json::to_vec_pretty(value)?;
    std::fs::write(path, data).with_context(|| format!("failed to write '{}'", path.display()))?;
    Ok(())
}

pub fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => format!("{}", d.as_secs()),
        Err(_) => "0".to_string(),
    }
}
