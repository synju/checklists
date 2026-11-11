// Electron main process: owns the window and the long-lived Rust `checklists serve` process.
// It bridges the renderer (Vue) <-> the service over stdio JSON-lines.

import { app, BrowserWindow, ipcMain } from 'electron'
import { spawn } from 'node:child_process'
import { createInterface } from 'node:readline'
import { existsSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const DEV_URL = process.env.VITE_DEV_SERVER_URL

// --- Rust service (stdio JSON-lines) ------------------------------------

let child = null
let nextId = 1
const pending = new Map() // id -> { resolve, reject }
let mainWindow = null

/** Locate the built `checklists` exe: env override, else the sibling Rust project's target. */
function resolveExe() {
  const exe = process.platform === 'win32' ? 'checklists.exe' : 'checklists'
  if (process.env.CHECKLISTS_EXE && existsSync(process.env.CHECKLISTS_EXE)) {
    return process.env.CHECKLISTS_EXE
  }
  // packaged: the sidecar bundled under resources/
  if (app.isPackaged) {
    const p = path.join(process.resourcesPath, exe)
    if (existsSync(p)) return p
  }
  // dev: app/electron -> repo root -> core/target/<profile>
  const base = path.resolve(__dirname, '..', '..', 'core', 'target')
  for (const profile of ['release', 'debug']) {
    const p = path.join(base, profile, exe)
    if (existsSync(p)) return p
  }
  return null
}

function startService() {
  const exe = resolveExe()
  if (!exe) {
    console.error('[checklists] service exe not found — build the Rust project first (cargo build --release)')
    return
  }
  console.log('[checklists] starting service:', exe)
  child = spawn(exe, ['serve'], { stdio: ['pipe', 'pipe', 'pipe'] })
  child.on('error', (e) => console.error('[checklists] service spawn error:', e))
  child.stderr.on('data', (d) => console.error('[checklists:svc]', String(d).trim()))
  child.on('exit', (code) => console.error('[checklists] service exited', code))

  const rl = createInterface({ input: child.stdout })
  rl.on('line', (line) => {
    if (!line.trim()) return
    let msg
    try { msg = JSON.parse(line) } catch { return }
    if (msg.event) {
      // pushed event (e.g. files changed on disk) -> tell the renderer
      mainWindow?.webContents.send('checklists:event', msg)
      return
    }
    const p = pending.get(msg.id)
    if (!p) return
    pending.delete(msg.id)
    if (msg.ok) p.resolve(msg.result)
    else p.reject(new Error(msg.error || 'service error'))
  })
}

function callService(method, params) {
  return new Promise((resolve, reject) => {
    if (!child || child.killed) return reject(new Error('service not running'))
    const id = nextId++
    pending.set(id, { resolve, reject })
    child.stdin.write(JSON.stringify({ id, method, params: params || {} }) + '\n')
    setTimeout(() => {
      if (pending.has(id)) { pending.delete(id); reject(new Error('service timeout')) }
    }, 5000)
  })
}

// --- window -------------------------------------------------------------

function createWindow() {
  const winIcon = path.join(__dirname, '..', 'build', 'icon.ico')
  mainWindow = new BrowserWindow({
    width: 1360,
    height: 860,
    minWidth: 900,
    minHeight: 600,
    backgroundColor: '#0c0c0c',
    frame: false,
    autoHideMenuBar: true,
    ...(existsSync(winIcon) ? { icon: winIcon } : {}),
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  })
  if (DEV_URL) {
    mainWindow.loadURL(DEV_URL)
  } else {
    mainWindow.loadFile(path.join(__dirname, '..', 'dist', 'index.html'))
  }
  mainWindow.webContents.on('did-finish-load', () => console.log('[checklists] window loaded'))
}

ipcMain.handle('checklists:call', (_e, method, params) => callService(method, params))

// Window controls for the custom (frameless) title bar.
ipcMain.on('win:minimize', () => mainWindow?.minimize())
ipcMain.on('win:toggleMaximize', () => {
  if (!mainWindow) return
  mainWindow.isMaximized() ? mainWindow.unmaximize() : mainWindow.maximize()
})
ipcMain.on('win:close', () => mainWindow?.close())

app.whenReady().then(() => {
  startService()
  createWindow()
  app.on('activate', () => { if (BrowserWindow.getAllWindows().length === 0) createWindow() })
})

app.on('window-all-closed', () => {
  try { child?.stdin.end() } catch {}
  if (process.platform !== 'darwin') app.quit()
})
