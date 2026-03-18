use anyhow::Result;
use std::path::Path;

pub fn ensure_store(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}
