# Spec 186: SvelteKit Project Setup

## Phase
Phase 9: UI Foundation

## Spec ID
186

## Status
Planned

## Dependencies
- Phase 1-8 completion (Core infrastructure)
- Tauri integration ready

## Estimated Context
~10%

---

## Objective

Initialize and configure a SvelteKit project optimized for Tauri desktop application integration. This establishes the foundational frontend architecture for Tachikoma's user interface with TypeScript support, proper build configuration, and Tauri-specific optimizations.

---

## Acceptance Criteria

- [x] SvelteKit 2.x project initialized with TypeScript
- [x] Vite configuration optimized for Tauri
- [x] Adapter-static configured for desktop builds
- [x] Path aliases configured (@lib, @components, @stores)
- [x] ESLint and Prettier configured
- [x] Development server integrates with Tauri
- [x] Production builds output to Tauri's expected directory
- [x] Hot Module Replacement (HMR) working in dev mode

---

## Implementation Details

### Project Structure

```
src/
├── lib/
│   ├── components/
│   │   ├── ui/           # Core UI components
│   │   ├── layout/       # Layout components
│   │   └── features/     # Feature-specific components
│   ├── stores/           # Svelte stores
│   ├── utils/            # Utility functions
│   ├── types/            # TypeScript types
│   └── ipc/              # Tauri IPC bindings
├── routes/               # SvelteKit routes
├── app.html              # HTML template
├── app.css               # Global styles
└── app.d.ts              # TypeScript declarations
```

### svelte.config.js

```javascript
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),

  kit: {
    adapter: adapter({
      pages: 'build',
      assets: 'build',
      fallback: 'index.html',
      precompress: false,
      strict: true
    }),

    alias: {
      '@lib': 'src/lib',
      '@components': 'src/lib/components',
      '@stores': 'src/lib/stores',
      '@utils': 'src/lib/utils',
      '@types': 'src/lib/types',
      '@ipc': 'src/lib/ipc'
    },

    // Disable SSR for Tauri
    prerender: {
      entries: []
    }
  }
};

export default config;
```

### vite.config.ts

```typescript
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
```

### src/app.html

```html
<!DOCTYPE html>
<html lang="en" data-theme="dark">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="description" content="Tachikoma - AI-Powered Penetration Testing Assistant" />
    <meta name="color-scheme" content="dark" />

    <!-- Prevent context menu in production -->
    <style>
      html, body {
        margin: 0;
        padding: 0;
        overflow: hidden;
        background-color: #0a0e14;
      }
    </style>

    %sveltekit.head%
  </head>
  <body data-sveltekit-preload-data="hover">
    <div style="display: contents">%sveltekit.body%</div>

    <script>
      // Prevent right-click context menu in production
      if (!import.meta.env.DEV) {
        document.addEventListener('contextmenu', (e) => e.preventDefault());
      }

      // Prevent text selection drag in production
      document.addEventListener('dragstart', (e) => {
        if (e.target instanceof HTMLImageElement || e.target instanceof HTMLAnchorElement) {
          e.preventDefault();
        }
      });
    </script>
  </body>
</html>
```

### src/app.css

```css
/* Tachikoma Global Styles */
@import './lib/styles/tokens.css';
@import './lib/styles/typography.css';
@import './lib/styles/reset.css';

:root {
  /* Base theme - Tachikoma Blue */
  --color-primary: #00d4ff;
  --color-primary-hover: #33ddff;
  --color-primary-active: #00a8cc;

  /* Background layers */
  --color-bg-base: #0a0e14;
  --color-bg-surface: #0d1117;
  --color-bg-elevated: #161b22;
  --color-bg-overlay: #1c2128;

  /* Text colors */
  --color-text-primary: #e6edf3;
  --color-text-secondary: #8b949e;
  --color-text-muted: #6e7681;

  /* Font stack */
  --font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  --font-mono: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

html, body {
  height: 100%;
  width: 100%;
  margin: 0;
  padding: 0;
  font-family: var(--font-sans);
  background-color: var(--color-bg-base);
  color: var(--color-text-primary);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* Scrollbar styling */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: var(--color-bg-surface);
}

::-webkit-scrollbar-thumb {
  background: var(--color-bg-overlay);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-muted);
}

/* Focus styles */
:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

/* Selection styles */
::selection {
  background-color: rgba(0, 212, 255, 0.3);
}
```

### src/app.d.ts

```typescript
/// <reference types="@sveltejs/kit" />
/// <reference types="vite/client" />

declare global {
  namespace App {
    interface Error {
      message: string;
      code?: string;
      details?: Record<string, unknown>;
    }

    interface Locals {
      // Add server-side locals here
    }

    interface PageData {
      // Add shared page data here
    }

    interface Platform {
      // Tauri platform specifics
    }
  }

  // Tauri globals
  interface Window {
    __TAURI__?: {
      invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
      event: {
        listen: <T>(event: string, handler: (event: { payload: T }) => void) => Promise<() => void>;
        emit: (event: string, payload?: unknown) => Promise<void>;
      };
    };
  }
}

export {};
```

### src/lib/utils/environment.ts

```typescript
/**
 * Environment detection utilities for Tachikoma
 */

export const isTauri = (): boolean => {
  return typeof window !== 'undefined' && window.__TAURI__ !== undefined;
};

export const isDev = (): boolean => {
  return import.meta.env.DEV;
};

export const isProd = (): boolean => {
  return import.meta.env.PROD;
};

export const getAppVersion = async (): Promise<string> => {
  if (isTauri()) {
    const { getVersion } = await import('@tauri-apps/api/app');
    return getVersion();
  }
  return import.meta.env.VITE_APP_VERSION || '0.0.0';
};

export const getPlatform = async (): Promise<string> => {
  if (isTauri()) {
    const { platform } = await import('@tauri-apps/api/os');
    return platform();
  }
  return 'web';
};
```

### package.json

```json
{
  "name": "tachikoma-ui",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch",
    "lint": "eslint . --ext .ts,.svelte",
    "format": "prettier --write .",
    "test": "vitest",
    "test:ui": "vitest --ui",
    "tauri": "tauri"
  },
  "devDependencies": {
    "@sveltejs/adapter-static": "^3.0.0",
    "@sveltejs/kit": "^2.0.0",
    "@sveltejs/vite-plugin-svelte": "^3.0.0",
    "@tauri-apps/api": "^1.5.0",
    "@tauri-apps/cli": "^1.5.0",
    "@testing-library/svelte": "^4.0.0",
    "@types/node": "^20.0.0",
    "@typescript-eslint/eslint-plugin": "^6.0.0",
    "@typescript-eslint/parser": "^6.0.0",
    "eslint": "^8.0.0",
    "eslint-plugin-svelte": "^2.0.0",
    "prettier": "^3.0.0",
    "prettier-plugin-svelte": "^3.0.0",
    "svelte": "^4.0.0",
    "svelte-check": "^3.0.0",
    "tslib": "^2.6.0",
    "typescript": "^5.0.0",
    "vite": "^5.0.0",
    "vitest": "^1.0.0"
  }
}
```

### tsconfig.json

```json
{
  "extends": "./.svelte-kit/tsconfig.json",
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "skipLibCheck": true,
    "sourceMap": true,
    "strict": true,
    "moduleResolution": "bundler",
    "target": "ES2021",
    "lib": ["ES2021", "DOM", "DOM.Iterable"],
    "types": ["vite/client"]
  }
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/setup.test.ts
import { describe, it, expect, vi } from 'vitest';
import { isTauri, isDev, isProd } from '@utils/environment';

describe('Environment Detection', () => {
  it('should detect Tauri environment', () => {
    // Mock Tauri presence
    window.__TAURI__ = {
      invoke: vi.fn(),
      event: { listen: vi.fn(), emit: vi.fn() }
    };

    expect(isTauri()).toBe(true);
  });

  it('should detect web environment', () => {
    delete window.__TAURI__;
    expect(isTauri()).toBe(false);
  });

  it('should detect development mode', () => {
    expect(typeof isDev()).toBe('boolean');
    expect(typeof isProd()).toBe('boolean');
  });
});
```

### Integration Tests

```typescript
// tests/integration/build.test.ts
import { describe, it, expect } from 'vitest';
import { existsSync } from 'fs';
import { resolve } from 'path';

describe('Build Configuration', () => {
  it('should have valid svelte.config.js', async () => {
    const config = await import('../../svelte.config.js');
    expect(config.default.kit.adapter).toBeDefined();
  });

  it('should have valid vite.config.ts', async () => {
    const config = await import('../../vite.config.ts');
    expect(config.default).toBeDefined();
  });
});
```

---

## Related Specs

- [187-routing-config.md](./187-routing-config.md) - Routing configuration
- [188-layout-system.md](./188-layout-system.md) - Layout system setup
- [189-store-architecture.md](./189-store-architecture.md) - Svelte stores
- [190-ipc-store-bindings.md](./190-ipc-store-bindings.md) - IPC integration
