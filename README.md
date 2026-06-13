# Lighthouse

Lighthouse is a port-awareness tool for local development. It's a desktop app
(Electron + Vite) that shows you what's listening on your machine, who owns each
port, and where conflicts are.

Features:
- Scans live listening ports via `lsof`
- Detects `PORTMASTER.md` files across your machine and annotates ports with
  their declared service/owner
- Per-process detail: PID, parent, start time, working directory, command
- Quick port check and free-port suggestions
- Conflict detection with a guided resolve flow

## Install

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
