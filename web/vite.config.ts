import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],

  // Electron renderer configuration
  base: './',

  build: {
    target: 'esnext',
    minify: 'esbuild'
  },

  server: {
    port: 5173,
    strictPort: true
  }
});