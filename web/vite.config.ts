import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],

  // Electron renderer configuration
  base: './',

  build: {
    target: 'esnext',
    minify: 'esbuild',
    sourcemap: true
  },

  server: {
    port: 5173,
    strictPort: true,
    hmr: {
      port: 5173
    }
  },

  // Enable source maps for development
  css: {
    devSourcemap: true
  }
});