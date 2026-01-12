import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],

  // Tauri expects a fixed port
  server: {
    port: 1420,
    strictPort: true,
    host: '127.0.0.1',
    hmr: {
      protocol: 'ws',
      host: '127.0.0.1',
      port: 1421
    },
    watch: {
      ignored: ['**/src-tauri/**']
    }
  },

  // Optimize for Tauri
  build: {
    target: ['es2021', 'chrome100', 'safari13'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['@tauri-apps/api']
        }
      }
    }
  },

  // Environment variables
  envPrefix: ['VITE_', 'TAURI_'],

  // Resolve configuration
  resolve: {
    alias: {
      $lib: '/src/lib'
    }
  }
});