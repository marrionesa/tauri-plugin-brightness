// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Adapted from the shared config of the official plugins:
// the ESM and CommonJS bundles are emitted next to the type declarations, and a
// second build emits the IIFE bundle used by applications configured with
// `withGlobalTauri`, which is what `build.rs` points at.

import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { cwd } from 'node:process'

import { nodeResolve } from '@rollup/plugin-node-resolve'
import terser from '@rollup/plugin-terser'
import typescript from '@rollup/plugin-typescript'

const pkg = JSON.parse(readFileSync(join(cwd(), 'package.json'), 'utf8'))

const pluginName = pkg.name.replace('@tauri-apps/plugin-', '')
const iifeVarName = `__TAURI_PLUGIN_${pluginName.replace('-', '_').toUpperCase()}__`
const jsGlobalName = pluginName.replace(/-./g, (x) => x[1].toUpperCase())

const onwarn = (warning) => {
  throw Object.assign(new Error(), warning)
}

export default [
  {
    input: 'guest-js/index.ts',
    output: [
      { file: pkg.exports.import, format: 'esm' },
      { file: pkg.exports.require, format: 'cjs' }
    ],
    plugins: [
      typescript({
        declaration: true,
        declarationDir: dirname(pkg.exports.import)
      })
    ],
    external: [/^@tauri-apps\/api/, ...Object.keys(pkg.dependencies || {})],
    onwarn
  },
  {
    input: 'guest-js/index.ts',
    output: {
      format: 'iife',
      name: iifeVarName,
      // The IIFE is wrapped in an existence check because it is only ever
      // loaded in applications built with `withGlobalTauri`, where the core API
      // is already on `window.__TAURI__`.
      banner: "if ('__TAURI__' in window) {",
      footer: `Object.defineProperty(window.__TAURI__, '${jsGlobalName}', { value: ${iifeVarName} }) }`,
      file: 'api-iife.js',
      // Resolve `@tauri-apps/api/*` to `window.__TAURI__.*` instead of bundling
      // a private copy, so that values crossing the two share the same classes
      // and `instanceof` checks keep working.
      globals: (id) =>
        id.startsWith('@tauri-apps/api/')
          ? `window.__TAURI__.${id.slice('@tauri-apps/api/'.length)}`
          : id
    },
    external: [/^@tauri-apps\/api\//],
    plugins: [typescript(), terser(), nodeResolve()],
    onwarn
  }
]
