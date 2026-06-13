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
