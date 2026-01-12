import { defineConfig, externalizeDepsPlugin } from 'electron-vite'
import { resolve } from 'path'

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin()],
    build: {
      outDir: 'dist/main',
      sourcemap: true,
      rollupOptions: {
        input: {
          index: resolve(__dirname, 'main/index.ts')
        }
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
  // Removed renderer section since we build web separately
})