import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { execSync } from 'child_process';

function getBuildInfo() {
  const version = process.env.npm_package_version ?? '0.0.0';
  let commit = 'unknown';
  try {
    commit = execSync('git rev-parse --short HEAD').toString().trim();
  } catch {}

  return {
    __APP_VERSION__: JSON.stringify(version),
    __GIT_COMMIT__: JSON.stringify(commit),
    __BUILD_TIME__: JSON.stringify(new Date().toISOString()),
    __PLATFORM__: JSON.stringify(process.platform)
  };
}

export default defineConfig({
  plugins: [sveltekit()],

  // Build-time constants
  define: getBuildInfo(),

  // Electron renderer configuration
  base: './',

  build: {
    target: 'esnext',
    minify: 'esbuild',
    sourcemap: true,
    rollupOptions: {
      // Optimize chunking for better caching
      output: {
        manualChunks: {
          // Only include external libraries that are not bundled by SvelteKit
          vendor: ['svelte']
        }
      }
    }
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