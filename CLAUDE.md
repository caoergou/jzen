# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`jzen` is a dual-interface JSON editor:
- **TUI mode** (human): tree navigation, inline editing, syntax highlighting via ratatui
- **Command mode** (AI agents): path-based CLI commands with minimal token output

Same binary, two modes, one engine. The core engine (`src/engine/`) has zero I/O dependencies and is fully testable.

## Development Commands

```bash
# Build
cargo build --release

# Run tests
cargo test

# Run a single test
cargo test fix_to_value

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format check
cargo fmt --all -- --check
```

## Architecture

```
main.rs → cli.rs (arg parsing)
           ↓
        ┌──┴──┐
        ▼     ▼
    TUI   Command Mode
   (tui/)   (command/)
        │     │
        └─────┴────┐
                  ▼
         ┌────────────┐
         │   Engine   │  ← pure in-memory operations
         │  (engine/) │
         └────────────┘
```

### Core Modules

| Module | Purpose |
|--------|---------|
| `engine/parser.rs` | Strict (serde_json) and lenient (custom tokenizer) JSON parsing. Lenient mode tolerates trailing commas, comments, single quotes, unquoted keys, Python literals (True/False/None), and BOM. |
| `engine/path.rs` | jq-inspired path expressions: `.key`, `.arr[0]`, `.[-1]` |
| `engine/fix.rs` | Auto-fix common JSON errors via lenient parser |
| `engine/format.rs` | Pretty-print and minify |
| `engine/edit.rs` | In-place modifications (set, del, add, mv) |
| `engine/schema.rs` | Generate type-only summaries without values |
| `command/` | CLI command handlers (get, set, fix, fmt, etc.) |
| `tui/` | Terminal UI using ratatui |

### Key Design Decisions

- **Object key order**: Uses `IndexMap` to preserve insertion order (minimizes diffs on save)
- **Atomic writes**: Write to `.tmp`, fsync, then rename (crash-safe)
- **Exit codes**: 0=success, 1=error, 2=not found, 3=type mismatch

## Auto-fix Capabilities

The `jzen fix` command repairs common JSON format errors:
- Trailing commas
- Single quotes → double quotes
- Unquoted keys
- Missing commas
- Line/block comments (with `--strip-comments`)
- Python literals (True→true, False→false, None→null)
- BOM stripping
