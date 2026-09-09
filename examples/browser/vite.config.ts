import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'

import wasmPlugin from "vite-plugin-wasm"

export default defineConfig({
  plugins: [react(), wasmPlugin()],
  base: '/superscript/',
  // vite-plugin-wasm emits top-level await; targeting esnext lets Vite keep
  // it natively instead of needing vite-plugin-top-level-await (whose SWC
  // transform breaks against current @swc/core versions).
  build: { target: 'esnext' },
})
