# Spec 495: Svelte Bundling

## Phase
23 - Build/Package System

## Spec ID
495

## Status
Planned

## Dependencies
- Spec 491 (Build System Orchestration)
- Spec 492 (Build Configuration)
- Spec 186 (SvelteKit Setup)

## Estimated Context
~10%

---

## Objective

Implement the Svelte/SvelteKit bundling pipeline using Vite. This includes optimization for Electron's renderer process, SSR disabling for desktop builds, code splitting, tree shaking, and asset optimization.

---

## Acceptance Criteria

- [ ] Vite configuration optimized for Electron renderer
- [ ] SvelteKit adapter for static output
- [ ] Development hot module replacement (HMR)
- [ ] Production minification and tree shaking
- [ ] CSS optimization and autoprefixing
- [ ] Asset optimization (images, fonts)
- [ ] Bundle analysis and size reporting
- [ ] Source map generation for development
- [ ] Environment variable handling
- [ ] Build caching for faster rebuilds

---

## Implementation Details

### Vite Configuration (web/vite.config.ts)

```typescript
// web/vite.config.ts
import { defineConfig, loadEnv, type Plugin } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { execSync } from 'child_process';
import path from 'path';

// Build info plugin
function buildInfoPlugin(): Plugin {
  return {
    name: 'build-info',
    config(_, { mode }) {
      const version = process.env.npm_package_version ?? '0.0.0';
      let gitCommit = 'unknown';
      let gitBranch = 'unknown';

      try {
        gitCommit = execSync('git rev-parse --short HEAD').toString().trim();
        gitBranch = execSync('git rev-parse --abbrev-ref HEAD').toString().trim();
      } catch {
        // Git not available
      }

      return {
        define: {
          __APP_VERSION__: JSON.stringify(version),
          __GIT_COMMIT__: JSON.stringify(gitCommit),
          __GIT_BRANCH__: JSON.stringify(gitBranch),
          __BUILD_TIME__: JSON.stringify(new Date().toISOString()),
          __BUILD_MODE__: JSON.stringify(mode),
          __IS_ELECTRON__: JSON.stringify(process.env.ELECTRON === 'true'),
        },
      };
    },
  };
}

// Bundle analyzer plugin (conditional)
async function conditionalAnalyzer(): Promise<Plugin | null> {
  if (process.env.ANALYZE !== 'true') return null;

  const { visualizer } = await import('rollup-plugin-visualizer');
  return visualizer({
    filename: 'dist/stats.html',
    open: true,
    gzipSize: true,
    brotliSize: true,
  }) as Plugin;
}

export default defineConfig(async ({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');
  const isProduction = mode === 'production';
  const isElectron = env.ELECTRON === 'true';
  const analyzerPlugin = await conditionalAnalyzer();

  return {
    plugins: [
      sveltekit(),
      buildInfoPlugin(),
      analyzerPlugin,
    ].filter(Boolean) as Plugin[],

    // Resolve configuration
    resolve: {
      alias: {
        $lib: path.resolve('./src/lib'),
        $components: path.resolve('./src/lib/components'),
        $stores: path.resolve('./src/lib/stores'),
        $utils: path.resolve('./src/lib/utils'),
        $types: path.resolve('./src/lib/types'),
      },
    },

    // Build configuration
    build: {
      target: isElectron ? 'chrome120' : 'es2020',
      outDir: 'dist',
      assetsDir: 'assets',
      sourcemap: !isProduction,
      minify: isProduction ? 'esbuild' : false,
      cssMinify: isProduction,
      reportCompressedSize: true,

      // Rollup options
      rollupOptions: {
        output: {
          // Manual chunk splitting
          manualChunks: isProduction
            ? {
                vendor: ['svelte', '@sveltejs/kit'],
                ui: ['@floating-ui/dom', 'lucide-svelte'],
              }
            : undefined,

          // Asset file naming
          assetFileNames: (assetInfo) => {
            const info = assetInfo.name?.split('.') ?? [];
            const ext = info[info.length - 1];

            if (/png|jpe?g|svg|gif|tiff|bmp|ico/i.test(ext)) {
              return `assets/images/[name]-[hash][extname]`;
            }
            if (/woff2?|eot|ttf|otf/i.test(ext)) {
              return `assets/fonts/[name]-[hash][extname]`;
            }
            return `assets/[name]-[hash][extname]`;
          },

          // Chunk file naming
          chunkFileNames: isProduction
            ? 'assets/js/[name]-[hash].js'
            : 'assets/js/[name].js',

          // Entry file naming
          entryFileNames: isProduction
            ? 'assets/js/[name]-[hash].js'
            : 'assets/js/[name].js',
        },

        // External modules for Electron
        external: isElectron
          ? ['electron', 'original-fs', 'path', 'fs', 'crypto']
          : [],
      },

      // Chunk size warnings
      chunkSizeWarningLimit: 1000,
    },

    // CSS configuration
    css: {
      devSourcemap: !isProduction,
      postcss: {
        plugins: [
          require('tailwindcss'),
          require('autoprefixer'),
          ...(isProduction
            ? [
                require('cssnano')({
                  preset: [
                    'default',
                    {
                      discardComments: { removeAll: true },
                      normalizeWhitespace: true,
                    },
                  ],
                }),
              ]
            : []),
        ],
      },
    },

    // Server configuration (development)
    server: {
      port: 5173,
      strictPort: true,
      host: true,
      hmr: {
        protocol: 'ws',
        port: 5173,
      },
      watch: {
        usePolling: false,
      },
    },

    // Preview configuration
    preview: {
      port: 4173,
      strictPort: true,
    },

    // Optimization configuration
    optimizeDeps: {
      include: ['svelte', '@sveltejs/kit'],
      exclude: ['@sveltejs/kit'],
    },

    // esbuild configuration
    esbuild: {
      drop: isProduction ? ['console', 'debugger'] : [],
      legalComments: 'none',
    },

    // Worker configuration
    worker: {
      format: 'es',
    },
  };
});
```

### SvelteKit Configuration (web/svelte.config.js)

```javascript
// web/svelte.config.js
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Preprocessors
  preprocess: vitePreprocess({
    postcss: true,
  }),

  // Compiler options
  compilerOptions: {
    // Enable runes mode for Svelte 5
    runes: true,
    // Development mode options
    dev: process.env.NODE_ENV !== 'production',
    // Accessibility warnings
    warningFilter: (warning) => {
      // Ignore specific warnings if needed
      if (warning.code === 'a11y-click-events-have-key-events') {
        return false;
      }
      return true;
    },
  },

  // Kit configuration
  kit: {
    // Static adapter for Electron
    adapter: adapter({
      pages: 'dist',
      assets: 'dist',
      fallback: 'index.html',
      precompress: true,
      strict: true,
    }),

    // App directory
    appDir: '_app',

    // Paths
    paths: {
      base: '',
      relative: true,
    },

    // Prerender configuration
    prerender: {
      crawl: true,
      entries: ['*'],
      handleHttpError: 'warn',
      handleMissingId: 'warn',
    },

    // CSP configuration (handled by Electron)
    csp: {
      mode: 'auto',
    },

    // Alias configuration
    alias: {
      $lib: 'src/lib',
      $components: 'src/lib/components',
      $stores: 'src/lib/stores',
      $utils: 'src/lib/utils',
      $types: 'src/lib/types',
    },

    // TypeScript configuration
    typescript: {
      config: (config) => ({
        ...config,
        compilerOptions: {
          ...config.compilerOptions,
          strict: true,
          noUncheckedIndexedAccess: true,
        },
      }),
    },

    // Environment configuration
    env: {
      publicPrefix: 'PUBLIC_',
      privatePrefix: '',
    },

    // Output configuration
    output: {
      preloadStrategy: 'modulepreload',
    },
  },

  // Vite plugin options
  vitePlugin: {
    inspector: {
      toggleKeyCombo: 'meta-shift-i',
      showToggleButton: 'active',
      toggleButtonPos: 'bottom-right',
    },
  },
};

export default config;
```

### PostCSS Configuration (web/postcss.config.js)

```javascript
// web/postcss.config.js
export default {
  plugins: {
    'tailwindcss/nesting': {},
    tailwindcss: {},
    autoprefixer: {},
    ...(process.env.NODE_ENV === 'production'
      ? {
          cssnano: {
            preset: [
              'default',
              {
                discardComments: { removeAll: true },
                normalizeWhitespace: true,
                colormin: true,
                minifyFontValues: true,
                minifyGradients: true,
              },
            ],
          },
        }
      : {}),
  },
};
```

### Tailwind Configuration (web/tailwind.config.ts)

```typescript
// web/tailwind.config.ts
import type { Config } from 'tailwindcss';
import forms from '@tailwindcss/forms';
import typography from '@tailwindcss/typography';

export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],

  darkMode: 'class',

  theme: {
    extend: {
      colors: {
        // Custom color palette
        primary: {
          50: '#f0f9ff',
          100: '#e0f2fe',
          200: '#bae6fd',
          300: '#7dd3fc',
          400: '#38bdf8',
          500: '#0ea5e9',
          600: '#0284c7',
          700: '#0369a1',
          800: '#075985',
          900: '#0c4a6e',
          950: '#082f49',
        },
        surface: {
          light: '#ffffff',
          dark: '#1e1e1e',
        },
      },

      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },

      animation: {
        'fade-in': 'fadeIn 0.2s ease-out',
        'slide-up': 'slideUp 0.3s ease-out',
        'spin-slow': 'spin 3s linear infinite',
      },

      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideUp: {
          '0%': { opacity: '0', transform: 'translateY(10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
      },
    },
  },

  plugins: [forms, typography],
} satisfies Config;
```

### Build Script (web/scripts/build.ts)

```typescript
// web/scripts/build.ts
import { build } from 'vite';
import { exec } from 'child_process';
import { promisify } from 'util';
import * as fs from 'fs';
import * as path from 'path';

const execAsync = promisify(exec);

interface BuildOptions {
  mode: 'development' | 'production';
  analyze: boolean;
  electron: boolean;
  sourcemaps: boolean;
}

async function buildWeb(options: BuildOptions): Promise<void> {
  console.log(`Building web app (${options.mode})...`);

  // Set environment variables
  process.env.NODE_ENV = options.mode;
  process.env.ANALYZE = options.analyze ? 'true' : 'false';
  process.env.ELECTRON = options.electron ? 'true' : 'false';

  try {
    // Run SvelteKit sync first
    console.log('Running svelte-kit sync...');
    await execAsync('npx svelte-kit sync');

    // Type check
    console.log('Running type check...');
    await execAsync('npx svelte-check --tsconfig ./tsconfig.json');

    // Build with Vite
    console.log('Building with Vite...');
    await build({
      mode: options.mode,
      build: {
        sourcemap: options.sourcemaps,
      },
    });

    // Generate build manifest
    await generateBuildManifest(options);

    console.log('Build complete!');
  } catch (error) {
    console.error('Build failed:', error);
    throw error;
  }
}

async function generateBuildManifest(options: BuildOptions): Promise<void> {
  const manifest = {
    version: process.env.npm_package_version,
    mode: options.mode,
    buildTime: new Date().toISOString(),
    gitCommit: await getGitCommit(),
    files: await getOutputFiles(),
  };

  const manifestPath = path.join('dist', 'build-manifest.json');
  fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));
}

async function getGitCommit(): Promise<string> {
  try {
    const { stdout } = await execAsync('git rev-parse --short HEAD');
    return stdout.trim();
  } catch {
    return 'unknown';
  }
}

async function getOutputFiles(): Promise<string[]> {
  const distPath = 'dist';
  const files: string[] = [];

  function walk(dir: string): void {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        walk(fullPath);
      } else {
        files.push(path.relative(distPath, fullPath));
      }
    }
  }

  if (fs.existsSync(distPath)) {
    walk(distPath);
  }

  return files;
}

// CLI entry point
const args = process.argv.slice(2);
const options: BuildOptions = {
  mode: args.includes('--production') ? 'production' : 'development',
  analyze: args.includes('--analyze'),
  electron: args.includes('--electron'),
  sourcemaps: args.includes('--sourcemaps'),
};

buildWeb(options).catch(() => process.exit(1));
```

### Package.json Scripts

```json
{
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "build:prod": "NODE_ENV=production vite build",
    "build:electron": "ELECTRON=true vite build",
    "build:analyze": "ANALYZE=true vite build",
    "preview": "vite preview",
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch",
    "lint": "eslint . --ext .js,.ts,.svelte",
    "lint:fix": "eslint . --ext .js,.ts,.svelte --fix",
    "format": "prettier --write .",
    "test": "vitest",
    "test:coverage": "vitest --coverage",
    "clean": "rimraf dist .svelte-kit"
  }
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// web/src/lib/__tests__/build-info.test.ts
import { describe, it, expect } from 'vitest';

describe('Build Info', () => {
  it('should have version defined', () => {
    expect(__APP_VERSION__).toBeDefined();
    expect(typeof __APP_VERSION__).toBe('string');
  });

  it('should have git commit defined', () => {
    expect(__GIT_COMMIT__).toBeDefined();
  });

  it('should have build time defined', () => {
    expect(__BUILD_TIME__).toBeDefined();
  });

  it('should have build mode defined', () => {
    expect(__BUILD_MODE__).toBeDefined();
    expect(['development', 'production', 'test']).toContain(__BUILD_MODE__);
  });
});

// Declare build-time constants
declare const __APP_VERSION__: string;
declare const __GIT_COMMIT__: string;
declare const __BUILD_TIME__: string;
declare const __BUILD_MODE__: string;
```

### Build Output Tests

```typescript
// web/__tests__/build-output.test.ts
import { describe, it, expect, beforeAll } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';

describe('Build Output', () => {
  const distDir = path.join(__dirname, '..', 'dist');

  beforeAll(() => {
    // Skip if dist doesn't exist (hasn't been built)
    if (!fs.existsSync(distDir)) {
      console.warn('Skipping build output tests - dist directory not found');
    }
  });

  it('should have index.html', () => {
    if (!fs.existsSync(distDir)) return;
    expect(fs.existsSync(path.join(distDir, 'index.html'))).toBe(true);
  });

  it('should have assets directory', () => {
    if (!fs.existsSync(distDir)) return;
    expect(fs.existsSync(path.join(distDir, 'assets'))).toBe(true);
  });

  it('should have build manifest', () => {
    if (!fs.existsSync(distDir)) return;
    const manifestPath = path.join(distDir, 'build-manifest.json');
    if (fs.existsSync(manifestPath)) {
      const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf-8'));
      expect(manifest.version).toBeDefined();
      expect(manifest.buildTime).toBeDefined();
    }
  });

  it('should have no source maps in production', () => {
    if (!fs.existsSync(distDir)) return;
    const assetsDir = path.join(distDir, 'assets');
    if (!fs.existsSync(assetsDir)) return;

    const files = fs.readdirSync(assetsDir, { recursive: true });
    const mapFiles = files.filter((f) => String(f).endsWith('.map'));

    // In production, there should be no .map files
    if (process.env.NODE_ENV === 'production') {
      expect(mapFiles.length).toBe(0);
    }
  });
});
```

---

## Related Specs

- Spec 491: Build System Orchestration
- Spec 492: Build Configuration
- Spec 496: Asset Handling
- Spec 186: SvelteKit Setup
- Spec 494: Electron Packaging
