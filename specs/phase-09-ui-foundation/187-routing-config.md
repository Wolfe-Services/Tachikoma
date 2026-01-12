# Spec 187: Routing Configuration

## Phase
Phase 9: UI Foundation

## Spec ID
187

## Status
Planned

## Dependencies
- Spec 186: SvelteKit Setup

## Estimated Context
~8%

---

## Objective

Configure SvelteKit routing for Tachikoma's desktop application, including file-based routing, navigation guards, route parameters, and deep linking support for Tauri.

---

## Acceptance Criteria

- [x] File-based routing structure established
- [x] Route guards for authentication
- [x] Dynamic route parameters working
- [x] Navigation store for route state
- [x] Deep linking from Tauri supported
- [x] Route transitions configured
- [x] 404 handling implemented
- [x] Route preloading optimized

---

## Implementation Details

### Route Structure

```
src/routes/
├── +layout.svelte           # Root layout
├── +layout.ts               # Layout load function
├── +page.svelte             # Home/Dashboard
├── +error.svelte            # Error page
├── (auth)/                  # Auth group (no layout)
│   ├── login/
│   │   └── +page.svelte
│   └── setup/
│       └── +page.svelte
├── projects/
│   ├── +page.svelte         # Project list
│   ├── +layout.svelte       # Project layout
│   └── [id]/
│       ├── +page.svelte     # Project detail
│       ├── +layout.svelte   # Project detail layout
│       ├── targets/
│       │   └── +page.svelte
│       ├── scans/
│       │   └── +page.svelte
│       └── reports/
│           └── +page.svelte
├── tools/
│   ├── +page.svelte         # Tools overview
│   ├── terminal/
│   │   └── +page.svelte
│   ├── reconnaissance/
│   │   └── +page.svelte
│   └── exploitation/
│       └── +page.svelte
├── ai/
│   ├── +page.svelte         # AI chat interface
│   └── history/
│       └── +page.svelte
└── settings/
    ├── +page.svelte
    ├── +layout.svelte
    ├── general/
    │   └── +page.svelte
    ├── api-keys/
    │   └── +page.svelte
    └── appearance/
        └── +page.svelte
```

### src/lib/stores/navigation.ts

```typescript
import { writable, derived, get } from 'svelte/store';
import { goto, beforeNavigate, afterNavigate } from '$app/navigation';
import { page } from '$app/stores';

export interface NavigationState {
  currentPath: string;
  previousPath: string | null;
  isNavigating: boolean;
  history: string[];
  canGoBack: boolean;
  canGoForward: boolean;
}

function createNavigationStore() {
  const { subscribe, set, update } = writable<NavigationState>({
    currentPath: '/',
    previousPath: null,
    isNavigating: false,
    history: ['/'],
    canGoBack: false,
    canGoForward: false
  });

  let historyIndex = 0;

  return {
    subscribe,

    navigate: async (path: string, options?: { replace?: boolean }) => {
      update(state => ({ ...state, isNavigating: true }));

      try {
        await goto(path, { replaceState: options?.replace });

        update(state => {
          const newHistory = options?.replace
            ? [...state.history.slice(0, historyIndex), path]
            : [...state.history.slice(0, historyIndex + 1), path];

          historyIndex = newHistory.length - 1;

          return {
            ...state,
            previousPath: state.currentPath,
            currentPath: path,
            history: newHistory,
            canGoBack: historyIndex > 0,
            canGoForward: false
          };
        });
      } finally {
        update(state => ({ ...state, isNavigating: false }));
      }
    },

    back: async () => {
      const state = get({ subscribe });
      if (state.canGoBack) {
        historyIndex--;
        const path = state.history[historyIndex];
        await goto(path);
        update(s => ({
          ...s,
          currentPath: path,
          previousPath: s.currentPath,
          canGoBack: historyIndex > 0,
          canGoForward: true
        }));
      }
    },

    forward: async () => {
      const state = get({ subscribe });
      if (state.canGoForward && historyIndex < state.history.length - 1) {
        historyIndex++;
        const path = state.history[historyIndex];
        await goto(path);
        update(s => ({
          ...s,
          currentPath: path,
          previousPath: s.currentPath,
          canGoBack: true,
          canGoForward: historyIndex < s.history.length - 1
        }));
      }
    },

    setPath: (path: string) => {
      update(state => ({
        ...state,
        currentPath: path
      }));
    }
  };
}

export const navigation = createNavigationStore();

// Derived stores
export const currentPath = derived(navigation, $nav => $nav.currentPath);
export const isNavigating = derived(navigation, $nav => $nav.isNavigating);
```

### src/lib/guards/auth.ts

```typescript
import type { LayoutLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import { get } from 'svelte/store';
import { authStore } from '@stores/auth';

export interface AuthGuardOptions {
  requireAuth?: boolean;
  redirectTo?: string;
  roles?: string[];
}

export function createAuthGuard(options: AuthGuardOptions = {}): LayoutLoad {
  const {
    requireAuth = true,
    redirectTo = '/login',
    roles = []
  } = options;

  return async ({ url }) => {
    const auth = get(authStore);

    // Check if authentication is required
    if (requireAuth && !auth.isAuthenticated) {
      const returnUrl = encodeURIComponent(url.pathname + url.search);
      throw redirect(302, `${redirectTo}?returnUrl=${returnUrl}`);
    }

    // Check roles if specified
    if (roles.length > 0 && auth.user) {
      const hasRole = roles.some(role => auth.user?.roles.includes(role));
      if (!hasRole) {
        throw redirect(302, '/unauthorized');
      }
    }

    return {
      user: auth.user
    };
  };
}

// Pre-built guards
export const requireAuth = createAuthGuard({ requireAuth: true });
export const requireAdmin = createAuthGuard({ requireAuth: true, roles: ['admin'] });
export const guestOnly = createAuthGuard({ requireAuth: false, redirectTo: '/' });
```

### src/routes/+layout.ts

```typescript
import type { LayoutLoad } from './$types';
import { isTauri } from '@utils/environment';

export const prerender = true;
export const ssr = false;
export const trailingSlash = 'never';

export const load: LayoutLoad = async ({ url }) => {
  return {
    pathname: url.pathname,
    isTauri: isTauri()
  };
};
```

### src/routes/+layout.svelte

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { beforeNavigate, afterNavigate } from '$app/navigation';
  import { page } from '$app/stores';
  import { navigation } from '@stores/navigation';
  import { setupDeepLinks } from '@lib/deeplinks';
  import '../app.css';

  export let data;

  // Track navigation for transitions
  let navigating = false;

  beforeNavigate(({ from, to }) => {
    navigating = true;
  });

  afterNavigate(({ from, to }) => {
    navigating = false;
    if (to?.url.pathname) {
      navigation.setPath(to.url.pathname);
    }
  });

  onMount(() => {
    if (data.isTauri) {
      setupDeepLinks();
    }
  });
</script>

<div class="app" class:navigating>
  <slot />
</div>

<style>
  .app {
    height: 100vh;
    width: 100vw;
    display: flex;
    flex-direction: column;
    background-color: var(--color-bg-base);
  }

  .app.navigating {
    opacity: 0.98;
  }

  :global(.page-transition) {
    animation: fadeIn 0.15s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
```

### src/lib/deeplinks.ts

```typescript
import { navigation } from '@stores/navigation';
import { isTauri } from '@utils/environment';

interface DeepLinkPayload {
  url: string;
}

export async function setupDeepLinks(): Promise<void> {
  if (!isTauri()) return;

  const { listen } = await import('@tauri-apps/api/event');

  // Listen for deep link events from Tauri
  await listen<DeepLinkPayload>('deep-link', (event) => {
    const url = event.payload.url;
    handleDeepLink(url);
  });
}

export function handleDeepLink(url: string): void {
  try {
    const parsed = new URL(url);

    // Handle tachikoma:// protocol
    if (parsed.protocol === 'tachikoma:') {
      const path = parsed.pathname || '/';
      const params = Object.fromEntries(parsed.searchParams);

      // Navigate to the path
      navigation.navigate(path);

      // Handle specific deep link actions
      handleDeepLinkAction(path, params);
    }
  } catch (error) {
    console.error('Invalid deep link URL:', url, error);
  }
}

function handleDeepLinkAction(path: string, params: Record<string, string>): void {
  // Handle specific deep link patterns
  const patterns: Record<string, (params: Record<string, string>) => void> = {
    '/project/open': (p) => {
      if (p.id) {
        navigation.navigate(`/projects/${p.id}`);
      }
    },
    '/scan/start': (p) => {
      if (p.target) {
        // Trigger scan start action
        window.dispatchEvent(new CustomEvent('deep-link:scan', { detail: p }));
      }
    },
    '/ai/chat': (p) => {
      navigation.navigate('/ai');
      if (p.message) {
        window.dispatchEvent(new CustomEvent('deep-link:ai-message', { detail: p }));
      }
    }
  };

  const handler = patterns[path];
  if (handler) {
    handler(params);
  }
}
```

### src/routes/+error.svelte

```svelte
<script lang="ts">
  import { page } from '$app/stores';
  import { navigation } from '@stores/navigation';
  import Button from '@components/ui/Button.svelte';

  $: status = $page.status;
  $: message = $page.error?.message || 'An error occurred';
</script>

<svelte:head>
  <title>Error {status} | Tachikoma</title>
</svelte:head>

<div class="error-page">
  <div class="error-content">
    <div class="error-code">{status}</div>
    <h1 class="error-title">
      {#if status === 404}
        Page Not Found
      {:else if status === 403}
        Access Denied
      {:else if status === 500}
        Server Error
      {:else}
        Something went wrong
      {/if}
    </h1>
    <p class="error-message">{message}</p>

    <div class="error-actions">
      <Button
        variant="primary"
        on:click={() => navigation.navigate('/')}
      >
        Go Home
      </Button>
      <Button
        variant="ghost"
        on:click={() => navigation.back()}
      >
        Go Back
      </Button>
    </div>
  </div>
</div>

<style>
  .error-page {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-bg-base);
    padding: var(--spacing-8);
  }

  .error-content {
    text-align: center;
    max-width: 480px;
  }

  .error-code {
    font-size: 8rem;
    font-weight: 700;
    color: var(--color-primary);
    line-height: 1;
    opacity: 0.3;
    margin-bottom: var(--spacing-4);
  }

  .error-title {
    font-size: var(--font-size-2xl);
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 var(--spacing-4);
  }

  .error-message {
    color: var(--color-text-secondary);
    margin: 0 0 var(--spacing-8);
    line-height: 1.6;
  }

  .error-actions {
    display: flex;
    gap: var(--spacing-4);
    justify-content: center;
  }
</style>
```

### src/routes/projects/[id]/+layout.ts

```typescript
import type { LayoutLoad } from './$types';
import { error } from '@sveltejs/kit';
import { getProject } from '@ipc/projects';

export const load: LayoutLoad = async ({ params }) => {
  const { id } = params;

  try {
    const project = await getProject(id);

    if (!project) {
      throw error(404, {
        message: 'Project not found'
      });
    }

    return {
      project
    };
  } catch (e) {
    if (e instanceof Error && 'status' in e) {
      throw e;
    }
    throw error(500, {
      message: 'Failed to load project'
    });
  }
};
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/routing/navigation.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { navigation, currentPath } from '@stores/navigation';

// Mock SvelteKit navigation
vi.mock('$app/navigation', () => ({
  goto: vi.fn().mockResolvedValue(undefined),
  beforeNavigate: vi.fn(),
  afterNavigate: vi.fn()
}));

describe('Navigation Store', () => {
  beforeEach(() => {
    // Reset store state
  });

  it('should track current path', () => {
    expect(get(currentPath)).toBeDefined();
  });

  it('should navigate to new path', async () => {
    await navigation.navigate('/projects');
    expect(get(navigation).currentPath).toBe('/projects');
  });

  it('should track navigation history', async () => {
    await navigation.navigate('/');
    await navigation.navigate('/projects');
    await navigation.navigate('/ai');

    const state = get(navigation);
    expect(state.history).toContain('/projects');
    expect(state.canGoBack).toBe(true);
  });
});
```

### Integration Tests

```typescript
// tests/routing/deeplinks.test.ts
import { describe, it, expect, vi } from 'vitest';
import { handleDeepLink } from '@lib/deeplinks';
import { navigation } from '@stores/navigation';

vi.mock('@stores/navigation', () => ({
  navigation: {
    navigate: vi.fn()
  }
}));

describe('Deep Links', () => {
  it('should parse and navigate to valid deep link', () => {
    handleDeepLink('tachikoma://projects/123');
    expect(navigation.navigate).toHaveBeenCalledWith('/projects/123');
  });

  it('should handle deep link with query params', () => {
    handleDeepLink('tachikoma://project/open?id=456');
    expect(navigation.navigate).toHaveBeenCalledWith('/projects/456');
  });

  it('should ignore invalid deep links', () => {
    handleDeepLink('invalid-url');
    expect(navigation.navigate).not.toHaveBeenCalled();
  });
});
```

---

## Related Specs

- [186-sveltekit-setup.md](./186-sveltekit-setup.md) - SvelteKit setup
- [188-layout-system.md](./188-layout-system.md) - Layout system
- [189-store-architecture.md](./189-store-architecture.md) - Store architecture
