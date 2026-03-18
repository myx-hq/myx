use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

pub const PROFILE_SCHEMA_VERSION: &str = "1";
pub const SUPPORTED_TARGETS: &[&str] = &["openai", "mcp", "skill"];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Identity {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub publisher: String,
    #[serde(default)]
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub homepage: String,
    #[serde(default)]
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Instructions {
    #[serde(default)]
    pub system: String,
    #[serde(default)]
    pub usage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolClass {
    HttpApi,
    LocalProcess,
    FilesystemAssisted,
    Composite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ToolExecution {
    Http {
        method: String,
        url: String,
        #[serde(default)]
        headers: BTreeMap<String, String>,
        #[serde(default)]
        timeout_ms: Option<u64>,
    },
    Subprocess {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        cwd: Option<String>,
        #[serde(default)]
        env_passthrough: Vec<String>,
        #[serde(default)]
        timeout_ms: Option<u64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub parameters: serde_json::Value,
    pub tool_class: ToolClass,
    pub execution: ToolExecution,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilesystemPermissions {
    #[serde(default)]
    pub read: Vec<String>,
    #[serde(default)]
    pub write: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubprocessPermissions {
    #[serde(default)]
    pub allowed_commands: Vec<String>,
    #[serde(default)]
    pub allowed_cwds: Vec<String>,
    #[serde(default)]
    pub allowed_env: Vec<String>,
    #[serde(default)]
    pub max_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Permissions {
    #[serde(default)]
    pub network: Vec<String>,
    #[serde(default)]
    pub secrets: Vec<String>,
    #[serde(default)]
    pub filesystem: FilesystemPermissions,
    #[serde(default)]
    pub subprocess: SubprocessPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Compatibility {
    #[serde(default)]
    pub runtimes: Vec<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilityProfile {
    pub schema_version: String,
    pub identity: Identity,
    #[serde(default)]
    pub metadata: Metadata,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub instructions: Instructions,
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default)]
    pub compatibility: Compatibility,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub publisher: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub ir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexConfig {
    #[serde(default)]
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PolicyMode {
    #[default]
    ReviewRequired,
    Permissive,
    Strict,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyConfig {
    #[serde(default)]
    pub mode: PolicyMode,
    #[serde(default)]
    pub allow_network: Vec<String>,
    #[serde(default)]
    pub allow_secrets: Vec<String>,
    #[serde(default)]
    pub allow_filesystem_read: Vec<String>,
    #[serde(default)]
    pub allow_filesystem_write: Vec<String>,
    #[serde(default)]
    pub allow_subprocess_commands: Vec<String>,
    #[serde(default)]
    pub allow_subprocess_cwds: Vec<String>,
    #[serde(default)]
    pub allow_subprocess_env: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MyxConfig {
    #[serde(default)]
    pub index: IndexConfig,
    #[serde(default)]
    pub policy: PolicyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub name: String,
    pub version: String,
    pub source: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StaticIndex {
    #[serde(default)]
    pub packages: Vec<IndexEntry>,
}

#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: String,
    pub source: PathBuf,
    pub expected_digest: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PackageBundle {
    pub manifest: PackageManifest,
    pub profile: CapabilityProfile,
    pub package_dir: PathBuf,
    pub profile_path: PathBuf,
}

fn merge_strings(target: &mut Vec<String>, incoming: Vec<String>) {
    if !incoming.is_empty() {
        *target = incoming;
    }
}

fn merge_config(base: &mut MyxConfig, incoming: MyxConfig) {
    merge_strings(&mut base.index.sources, incoming.index.sources);

    base.policy.mode = incoming.policy.mode;
    merge_strings(
        &mut base.policy.allow_network,
        incoming.policy.allow_network,
    );
    merge_strings(
        &mut base.policy.allow_secrets,
        incoming.policy.allow_secrets,
    );
    merge_strings(
        &mut base.policy.allow_filesystem_read,
        incoming.policy.allow_filesystem_read,
    );
    merge_strings(
        &mut base.policy.allow_filesystem_write,
        incoming.policy.allow_filesystem_write,
    );
    merge_strings(
        &mut base.policy.allow_subprocess_commands,
        incoming.policy.allow_subprocess_commands,
    );
    merge_strings(
        &mut base.policy.allow_subprocess_cwds,
        incoming.policy.allow_subprocess_cwds,
    );
    merge_strings(
        &mut base.policy.allow_subprocess_env,
        incoming.policy.allow_subprocess_env,
    );
}

fn parse_config_file(path: &Path) -> Result<MyxConfig> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let cfg: MyxConfig =
        toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(cfg)
}

fn apply_env_overrides(cfg: &mut MyxConfig) {
    if let Ok(v) = std::env::var("MYX_INDEX_SOURCES") {
        let parts = v
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        if !parts.is_empty() {
            cfg.index.sources = parts;
        }
    }

    if let Ok(v) = std::env::var("MYX_POLICY_MODE") {
        cfg.policy.mode = match v.trim().to_ascii_lowercase().as_str() {
            "permissive" => PolicyMode::Permissive,
            "strict" => PolicyMode::Strict,
            _ => PolicyMode::ReviewRequired,
        };
    }
}

pub fn load_config(config_override: Option<&Path>, cwd: &Path) -> Result<MyxConfig> {
    let mut cfg = MyxConfig::default();

    if let Some(home) = dirs::home_dir() {
        let global = home.join(".config").join("myx").join("config.toml");
        if global.exists() {
            merge_config(&mut cfg, parse_config_file(&global)?);
        }
    }

    let project = cwd.join("myx.config.toml");
    if project.exists() {
        merge_config(&mut cfg, parse_config_file(&project)?);
    }

    apply_env_overrides(&mut cfg);

    if let Some(path) = config_override {
        merge_config(&mut cfg, parse_config_file(path)?);
    }

    Ok(cfg)
}

pub fn ir_path_from_manifest(package_dir: &Path, manifest: &PackageManifest) -> PathBuf {
    let rel = manifest.ir.as_deref().unwrap_or("./capability.json");
    package_dir.join(rel)
}

pub fn load_manifest(package_dir: &Path) -> Result<PackageManifest> {
    let manifest_path = package_dir.join("myx.yaml");
    let text = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let manifest: PackageManifest = serde_yaml::from_str(&text)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    Ok(manifest)
}

pub fn load_profile(profile_path: &Path) -> Result<CapabilityProfile> {
    let text = std::fs::read_to_string(profile_path)
        .with_context(|| format!("failed to read {}", profile_path.display()))?;
    let profile: CapabilityProfile = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse {}", profile_path.display()))?;
    Ok(profile)
}

pub fn load_package_bundle(package_dir: &Path) -> Result<PackageBundle> {
    let manifest = load_manifest(package_dir)?;
    let profile_path = ir_path_from_manifest(package_dir, &manifest);
    let profile = load_profile(&profile_path)?;
    validate_package(&manifest, &profile)?;
    Ok(PackageBundle {
        manifest,
        profile,
        package_dir: package_dir.to_path_buf(),
        profile_path,
    })
}

pub fn validate_package(manifest: &PackageManifest, profile: &CapabilityProfile) -> Result<()> {
    if profile.schema_version != PROFILE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported profile schema_version '{}'; expected '{}'",
            profile.schema_version,
            PROFILE_SCHEMA_VERSION
        ));
    }
    if manifest.name != profile.identity.name {
        return Err(anyhow!(
            "manifest/profile name mismatch: '{}' vs '{}'",
            manifest.name,
            profile.identity.name
        ));
    }
    if manifest.version != profile.identity.version {
        return Err(anyhow!(
            "manifest/profile version mismatch: '{}' vs '{}'",
            manifest.version,
            profile.identity.version
        ));
    }
    if profile.tools.is_empty() {
        return Err(anyhow!("profile must define at least one tool"));
    }
    for tool in &profile.tools {
        if tool.name.trim().is_empty() {
            return Err(anyhow!("tool name cannot be empty"));
        }
        match &tool.execution {
            ToolExecution::Http { method, url, .. } => {
                if method.trim().is_empty() || url.trim().is_empty() {
                    return Err(anyhow!("http execution requires method and url"));
                }
            }
            ToolExecution::Subprocess {
                command,
                timeout_ms,
                ..
            } => {
                if command.trim().is_empty() {
                    return Err(anyhow!("subprocess execution requires command"));
                }
                if command.contains(' ') {
                    return Err(anyhow!(
                        "subprocess command must be a single executable token"
                    ));
                }
                if timeout_ms.is_none() {
                    return Err(anyhow!("subprocess execution requires timeout_ms in MVP"));
                }
                if profile.permissions.subprocess.allowed_commands.is_empty() {
                    return Err(anyhow!(
                        "subprocess tools require permissions.subprocess.allowed_commands"
                    ));
                }
                if profile.permissions.subprocess.allowed_cwds.is_empty() {
                    return Err(anyhow!(
                        "subprocess tools require permissions.subprocess.allowed_cwds"
                    ));
                }
                if profile.permissions.subprocess.max_timeout_ms.is_none() {
                    return Err(anyhow!(
                        "subprocess tools require permissions.subprocess.max_timeout_ms"
                    ));
                }
            }
        }
    }
    Ok(())
}

pub fn assert_supported_target(target: &str) -> Result<()> {
    if SUPPORTED_TARGETS.contains(&target) {
        return Ok(());
    }
    Err(anyhow!(
        "unsupported target '{}'; expected one of {}",
        target,
        SUPPORTED_TARGETS.join(", ")
    ))
}
