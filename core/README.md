# checklists-app (Rust core)

The Rust backend for the checklists desktop app (Electron + Vue frontend, added later). It is the
same JSON contract as the original terminal `checklist-app`, so the TUI, this app, and
Claude/agents all read and write **one shared `checklists/` directory**.

## Layout
- `src/model.rs` — `Item`, `Checklist`, `Progress`; tolerant parse of the shared JSON
  (bare array **or** `{ name?, items }`), and a writer that matches the original format.
- `src/store.rs` — `Store`: `list_ids`, `read`, `read_all`, `write` (atomic), `delete`,
  `toggle_item`.
- `src/config.rs` — `Config { checklist_dir }`, per-user `config.json`, default resolution.
- `src/main.rs` — a thin CLI (`checklists`) over the core.

## Shared store
The checklist directory is **read from config, never hardcoded**. Resolution order:
1. `config.json` (`checklist_dir`) — per user, at:
   - Windows: `%APPDATA%\checklists-app\config.json`
   - Linux/WSL: `$XDG_CONFIG_HOME/checklists-app/config.json` (or `~/.config/...`)
2. First-run default: `$CHECKLISTS_DIR`, else `C:\charlie\checklists`
   (WSL maps to `/mnt/c/charlie/checklists` when present).

## CLI
```
checklists [list]            list every checklist with progress
checklists show <id>         show one checklist's items
checklists toggle <id> <i>   toggle item <i>
checklists config            print the config file + store path
checklists init-config       write config.json with the current/default path
```

## Build & test
Requires the Rust toolchain (`cargo`). Currently available in WSL only:
```
cargo test
cargo run -- list
```

## JSON contract
One `<id>.json` per checklist. Either a bare array, or an object with a display `name`:
```json
{ "name": "Feature X", "items": [ { "label": "Do it", "done": false, "detail": "", "notes": "" } ] }
```
`detail` and `notes` are markdown. `notes` is kept only when non-empty. The `name` is preserved
on every write; the file is never renamed.
