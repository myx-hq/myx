use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use myx_core::{MyxConfig, ResolvedPackage, StaticIndex};
use semver::Version;

enum IndexOrigin {
    File(PathBuf),
    Url(url::Url),
}

fn parse_spec(spec: &str) -> (String, Option<String>) {
    if let Some((name, version)) = spec.rsplit_once('@') {
        if !name.is_empty() && !version.is_empty() {
            return (name.to_string(), Some(version.to_string()));
        }
    }
    (spec.to_string(), None)
}

fn resolve_local_path(spec: &str, cwd: &Path) -> Option<PathBuf> {
    let p = Path::new(spec);
    if p.is_absolute() && p.exists() {
        return Some(p.to_path_buf());
    }
    let rel = cwd.join(p);
    if rel.exists() {
        return Some(rel);
    }
    None
}

fn load_index(source: &str, cwd: &Path) -> Result<(StaticIndex, IndexOrigin)> {
    if source.starts_with("http://") || source.starts_with("https://") {
        let response = ureq::get(source)
            .call()
            .with_context(|| format!("failed to fetch index source '{}'", source))?;
        let body = response
            .into_body()
            .read_to_string()
            .with_context(|| format!("failed to read index source '{}'", source))?;
        let index: StaticIndex = serde_json::from_str(&body)
            .with_context(|| format!("failed to parse index source '{}'", source))?;
        let url = url::Url::parse(source)
            .with_context(|| format!("failed to parse index URL '{}'", source))?;
        return Ok((index, IndexOrigin::Url(url)));
    }

    let path = if Path::new(source).is_absolute() {
        PathBuf::from(source)
    } else {
        cwd.join(source)
    };
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read index file '{}'", path.display()))?;
    let index: StaticIndex = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse index file '{}'", path.display()))?;
    Ok((index, IndexOrigin::File(path)))
}

fn cmp_version(a: &str, b: &str) -> Ordering {
    match (Version::parse(a), Version::parse(b)) {
        (Ok(va), Ok(vb)) => va.cmp(&vb),
        _ => a.cmp(b),
    }
}

fn resolve_entry_source(origin: &IndexOrigin, source: &str) -> Result<PathBuf> {
    if source.starts_with("http://") || source.starts_with("https://") {
        return Err(anyhow!(
            "index entry source '{}' is remote; MVP expects local package paths",
            source
        ));
    }

    if let Some(rest) = source.strip_prefix("file://") {
        return Ok(PathBuf::from(rest));
    }

    let p = Path::new(source);
    if p.is_absolute() {
        return Ok(p.to_path_buf());
    }

    match origin {
        IndexOrigin::File(path) => {
            let base = path
                .parent()
                .ok_or_else(|| anyhow!("index path '{}' has no parent", path.display()))?;
            Ok(base.join(p))
        }
        IndexOrigin::Url(url) => Err(anyhow!(
            "index '{}' has relative package source '{}'; use absolute file paths in MVP",
            url,
            source
        )),
    }
}

pub fn resolve(spec: &str, config: &MyxConfig, cwd: &Path) -> Result<ResolvedPackage> {
    if spec.trim().is_empty() {
        return Err(anyhow!("empty package spec"));
    }

    if let Some(path) = resolve_local_path(spec, cwd) {
        let manifest = myx_core::load_manifest(&path)?;
        return Ok(ResolvedPackage {
            name: manifest.name,
            version: manifest.version,
            source: path,
            expected_digest: None,
        });
    }

    if config.index.sources.is_empty() {
        return Err(anyhow!(
            "no index sources configured and '{}' is not a local path",
            spec
        ));
    }

    let (name, requested_version) = parse_spec(spec);

    let mut candidates: Vec<(String, String, PathBuf, String)> = Vec::new();
    for source in &config.index.sources {
        let (index, origin) = load_index(source, cwd)?;
        for pkg in index.packages {
            if pkg.name != name {
                continue;
            }
            if let Some(req) = &requested_version {
                if &pkg.version != req {
                    continue;
                }
            }
            let source_path = resolve_entry_source(&origin, &pkg.source)?;
            candidates.push((pkg.name, pkg.version, source_path, pkg.digest));
        }
    }

    if candidates.is_empty() {
        if let Some(v) = requested_version {
            return Err(anyhow!(
                "package '{}@{}' not found in configured indexes",
                name,
                v
            ));
        }
        return Err(anyhow!(
            "package '{}' not found in configured indexes",
            name
        ));
    }

    candidates.sort_by(|a, b| cmp_version(&a.1, &b.1));
    let selected = candidates
        .pop()
        .ok_or_else(|| anyhow!("no matching package after resolution"))?;
    Ok(ResolvedPackage {
        name: selected.0,
        version: selected.1,
        source: selected.2,
        expected_digest: Some(selected.3),
    })
}
