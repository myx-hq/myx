use anyhow::{bail, Result};
use myx_core::PackageRef;

pub fn resolve(spec: &str) -> Result<PackageRef> {
    if spec.trim().is_empty() {
        bail!("empty package spec");
    }

    Ok(PackageRef {
        name: spec.to_string(),
        version: "latest".to_string(),
    })
}
