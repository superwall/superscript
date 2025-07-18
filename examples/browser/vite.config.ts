import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'

import wasmPlugin from "vite-plugin-wasm"

import topLevelAwait from "vite-plugin-top-level-await";


export default defineConfig({
  plugins: [react(), topLevelAwait(), wasmPlugin()],
  base: '/superscript/',
})

