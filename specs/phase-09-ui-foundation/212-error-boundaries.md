# Spec 212: Error Boundaries and Error Handling

## Phase
Phase 9: UI Foundation

## Spec ID
212

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 191-195: Design System
- Spec 203: Toast Component

## Estimated Context
~10%

---

## Objective

Implement comprehensive error handling components for Tachikoma including error boundaries, fallback UI components, retry mechanisms, and global error tracking to gracefully handle runtime errors and provide meaningful feedback to users.

---

## Acceptance Criteria

- [x] Error boundary component wrapping
- [x] Customizable fallback UI
- [x] Error recovery/retry mechanism
- [x] Global error handler
- [x] Error logging integration
- [x] Network error handling
- [x] 404/Not Found page
- [x] Generic error page
- [x] Error store for tracking
- [x] Development vs production error display

---

## Implementation Details

### src/lib/components/ui/Error/ErrorBoundary.svelte

```svelte
<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { cn } from '@utils/component';
  import ErrorFallback from './ErrorFallback.svelte';
  import { errorStore } from '@stores/error';

  export let fallback: typeof ErrorFallback = ErrorFallback;
  export let onError: ((error: Error, errorInfo: string) => void) | undefined = undefined;
  export let resetKeys: any[] = [];
  export let isolate: boolean = true;
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    error: { error: Error; errorInfo: string };
    reset: void;
  }>();

  let hasError = false;
  let error: Error | null = null;
  let errorInfo: string = '';
  let key = 0;

  // Reset when resetKeys change
  $: if (resetKeys.length > 0) {
    resetError();
  }

  function handleError(event: ErrorEvent) {
    if (!isolate) return;

    event.preventDefault();
    captureError(event.error || new Error(event.message), event.filename || 'unknown');
  }

  function handleUnhandledRejection(event: PromiseRejectionEvent) {
    if (!isolate) return;

    event.preventDefault();
    const err = event.reason instanceof Error
      ? event.reason
      : new Error(String(event.reason));
    captureError(err, 'Promise rejection');
  }

  function captureError(err: Error, info: string) {
    hasError = true;
    error = err;
    errorInfo = info;

    // Log to error store
    errorStore.capture({
      error: err,
      componentStack: info,
      timestamp: new Date()
    });

    // Call custom error handler
    onError?.(err, info);

    // Dispatch event
    dispatch('error', { error: err, errorInfo: info });

    // Log in development
    if (import.meta.env.DEV) {
      console.error('Error caught by boundary:', err);
      console.error('Component stack:', info);
    }
  }

  export function resetError() {
    hasError = false;
    error = null;
    errorInfo = '';
    key += 1;
    dispatch('reset');
  }

  onMount(() => {
    if (isolate) {
      window.addEventListener('error', handleError);
      window.addEventListener('unhandledrejection', handleUnhandledRejection);
    }
  });

  onDestroy(() => {
    if (isolate) {
      window.removeEventListener('error', handleError);
      window.removeEventListener('unhandledrejection', handleUnhandledRejection);
    }
  });

  $: classes = cn('error-boundary', className);
</script>

<div class={classes}>
  {#if hasError && error}
    <svelte:component
      this={fallback}
      {error}
      {errorInfo}
      onReset={resetError}
    />
  {:else}
    {#key key}
      <slot />
    {/key}
  {/if}
</div>

<style>
  .error-boundary {
    display: contents;
  }
</style>
```

### src/lib/components/ui/Error/ErrorFallback.svelte

```svelte
<script lang="ts">
  import { cn } from '@utils/component';
  import Button from '../Button/Button.svelte';
  import Icon from '../Icon/Icon.svelte';

  export let error: Error;
  export let errorInfo: string = '';
  export let onReset: (() => void) | undefined = undefined;
  export let showDetails: boolean = import.meta.env.DEV;
  export let title: string = 'Something went wrong';
  export let message: string = 'An unexpected error occurred. Please try again.';
  let className: string = '';
  export { className as class };

  let showStack = false;

  $: classes = cn('error-fallback', className);
</script>

<div class={classes} role="alert">
  <div class="error-fallback-icon">
    <Icon name="alert-triangle" size={48} />
  </div>

  <h2 class="error-fallback-title">{title}</h2>
  <p class="error-fallback-message">{message}</p>

  {#if showDetails}
    <div class="error-fallback-details">
      <button
        class="error-fallback-toggle"
        on:click={() => showStack = !showStack}
        type="button"
      >
        <Icon name={showStack ? 'chevron-down' : 'chevron-right'} size={16} />
        Error Details
      </button>

      {#if showStack}
        <div class="error-fallback-stack">
          <div class="error-fallback-error-name">
            {error.name}: {error.message}
          </div>
          {#if error.stack}
            <pre class="error-fallback-stacktrace">{error.stack}</pre>
          {/if}
          {#if errorInfo}
            <div class="error-fallback-component-stack">
              <strong>Component Stack:</strong>
              <pre>{errorInfo}</pre>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}

  <div class="error-fallback-actions">
    {#if onReset}
      <Button variant="primary" on:click={onReset}>
        <Icon name="refresh-cw" slot="icon-left" size={16} />
        Try Again
      </Button>
    {/if}
    <Button variant="ghost" on:click={() => window.location.reload()}>
      Reload Page
    </Button>
  </div>
</div>

<style>
  .error-fallback {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-8);
    text-align: center;
    min-height: 300px;
  }

  .error-fallback-icon {
    color: var(--color-danger-500);
    margin-bottom: var(--spacing-4);
  }

  .error-fallback-title {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
    color: var(--color-fg-default);
    margin: 0 0 var(--spacing-2);
  }

  .error-fallback-message {
    font-size: var(--text-sm);
    color: var(--color-fg-muted);
    margin: 0 0 var(--spacing-6);
    max-width: 400px;
  }

  .error-fallback-details {
    width: 100%;
    max-width: 600px;
    margin-bottom: var(--spacing-6);
    text-align: left;
  }

  .error-fallback-toggle {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    background: none;
    border: none;
    color: var(--color-fg-muted);
    font-size: var(--text-sm);
    cursor: pointer;
    padding: var(--spacing-2);
    border-radius: var(--radius-sm);
  }

  .error-fallback-toggle:hover {
    background-color: var(--color-bg-hover);
    color: var(--color-fg-default);
  }

  .error-fallback-stack {
    margin-top: var(--spacing-2);
    padding: var(--spacing-4);
    background-color: var(--color-bg-subtle);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border-subtle);
  }

  .error-fallback-error-name {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    color: var(--color-danger-500);
    margin-bottom: var(--spacing-2);
  }

  .error-fallback-stacktrace {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--color-fg-muted);
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-all;
    margin: 0;
    max-height: 200px;
    overflow-y: auto;
  }

  .error-fallback-component-stack {
    margin-top: var(--spacing-3);
    padding-top: var(--spacing-3);
    border-top: 1px solid var(--color-border-subtle);
  }

  .error-fallback-component-stack pre {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--color-fg-muted);
    margin: var(--spacing-1) 0 0;
  }

  .error-fallback-actions {
    display: flex;
    gap: var(--spacing-3);
  }
</style>
```

### src/lib/components/ui/Error/NetworkError.svelte

```svelte
<script lang="ts">
  import { cn } from '@utils/component';
  import Button from '../Button/Button.svelte';
  import Icon from '../Icon/Icon.svelte';

  export let onRetry: (() => void) | undefined = undefined;
  export let retrying: boolean = false;
  export let statusCode: number | undefined = undefined;
  export let message: string = 'Unable to connect to the server. Please check your connection.';
  let className: string = '';
  export { className as class };

  $: title = statusCode
    ? `Error ${statusCode}`
    : 'Connection Error';

  $: icon = statusCode === 404
    ? 'file-question'
    : statusCode && statusCode >= 500
      ? 'server-off'
      : 'wifi-off';

  $: classes = cn('network-error', className);
</script>

<div class={classes} role="alert">
  <div class="network-error-icon">
    <Icon name={icon} size={64} />
  </div>

  <h2 class="network-error-title">{title}</h2>
  <p class="network-error-message">{message}</p>

  {#if onRetry}
    <Button
      variant="primary"
      on:click={onRetry}
      loading={retrying}
      disabled={retrying}
    >
      <Icon name="refresh-cw" slot="icon-left" size={16} />
      {retrying ? 'Retrying...' : 'Retry'}
    </Button>
  {/if}
</div>

<style>
  .network-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-8);
    text-align: center;
    min-height: 300px;
  }

  .network-error-icon {
    color: var(--color-fg-muted);
    margin-bottom: var(--spacing-4);
    opacity: 0.5;
  }

  .network-error-title {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
    color: var(--color-fg-default);
    margin: 0 0 var(--spacing-2);
  }

  .network-error-message {
    font-size: var(--text-sm);
    color: var(--color-fg-muted);
    margin: 0 0 var(--spacing-6);
    max-width: 400px;
  }
</style>
```

### src/lib/components/ui/Error/NotFound.svelte

```svelte
<script lang="ts">
  import { cn } from '@utils/component';
  import Button from '../Button/Button.svelte';
  import Icon from '../Icon/Icon.svelte';

  export let title: string = 'Page Not Found';
  export let message: string = "The page you're looking for doesn't exist or has been moved.";
  export let showHomeButton: boolean = true;
  export let homeHref: string = '/';
  let className: string = '';
  export { className as class };

  $: classes = cn('not-found', className);
</script>

<div class={classes}>
  <div class="not-found-content">
    <div class="not-found-code">404</div>
    <h1 class="not-found-title">{title}</h1>
    <p class="not-found-message">{message}</p>

    <div class="not-found-actions">
      {#if showHomeButton}
        <Button variant="primary" href={homeHref}>
          <Icon name="home" slot="icon-left" size={16} />
          Go Home
        </Button>
      {/if}
      <Button variant="ghost" on:click={() => history.back()}>
        <Icon name="arrow-left" slot="icon-left" size={16} />
        Go Back
      </Button>
    </div>
  </div>
</div>

<style>
  .not-found {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: var(--spacing-8);
    text-align: center;
    background: var(--color-bg-canvas);
  }

  .not-found-content {
    max-width: 500px;
  }

  .not-found-code {
    font-size: 8rem;
    font-weight: var(--font-bold);
    line-height: 1;
    background: linear-gradient(
      135deg,
      var(--color-primary-500),
      var(--color-primary-300)
    );
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    margin-bottom: var(--spacing-4);
  }

  .not-found-title {
    font-size: var(--text-2xl);
    font-weight: var(--font-semibold);
    color: var(--color-fg-default);
    margin: 0 0 var(--spacing-2);
  }

  .not-found-message {
    font-size: var(--text-base);
    color: var(--color-fg-muted);
    margin: 0 0 var(--spacing-8);
  }

  .not-found-actions {
    display: flex;
    justify-content: center;
    gap: var(--spacing-3);
  }
</style>
```

### src/lib/components/ui/Error/GenericError.svelte

```svelte
<script lang="ts">
  import { cn } from '@utils/component';
  import Button from '../Button/Button.svelte';
  import Icon from '../Icon/Icon.svelte';

  export let statusCode: number = 500;
  export let title: string = 'Server Error';
  export let message: string = 'An unexpected error occurred on our servers. Please try again later.';
  export let showSupport: boolean = false;
  export let supportEmail: string = 'support@tachikoma.io';
  let className: string = '';
  export { className as class };

  const statusMessages: Record<number, { title: string; message: string }> = {
    400: { title: 'Bad Request', message: 'The request could not be understood by the server.' },
    401: { title: 'Unauthorized', message: 'You need to be logged in to access this resource.' },
    403: { title: 'Forbidden', message: "You don't have permission to access this resource." },
    404: { title: 'Not Found', message: 'The requested resource could not be found.' },
    408: { title: 'Request Timeout', message: 'The request took too long to complete.' },
    429: { title: 'Too Many Requests', message: 'You have made too many requests. Please try again later.' },
    500: { title: 'Internal Server Error', message: 'An unexpected error occurred on our servers.' },
    502: { title: 'Bad Gateway', message: 'The server received an invalid response.' },
    503: { title: 'Service Unavailable', message: 'The service is temporarily unavailable. Please try again later.' },
    504: { title: 'Gateway Timeout', message: 'The server did not respond in time.' }
  };

  $: displayTitle = title || statusMessages[statusCode]?.title || 'Error';
  $: displayMessage = message || statusMessages[statusCode]?.message || 'An unexpected error occurred.';

  $: classes = cn('generic-error', className);
</script>

<div class={classes}>
  <div class="generic-error-content">
    <div class="generic-error-code">{statusCode}</div>
    <h1 class="generic-error-title">{displayTitle}</h1>
    <p class="generic-error-message">{displayMessage}</p>

    <div class="generic-error-actions">
      <Button variant="primary" on:click={() => window.location.reload()}>
        <Icon name="refresh-cw" slot="icon-left" size={16} />
        Refresh Page
      </Button>
      <Button variant="ghost" href="/">
        <Icon name="home" slot="icon-left" size={16} />
        Go Home
      </Button>
    </div>

    {#if showSupport}
      <p class="generic-error-support">
        If the problem persists, please contact
        <a href="mailto:{supportEmail}">{supportEmail}</a>
      </p>
    {/if}
  </div>
</div>

<style>
  .generic-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: var(--spacing-8);
    text-align: center;
    background: var(--color-bg-canvas);
  }

  .generic-error-content {
    max-width: 500px;
  }

  .generic-error-code {
    font-size: 6rem;
    font-weight: var(--font-bold);
    line-height: 1;
    color: var(--color-danger-500);
    opacity: 0.5;
    margin-bottom: var(--spacing-4);
  }

  .generic-error-title {
    font-size: var(--text-2xl);
    font-weight: var(--font-semibold);
    color: var(--color-fg-default);
    margin: 0 0 var(--spacing-2);
  }

  .generic-error-message {
    font-size: var(--text-base);
    color: var(--color-fg-muted);
    margin: 0 0 var(--spacing-8);
  }

  .generic-error-actions {
    display: flex;
    justify-content: center;
    gap: var(--spacing-3);
    margin-bottom: var(--spacing-6);
  }

  .generic-error-support {
    font-size: var(--text-sm);
    color: var(--color-fg-muted);
  }

  .generic-error-support a {
    color: var(--color-primary-500);
    text-decoration: none;
  }

  .generic-error-support a:hover {
    text-decoration: underline;
  }
</style>
```

### src/lib/stores/error.ts

```typescript
import { writable, derived, get } from 'svelte/store';

export interface ErrorEntry {
  id: string;
  error: Error;
  componentStack?: string;
  timestamp: Date;
  handled: boolean;
  context?: Record<string, any>;
}

interface ErrorState {
  errors: ErrorEntry[];
  lastError: ErrorEntry | null;
}

function generateId(): string {
  return `err_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

function createErrorStore() {
  const { subscribe, update, set } = writable<ErrorState>({
    errors: [],
    lastError: null
  });

  return {
    subscribe,

    capture(params: {
      error: Error;
      componentStack?: string;
      timestamp?: Date;
      context?: Record<string, any>;
    }) {
      const entry: ErrorEntry = {
        id: generateId(),
        error: params.error,
        componentStack: params.componentStack,
        timestamp: params.timestamp || new Date(),
        handled: false,
        context: params.context
      };

      update(state => ({
        errors: [...state.errors, entry],
        lastError: entry
      }));

      // Log to console in development
      if (import.meta.env.DEV) {
        console.group(`[Error Store] ${params.error.name}`);
        console.error(params.error);
        if (params.componentStack) {
          console.log('Component Stack:', params.componentStack);
        }
        if (params.context) {
          console.log('Context:', params.context);
        }
        console.groupEnd();
      }

      return entry.id;
    },

    markHandled(id: string) {
      update(state => ({
        ...state,
        errors: state.errors.map(e =>
          e.id === id ? { ...e, handled: true } : e
        )
      }));
    },

    dismiss(id: string) {
      update(state => ({
        ...state,
        errors: state.errors.filter(e => e.id !== id),
        lastError: state.lastError?.id === id ? null : state.lastError
      }));
    },

    clear() {
      set({ errors: [], lastError: null });
    },

    getUnhandled() {
      return derived({ subscribe }, $state =>
        $state.errors.filter(e => !e.handled)
      );
    }
  };
}

export const errorStore = createErrorStore();

// Derived stores
export const hasErrors = derived(errorStore, $state => $state.errors.length > 0);
export const errorCount = derived(errorStore, $state => $state.errors.length);
export const lastError = derived(errorStore, $state => $state.lastError);

// Global error handler setup
export function setupGlobalErrorHandler() {
  if (typeof window === 'undefined') return;

  window.addEventListener('error', (event) => {
    errorStore.capture({
      error: event.error || new Error(event.message),
      context: {
        filename: event.filename,
        lineno: event.lineno,
        colno: event.colno
      }
    });
  });

  window.addEventListener('unhandledrejection', (event) => {
    const error = event.reason instanceof Error
      ? event.reason
      : new Error(String(event.reason));

    errorStore.capture({
      error,
      context: { type: 'unhandledrejection' }
    });
  });
}
```

### src/lib/utils/errorHandling.ts

```typescript
import { errorStore } from '@stores/error';

/**
 * Wrap an async function with error handling
 */
export function withErrorHandling<T extends (...args: any[]) => Promise<any>>(
  fn: T,
  options: {
    context?: Record<string, any>;
    onError?: (error: Error) => void;
    rethrow?: boolean;
  } = {}
): T {
  return (async (...args: Parameters<T>) => {
    try {
      return await fn(...args);
    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));

      errorStore.capture({
        error: err,
        context: options.context
      });

      options.onError?.(err);

      if (options.rethrow !== false) {
        throw err;
      }
    }
  }) as T;
}

/**
 * Retry an async operation with exponential backoff
 */
export async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  options: {
    maxRetries?: number;
    baseDelay?: number;
    maxDelay?: number;
    onRetry?: (attempt: number, error: Error) => void;
  } = {}
): Promise<T> {
  const {
    maxRetries = 3,
    baseDelay = 1000,
    maxDelay = 10000,
    onRetry
  } = options;

  let lastError: Error;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));

      if (attempt < maxRetries) {
        const delay = Math.min(baseDelay * Math.pow(2, attempt), maxDelay);
        onRetry?.(attempt + 1, lastError);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
  }

  throw lastError!;
}

/**
 * Create a safe version of a function that catches errors
 */
export function safe<T extends (...args: any[]) => any>(
  fn: T,
  fallback?: ReturnType<T>
): (...args: Parameters<T>) => ReturnType<T> | undefined {
  return (...args: Parameters<T>) => {
    try {
      return fn(...args);
    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));
      errorStore.capture({ error: err });
      return fallback;
    }
  };
}

/**
 * Parse API error response
 */
export function parseApiError(response: Response, body?: any): Error {
  const message = body?.message || body?.error || response.statusText;
  const error = new Error(message);
  (error as any).status = response.status;
  (error as any).body = body;
  return error;
}
```

### Usage Examples

```svelte
<script>
  import {
    ErrorBoundary,
    ErrorFallback,
    NetworkError,
    NotFound,
    GenericError
  } from '@components/ui';
  import { errorStore, setupGlobalErrorHandler } from '@stores/error';
  import { retryWithBackoff, withErrorHandling } from '@utils/errorHandling';
  import { onMount } from 'svelte';

  onMount(() => {
    setupGlobalErrorHandler();
  });

  // Component that might throw
  function BuggyComponent() {
    throw new Error('Intentional error for testing');
  }

  // With retry logic
  let data = null;
  let loading = false;
  let networkError = null;

  async function fetchData() {
    loading = true;
    networkError = null;

    try {
      data = await retryWithBackoff(
        () => fetch('/api/data').then(r => r.json()),
        {
          maxRetries: 3,
          onRetry: (attempt, error) => {
            console.log(`Retry attempt ${attempt}:`, error.message);
          }
        }
      );
    } catch (error) {
      networkError = error;
    } finally {
      loading = false;
    }
  }

  // Wrapped function with automatic error tracking
  const safeOperation = withErrorHandling(
    async () => {
      // Some risky operation
    },
    { context: { component: 'MyComponent' } }
  );
</script>

<!-- Error boundary wrapping risky components -->
<ErrorBoundary on:error={(e) => console.log('Caught:', e.detail.error)}>
  <RiskyComponent />
</ErrorBoundary>

<!-- Custom fallback -->
<ErrorBoundary>
  <SomeComponent />

  <svelte:fragment slot="fallback" let:error let:reset>
    <div class="custom-error">
      <p>Oops! {error.message}</p>
      <button on:click={reset}>Retry</button>
    </div>
  </svelte:fragment>
</ErrorBoundary>

<!-- Network error with retry -->
{#if networkError}
  <NetworkError
    statusCode={networkError.status}
    message={networkError.message}
    onRetry={fetchData}
    retrying={loading}
  />
{:else if data}
  <DataDisplay {data} />
{/if}

<!-- Error pages -->
<NotFound />
<GenericError statusCode={500} showSupport />
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/Error.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import { get } from 'svelte/store';
import ErrorBoundary from '@components/ui/Error/ErrorBoundary.svelte';
import ErrorFallback from '@components/ui/Error/ErrorFallback.svelte';
import { errorStore } from '@stores/error';
import { retryWithBackoff } from '@utils/errorHandling';

describe('ErrorFallback', () => {
  it('should display error message', () => {
    const error = new Error('Test error');
    const { getByText } = render(ErrorFallback, { props: { error } });

    expect(getByText('Something went wrong')).toBeInTheDocument();
  });

  it('should call onReset when retry clicked', async () => {
    const onReset = vi.fn();
    const { getByText } = render(ErrorFallback, {
      props: { error: new Error('Test'), onReset }
    });

    await fireEvent.click(getByText('Try Again'));
    expect(onReset).toHaveBeenCalled();
  });

  it('should show error details in dev mode', () => {
    const error = new Error('Test error');
    error.stack = 'Error: Test error\n  at test.js:1:1';

    const { getByText, container } = render(ErrorFallback, {
      props: { error, showDetails: true }
    });

    fireEvent.click(getByText('Error Details'));
    expect(container.querySelector('.error-fallback-stacktrace')).toBeInTheDocument();
  });
});

describe('errorStore', () => {
  beforeEach(() => {
    errorStore.clear();
  });

  it('should capture errors', () => {
    const error = new Error('Test error');
    errorStore.capture({ error });

    const state = get(errorStore);
    expect(state.errors).toHaveLength(1);
    expect(state.lastError?.error.message).toBe('Test error');
  });

  it('should mark errors as handled', () => {
    const error = new Error('Test error');
    const id = errorStore.capture({ error });

    errorStore.markHandled(id);

    const state = get(errorStore);
    expect(state.errors[0].handled).toBe(true);
  });

  it('should dismiss errors', () => {
    const error = new Error('Test error');
    const id = errorStore.capture({ error });

    errorStore.dismiss(id);

    const state = get(errorStore);
    expect(state.errors).toHaveLength(0);
  });
});

describe('retryWithBackoff', () => {
  it('should retry on failure', async () => {
    let attempts = 0;
    const fn = vi.fn().mockImplementation(() => {
      attempts++;
      if (attempts < 3) throw new Error('Fail');
      return 'success';
    });

    const result = await retryWithBackoff(fn, {
      maxRetries: 3,
      baseDelay: 10
    });

    expect(result).toBe('success');
    expect(fn).toHaveBeenCalledTimes(3);
  });

  it('should throw after max retries', async () => {
    const fn = vi.fn().mockRejectedValue(new Error('Always fails'));

    await expect(
      retryWithBackoff(fn, { maxRetries: 2, baseDelay: 10 })
    ).rejects.toThrow('Always fails');

    expect(fn).toHaveBeenCalledTimes(3); // Initial + 2 retries
  });

  it('should call onRetry callback', async () => {
    const onRetry = vi.fn();
    const fn = vi.fn()
      .mockRejectedValueOnce(new Error('Fail 1'))
      .mockResolvedValue('success');

    await retryWithBackoff(fn, { maxRetries: 2, baseDelay: 10, onRetry });

    expect(onRetry).toHaveBeenCalledWith(1, expect.any(Error));
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [203-toast-component.md](./203-toast-component.md) - Toast notifications
- [211-loading-states.md](./211-loading-states.md) - Loading states
