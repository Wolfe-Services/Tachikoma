import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';
import path from 'path';
import { execSync } from 'child_process';
import { visualizer } from 'rollup-plugin-visualizer';

// Get build info
function getBuildInfo() {
  const version = process.env.npm_package_version ?? '0.1.0';
  let commit = 'unknown';
  let branch = 'unknown';

  try {
    commit = execSync('git rev-parse --short HEAD').toString().trim();
    branch = execSync('git rev-parse --abbrev-ref HEAD').toString().trim();
  } catch {}

  return {
    __APP_VERSION__: JSON.stringify(version),
    __GIT_COMMIT__: JSON.stringify(commit),
    __GIT_BRANCH__: JSON.stringify(branch),
    __BUILD_TIME__: JSON.stringify(new Date().toISOString()),
    __BUILD_ENV__: JSON.stringify(process.env.NODE_ENV || 'development'),
  };
}

export default defineConfig(({ mode }) => {
  // Load env from both web/ and parent directory (repo root)
  const env = {
    ...loadEnv(mode, path.resolve(__dirname, '..'), ''),
    ...loadEnv(mode, process.cwd(), ''),
  };
  const isProduction = mode === 'production';
  const isAnalyze = process.env.ANALYZE === 'true';
  const isTauri = process.env.TAURI_FAMILY;

  return {
    plugins: [
      sveltekit(),

      // Bundle analyzer
      isAnalyze && visualizer({
        filename: 'build/stats.html',
        open: true,
        gzipSize: true,
        brotliSize: true,
      }),
    ].filter(Boolean),

    // Path aliases - handled by SvelteKit
    resolve: {
      alias: {
        // Keep only non-SvelteKit aliases that might be needed
        '@': path.resolve(__dirname, 'src'),
      },
    },

    // Build-time constants
    define: {
      ...getBuildInfo(),
      'process.env.NODE_ENV': JSON.stringify(mode),
    },

    // Development server - different config for Tauri vs web
    server: isTauri ? {
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
    } : {
      port: 5173,
      strictPort: true,
      host: true,
    },

    // Environment variables - look in parent directory for .env
    envDir: path.resolve(__dirname, '..'),
    envPrefix: ['VITE_', 'TAURI_', 'ANTHROPIC_', 'OPENAI_', 'GEMINI_', 'GOOGLE_'],

    // Build configuration
    build: {
      outDir: 'build',
      sourcemap: isProduction ? 'hidden' : true,
      minify: isProduction ? 'esbuild' : false,

      // Target modern browsers for web, specific targets for Tauri
      target: isTauri ? ['es2021', 'chrome100', 'safari13'] : 'es2020',

      // Chunk size warnings
      chunkSizeWarningLimit: 500,

      // Rollup options
      rollupOptions: {
        output: {
          // Manual chunk splitting - avoid SvelteKit as it's externalized
          manualChunks(id) {
            // Node modules chunking
            if (id.includes('node_modules')) {
              // Don't chunk SvelteKit components as they're externalized
              if (id.includes('@sveltejs/kit')) {
                return;
              }
              
              // Svelte runtime
              if (id.includes('svelte')) {
                return 'vendor-svelte';
              }
              
              // Tauri API
              if (isTauri && id.includes('@tauri-apps/api')) {
                return 'vendor-tauri';
              }
              
              // Large utility libraries
              if (id.includes('highlight.js')) {
                return 'vendor-highlight';
              }
              
              if (id.includes('marked')) {
                return 'vendor-markdown';
              }
              
              if (id.includes('mermaid')) {
                return 'vendor-diagram';
              }
              
              // Other vendor dependencies
              return 'vendor-utils';
            }
            
            // Feature-based chunking for our own code
            if (id.includes('/lib/components/forge/')) {
              return 'feature-forge';
            }
            if (id.includes('/lib/components/mission/')) {
              return 'feature-mission';
            }
            if (id.includes('/lib/components/spec-browser/')) {
              return 'feature-spec-browser';
            }
          },

          // Asset naming
          assetFileNames: (assetInfo) => {
            const info = assetInfo.name?.split('.') ?? [];
            const ext = info[info.length - 1];

            if (/png|jpe?g|svg|gif|tiff|bmp|ico/i.test(ext)) {
              return 'assets/images/[name]-[hash][extname]';
            }
            if (/woff2?|eot|ttf|otf/i.test(ext)) {
              return 'assets/fonts/[name]-[hash][extname]';
            }
            return 'assets/[name]-[hash][extname]';
          },

          chunkFileNames: 'assets/js/[name]-[hash].js',
          entryFileNames: 'assets/js/[name]-[hash].js',
        },
      },

      // CSS configuration
      cssCodeSplit: true,
      cssMinify: isProduction,
    },

    // CSS preprocessing
    css: {
      preprocessorOptions: {
        scss: {
          additionalData: `@import "src/lib/styles/variables.scss";`,
        },
      },
      devSourcemap: true,
    },

    // Optimization
    optimizeDeps: {
      include: ['svelte', '@sveltejs/kit'],
      exclude: isTauri ? ['@tauri-apps/api'] : [],
    },

    // esbuild configuration
    esbuild: {
      drop: isProduction ? ['console', 'debugger'] : [],
      legalComments: 'none',
    },

    // Test configuration
    test: {
      include: ['src/**/*.{test,spec}.{js,ts}'],
      environment: 'jsdom',
    },
  };
});