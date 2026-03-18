use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

pub fn ensure_store(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

pub fn compute_package_digest(source_dir: &Path) -> Result<String> {
    if !source_dir.is_dir() {
        return Err(anyhow!(
            "package source '{}' is not a directory",
            source_dir.display()
        ));
    }

    let mut files = Vec::new();
    for entry in WalkDir::new(source_dir).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix(source_dir).with_context(|| {
            format!(
                "failed to compute relative path for '{}'",
                entry.path().display()
            )
        })?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        files.push(rel.to_path_buf());
    }

    files.sort_by_key(|a| path_key(a));

    let mut hasher = Sha256::new();
    for rel in files {
        let rel_text = path_key(&rel);
        let abs = source_dir.join(&rel);
        let bytes =
            std::fs::read(&abs).with_context(|| format!("failed to read '{}'", abs.display()))?;

        hasher.update(b"file\0");
        hasher.update(rel_text.as_bytes());
        hasher.update(b"\0");
        hasher.update(bytes.len().to_string().as_bytes());
        hasher.update(b"\0");
        hasher.update(&bytes);
        hasher.update(b"\0");
    }

    Ok(format!("{:x}", hasher.finalize()))
}

pub fn install_package_to_store(
    source_dir: &Path,
    workspace_root: &Path,
    name: &str,
    version: &str,
) -> Result<PathBuf> {
    let store_dir = workspace_root
        .join(".myx")
        .join("store")
        .join(name)
        .join(version);
    if store_dir.is_file() {
        return Err(anyhow!(
            "store path '{}' is a file, expected directory",
            store_dir.display()
        ));
    }
    if store_dir.is_dir() {
        return Ok(store_dir);
    }

    let store_parent = store_dir
        .parent()
        .ok_or_else(|| anyhow!("invalid store path '{}'", store_dir.display()))?;
    std::fs::create_dir_all(store_parent)
        .with_context(|| format!("failed to create '{}'", store_parent.display()))?;

    let staging_dir = unique_staging_dir(store_parent, name, version);
    copy_tree(source_dir, &staging_dir)?;

    match std::fs::rename(&staging_dir, &store_dir) {
        Ok(()) => Ok(store_dir),
        Err(_err) if store_dir.is_dir() => {
            let _ = std::fs::remove_dir_all(&staging_dir);
            Ok(store_dir)
        }
        Err(err) => {
            let _ = std::fs::remove_dir_all(&staging_dir);
            Err(err).with_context(|| {
                format!(
                    "failed to atomically move '{}' into '{}'",
                    staging_dir.display(),
                    store_dir.display()
                )
            })
        }
    }
}

fn copy_tree(source_dir: &Path, dest_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dest_dir)
        .with_context(|| format!("failed to create '{}'", dest_dir.display()))?;

    for entry in WalkDir::new(source_dir).follow_links(false) {
        let entry = entry?;
        let src = entry.path();
        let rel = src.strip_prefix(source_dir)?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        let dest = dest_dir.join(rel);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(src, &dest)?;
        }
    }

    Ok(())
}

fn unique_staging_dir(parent: &Path, name: &str, version: &str) -> PathBuf {
    let pid = std::process::id();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    parent.join(format!(".{name}-{version}.staging-{pid}-{now}"))
}

fn path_key(path: &Path) -> String {
    let mut key = String::new();
    for (idx, component) in path.components().enumerate() {
        if idx > 0 {
            key.push('/');
        }
        key.push_str(&component.as_os_str().to_string_lossy());
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_package(root: &Path, marker: &str, prompt: &str) -> PathBuf {
        let package_dir = root.join(marker);
        std::fs::create_dir_all(package_dir.join("prompts")).expect("create prompts dir");
        std::fs::write(
            package_dir.join("myx.yaml"),
            "name: github\nversion: 1.0.0\nir: ./capability.json\n",
        )
        .expect("write manifest");
        std::fs::write(
            package_dir.join("capability.json"),
            r#"{"schema_version":"1","identity":{"name":"github","version":"1.0.0"},"tools":[]}"#,
        )
        .expect("write profile");
        std::fs::write(package_dir.join("prompts/system.md"), prompt).expect("write prompt");
        package_dir
    }

    #[test]
    fn package_digest_changes_when_non_profile_file_changes() {
        let tmp = TempDir::new().expect("tempdir");
        let first = write_package(tmp.path(), "pkg-a", "one");
        let second = write_package(tmp.path(), "pkg-b", "two");

        let digest_a = compute_package_digest(&first).expect("digest a");
        let digest_b = compute_package_digest(&second).expect("digest b");
        assert_ne!(digest_a, digest_b);
    }

    #[test]
    fn install_is_non_destructive_when_package_already_exists() {
        let tmp = TempDir::new().expect("tempdir");
        let first = write_package(tmp.path(), "source-a", "first");
        let second = write_package(tmp.path(), "source-b", "second");

        let installed =
            install_package_to_store(&first, tmp.path(), "github", "1.0.0").expect("first install");
        let prompt_path = installed.join("prompts/system.md");
        assert_eq!(
            std::fs::read_to_string(&prompt_path).expect("read first prompt"),
            "first"
        );

        let installed_again = install_package_to_store(&second, tmp.path(), "github", "1.0.0")
            .expect("second install");
        assert_eq!(installed_again, installed);
        assert_eq!(
            std::fs::read_to_string(&prompt_path).expect("read second prompt"),
            "first"
        );
    }
}
