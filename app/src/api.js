// Client for the Rust service. In Electron it goes over IPC -> `checklists serve`.
// In a plain browser (vite preview) it falls back to in-memory sample data so the UI renders.
import {
  mockList, mockGet, mockToggle, mockRename, mockEditItem,
  mockAddItem, mockDeleteItem, mockDelete, mockCreate, mockConfig, mockSetConfig,
} from './sample.js'

const bridge = typeof window !== 'undefined' ? window.checklists : null
export const isLive = !!bridge

export async function call(method, params = {}) {
  if (bridge) return bridge.call(method, params)
  // --- browser preview fallback ---
  switch (method) {
    case 'list': return mockList()
    case 'get': return mockGet(params.id)
    case 'toggle': return mockToggle(params.id, params.index)
    case 'set_done': return mockToggle(params.id, params.index, params.done)
    case 'rename': return mockRename(params.id, params.name)
    case 'edit_item': return mockEditItem(params.id, params.index, params.label, params.detail)
    case 'add_item': return mockAddItem(params.id, params.label)
    case 'delete_item': return mockDeleteItem(params.id, params.index)
    case 'delete': return mockDelete(params.id)
    case 'create': return mockCreate(params.id, params.name)
    case 'config': return mockConfig()
    case 'get_config': return mockConfig()
    case 'set_config': return mockSetConfig(params)
    default: throw new Error('unknown method ' + method)
  }
}

export function onEvent(cb) {
  return bridge ? bridge.onEvent(cb) : () => {}
}

// Window controls (Electron frameless). No-ops in the browser.
export const win = (typeof window !== 'undefined' && window.win) || {
  minimize() {}, toggleMaximize() {}, close() {},
}
