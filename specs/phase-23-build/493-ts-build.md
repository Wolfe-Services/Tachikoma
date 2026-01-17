# 493 - TypeScript Build Configuration

**Phase:** 23 - Build & Distribution
**Spec ID:** 493
**Status:** Planned
**Dependencies:** 491-build-overview, 004-svelte-integration
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Configure optimized TypeScript and Svelte build settings using Vite, including code splitting, tree shaking, minification, and asset optimization for the web frontend.

---

## Acceptance Criteria

- [x] Vite configured for optimal production builds
- [x] Code splitting for lazy loading
- [x] Tree shaking removes unused code
- [x] Assets are compressed and optimized
- [x] Source maps available for debugging
- [x] Build analytics for bundle size tracking

---

## Implementation Details

### 1. Vite Configuration

Update `web/vite.config.ts`:

```typescript
import { defineConfig, loadEnv } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import path from 'path';
import { execSync } from 'child_process';
import { visualizer } from 'rollup-plugin-visualizer';

// Get build info
function getBuildInfo() {
  const version = process.env.npm_package_version ?? '0.0.0';
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
  const env = loadEnv(mode, process.cwd(), '');
  const isProduction = mode === 'production';
  const isAnalyze = process.env.ANALYZE === 'true';

  return {
    plugins: [
      svelte({
        compilerOptions: {
          dev: !isProduction,
        },
        hot: !isProduction,
      }),

      // Bundle analyzer
      isAnalyze && visualizer({
        filename: 'dist/stats.html',
        open: true,
        gzipSize: true,
        brotliSize: true,
      }),
    ].filter(Boolean),

    // Path aliases
    resolve: {
      alias: {
        '@': path.resolve(__dirname, 'src'),
        '@components': path.resolve(__dirname, 'src/lib/components'),
        '@stores': path.resolve(__dirname, 'src/lib/stores'),
        '@utils': path.resolve(__dirname, 'src/lib/utils'),
        '@types': path.resolve(__dirname, 'src/lib/types'),
      },
    },

    // Build-time constants
    define: {
      ...getBuildInfo(),
      'process.env.NODE_ENV': JSON.stringify(mode),
    },

    // Development server
    server: {
      port: 5173,
      strictPort: true,
      host: true,
    },

    // Build configuration
    build: {
      outDir: 'dist',
      sourcemap: isProduction ? 'hidden' : true,
      minify: isProduction ? 'esbuild' : false,

      // Target modern browsers
      target: 'es2020',

      // Chunk size warnings
      chunkSizeWarningLimit: 500,

      // Rollup options
      rollupOptions: {
        output: {
          // Manual chunk splitting
          manualChunks: {
            // Vendor chunks
            'vendor-svelte': ['svelte', 'svelte/store', 'svelte/transition'],
            'vendor-utils': ['lodash-es', 'date-fns'],

            // Feature chunks
            'feature-editor': [
              './src/lib/components/editor/index.ts',
            ],
            'feature-terminal': [
              './src/lib/components/terminal/index.ts',
            ],
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
          additionalData: `@import "@/styles/variables.scss";`,
        },
      },
      devSourcemap: true,
    },

    // Optimization
    optimizeDeps: {
      include: ['svelte', 'lodash-es'],
      exclude: ['@electron/remote'],
    },

    // esbuild configuration
    esbuild: {
      drop: isProduction ? ['console', 'debugger'] : [],
      legalComments: 'none',
    },
  };
});
```

### 2. TypeScript Configuration

Update `web/tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,

    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,

    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,

    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"],
      "@components/*": ["src/lib/components/*"],
      "@stores/*": ["src/lib/stores/*"],
      "@utils/*": ["src/lib/utils/*"],
      "@types/*": ["src/lib/types/*"]
    },

    "types": ["svelte", "vite/client"]
  },
  "include": ["src/**/*", "src/**/*.svelte"],
  "exclude": ["node_modules", "dist"]
}
```

### 3. Svelte Configuration

Update `web/svelte.config.js`:

```javascript
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/vite-plugin-svelte').SvelteConfig} */
export default {
  preprocess: vitePreprocess(),

  compilerOptions: {
    // Enable runes (Svelte 5)
    runes: true,

    // Accessibility warnings
    accessors: false,

    // CSS handling
    css: 'injected',
  },

  // Warning filters
  onwarn: (warning, handler) => {
    // Ignore specific warnings
    if (warning.code === 'css-unused-selector') return;
    if (warning.code === 'a11y-click-events-have-key-events') return;

    handler(warning);
  },

  // Extensions
  extensions: ['.svelte'],
};
```

### 4. Build Scripts

Update `web/package.json`:

```json
{
  "name": "tachikoma-web",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "build:analyze": "ANALYZE=true vite build",
    "preview": "vite preview",
    "check": "svelte-check --tsconfig ./tsconfig.json",
    "check:watch": "svelte-check --tsconfig ./tsconfig.json --watch",
    "lint": "eslint src --ext .ts,.svelte",
    "lint:fix": "eslint src --ext .ts,.svelte --fix",
    "format": "prettier --write src",
    "test": "vitest",
    "test:run": "vitest run",
    "test:coverage": "vitest run --coverage",
    "clean": "rm -rf dist .svelte-kit"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^3.0.0",
    "@types/node": "^20.0.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0",
    "rollup-plugin-visualizer": "^5.12.0",
    "sass": "^1.70.0",
    "svelte": "^5.0.0",
    "svelte-check": "^3.6.0",
    "typescript": "^5.3.0",
    "vite": "^5.0.0"
  }
}
```

### 5. Asset Optimization

Create `web/scripts/optimize-assets.ts`:

```typescript
#!/usr/bin/env ts-node
/**
 * Asset optimization script for production builds.
 */

import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

const DIST_DIR = path.join(__dirname, '../dist');
const ASSETS_DIR = path.join(DIST_DIR, 'assets');

interface OptimizationResult {
  file: string;
  originalSize: number;
  optimizedSize: number;
  savings: number;
}

const results: OptimizationResult[] = [];

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

function optimizeImages(): void {
  const imageDir = path.join(ASSETS_DIR, 'images');
  if (!fs.existsSync(imageDir)) return;

  console.log('Optimizing images...');

  const images = fs.readdirSync(imageDir);
  for (const image of images) {
    const imagePath = path.join(imageDir, image);
    const originalSize = fs.statSync(imagePath).size;

    // Use sharp or imagemin for optimization
    try {
      if (image.endsWith('.png')) {
        execSync(`pngquant --quality=65-80 --force --output "${imagePath}" "${imagePath}"`, {
          stdio: 'ignore',
        });
      } else if (image.endsWith('.jpg') || image.endsWith('.jpeg')) {
        execSync(`jpegoptim --max=80 "${imagePath}"`, { stdio: 'ignore' });
      }

      const optimizedSize = fs.statSync(imagePath).size;
      results.push({
        file: image,
        originalSize,
        optimizedSize,
        savings: originalSize - optimizedSize,
      });
    } catch (e) {
      // Tool not available, skip
    }
  }
}

function generateReport(): void {
  console.log('\n=== Asset Optimization Report ===\n');

  let totalOriginal = 0;
  let totalOptimized = 0;

  for (const result of results) {
    const percent = ((result.savings / result.originalSize) * 100).toFixed(1);
    console.log(
      `${result.file}: ${formatBytes(result.originalSize)} -> ${formatBytes(result.optimizedSize)} (${percent}% saved)`
    );
    totalOriginal += result.originalSize;
    totalOptimized += result.optimizedSize;
  }

  if (results.length > 0) {
    const totalSavings = totalOriginal - totalOptimized;
    const totalPercent = ((totalSavings / totalOriginal) * 100).toFixed(1);
    console.log(`\nTotal: ${formatBytes(totalOriginal)} -> ${formatBytes(totalOptimized)} (${totalPercent}% saved)`);
  }
}

// Run optimization
optimizeImages();
generateReport();
```

### 6. Build Verification

Create `web/scripts/verify-build.ts`:

```typescript
#!/usr/bin/env ts-node
/**
 * Verify production build integrity.
 */

import fs from 'fs';
import path from 'path';

const DIST_DIR = path.join(__dirname, '../dist');

interface VerificationResult {
  check: string;
  passed: boolean;
  message: string;
}

const results: VerificationResult[] = [];

function check(name: string, condition: boolean, message: string): void {
  results.push({ check: name, passed: condition, message });
}

// Check dist directory exists
check(
  'Dist directory exists',
  fs.existsSync(DIST_DIR),
  'dist/ directory should exist after build'
);

// Check index.html exists
check(
  'Index HTML exists',
  fs.existsSync(path.join(DIST_DIR, 'index.html')),
  'index.html should be in dist/'
);

// Check for JS assets
const assetsDir = path.join(DIST_DIR, 'assets/js');
const hasJsAssets = fs.existsSync(assetsDir) && fs.readdirSync(assetsDir).some(f => f.endsWith('.js'));
check('JS assets exist', hasJsAssets, 'JavaScript bundles should be in assets/js/');

// Check for no source maps in production (if configured)
const hasSourceMaps = fs.existsSync(assetsDir) && fs.readdirSync(assetsDir).some(f => f.endsWith('.map'));
check(
  'No exposed source maps',
  !hasSourceMaps || process.env.INCLUDE_SOURCE_MAPS === 'true',
  'Source maps should not be in production bundle'
);

// Check bundle sizes
if (fs.existsSync(assetsDir)) {
  const jsFiles = fs.readdirSync(assetsDir).filter(f => f.endsWith('.js'));
  for (const file of jsFiles) {
    const size = fs.statSync(path.join(assetsDir, file)).size;
    const sizeKB = size / 1024;
    check(
      `Bundle size: ${file}`,
      sizeKB < 500,
      `${file} is ${sizeKB.toFixed(1)}KB (should be <500KB)`
    );
  }
}

// Report results
console.log('\n=== Build Verification ===\n');
let allPassed = true;
for (const result of results) {
  const icon = result.passed ? '✓' : '✗';
  console.log(`${icon} ${result.check}: ${result.message}`);
  if (!result.passed) allPassed = false;
}

process.exit(allPassed ? 0 : 1);
```

---

## Testing Requirements

1. `npm run build` completes without errors
2. Bundle sizes are within acceptable limits
3. Code splitting creates appropriate chunks
4. Source maps are correctly configured
5. Build verification passes all checks

---

## Related Specs

- Depends on: [491-build-overview.md](491-build-overview.md), [004-svelte-integration.md](../phase-00-setup/004-svelte-integration.md)
- Next: [494-electron-packaging.md](494-electron-packaging.md)
- Related: [186-sveltekit-setup.md](../phase-09-ui-foundation/186-sveltekit-setup.md)
