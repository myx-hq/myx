use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    pub name: String,
    pub version: String,
    pub source: String,
    pub digest: String,
    #[serde(default)]
    pub permissions_snapshot: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyxLock {
    pub lockfile_version: u32,
    pub packages: Vec<LockEntry>,
}

impl Default for MyxLock {
    fn default() -> Self {
        Self {
            lockfile_version: 1,
            packages: Vec::new(),
        }
    }
}

pub fn load_lock(path: &Path) -> Result<MyxLock> {
    if !path.exists() {
        return Ok(MyxLock::default());
    }
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read lockfile '{}'", path.display()))?;
    let lock: MyxLock = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse lockfile '{}'", path.display()))?;
    Ok(lock)
}

pub fn upsert_entry(lock: &mut MyxLock, entry: LockEntry) {
    if let Some(existing) = lock
        .packages
        .iter_mut()
        .find(|p| p.name == entry.name && p.version == entry.version)
    {
        *existing = entry;
    } else {
        lock.packages.push(entry);
    }
    lock.packages
        .sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));
}

pub fn write_lock_atomic(path: &Path, lock: &MyxLock) -> Result<()> {
    let text = serde_json::to_string_pretty(lock).context("failed to serialize lockfile")?;
    let tmp = path.with_extension("lock.tmp");
    std::fs::write(&tmp, text)
        .with_context(|| format!("failed to write temp lockfile '{}'", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("failed to atomically replace '{}'", path.display()))?;
    Ok(())
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    format!("{out:x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn entry(name: &str, version: &str) -> LockEntry {
        LockEntry {
            name: name.to_string(),
            version: version.to_string(),
            source: format!("/tmp/{name}/{version}"),
            digest: format!("sha256:{name}{version}"),
            permissions_snapshot: serde_json::json!({"network":["api.example.com"]}),
        }
    }

    #[test]
    fn upsert_keeps_deterministic_order() {
        let mut lock = MyxLock::default();
        upsert_entry(&mut lock, entry("zeta", "1.0.0"));
        upsert_entry(&mut lock, entry("alpha", "2.0.0"));
        upsert_entry(&mut lock, entry("alpha", "1.0.0"));

        let keys = lock
            .packages
            .iter()
            .map(|p| format!("{}@{}", p.name, p.version))
            .collect::<Vec<_>>();
        assert_eq!(
            keys,
            vec![
                "alpha@1.0.0".to_string(),
                "alpha@2.0.0".to_string(),
                "zeta@1.0.0".to_string()
            ]
        );
    }

    #[test]
    fn write_and_load_round_trip() {
        let tmp = TempDir::new().expect("tempdir");
        let path = tmp.path().join("myx.lock");

        let mut lock = MyxLock::default();
        upsert_entry(&mut lock, entry("github", "1.2.3"));
        write_lock_atomic(&path, &lock).expect("write lock");

        let loaded = load_lock(&path).expect("load lock");
        assert_eq!(loaded.lockfile_version, 1);
        assert_eq!(loaded.packages.len(), 1);
        assert_eq!(loaded.packages[0].name, "github");
        assert_eq!(loaded.packages[0].version, "1.2.3");
    }

    #[test]
    fn sha256_is_stable() {
        let digest = sha256_hex(b"hello");
        assert_eq!(
            digest,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
}
