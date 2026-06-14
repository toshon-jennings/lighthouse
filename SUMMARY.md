# SUMMARY.md — Agent Orientation Cheat Sheet

> **PURPOSE:** This file exists to prevent unnecessary full-repo scans. AI
> agents MUST read this file first when they need orientation. It provides
> a map of the codebase so agents can jump directly to the relevant files
> instead of scanning every directory.

> **RULE:** Do NOT grep or glob the entire repo to find things. Use this
> document's pointers to navigate directly. Only search broadly when the
> answer isn't found via these pointers.

---

## Project

**Name:** Lighthouse
**Description:** Desktop port-awareness tool — scans local listening ports, detects conflicts, annotates with PORTMASTER.md registry.
**Target Platform:** Desktop (Electron, macOS)
**Primary Language:** JavaScript/TypeScript
**Framework:** Electron + Vite

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | Vanilla renderer with nodeIntegration (direct Node.js access) |
| Backend | None — port scanning runs in-process via `child_process` |
| Desktop Shell | Electron |
| Build | Vite + electron-builder |
| Testing | None |
| Styling | CSS (no framework) |

---

## Source Map

> Paths are relative to repo root (`/Users/toshonjennings/lighthouse`).

### Entry Points
| File | Role |
|------|------|
| `electron/main.js` | Electron main process — creates BrowserWindow |
| `src/main.js` | Renderer entry (Vite entry point) |
| `index.html` | HTML shell loaded by Electron |

### Core Directories
| Directory | Contains |
|-----------|----------|
| `src/` | Renderer source (UI + port scanning logic) |
| `electron/` | Electron main process (single `main.js`) |
| `build/` | Build artifacts |
| `release/` | Packaged DMG output |

### Key Files
| File | Role |
|------|------|
| `vite.config.ts` | Vite configuration (dev server on port 5189) |
| `package.json` | Dependencies + scripts (`npm run electron`) |
| `PORTMASTER.md` (global) | Port registry at `~/.config/agent-rules/PORTMASTER.md` |

---

## Key Architectural Patterns

1. **No IPC bridge:** Unlike Perci/Mercury, Lighthouse uses `nodeIntegration: true` — the renderer has direct Node.js access. Port scanning (`lsof`), PORTMASTER.md parsing, and process lookups run in-process via `child_process` — there's no preload/contextBridge layer.

2. **Port scanning pipeline:** (1) Run `lsof` to discover listening ports → (2) Parse PORTMASTER.md files (global + per-project) to label known services → (3) Cross-reference to detect conflicts → (4) Render table with process details (PID, parent PID, working dir, command line).

3. **CSS Grid layout:** Uses CSS Grid with `grid-auto-rows: 1fr` for equal-height port cards. All styling is plain CSS — no Tailwind or CSS-in-JS.

---

## What NOT to Touch

| Path | Why |
|------|-----|
| `dist/` | Build output — regenerated |
| `build/` | Intermediate build artifacts |
| `release/` | Packaged DMG files |
| `node_modules/` | Dependencies |

---

## Common Gotchas

1. **No preload bridge** — `nodeIntegration` is enabled. There's no `preload.cjs` or `contextBridge`. The renderer can directly `require('child_process')`. This is intentional for port scanning but means XSS is more dangerous here.

2. **PORTMASTER.md locations** — The global registry is at `~/.config/agent-rules/PORTMASTER.md`. Repo-local PORTMASTER.md files are also scanned. If a port shows as "Unknown," check whether a PORTMASTER.md exists in the relevant project.

3. **lsof requires permissions** — Port scanning uses `lsof -iTCP -sTCP:LISTEN -P -n`. On macOS this works for standard ports but may require sudo for some processes. If scan returns empty, check permissions.

4. **Parent PID tracking** — Lighthouse shows parent PID, working directory, and full command line for each process. This distinguishes it from simpler port scanners. Users rely on parent PID to identify which Docker container or dev server spawned a child process.

5. **Tauri was banned for this project** — Lighthouse was migrated from Tauri to Electron on 2026-06-13. Never attempt to add Tauri. Use Electron for all desktop apps.

---

## How to Navigate by Task

| If you need to... | Go to... |
|--------------------|------------------|
| Change port scanning logic | `src/main.js` (the `lsof` invocation + parsing) |
| Change UI/layout | `src/main.js` (UI lives in the renderer entry — no component separation) |
| Add PORTMASTER.md parsing | `src/main.js` (file I/O parsing) |
| Change Electron window | `electron/main.js` |
| Change build config | `vite.config.ts` + `package.json` |
| Change styling | `src/main.js` or separate CSS files |
| Build/release | `package.json` scripts + `electron/main.js` |
| PORTMASTER.md registry | `~/.config/agent-rules/PORTMASTER.md` |