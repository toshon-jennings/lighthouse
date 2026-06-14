# Lighthouse Handoff

## Status: ELECTRON + VITE (Tauri banned -- see HITL.md)

## What Works
- Vite dev server on port 5189 serves the frontend
- Electron app loads the Vite dev server URL
- `nodeIntegration: true` + `contextIsolation: false` means renderer can use `require('child_process')` directly
- `invoke()` function in `src/index.html` calls `execSync('lsof ...')` directly -- no IPC bridge needed
- `loadPortmasters()` reads PORTMASTER.md files from disk
- CSS loads from `styles/main.css` via Vite dev server
- Logo at `assets/lh-logo.png`

## Real Engine (2026-06-13) -- no longer a stub
The headline differentiators (conflict detection + guided resolution) and the
inventory are now fully implemented and verified against the live machine:
- **Full socket inventory:** single `lsof -nP -iTCP -iUDP -F pcftPnT` pass via
  `scanSockets()` -> covers TCP **and UDP**, IPv4 **and IPv6**, and all socket
  states (parses the `t`/`P`/`n`/`T` fields, bracketed v6 addresses).
- **Conflict engine** (`detectConflicts`): reconciles live listeners vs
  PORTMASTER declarations. Detects (1) duplicate declarations (same port -> two
  owners) and (2) owner mismatch (live process != declared owner). Token-overlap
  matching avoids false positives (e.g. Docker Desktop == com.docker.backend).
- **Real `suggest_port`**: first free port excluding both live + declared (was
  hardcoded `3007`).
- **Honest `check_port`**: reports lingering `TIME_WAIT`/`CLOSE_WAIT` sockets so
  "free" never lies; suggestion range fixed (was `port+1..3999`, now ..65535).
- **Real resolve flow**: `find_port_references` greps config files under `$HOME`
  (depth 4, skips Library/node_modules/etc.); `apply_fix` rewrites a single
  validated line (refuses if the line drifted since preview). Preview + per-item
  confirm -- nothing is killed or rewritten without an explicit check.
- **Exposure flag**: wildcard binds (`0.0.0.0`/`::`) flagged in the table per
  PORTMASTER policy; `undeclared` ports marked.
- **`friendlyProcessName`**: recovers lsof's legacy 9-char-truncated names after
  the switch to `-F` full names (interpreter version-strip runs first so generic
  python/node aren't hijacked by machine-specific keys).

Verified by extracting the pure functions and running against live `lsof` +
real PORTMASTER.md (36 listeners, UDP+IPv6 seen, 5 real conflicts incl. the
duplicate port-8000 declaration). `npm run build` is clean.

## Second copy (Opal / Perci)
Opal embeds a "Lighthouse Mode" with a *different* architecture: React UI
(`opal/src/components/LighthouseMode.jsx`) -> `window.electron.lighthouse*`
(`opal/electron/preload.cjs`) -> IPC handlers (`opal/electron/main.cjs`). The
**same engine was mirrored** into `main.cjs` (scanSockets/detectConflicts/
suggestFree/findPortReferences/friendlyProcessName), two new bridges added
(`lighthouseFindReferences`, `lighthouseApplyFix`), and the JSX now renders a
conflicts banner + resolve modal (preview+confirm) + exposure/undeclared/proto
flags + transient-state in Quick Check. Engine verified identically; both
`vite build`s pass. NOTE: logic is duplicated across the two repos (separate
apps, different integration) -- keep them in sync when editing.

## Project Structure
- `src/index.html` -- main UI (all JS inline)
- `src/styles/main.css` -- dark theme, side rail layout
- `src/assets/lh-logo.png` -- logo
- `electron/main.js` -- minimal Electron main process
- `vite.config.ts` -- Vite config (root: src, port 5189)
- `package.json` -- scripts: `dev` (vite), `electron`, `start` (both)

## How to Run
```
npm run electron
```
This runs `concurrently -k -s first` to start the Vite dev server, `wait-on`s
port 5189, then launches Electron. `-k` kills Vite when Electron quits; `-s
first` makes the command exit with Electron's exit code. `npm start` is an
alias for the same thing.

## Key Architecture Decisions
- **NO Tauri** (banned per HITL.md)
- **NO preload script** -- `nodeIntegration: true` gives renderer direct Node.js access
- **NO IPC bridge** -- `invoke()` uses `require('child_process').execSync()` directly
- **NO separate API server** -- everything runs in the Electron renderer
- PORTMASTER.md is read from `~/.config/agent-rules/PORTMASTER.md` and walked directories

## Known Issues
- None currently.
- Possible follow-up: extract the shared engine into one module instead of
  duplicating it across `lighthouse/` and `opal/` (deferred -- separate apps,
  cross-repo packaging not worth it yet).

## Resolved
- **macOS window now shows reliably.** `electron/main.js` sets
  `app.setActivationPolicy('regular')` (NSApplicationActivationPolicyRegular) +
  `app.dock.show()` on darwin, creates the window with `show: false`, and shows
  + focuses it on `ready-to-show`, with `app.focus({ steal: true })` after
  `whenReady`.
- **`npm run electron` starts Vite + Electron together** via concurrently (see
  How to Run).
- **Removed unused `mockState`** from `src/index.html`.
- **Fixed duplicate PORTMASTER.md file rows** -- `get_portmaster_files` now
  dedupes by file path (was returning one entry per table row).
- `server.js` and `src/dist/` removed (no longer needed).
