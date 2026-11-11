import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// base './' so the built index.html loads from file:// inside Electron.
export default defineConfig({
  plugins: [vue()],
  base: './',
  build: { outDir: 'dist', emptyOutDir: true },
  server: { port: 5173, strictPort: true },
})
