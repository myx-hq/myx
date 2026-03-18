use anyhow::Result;
use myx_core::ToolExecution;

pub fn validate_execution(exec: &ToolExecution) -> Result<()> {
    match exec {
        ToolExecution::Http { .. } => Ok(()),
        ToolExecution::Subprocess { command, .. } => {
            if command.contains(' ') {
                anyhow::bail!("subprocess command must be a single executable token");
            }
            Ok(())
        }
    }
}
