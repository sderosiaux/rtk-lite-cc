//! Shared command execution skeleton for filter modules.

use anyhow::{Context, Result};
use std::process::Command;

use crate::core::utils::{exit_code_from_output, exit_code_from_status};

#[derive(Default)]
pub struct RunOptions {
    pub filter_stdout_only: bool,
    pub skip_filter_on_failure: bool,
    pub no_trailing_newline: bool,
}

impl RunOptions {
    pub fn stdout_only() -> Self {
        Self {
            filter_stdout_only: true,
            ..Default::default()
        }
    }

    pub fn early_exit_on_failure(mut self) -> Self {
        self.skip_filter_on_failure = true;
        self
    }

    pub fn no_trailing_newline(mut self) -> Self {
        self.no_trailing_newline = true;
        self
    }
}

pub fn run_filtered<F>(
    mut cmd: Command,
    tool_name: &str,
    _args_display: &str,
    filter_fn: F,
    opts: RunOptions,
) -> Result<i32>
where
    F: Fn(&str) -> String,
{
    let output = cmd
        .output()
        .with_context(|| format!("Failed to run {}", tool_name))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let raw = format!("{}\n{}", stdout, stderr);
    let exit_code = exit_code_from_output(&output, tool_name);

    // On failure, skip filtering and return early (e.g. psql error messages
    // containing '|' would be misinterpreted by the table parser)
    if opts.skip_filter_on_failure && exit_code != 0 {
        if !stderr.trim().is_empty() {
            eprintln!("{}", stderr.trim());
        }
        return Ok(exit_code);
    }

    let text_to_filter = if opts.filter_stdout_only {
        &stdout
    } else {
        raw.as_str()
    };
    let filtered = filter_fn(text_to_filter);

    if opts.no_trailing_newline {
        print!("{}", filtered);
    } else {
        println!("{}", filtered);
    }

    if opts.filter_stdout_only && !stderr.trim().is_empty() {
        eprintln!("{}", stderr.trim());
    }

    Ok(exit_code)
}

pub fn run_passthrough(tool: &str, args: &[std::ffi::OsString], verbose: u8) -> Result<i32> {
    if verbose > 0 {
        eprintln!("{} passthrough: {:?}", tool, args);
    }
    let status = crate::core::utils::resolved_command(tool)
        .args(args)
        .status()
        .with_context(|| format!("Failed to run {}", tool))?;
    Ok(exit_code_from_status(&status, tool))
}
