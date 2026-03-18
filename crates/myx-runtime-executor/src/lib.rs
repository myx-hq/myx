use anyhow::Result;
use myx_core::ToolExecution;

pub fn validate_execution(exec: &ToolExecution) -> Result<()> {
    match exec {
        ToolExecution::Http { method, url, .. } => {
            if method.trim().is_empty() || url.trim().is_empty() {
                anyhow::bail!("http execution requires non-empty method and url");
            }
            Ok(())
        }
        ToolExecution::Subprocess {
            command,
            timeout_ms,
            ..
        } => {
            if command.contains(' ') {
                anyhow::bail!("subprocess command must be a single executable token");
            }
            if command.contains('/') {
                // Use explicit allowlisted command names, not arbitrary shell paths.
                anyhow::bail!("subprocess command must be a command name, not a path");
            }
            if timeout_ms.is_none() {
                anyhow::bail!("subprocess execution requires timeout_ms");
            }
            Ok(())
        }
    }
}
