//! Translates a raw shell command into its RTK-optimized equivalent.

use crate::discover::registry;
use std::io::Write;

/// Run the `rtk rewrite` command.
///
/// Prints the RTK-rewritten command to stdout and exits with a code:
///
/// | Exit | Stdout   | Meaning                                                      |
/// |------|----------|--------------------------------------------------------------|
/// | 0    | rewritten| Rewrite allowed — hook may auto-allow the rewritten command. |
/// | 1    | (none)   | No RTK equivalent — hook passes through unchanged.           |
pub fn run(cmd: &str) -> anyhow::Result<()> {
    let config = crate::core::config::Config::load().unwrap_or_default();
    let excluded = &config.hooks.exclude_commands;
    let included = &config.hooks.include_commands;

    // include_commands takes precedence: if non-empty, only rewrite those
    if !included.is_empty() {
        let base_cmd = cmd.split_whitespace().next().unwrap_or("");
        if !included.iter().any(|inc| base_cmd == inc.as_str()) {
            std::process::exit(1); // not in include list → passthrough
        }
    }

    match registry::rewrite_command(cmd, excluded) {
        Some(rewritten) => {
            print!("{}", rewritten);
            let _ = std::io::stdout().flush();
            Ok(())
        }
        None => {
            // No RTK equivalent. Exit 1 = passthrough.
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_supported_command_succeeds() {
        assert!(registry::rewrite_command("git status", &[]).is_some());
    }

    #[test]
    fn test_run_unsupported_returns_none() {
        assert!(registry::rewrite_command("htop", &[]).is_none());
    }

    #[test]
    fn test_run_already_rtk_returns_some() {
        assert_eq!(
            registry::rewrite_command("rtk git status", &[]),
            Some("rtk git status".into())
        );
    }
}
