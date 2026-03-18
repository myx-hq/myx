use anyhow::Result;
use std::path::Path;

pub fn ensure_store(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

pub fn install_package_to_store(
    source_dir: &Path,
    workspace_root: &Path,
    name: &str,
    version: &str,
) -> Result<std::path::PathBuf> {
    let store_dir = workspace_root
        .join(".myx")
        .join("store")
        .join(name)
        .join(version);
    if store_dir.exists() {
        std::fs::remove_dir_all(&store_dir)?;
    }
    std::fs::create_dir_all(&store_dir)?;

    for entry in walkdir::WalkDir::new(source_dir) {
        let entry = entry?;
        let src = entry.path();
        let rel = src.strip_prefix(source_dir)?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        let dest = store_dir.join(rel);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(src, &dest)?;
        }
    }

    Ok(store_dir)
}
