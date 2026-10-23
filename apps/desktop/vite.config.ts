import { fileURLToPath, URL } from 'node:url'

import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

// Tauri expects a fixed port; build output and coverage are not part of the app.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  resolve: { alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) } },
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ['**/coverage/**', '**/target/**', '**/src-tauri/**'] },
  },
  build: { target: 'safari16', outDir: 'dist' },
})
