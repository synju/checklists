# Checklists

A desktop checklist app with a live, **agent-friendly JSON store**. A Rust core + stdio service, an
Electron + Vue 3 GUI. One `*.json` file per checklist in a shared folder — editable by hand, by the
app, or by automation/agents — with live updates, markdown detail & notes, six themes, keyboard
navigation, and a completion chime.

It started as a terminal TUI; this is the desktop port that keeps the exact same JSON contract, so a
TUI, the GUI, and scripts/agents can all drive one shared store at once.

## Structure

- **`core/`** — Rust: the checklist model, file store, per-app config, a small CLI, and a stdio
  JSON-lines service (`checklists serve`). No GUI dependencies; cross-platform.
- **`app/`** — Electron + Vue 3 desktop GUI. Spawns the core as a sidecar and talks to it over stdio
  JSON-lines; renders the store live and writes changes back through it.

## The JSON contract

One `<id>.json` per checklist in the store directory (default `~/checklists`, override with the
`CHECKLISTS_DIR` env var or each app's `config.json`). A file is **either** a bare array **or**
`{ "name"?: string, "items": [...] }`:

```json
{
  "name": "Ship v1.0",
  "items": [
    { "label": "Run the tests", "done": true },
    { "label": "Publish the notes", "done": false, "detail": "Highlight **breaking changes**.", "notes": "Draft during the freeze." }
  ]
}
```

- `label` (string), `done` (bool), `detail?` and `notes?` (both **markdown**).
- The display `name` is preserved on every write; the file is never renamed.
- Agents just drop or edit files; the running app polls the folder and reacts live.

## Build & run

**Core (Rust):**
```
cd core
cargo build --release      # produces the `checklists` binary
cargo run -- list          # or: show <id>, toggle <id> <i>, serve, config
```

**App (Electron + Vue):**
```
cd app
npm install
npm run dev                # Electron window (spawns the core, reads ~/checklists)
npm run vite               # browser-only preview at http://localhost:5173 (sample data)
```

**Package (Windows installer):** build the core in release first, then
```
cd app
npm run dist               # -> app/release/Checklist-Setup-<ver>.exe  (bundles the core as a sidecar)
```

## Features

Live store sync, keyboard nav (`↑↓`/`jk`, `space`, `←→`, `Enter`), inline detail mode, markdown
detail/notes, completion chime + toast, a tick sound on agent ticks, editing modals (right-click or
`e` `a` `r` `n`), six switchable themes (Config, `c`), sidebar toggle (`Ctrl+L`), and focus mode
(`f`). Settings persist to a per-app `config.json`.

## License

[MIT](LICENSE).
