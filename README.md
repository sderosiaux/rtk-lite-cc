<p align="center">
  <img src="hero.png" alt="rtk-lite-cc" width="600">
</p>

<p align="center">
  CLI proxy that compresses command outputs before they eat your Claude Code context window.
</p>

<p align="center">
  <a href="https://crates.io/crates/rtk-lite-cc"><img src="https://img.shields.io/crates/v/rtk-lite-cc" alt="crates.io"></a>
  <a href="https://github.com/sderosiaux/rtk-lite-cc/actions"><img src="https://github.com/sderosiaux/rtk-lite-cc/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT"></a>
</p>

---

Stripped-down fork of [rtk-ai/rtk](https://github.com/rtk-ai/rtk). Same filters, none of the overhead.

Single Rust binary, ~3.8 MB, <10ms overhead. No database, no network calls, no telemetry.

<br>

## How it works

```
Claude Code runs "git status"
        |
        v
  Hook intercepts (PreToolUse)
        |
        v
  "rtk rewrite" returns "rtk git status"
        |
        v
  rtk executes, filters output, prints compressed version
        |
        v
  Claude Code sees ~80% fewer tokens
```

Claude Code doesn't know RTK exists. The hook rewrites commands silently before execution.

<br>

## Install

```bash
# Pre-built binary (Linux/macOS)
curl -fsSL https://raw.githubusercontent.com/sderosiaux/rtk-lite-cc/master/install.sh | sh

# From crates.io
cargo install rtk-lite-cc

# From source
cargo install --git https://github.com/sderosiaux/rtk-lite-cc
```

Then:

```bash
rtk init -g              # install hook + patch settings.json
rtk init -g --auto-patch # same, skip the prompt
```

This does two things:
1. Installs `~/.claude/hooks/rtk-rewrite.sh`
2. Adds a PreToolUse entry in `~/.claude/settings.json`

To undo: `rtk init -g --uninstall`

<br>

## Usage

After `rtk init -g`, commands are transparently rewritten:

```
git status   -->  rtk git status    (filtered)
cargo test   -->  rtk cargo test    (only failures)
docker ps    -->  rtk docker ps     (compact table)
```

Or use rtk manually:

```bash
rtk git status
rtk cargo test
rtk ls -la src/
rtk grep "pattern" .
```

<br>

## Config

`~/.config/rtk/config.toml`:

```toml
[hooks]
# Only rewrite these commands (everything else passes through raw)
include_commands = ["git", "cargo", "docker"]

# Or exclude specific commands
exclude_commands = ["curl", "playwright"]
```

`include_commands` takes precedence. If both are empty (default), everything gets rewritten.

<br>

## Token savings

Rough numbers from a 30-minute Claude Code session on a medium TypeScript/Rust project:

| Command | Raw | Filtered | Savings |
|---|---|---|---|
| `git status/log/diff` (20x) | 15,500 | 3,600 | -77% |
| `cargo test` / `npm test` (5x) | 25,000 | 2,500 | -90% |
| `ls` / `tree` / `cat` (30x) | 42,000 | 12,400 | -70% |
| `grep` / `rg` (8x) | 16,000 | 3,200 | -80% |
| `docker ps` / `kubectl` (3x) | 900 | 180 | -80% |
| **Total** | **~100k** | **~22k** | **-78%** |

<br>

## What it doesn't do

- No network calls. Zero HTTP crates in the binary.
- No disk writes except during `rtk init`.
- No database. No SQLite, no tracking, no metrics.
- No CLAUDE.md modification. The hook is invisible to Claude.
- No telemetry, no analytics, no phone-home.

<br>

## What changed from upstream

~15,000 lines removed from [rtk-ai/rtk](https://github.com/rtk-ai/rtk):

| Removed | Why |
|---|---|
| Telemetry (HTTP pings) | No transmission outside my machine |
| SQLite tracking database | No disk writes per command |
| Token analytics | I care about filtering, not measuring it |
| 6 non-Claude agents | Claude Code only |
| RTK.md / CLAUDE.md patching | Hook is transparent |
| Permission / trust / integrity systems | Claude Code handles this already |
| Hook warnings, proxy command, verify | Not needed |

What stayed: 30 compiled Rust filters + 58 TOML declarative filters, covering git, cargo, npm, docker, kubectl, go, python, ruby, dotnet, aws, curl, and more.

<br>

## Credits

Based on [rtk-ai/rtk](https://github.com/rtk-ai/rtk) by Patrick Szymkowiak. MIT license.
