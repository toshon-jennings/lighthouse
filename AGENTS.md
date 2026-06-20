# AGENTS.md — Lighthouse

> Agent rules and operating procedures for the Lighthouse project.
> Read this before making any changes.

## Project Context

Lighthouse is a port-awareness tool for local development — scans live ports,
detects conflicts, reads PORTMASTER.md files, and suggests fixes. It is a Tauri
desktop app with a CLI companion (`lh`).

- **Local path:** `/Users/toshonjennings/lighthouse`
- **Repo:** `github.com/toshon-jennings/lighthouse`
- **Core stack:** Rust (Tauri v2), HTML/CSS/JS frontend
- **Cross-platform:** macOS / Linux / Windows

## Architecture

### Key Modules
| Module | Purpose |
|--------|---------|
| `scanner` | Live port scanning via lsof/ss/netstat |
| `portmaster` | PORTMASTER.md parsing |
| `projects` | Config file scanning |
| `resolver` | Conflict detection + port suggestion |
| `config_editor` | Fix preview/apply |
| `monitor` | 5s background polling |

### CLI Commands
- `lh list` — list live ports
- `lh check <port>` — check if a port is free
- `lh suggest <port> <range>` — suggest alternative port
- `lh portmasters` — list PORTMASTER.md files
- `lh projects` — list project configs

### Known Bug
- `resolver.rs` line ~80: `pp.port == pp.port` is always true (should be `pp.port == lp.port`). Causes false-positive conflict reports.

## Design Gate

Before writing any new feature code or making non-trivial changes, state what you're
planning to build and wait for explicit approval. Do not start implementation until
confirmed.

## Git Workflow
- Treat `origin/main` as source of truth.
- Work on `main` directly unless explicitly asked for a branch.
- Sync local `main` from `origin/main` before editing or pushing.

## PORTMASTER.md
- Global spec at `~/.config/agent-rules/PORTMASTER.md` — single source of truth for all service ports on this machine.
- Do not modify without explicit user approval.
