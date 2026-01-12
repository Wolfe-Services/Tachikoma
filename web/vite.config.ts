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
  },

  // Enable source maps for development
  css: {
    devSourcemap: true
  }
});