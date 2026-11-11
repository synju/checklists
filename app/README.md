# checklists-app (desktop GUI)

Electron + Vue 3 desktop app for the shared JSON checklist store — the GUI counterpart to the
terminal `checklist-app`. Talks to the Rust core (`checklists serve`) over stdio JSON-lines.

## Layout
- `electron/main.js` — frameless window; spawns the Rust `checklists serve` sidecar; bridges the
  renderer ↔ service over IPC; forwards live `changed` events; window controls.
- `electron/preload.cjs` — exposes a safe `window.checklists` + `window.win` API.
- `src/` — Vue app: Sidebar, ProgressHeader, Checklist, Detail/Notes panels, DetailView, modals
  (actions, edit, confirm, prompt, config). `api.js` talks to the service, or falls back to
  `sample.js` data in a plain browser so the UI previews without the backend. `themes.js` + the
  `:root[data-theme]` blocks in `style.css` drive theming. `chime.js` synthesizes the sounds.

## Develop
```
npm install
npm run dev      # Electron window (spawns the Rust exe, reads C:\charlie\checklists)
npm run vite     # browser-only preview at http://localhost:5173 (sample data)
```

## Package
First build the Rust sidecar: in `../core`, `cargo build --release`. Then:
```
npm run dist     # NSIS installer -> release/Checklist-Setup-<ver>.exe
npm run pack     # unpacked app   -> release/win-unpacked/
```
electron-builder bundles `core/target/release/checklists.exe` as a sidecar under
`resources/`; the main process resolves it via `process.resourcesPath` when packaged. Unsigned
(SmartScreen will warn); add an icon at `build/icon.ico` and a signing cert for a release build.

## Features
Live store sync, keyboard nav (↑↓/jk, space, ←→, Enter), inline detail mode, markdown
detail/notes, completion chime + toast, agent-tick sound, editing modals (right-click / `e` `a`
`r` `n`), 6 themes (Config modal, `c`), sidebar toggle (`Ctrl+L` / ☰), focus mode (`f`).
