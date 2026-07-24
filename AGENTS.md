# AGENTS.md — Lighthouse

> Agent rules and operating procedures for the Lighthouse project.
> Read this before making any changes.

## Project Context

Lighthouse is a port-awareness tool for local development — scans live ports,
detects conflicts, reads PORTMASTER.md files, and suggests fixes. It ships as
both an Electron desktop app and a CLI (`lh`).

- **Local path:** `/Users/toshonjennings/lighthouse`
- **Repo:** `github.com/toshon-jennings/lighthouse`
- **Core stack:** Electron + Vite (desktop), Node.js + Commander (CLI)
- **Cross-platform:** macOS (primary), Linux (planned)

## Architecture

### Engine (`lib/engine.js`)
Shared logic used by both the Electron app and CLI:
| Function | Purpose |
|----------|---------|
| `scanSockets()` | Live port scanning via lsof (TCP + UDP, IPv4 + IPv6) |
| `loadPortmasters()` | PORTMASTER.md parsing and walking |
| `detectConflicts()` | Conflict detection between live and declared ports |
| `suggestFree()` | Free port suggestion |
| `friendlyProcessName()` | Human-readable process names |

### CLI (`bin/lh.js`)
- `lh list` — list live ports
- `lh check <port>` — check if a port is free
- `lh suggest` — suggest a free port (option: `-r 3000:3999`)
- `lh portmasters` — list PORTMASTER.md files

All commands support `--json` for machine-readable output.

### Desktop App
- `electron/main.js` — Electron main process
- `src/index.html` — UI (all JS inline, uses engine via `require()`)
- `vite.config.ts` — dev server on port 5189

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
