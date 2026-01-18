import { defineConfig, externalizeDepsPlugin } from 'electron-vite'
import { resolve } from 'path'
import copy from 'rollup-plugin-copy'

export default defineConfig({
  main: {
    plugins: [
      externalizeDepsPlugin(),
      copy({
        targets: [
          { src: 'main/native-bindings/*.node', dest: 'dist/main/native-bindings' },
          { src: 'main/native-bindings/index.js', dest: 'dist/main/native-bindings' },
          { src: 'main/native-bindings/index.d.ts', dest: 'dist/main/native-bindings' }
        ],
        hook: 'writeBundle'
      })
    ],
    build: {
      outDir: 'dist/main',
      sourcemap: true,
      rollupOptions: {
        input: {
          index: resolve(__dirname, 'main/index.ts')
        },
        external: [/\.node$/]
      }
    }
  },
  preload: {
    plugins: [externalizeDepsPlugin()],
    build: {
      outDir: 'dist/preload',
      sourcemap: true,
      rollupOptions: {
        input: {
          index: resolve(__dirname, 'preload/index.ts')
        }
      }
    }
  }
})