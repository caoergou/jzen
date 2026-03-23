# Jed — JSON Editor

A dual-interface JSON tool built for **humans and AI agents** alike.

- **Humans** get a full TUI with tree navigation, inline editing, and syntax highlighting.
- **AI Agents** get path-based CLI commands with minimal token output.

Same binary. Two modes. One engine.

[![CI](https://github.com/caoergou/jed/actions/workflows/ci.yml/badge.svg)](https://github.com/caoergou/jed/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## Quick Start

```bash
# TUI mode (human)
jed config.json

# Command mode (agent / script)
jed config.json get .name
jed config.json set .name '"Bob"'
jed config.json fix --strip-comments
```

---

## Installation

### Pre-built binaries (recommended)

Download from the [Releases](https://github.com/caoergou/jed/releases) page, or use the install script:

```bash
# Linux / macOS — auto-detects platform and installs to /usr/local/bin
curl -fsSL https://github.com/caoergou/jed/releases/latest/download/install.sh | sh
```

### From crates.io

```bash
cargo install jed-json
```

### From source (requires Rust)

```bash
cargo install --git https://github.com/caoergou/jed
```

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `jed-linux-x86_64` |
| Linux aarch64 | `jed-linux-aarch64` |
| macOS x86_64 | `jed-macos-x86_64` |
| macOS Apple Silicon | `jed-macos-aarch64` |
| Windows x86_64 | `jed-windows-x86_64.exe` |

Place the binary somewhere on your `$PATH` and rename it to `jed`.

---

## TUI Mode

Launch by passing only a filename:

```bash
jed settings.json
```

| Key | Action |
|-----|--------|
| `↑/↓` | Move up/down |
| `←` | Collapse / go to parent |
| `→` / `Space` | Expand / toggle |
| `Enter` | Edit leaf node / expand container |
| `N` / `Insert` | Add new node |
| `Delete` | Delete current node |
| `Ctrl+S` | Save |
| `Ctrl+F` / `/` | Search |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `F2` | Context menu |
| `F1` | Help |

---

## Command Mode

Designed for **AI agents** to read and write JSON with minimal token usage.

### Read

```bash
jed file.json get .key              # get value at path
jed file.json get '.servers[0].host'
jed file.json keys .                # list all top-level keys
jed file.json len .tags             # array / object length
jed file.json type .count           # type name: string|number|boolean|null|object|array
jed file.json exists .key           # exit 0=exists, 2=not found
jed file.json schema                # infer structure (no values)
jed file.json check                 # validate; errors to stderr
```

### Write

```bash
jed file.json set .name '"Bob"'     # set value
jed file.json del .legacy           # delete key
jed file.json add .tags '"go"'      # append to array
jed file.json mv .oldKey .newKey    # rename key

# Batch (JSON Patch RFC 6902) — one call, atomic
jed file.json patch '[
  {"op": "replace", "path": ".name",    "value": "Bob"},
  {"op": "add",     "path": ".tags/-",  "value": "go"},
  {"op": "remove",  "path": ".legacy"}
]'
```

### Format / Repair

```bash
jed file.json fmt                   # pretty-print in place
jed file.json fix --strip-comments  # auto-fix JSONC, trailing commas, etc.
jed file.json fix --dry-run         # preview repairs without writing
jed file.json minify                # compact JSON
jed file.json diff other.json       # structural diff
```

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Path not found |
| 3 | Type mismatch |

---

## Why Jed for AI Agents?

| Traditional approach | jed command mode |
|----------------------|-----------------|
| Read full file into context | `get .key` → only the target value |
| Rewrite full file after change | `set .key val` → returns `ok` |
| Agent parses JSON manually | Path addressing handles navigation |
| Agent retries on format errors | `fix` repairs errors automatically |
| Multiple round-trips | `patch` applies batch changes in one call |

### Example: configure Claude Code MCP server

```bash
# 1. Check file structure without reading values
jed ~/.claude/settings.json schema

# 2. Check if a server exists
jed ~/.claude/settings.json exists .mcpServers.github

# 3. Read only the specific value needed
jed ~/.claude/settings.json get .mcpServers.github.command

# 4. Update a single field
jed ~/.claude/settings.json set .mcpServers.github.env.TOKEN '"ghp_xxxx"'

# 5. Batch update (one call)
jed ~/.claude/settings.json patch '[
  {"op": "replace", "path": ".defaultMode", "value": "acceptEdits"},
  {"op": "add",     "path": ".mcpServers.github.enabled", "value": true}
]'
```

---

## Auto-fix Capabilities

`jed fix` repairs the most common JSON format errors found in the wild:

| Error | Example | Fix |
|-------|---------|-----|
| Trailing comma | `{"a": 1,}` | Remove |
| Single quotes | `{'key': 'val'}` | Replace with double quotes |
| Unquoted keys | `{key: "val"}` | Add quotes |
| Missing comma | `{"a": 1 "b": 2}` | Insert comma |
| Line comments | `// comment` | Strip |
| Block comments | `/* comment */` | Strip |
| Python literals | `True`, `False`, `None` | Replace with JSON equivalents |
| BOM | Leading `\uFEFF` | Strip |

---

## Path Syntax

Uses jq-inspired path syntax:

```
.                      # root
.key                   # object field
.key.nested            # nested field
.array[0]              # array index
.array[-1]             # last element
.key.array[2].field    # deep path
```

---

## Building from Source

```bash
git clone https://github.com/caoergou/jed
cd jed
cargo build --release
./target/release/jed --version
```

---

## License

MIT
