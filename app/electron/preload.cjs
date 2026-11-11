// Preload: expose a tiny, safe API to the renderer (no Node access in the page).
const { contextBridge, ipcRenderer } = require('electron')

contextBridge.exposeInMainWorld('checklists', {
  // call a service method, returns a Promise of the result
  call: (method, params) => ipcRenderer.invoke('checklists:call', method, params),
  // subscribe to pushed events (e.g. { event: "changed" }); returns an unsubscribe fn
  onEvent: (cb) => {
    const handler = (_e, msg) => cb(msg)
    ipcRenderer.on('checklists:event', handler)
    return () => ipcRenderer.removeListener('checklists:event', handler)
  },
})

contextBridge.exposeInMainWorld('win', {
  minimize: () => ipcRenderer.send('win:minimize'),
  toggleMaximize: () => ipcRenderer.send('win:toggleMaximize'),
  close: () => ipcRenderer.send('win:close'),
})

contextBridge.exposeInMainWorld('store', {
  change: () => ipcRenderer.invoke('store:change'),
})
