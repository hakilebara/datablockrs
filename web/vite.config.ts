import react from '@vitejs/plugin-react'
import {defineConfig} from 'vite'
import type {Plugin} from 'vite'

// jco emits `"use components"` at the top of the generated module as an
// asm.js-style telemetry marker for browser engines (formerly "use jco").
// Its whole job is to be seen by engines/tooling, so bundlers drop it by
// design -- rolldown just warns when doing so. Nothing in the app (or in the
// bundle) reads it, so strip it before the bundler sees it.
const stripUseComponents: Plugin = {
  name: 'strip-use-components',
  enforce: 'pre',
  transform(code, id) {
    if (!id.endsWith('/wasm-bindings/datablockrs.js')) return
    return code.replace(/^"use components";\r?\n/, '')
  },
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [stripUseComponents, react()],
  build: {
    target: "es2022",
    // The entry chunk is the jco runtime glue plus preview2-shim (~530 kB
    // minified, ~112 kB gzip); the 2.4 MB component is a separate .wasm asset.
    chunkSizeWarningLimit: 700,
  },
  // Keep the shim out of Vite's dependency prebundling: prebundled chunks each
  // bundle their own copy of its internals, so instanceof checks on its
  // resources (e.g. InputStream) fail across copies.
  optimizeDeps: {exclude: ["@bytecodealliance/preview2-shim"]},
})
