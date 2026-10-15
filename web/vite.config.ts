import react from '@vitejs/plugin-react'
import {defineConfig} from 'vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  build: {target: "es2022"},
  // Keep the shim out of Vite's dependency prebundling: prebundled chunks each
  // bundle their own copy of its internals, so instanceof checks on its
  // resources (e.g. InputStream) fail across copies.
  optimizeDeps: {exclude: ["@bytecodealliance/preview2-shim"]},
})
