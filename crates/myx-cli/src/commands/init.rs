use std::path::PathBuf;

use anyhow::Result;
use myx_core::{CapabilityProfile, PackageManifest, ToolClass, ToolDefinition, ToolExecution};
use serde_json::json;

pub fn command_init(path: Option<PathBuf>, force: bool) -> Result<()> {
    let target = path.unwrap_or_else(|| PathBuf::from("."));
    std::fs::create_dir_all(&target)?;

    let manifest_path = target.join("myx.yaml");
    let profile_path = target.join("capability.json");
    if !force && (manifest_path.exists() || profile_path.exists()) {
        anyhow::bail!(
            "refusing to overwrite existing package files in '{}'; use --force",
            target.display()
        );
    }

    let package_name = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("my-capability")
        .replace(' ', "-");

    let manifest = PackageManifest {
        name: package_name.clone(),
        version: "0.1.0".to_string(),
        description: "myx capability package".to_string(),
        publisher: "local".to_string(),
        license: "Apache-2.0".to_string(),
        ir: Some("./capability.json".to_string()),
    };

    let profile = CapabilityProfile {
        schema_version: myx_core::PROFILE_SCHEMA_VERSION.to_string(),
        identity: myx_core::Identity {
            name: package_name,
            version: "0.1.0".to_string(),
            publisher: "local".to_string(),
            license: "Apache-2.0".to_string(),
        },
        metadata: myx_core::Metadata {
            description: "Scaffolded myx capability".to_string(),
            homepage: String::new(),
            source: String::new(),
        },
        capabilities: vec!["example".to_string()],
        instructions: myx_core::Instructions {
            system: "Use these tools to help the user.".to_string(),
            usage: "Call tools only when they reduce ambiguity.".to_string(),
        },
        tools: vec![ToolDefinition {
            name: "example_http_tool".to_string(),
            description: "Example HTTP action.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"}
                },
                "required": ["query"]
            }),
            tool_class: ToolClass::HttpApi,
            execution: ToolExecution::Http {
                method: "GET".to_string(),
                url: "https://api.example.com/search?q={{query}}".to_string(),
                headers: Default::default(),
                timeout_ms: Some(10_000),
            },
        }],
        permissions: myx_core::Permissions {
            network: vec!["api.example.com".to_string()],
            secrets: Vec::new(),
            filesystem: myx_core::FilesystemPermissions::default(),
            subprocess: myx_core::SubprocessPermissions::default(),
        },
        compatibility: myx_core::Compatibility {
            runtimes: vec!["openai".to_string(), "mcp".to_string(), "skill".to_string()],
            platforms: vec!["darwin".to_string(), "linux".to_string()],
        },
    };

    std::fs::create_dir_all(target.join("prompts"))?;
    std::fs::create_dir_all(target.join("tools"))?;
    std::fs::write(
        target.join("prompts/system.md"),
        "# System Prompt\n\nDescribe how the capability should be used.\n",
    )?;
    std::fs::write(
        target.join("tools/schema.json"),
        serde_json::to_vec_pretty(&json!({
            "type": "object",
            "properties": {
                "query": {"type":"string"}
            },
            "required": ["query"]
        }))?,
    )?;

    std::fs::write(&manifest_path, serde_yaml::to_string(&manifest)?)?;
    std::fs::write(&profile_path, serde_json::to_vec_pretty(&profile)?)?;
    println!("initialized package scaffold in {}", target.display());
    Ok(())
}
