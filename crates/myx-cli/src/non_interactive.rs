use std::io::IsTerminal;

use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonInteractiveMode {
    pub enabled: bool,
    pub reason: String,
}

fn parse_boolish(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

pub fn resolve_non_interactive_from_inputs(
    explicit_flag: bool,
    env_override: Option<&str>,
    ci_env: Option<&str>,
    stdin_is_tty: bool,
    stdout_is_tty: bool,
) -> Result<NonInteractiveMode> {
    if explicit_flag {
        return Ok(NonInteractiveMode {
            enabled: true,
            reason: "explicit --non-interactive flag".to_string(),
        });
    }

    if let Some(raw) = env_override {
        let enabled = parse_boolish(raw).ok_or_else(|| {
            anyhow::anyhow!(
                "invalid MYX_NON_INTERACTIVE value '{}'; expected one of: 1,0,true,false,yes,no,on,off",
                raw
            )
        })?;
        return Ok(NonInteractiveMode {
            enabled,
            reason: format!("MYX_NON_INTERACTIVE={raw}"),
        });
    }

    if let Some(raw_ci) = ci_env {
        let ci_enabled = match parse_boolish(raw_ci) {
            Some(false) => false,
            Some(true) => true,
            None => !raw_ci.trim().is_empty(),
        };
        if ci_enabled {
            return Ok(NonInteractiveMode {
                enabled: true,
                reason: format!("CI={raw_ci}"),
            });
        }
    }

    if !stdin_is_tty || !stdout_is_tty {
        return Ok(NonInteractiveMode {
            enabled: true,
            reason: "stdio is not a TTY".to_string(),
        });
    }

    Ok(NonInteractiveMode {
        enabled: false,
        reason: "interactive TTY session".to_string(),
    })
}

pub fn resolve_non_interactive_mode(explicit_flag: bool) -> Result<NonInteractiveMode> {
    let env_override = std::env::var("MYX_NON_INTERACTIVE").ok();
    let ci = std::env::var("CI").ok();
    resolve_non_interactive_from_inputs(
        explicit_flag,
        env_override.as_deref(),
        ci.as_deref(),
        std::io::stdin().is_terminal(),
        std::io::stdout().is_terminal(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_interactive_flag_has_highest_priority() {
        let mode =
            resolve_non_interactive_from_inputs(true, Some("false"), Some("false"), true, true)
                .expect("resolve mode");
        assert!(mode.enabled);
        assert_eq!(mode.reason, "explicit --non-interactive flag");
    }

    #[test]
    fn myx_non_interactive_env_overrides_ci_and_tty() {
        let mode =
            resolve_non_interactive_from_inputs(false, Some("false"), Some("true"), false, false)
                .expect("resolve mode");
        assert!(!mode.enabled);
        assert_eq!(mode.reason, "MYX_NON_INTERACTIVE=false");
    }

    #[test]
    fn ci_env_enables_non_interactive_mode() {
        let mode = resolve_non_interactive_from_inputs(false, None, Some("1"), true, true)
            .expect("resolve mode");
        assert!(mode.enabled);
        assert_eq!(mode.reason, "CI=1");
    }

    #[test]
    fn non_tty_enables_non_interactive_mode() {
        let mode =
            resolve_non_interactive_from_inputs(false, None, None, false, true).expect("resolve");
        assert!(mode.enabled);
        assert_eq!(mode.reason, "stdio is not a TTY");
    }

    #[test]
    fn interactive_tty_stays_interactive() {
        let mode =
            resolve_non_interactive_from_inputs(false, None, None, true, true).expect("resolve");
        assert!(!mode.enabled);
        assert_eq!(mode.reason, "interactive TTY session");
    }

    #[test]
    fn invalid_myx_non_interactive_env_is_rejected() {
        let err = resolve_non_interactive_from_inputs(false, Some("maybe"), None, true, true)
            .unwrap_err();
        assert!(err
            .to_string()
            .contains("invalid MYX_NON_INTERACTIVE value"));
    }
}
