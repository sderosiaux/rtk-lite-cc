# rtk-lite-cc

A stripped-down fork of [rtk-ai/rtk](https://github.com/rtk-ai/rtk) (Rust Token Killer). Single Rust binary that sits between Claude Code and your shell, compressing command outputs before they eat your context window.

Same proxy, same filters, none of the overhead.

## What changed from upstream

The original rtk is a multi-agent tool with analytics, telemetry, session tracking, and support for 7 AI coding assistants. I only use Claude Code, and I don't want my CLI proxy phoning home or writing to a SQLite database every time I run `git status`.

Here's what got cut (~12,000 lines removed):

| Removed | Why |
|---|---|
| Telemetry (`ureq` HTTP pings to external server) | No transmission outside my machine |
| SQLite tracking database (`rusqlite`) | No database, no disk writes per command |
| Token analytics (`rtk gain`, `rtk cc-economics`, `rtk session`) | I care about the filtering, not measuring it |
| Discover command (session history scanning) | Depended on provider/report modules I don't need |
| Learn module (CLI correction detection) | Same |
| Tee system (raw output saved to disk on failure) | Noise. If I need raw output, I run the command directly |
| Gemini, Copilot, Cursor, Windsurf, Cline, Codex, OpenCode support | Claude Code only |
| Legacy `--claude-md` injection mode (137-line block in CLAUDE.md) | Modern hook-based approach is the default now |
| `colored` crate (terminal colors for analytics display) | Analytics is gone, so this goes too |
| `getrandom`, `hostname` crates | Telemetry-only dependencies |

Everything else stayed: the full proxy + filter pipeline (30 compiled Rust filters, 58 TOML declarative filters), Claude Code hook integration, TOML filter DSL, hook integrity verification, permission system, trust system, and all command filter modules across every ecosystem (git, cargo, npm, docker, kubectl, go, python, ruby, dotnet, aws, curl, etc.).

One addition: `include_commands` config option. If set, only listed commands get rewritten by the hook. Opt-in instead of the default opt-out.

## How it works

```
Claude Code runs "git status"
       |
       v
Hook intercepts (PreToolUse) --> rtk rewrite "git status"
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

Two layers of filtering:

1. **Compiled filters** (Rust) -- for commands that need multi-pass parsing (git diff compaction, cargo test error grouping, gh pr JSON extraction)
2. **TOML filters** (declarative) -- for the long tail. Regex-based strip/keep/truncate rules, no recompilation needed

## Install

```bash
cargo install --git https://github.com/sderosiaux/rtk-lite-cc
```

Then set up the Claude Code hook:

```bash
rtk init -g              # hook + RTK.md + settings.json patch
rtk init -g --auto-patch # same, no prompt
```

This installs `~/.claude/hooks/rtk-rewrite.sh` and patches `~/.claude/settings.json` so Claude Code routes commands through rtk automatically.

## Usage

Automatic (after `rtk init -g`):
```bash
# Claude Code transparently rewrites commands:
# git status     --> rtk git status
# cargo test     --> rtk cargo test
# docker ps      --> rtk docker ps
```

Manual:
```bash
rtk git status        # filtered output
rtk cargo test        # only failures shown
rtk ls -la src/       # compact listing
rtk grep "pattern" .  # grouped results
```

Passthrough (no filter, just execute):
```bash
rtk proxy curl https://api.example.com
```

## Config

`~/.config/rtk/config.toml`:

```toml
[hooks]
# Only rewrite these commands (if set, everything else passes through raw)
include_commands = ["git", "cargo", "docker"]

# Or exclude specific commands from rewriting
exclude_commands = ["curl", "playwright"]
```

`include_commands` takes precedence. If it's non-empty, only listed commands get rewritten. If it's empty (default), everything is rewritten except what's in `exclude_commands`.

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
- Zero runtime dependencies (no database, no network, no temp files)
- Single static binary

## Credits

Based on [rtk-ai/rtk](https://github.com/rtk-ai/rtk) by Patrick Szymkowiak. MIT license.
