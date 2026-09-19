# Lighthouse

Lighthouse is a port-awareness tool for local development. It shows you what's
listening on your machine, who owns each port, and where conflicts are.

Ships as both an **Electron desktop app** and a **CLI** (`lh`).

## What this isn't

- **Not a firewall or a process manager.** Lighthouse reports; it doesn't block traffic,
  and killing a process is something you do yourself after it tells you what's there.
- **Not a daemon.** There's no background service holding a live view. Each run shells
  out to `lsof` and reads the `PORTMASTER.md` files it finds, so what you see is a
  snapshot from the moment you asked.
- **Not a source of truth on its own.** Ownership labels come from `PORTMASTER.md`
  files you maintain. A port with no declared owner shows up as whatever the process
  table says, which is often just a runtime name.
- **Not cross-platform yet.** It depends on `lsof` and is developed and tested on macOS.
  Cross-platform is a design goal, not a current property.

Features:
- Scans live listening ports via `lsof`
- Detects `PORTMASTER.md` files across your machine and annotates ports with
  their declared service/owner
- Per-process detail: PID, parent, start time, working directory, command
- Quick port check and free-port suggestions
- Conflict detection with a guided resolve flow

## Install

### CLI

```bash
npm install -g @toshon-jennings/lighthouse
lh list               # list live ports
lh check 3000         # check if a port is free
lh suggest            # suggest a free port
lh portmasters        # list PORTMASTER.md files
```

All commands support `--json` for machine-readable output.

### Desktop App

Grab the latest `.dmg` from the
[Releases](https://github.com/toshon-jennings/lighthouse/releases) page (Apple
Silicon), open it, and drag Lighthouse to Applications.

The build is unsigned, so on first launch macOS Gatekeeper will block it.
Right-click the app → **Open**, or run:

```bash
xattr -dr com.apple.quarantine /Applications/Lighthouse.app
```

## Development

Requires Node.js.

```bash
npm install
npm run electron   # starts Vite + Electron together
```

The renderer has direct Node.js access (`nodeIntegration`), so port scanning,
`PORTMASTER.md` parsing, and process lookups run in-process via `child_process`
— there's no separate backend or IPC bridge.

Core logic lives in `lib/engine.js` and is shared by both the CLI and desktop app.

## Building a release

```bash
npm run dist       # vite build + electron-builder → release/*.dmg
```

## Design goals

- Keep track of which app is using which port
- Detect conflicts proactively
- Understand both live listeners and configured port claims
- Work with all `PORTMASTER.md` files, including global agent rules and repo-local ones
- Be cross-platform over time
