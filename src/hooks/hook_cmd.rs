//! Processes incoming hook calls from Claude Code and rewrites commands on the fly.

use super::constants::PRE_TOOL_USE_KEY;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::io::{self, Read};

use crate::discover::registry::rewrite_command;
use crate::hooks::permissions::{PermissionVerdict, check_command};

/// Run the Claude Code preToolUse hook.
pub fn run_claude_hook() -> Result<()> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context("Failed to read stdin")?;

    let input = input.trim();
    if input.is_empty() {
        return Ok(());
    }

    let v: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[rtk hook] Failed to parse JSON input: {e}");
            return Ok(());
        }
    };

    // Claude Code uses snake_case: tool_name + tool_input.command
    let tool_name = match v.get("tool_name").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => return Ok(()),
    };

    if !matches!(tool_name, "Bash" | "bash" | "runTerminalCommand") {
        return Ok(());
    }

    let cmd = match v
        .pointer("/tool_input/command")
        .and_then(|c| c.as_str())
        .filter(|c| !c.is_empty())
    {
        Some(c) => c,
        None => return Ok(()),
    };

    handle_rewrite(cmd)
}

fn get_rewritten(cmd: &str) -> Option<String> {
    if cmd.contains("<<") {
        return None;
    }

    let config = crate::core::config::Config::load().unwrap_or_default();
    let excluded = &config.hooks.exclude_commands;
    let included = &config.hooks.include_commands;

    // include_commands takes precedence: if non-empty, only rewrite those
    if !included.is_empty() {
        let base_cmd = cmd.split_whitespace().next().unwrap_or("");
        if !included.iter().any(|inc| base_cmd == inc.as_str()) {
            return None;
        }
    }

    let rewritten = rewrite_command(cmd, excluded)?;

    if rewritten == cmd {
        return None;
    }

    Some(rewritten)
}

fn handle_rewrite(cmd: &str) -> Result<()> {
    let rewritten = match get_rewritten(cmd) {
        Some(r) => r,
        None => return Ok(()),
    };

    let verdict = check_command(cmd);

    if verdict == PermissionVerdict::Deny {
        return Ok(());
    }

    let decision = match verdict {
        PermissionVerdict::Allow => "allow",
        _ => "ask",
    };

    let output = json!({
        "hookSpecificOutput": {
            "hookEventName": PRE_TOOL_USE_KEY,
            "permissionDecision": decision,
            "permissionDecisionReason": "RTK auto-rewrite",
            "updatedInput": { "command": rewritten }
        }
    });
    println!("{output}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_rewritten_supported() {
        assert!(get_rewritten("git status").is_some());
    }

    #[test]
    fn test_get_rewritten_unsupported() {
        assert!(get_rewritten("htop").is_none());
    }

    #[test]
    fn test_get_rewritten_already_rtk() {
        assert!(get_rewritten("rtk git status").is_none());
    }

    #[test]
    fn test_get_rewritten_heredoc() {
        assert!(get_rewritten("cat <<'EOF'\nhello\nEOF").is_none());
    }

    #[test]
    fn test_rewrite_command_basic() {
        assert_eq!(
            rewrite_command("git status", &[]),
            Some("rtk git status".into())
        );
        assert_eq!(
            rewrite_command("cargo test", &[]),
            Some("rtk cargo test".into())
        );
    }

    #[test]
    fn test_rewrite_command_excluded() {
        let excluded = vec!["curl".to_string()];
        assert_eq!(rewrite_command("curl https://example.com", &excluded), None);
        assert_eq!(
            rewrite_command("git status", &excluded),
            Some("rtk git status".into())
        );
    }

    #[test]
    fn test_rewrite_command_env_prefix() {
        assert_eq!(
            rewrite_command("RUST_LOG=debug cargo test", &[]),
            Some("RUST_LOG=debug rtk cargo test".into())
        );
    }
}
