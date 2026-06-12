# Lighthouse

Lighthouse is a port-awareness tool for local development.

Current prototype includes:
- Desktop app (Tauri) for scanning live ports
- Detection of `PORTMASTER.md` files across your machine
- Scanning project config files for claimed ports
- CLI companion: `lh`

## Current CLI usage

From the repo root:

```bash
cargo run --bin lh -- list
cargo run --bin lh -- check 3000
cargo run --bin lh -- suggest 3000 3999
cargo run --bin lh -- portmasters
cargo run --bin lh -- projects
```

## Current desktop usage

```bash
cargo run --bin lighthouse --manifest-path src-tauri/Cargo.toml
```

## Design goals

- Keep track of which app is using which port
- Detect conflicts proactively
- Understand both live listeners and configured port claims
- Work with all `PORTMASTER.md` files, including global agent rules and repo-local ones
- Be cross-platform over time
