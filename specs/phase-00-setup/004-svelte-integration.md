# 004 - Svelte Integration

**Phase:** 0 - Setup
**Spec ID:** 004
**Status:** Planned
**Dependencies:** 001-project-structure, 003-electron-shell
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Set up SvelteKit in the `web/` directory configured for Electron's renderer process with static adapter and proper TypeScript support.

---

## Acceptance Criteria

- [ ] SvelteKit project initialized in `web/`
- [ ] Static adapter configured for Electron
- [ ] TypeScript properly configured
- [ ] Vite configured for Electron renderer
- [ ] Global types for Tachikoma IPC bridge
- [ ] Base layout and routing structure

---

## Implementation Details

### 1. web/package.json

```json
{
  "name": "tachikoma-web",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite dev --port 5173",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch",
    "test": "vitest",
    "lint": "eslint ."
  },
  "devDependencies": {
    "@sveltejs/adapter-static": "^3.0.0",
    "@sveltejs/kit": "^2.0.0",
    "@sveltejs/vite-plugin-svelte": "^3.0.0",
    "@types/node": "^20.10.0",
    "eslint": "^8.55.0",
    "eslint-plugin-svelte": "^2.35.0",
    "svelte": "^4.2.0",
    "svelte-check": "^3.6.0",
    "tslib": "^2.6.0",
    "typescript": "^5.3.0",
    "vite": "^5.0.0",
    "vitest": "^1.0.0"
  }
}
```

### 2. web/svelte.config.js

```javascript
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),

  kit: {
    adapter: adapter({
      pages: 'dist',
      assets: 'dist',
      fallback: 'index.html',
      precompress: false,
      strict: true
    }),
    paths: {
      base: ''
    }
  }
};

export default config;
```

### 3. web/vite.config.ts

```typescript
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
```

### 4. web/src/app.d.ts

```typescript
// Global type definitions

declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    // interface Platform {}
  }

  // Tachikoma IPC bridge exposed by preload
  interface Window {
    tachikoma: {
      platform: NodeJS.Platform;
      invoke: (channel: string, ...args: unknown[]) => Promise<unknown>;
      on: (channel: string, callback: (...args: unknown[]) => void) => void;
      off: (channel: string, callback: (...args: unknown[]) => void) => void;
    };
  }
}

export {};
```

### 5. web/src/app.html

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="color-scheme" content="dark light" />
    <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';" />
    <link rel="icon" href="%sveltekit.assets%/favicon.png" />
    %sveltekit.head%
  </head>
  <body data-sveltekit-preload-data="hover">
    <div style="display: contents">%sveltekit.body%</div>
  </body>
</html>
```

### 6. web/src/routes/+layout.svelte

```svelte
<script lang="ts">
  import '../app.css';
</script>

<div class="app">
  <slot />
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }
</style>
```

### 7. web/src/routes/+page.svelte

```svelte
<script lang="ts">
  let platform = 'unknown';

  if (typeof window !== 'undefined' && window.tachikoma) {
    platform = window.tachikoma.platform;
  }
</script>

<main>
  <h1>Tachikoma</h1>
  <p>Your squad of tireless AI coders</p>
  <p class="platform">Running on: {platform}</p>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    font-family: system-ui, -apple-system, sans-serif;
  }

  h1 {
    font-size: 3rem;
    margin-bottom: 0.5rem;
  }

  .platform {
    color: var(--text-muted, #666);
    font-size: 0.875rem;
  }
</style>
```

### 8. web/src/app.css

```css
:root {
  --bg: #0a0a0a;
  --bg-secondary: #141414;
  --text: #fafafa;
  --text-muted: #666;
  --accent: #00b4d8;
  --border: #333;
}

* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  background: var(--bg);
  color: var(--text);
  font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}
```

---

## Testing Requirements

1. `npm install` succeeds in web/
2. `npm run check` passes
3. `npm run dev` starts dev server on port 5173
4. `npm run build` produces static files in dist/

---

## Related Specs

- Depends on: [001-project-structure.md](001-project-structure.md), [003-electron-shell.md](003-electron-shell.md)
- Next: [005-ipc-bridge.md](005-ipc-bridge.md)
