# rtk-lite-cc

A stripped-down fork of [rtk-ai/rtk](https://github.com/rtk-ai/rtk) (Rust Token Killer). Single Rust binary that sits between Claude Code and your shell, compressing command outputs before they eat your context window.

Same proxy, same filters, none of the overhead.

## What changed from upstream

The original rtk is a multi-agent tool with analytics, telemetry, session tracking, and support for 7 AI coding assistants. I only use Claude Code, and I don't want my CLI proxy phoning home or writing to a SQLite database every time I run `git status`.

Here's what got cut (~15,000 lines removed):

| Removed | Why |
|---|---|
| Telemetry (`ureq` HTTP pings to external server) | No transmission outside my machine |
| SQLite tracking database (`rusqlite`) | No database, no disk writes per command |
| Token analytics (`rtk gain`, `rtk cc-economics`, `rtk session`) | I care about the filtering, not measuring it |
| Discover / Learn / Tee modules | Session scanning, CLI correction, raw output recovery -- not core |
| Gemini, Copilot, Cursor, Windsurf, Cline, Codex, OpenCode support | Claude Code only |
| RTK.md / CLAUDE.md patching | The hook is transparent -- Claude doesn't need to know RTK exists |
| Permission system (`permissions.rs`) | Claude Code already handles deny/ask/allow |
| Trust system (`trust.rs`) | Project filter trust management -- not needed |
| Hook integrity verification (`integrity.rs`, `sha2`) | SHA-256 hook verification -- not needed |
| Hook outdated warnings (`hook_check.rs`) | Daily nag -- not needed |
| `rtk proxy` command | Redundant without tracking |
| `rtk verify` / `rtk trust` / `rtk untrust` | Dev tooling, not user-facing |
| `colored`, `getrandom`, `hostname` crates | Only used by removed modules |

What stayed: the full filter pipeline (30 compiled Rust filters, 58 TOML declarative filters) and all command modules across every ecosystem (git, cargo, npm, docker, kubectl, go, python, ruby, dotnet, aws, curl, etc.).

One addition: `include_commands` config option. If set, only listed commands get rewritten by the hook.

## How it works

```
Claude Code runs "git status"
       |
       v
Hook intercepts (PreToolUse) --> calls "rtk rewrite"
       |
       v
Returns "rtk git status" --> Claude Code runs that instead
       |
       v
rtk executes git status, filters the output, prints compressed version
       |
       v
Claude Code sees ~80% fewer tokens
```

Claude Code doesn't know RTK exists. The hook silently rewrites commands before execution. Two layers of filtering handle the compression:

- Compiled filters (Rust) -- for commands that need multi-pass parsing (git diff compaction, cargo test error grouping, gh pr JSON extraction)
- TOML filters (declarative) -- regex-based strip/keep/truncate rules for the long tail, no recompilation needed

## What it doesn't do

- No network calls. Ever. Zero HTTP crates in the binary.
- No disk writes except during `rtk init` (installs the hook) and `rtk config --create`.
- No database. No SQLite, no tracking, no metrics.
- No CLAUDE.md or RTK.md modification. The hook is invisible to Claude.
- No telemetry, no analytics, no phone-home.

## Install

```bash
# Pre-built binary (Linux/macOS)
curl -fsSL https://raw.githubusercontent.com/sderosiaux/rtk-lite-cc/master/install.sh | sh

# From crates.io
cargo install rtk-lite-cc

# From source
cargo install --git https://github.com/sderosiaux/rtk-lite-cc
```

Then set up the Claude Code hook:

```bash
rtk init -g              # install hook + patch settings.json
rtk init -g --auto-patch # same, skip the [y/N] prompt
```

This does two things:
1. Installs `~/.claude/hooks/rtk-rewrite.sh`
2. Adds a PreToolUse entry in `~/.claude/settings.json`

That's it.

To undo: `rtk init -g --uninstall`

## Usage

After `rtk init -g`, Claude Code commands are transparently rewritten:

```
git status     --> rtk git status     (filtered)
cargo test     --> rtk cargo test     (only failures shown)
docker ps      --> rtk docker ps      (compact table)
```

You can also use rtk manually:

```bash
rtk git status
rtk cargo test
rtk ls -la src/
rtk grep "pattern" .
```

## Config

`~/.config/rtk/config.toml`:

```toml
[hooks]
# Only rewrite these commands (everything else passes through raw)
include_commands = ["git", "cargo", "docker"]

# Or exclude specific commands from rewriting
exclude_commands = ["curl", "playwright"]
```

`include_commands` takes precedence over `exclude_commands`. If both are empty (default), everything gets rewritten.

## Token savings

Rough numbers from a 30-minute Claude Code session on a medium TypeScript/Rust project:

| Command | Raw tokens | Filtered | Savings |
|---|---|---|---|
| `git status/log/diff` (20x) | 15,500 | 3,600 | -77% |
| `cargo test` / `npm test` (5x) | 25,000 | 2,500 | -90% |
| `ls` / `tree` / `cat` (30x) | 42,000 | 12,400 | -70% |
| `grep` / `rg` (8x) | 16,000 | 3,200 | -80% |
| `docker ps` / `kubectl` (3x) | 900 | 180 | -80% |
| Total | ~100k | ~22k | **-78%** |

## Binary

- 3.8 MB release build
- <10ms startup overhead
- Single static binary, zero runtime dependencies

## Credits

Based on [rtk-ai/rtk](https://github.com/rtk-ai/rtk) by Patrick Szymkowiak. MIT license.
