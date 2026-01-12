# Spec 211: Loading States and Skeleton Components

## Phase
Phase 9: UI Foundation

## Spec ID
211

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 191-195: Design System

## Estimated Context
~10%

---

## Objective

Implement comprehensive loading state components for Tachikoma including skeleton loaders, spinners, progress indicators, and shimmer effects to provide visual feedback during async operations.

---

## Acceptance Criteria

- [ ] Skeleton component with customizable shapes
- [ ] Shimmer animation effect
- [ ] Spinner component with sizes
- [ ] Progress bar (determinate/indeterminate)
- [ ] Circular progress indicator
- [ ] Loading overlay for containers
- [ ] Skeleton presets for common layouts
- [ ] Reduced motion support
- [ ] Loading state store for global tracking

---

## Implementation Details

### src/lib/components/ui/Loading/Skeleton.svelte

```svelte
<script lang="ts">
  import { cn } from '@utils/component';

  export let variant: 'text' | 'circular' | 'rectangular' | 'rounded' = 'text';
  export let width: string | number = '100%';
  export let height: string | number | undefined = undefined;
  export let lines: number = 1;
  export let animate: boolean = true;
  let className: string = '';
  export { className as class };

  $: computedWidth = typeof width === 'number' ? `${width}px` : width;
  $: computedHeight = typeof height === 'number' ? `${height}px` : height;

  $: defaultHeight = variant === 'text' ? '1em' : variant === 'circular' ? computedWidth : '100px';

  $: classes = cn(
    'skeleton',
    `skeleton-${variant}`,
    animate && 'skeleton-animate',
    className
  );
</script>

{#if lines > 1 && variant === 'text'}
  <div class="skeleton-lines">
    {#each Array(lines) as _, i}
      <div
        class={classes}
        style="
          width: {i === lines - 1 ? '80%' : computedWidth};
          height: {computedHeight || defaultHeight};
        "
        role="presentation"
        aria-hidden="true"
      />
    {/each}
  </div>
{:else}
  <div
    class={classes}
    style="
      width: {computedWidth};
      height: {computedHeight || defaultHeight};
    "
    role="presentation"
    aria-hidden="true"
  />
{/if}

<style>
  .skeleton {
    background-color: var(--color-bg-muted);
    position: relative;
    overflow: hidden;
  }

  .skeleton-text {
    border-radius: var(--radius-sm);
  }

  .skeleton-circular {
    border-radius: 50%;
  }

  .skeleton-rectangular {
    border-radius: 0;
  }

  .skeleton-rounded {
    border-radius: var(--radius-lg);
  }

  .skeleton-animate {
    background: linear-gradient(
      90deg,
      var(--color-bg-muted) 25%,
      var(--color-bg-subtle) 50%,
      var(--color-bg-muted) 75%
    );
    background-size: 200% 100%;
    animation: skeleton-shimmer 1.5s ease-in-out infinite;
  }

  @keyframes skeleton-shimmer {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }

  .skeleton-lines {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
  }

  @media (prefers-reduced-motion: reduce) {
    .skeleton-animate {
      animation: none;
      background: var(--color-bg-muted);
    }
  }
</style>
```

### src/lib/components/ui/Loading/Spinner.svelte

```svelte
<script lang="ts">
  import { cn } from '@utils/component';

  export let size: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | number = 'md';
  export let color: string = 'currentColor';
  export let thickness: number = 2;
  export let speed: 'slow' | 'normal' | 'fast' = 'normal';
  export let label: string = 'Loading';
  let className: string = '';
  export { className as class };

  const sizeMap = {
    xs: 12,
    sm: 16,
    md: 24,
    lg: 32,
    xl: 48
  };

  const speedMap = {
    slow: '1.2s',
    normal: '0.75s',
    fast: '0.5s'
  };

  $: computedSize = typeof size === 'number' ? size : sizeMap[size];
  $: computedSpeed = speedMap[speed];

  $: classes = cn('spinner', className);
</script>

<svg
  class={classes}
  width={computedSize}
  height={computedSize}
  viewBox="0 0 24 24"
  fill="none"
  style="--spinner-speed: {computedSpeed}; --spinner-color: {color}"
  role="status"
  aria-label={label}
>
  <circle
    class="spinner-track"
    cx="12"
    cy="12"
    r={10 - thickness / 2}
    stroke-width={thickness}
  />
  <circle
    class="spinner-arc"
    cx="12"
    cy="12"
    r={10 - thickness / 2}
    stroke-width={thickness}
    stroke-linecap="round"
  />
  <title>{label}</title>
</svg>

<style>
  .spinner {
    animation: spinner-rotate var(--spinner-speed) linear infinite;
  }

  .spinner-track {
    stroke: var(--color-border-subtle);
  }

  .spinner-arc {
    stroke: var(--spinner-color);
    stroke-dasharray: 60 200;
    stroke-dashoffset: 0;
    animation: spinner-dash calc(var(--spinner-speed) * 2) ease-in-out infinite;
  }

  @keyframes spinner-rotate {
    100% {
      transform: rotate(360deg);
    }
  }

  @keyframes spinner-dash {
    0% {
      stroke-dasharray: 1 200;
      stroke-dashoffset: 0;
    }
    50% {
      stroke-dasharray: 60 200;
      stroke-dashoffset: -25;
    }
    100% {
      stroke-dasharray: 60 200;
      stroke-dashoffset: -125;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .spinner {
      animation-duration: 1.5s;
    }
    .spinner-arc {
      animation: none;
      stroke-dasharray: 60 200;
    }
  }
</style>
```

### src/lib/components/ui/Loading/Progress.svelte

```svelte
<script lang="ts">
  import { cn } from '@utils/component';
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';

  export let value: number | undefined = undefined;
  export let max: number = 100;
  export let size: 'sm' | 'md' | 'lg' = 'md';
  export let color: 'primary' | 'success' | 'warning' | 'danger' = 'primary';
  export let showValue: boolean = false;
  export let label: string = 'Progress';
  export let animated: boolean = true;
  let className: string = '';
  export { className as class };

  const progress = tweened(0, {
    duration: animated ? 300 : 0,
    easing: cubicOut
  });

  $: isIndeterminate = value === undefined;
  $: percentage = isIndeterminate ? 0 : Math.min(Math.max((value! / max) * 100, 0), 100);
  $: progress.set(percentage);

  const sizeClasses = {
    sm: 'progress-sm',
    md: 'progress-md',
    lg: 'progress-lg'
  };

  const colorClasses = {
    primary: 'progress-primary',
    success: 'progress-success',
    warning: 'progress-warning',
    danger: 'progress-danger'
  };

  $: classes = cn(
    'progress',
    sizeClasses[size],
    colorClasses[color],
    isIndeterminate && 'progress-indeterminate',
    className
  );
</script>

<div
  class={classes}
  role="progressbar"
  aria-valuenow={isIndeterminate ? undefined : value}
  aria-valuemin={0}
  aria-valuemax={max}
  aria-label={label}
>
  <div
    class="progress-bar"
    style={isIndeterminate ? '' : `width: ${$progress}%`}
  />

  {#if showValue && !isIndeterminate}
    <span class="progress-value">{Math.round(percentage)}%</span>
  {/if}
</div>

<style>
  .progress {
    position: relative;
    width: 100%;
    background-color: var(--color-bg-muted);
    border-radius: var(--radius-full);
    overflow: hidden;
  }

  .progress-sm {
    height: 4px;
  }

  .progress-md {
    height: 8px;
  }

  .progress-lg {
    height: 12px;
  }

  .progress-bar {
    height: 100%;
    border-radius: var(--radius-full);
    transition: width 0.3s ease;
  }

  .progress-primary .progress-bar {
    background: linear-gradient(90deg, var(--color-primary-500), var(--color-primary-400));
  }

  .progress-success .progress-bar {
    background: linear-gradient(90deg, var(--color-success-500), var(--color-success-400));
  }

  .progress-warning .progress-bar {
    background: linear-gradient(90deg, var(--color-warning-500), var(--color-warning-400));
  }

  .progress-danger .progress-bar {
    background: linear-gradient(90deg, var(--color-danger-500), var(--color-danger-400));
  }

  .progress-indeterminate .progress-bar {
    width: 30%;
    animation: progress-indeterminate 1.5s ease-in-out infinite;
  }

  @keyframes progress-indeterminate {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(400%);
    }
  }

  .progress-value {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    color: var(--color-fg-default);
  }

  @media (prefers-reduced-motion: reduce) {
    .progress-indeterminate .progress-bar {
      animation-duration: 3s;
    }
  }
</style>
```

### src/lib/components/ui/Loading/CircularProgress.svelte

```svelte
<script lang="ts">
  import { cn } from '@utils/component';
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';

  export let value: number | undefined = undefined;
  export let max: number = 100;
  export let size: number = 48;
  export let thickness: number = 4;
  export let color: 'primary' | 'success' | 'warning' | 'danger' = 'primary';
  export let showValue: boolean = false;
  export let label: string = 'Progress';
  let className: string = '';
  export { className as class };

  const progress = tweened(0, {
    duration: 300,
    easing: cubicOut
  });

  $: isIndeterminate = value === undefined;
  $: percentage = isIndeterminate ? 0 : Math.min(Math.max((value! / max) * 100, 0), 100);
  $: progress.set(percentage);

  $: radius = (size - thickness) / 2;
  $: circumference = 2 * Math.PI * radius;
  $: strokeDashoffset = circumference - ($progress / 100) * circumference;

  const colorMap = {
    primary: 'var(--color-primary-500)',
    success: 'var(--color-success-500)',
    warning: 'var(--color-warning-500)',
    danger: 'var(--color-danger-500)'
  };

  $: classes = cn(
    'circular-progress',
    isIndeterminate && 'circular-progress-indeterminate',
    className
  );
</script>

<div
  class={classes}
  style="width: {size}px; height: {size}px"
  role="progressbar"
  aria-valuenow={isIndeterminate ? undefined : value}
  aria-valuemin={0}
  aria-valuemax={max}
  aria-label={label}
>
  <svg viewBox="0 0 {size} {size}">
    <circle
      class="circular-progress-track"
      cx={size / 2}
      cy={size / 2}
      r={radius}
      stroke-width={thickness}
      fill="none"
    />
    <circle
      class="circular-progress-bar"
      cx={size / 2}
      cy={size / 2}
      r={radius}
      stroke-width={thickness}
      fill="none"
      stroke={colorMap[color]}
      stroke-linecap="round"
      stroke-dasharray={circumference}
      stroke-dashoffset={isIndeterminate ? circumference * 0.75 : strokeDashoffset}
      transform="rotate(-90 {size / 2} {size / 2})"
    />
  </svg>

  {#if showValue && !isIndeterminate}
    <span class="circular-progress-value">
      {Math.round(percentage)}%
    </span>
  {/if}
</div>

<style>
  .circular-progress {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .circular-progress svg {
    width: 100%;
    height: 100%;
  }

  .circular-progress-track {
    stroke: var(--color-bg-muted);
  }

  .circular-progress-bar {
    transition: stroke-dashoffset 0.3s ease;
  }

  .circular-progress-indeterminate svg {
    animation: circular-rotate 1.4s linear infinite;
  }

  .circular-progress-indeterminate .circular-progress-bar {
    animation: circular-dash 1.4s ease-in-out infinite;
  }

  @keyframes circular-rotate {
    100% {
      transform: rotate(360deg);
    }
  }

  @keyframes circular-dash {
    0% {
      stroke-dasharray: 1, 200;
      stroke-dashoffset: 0;
    }
    50% {
      stroke-dasharray: 100, 200;
      stroke-dashoffset: -15;
    }
    100% {
      stroke-dasharray: 100, 200;
      stroke-dashoffset: -125;
    }
  }

  .circular-progress-value {
    position: absolute;
    font-size: calc(var(--text-xs));
    font-weight: var(--font-semibold);
    color: var(--color-fg-default);
  }

  @media (prefers-reduced-motion: reduce) {
    .circular-progress-indeterminate svg {
      animation-duration: 2.8s;
    }
    .circular-progress-indeterminate .circular-progress-bar {
      animation: none;
      stroke-dasharray: 80 200;
    }
  }
</style>
```

### src/lib/components/ui/Loading/LoadingOverlay.svelte

```svelte
<script lang="ts">
  import { fade } from 'svelte/transition';
  import { cn } from '@utils/component';
  import Spinner from './Spinner.svelte';

  export let loading: boolean = false;
  export let text: string = '';
  export let blur: boolean = false;
  export let opacity: number = 0.8;
  export let spinnerSize: 'sm' | 'md' | 'lg' = 'md';
  let className: string = '';
  export { className as class };

  $: classes = cn(
    'loading-overlay',
    blur && 'loading-overlay-blur',
    className
  );
</script>

<div class="loading-overlay-container">
  <slot />

  {#if loading}
    <div
      class={classes}
      style="--overlay-opacity: {opacity}"
      transition:fade={{ duration: 150 }}
    >
      <div class="loading-overlay-content">
        <Spinner size={spinnerSize} color="var(--color-primary-500)" />
        {#if text}
          <span class="loading-overlay-text">{text}</span>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .loading-overlay-container {
    position: relative;
  }

  .loading-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: rgba(var(--color-bg-surface-rgb), var(--overlay-opacity));
    z-index: var(--z-overlay);
  }

  .loading-overlay-blur {
    backdrop-filter: blur(4px);
  }

  .loading-overlay-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-3);
  }

  .loading-overlay-text {
    font-size: var(--text-sm);
    color: var(--color-fg-muted);
  }
</style>
```

### src/lib/components/ui/Loading/SkeletonPresets.svelte

```svelte
<script lang="ts" context="module">
  export type PresetType = 'card' | 'list-item' | 'avatar-text' | 'table-row' | 'article';
</script>

<script lang="ts">
  import Skeleton from './Skeleton.svelte';

  export let type: PresetType = 'card';
  export let count: number = 1;
</script>

{#each Array(count) as _, i}
  {#if type === 'card'}
    <div class="skeleton-card">
      <Skeleton variant="rectangular" height={160} />
      <div class="skeleton-card-content">
        <Skeleton variant="text" width="60%" />
        <Skeleton variant="text" lines={2} />
        <div class="skeleton-card-footer">
          <Skeleton variant="circular" width={32} height={32} />
          <Skeleton variant="text" width="30%" />
        </div>
      </div>
    </div>
  {:else if type === 'list-item'}
    <div class="skeleton-list-item">
      <Skeleton variant="circular" width={40} height={40} />
      <div class="skeleton-list-item-content">
        <Skeleton variant="text" width="40%" />
        <Skeleton variant="text" width="70%" />
      </div>
    </div>
  {:else if type === 'avatar-text'}
    <div class="skeleton-avatar-text">
      <Skeleton variant="circular" width={48} height={48} />
      <div class="skeleton-avatar-text-content">
        <Skeleton variant="text" width="120px" />
        <Skeleton variant="text" width="80px" />
      </div>
    </div>
  {:else if type === 'table-row'}
    <div class="skeleton-table-row">
      <Skeleton variant="text" width="15%" />
      <Skeleton variant="text" width="25%" />
      <Skeleton variant="text" width="20%" />
      <Skeleton variant="text" width="15%" />
      <Skeleton variant="text" width="10%" />
    </div>
  {:else if type === 'article'}
    <div class="skeleton-article">
      <Skeleton variant="text" width="80%" height="2em" />
      <div class="skeleton-article-meta">
        <Skeleton variant="circular" width={24} height={24} />
        <Skeleton variant="text" width="100px" />
        <Skeleton variant="text" width="80px" />
      </div>
      <Skeleton variant="rectangular" height={200} />
      <Skeleton variant="text" lines={4} />
    </div>
  {/if}
{/each}

<style>
  .skeleton-card {
    background: var(--color-bg-surface);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .skeleton-card-content {
    padding: var(--spacing-4);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-3);
  }

  .skeleton-card-footer {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    margin-top: var(--spacing-2);
  }

  .skeleton-list-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-3);
    padding: var(--spacing-3);
  }

  .skeleton-list-item-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-2);
  }

  .skeleton-avatar-text {
    display: flex;
    align-items: center;
    gap: var(--spacing-3);
  }

  .skeleton-avatar-text-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
  }

  .skeleton-table-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-4);
    padding: var(--spacing-3) 0;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .skeleton-article {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-4);
  }

  .skeleton-article-meta {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
  }
</style>
```

### src/lib/stores/loading.ts

```typescript
import { writable, derived } from 'svelte/store';

interface LoadingState {
  [key: string]: boolean;
}

function createLoadingStore() {
  const { subscribe, update, set } = writable<LoadingState>({});

  return {
    subscribe,

    start(key: string) {
      update(state => ({ ...state, [key]: true }));
    },

    stop(key: string) {
      update(state => ({ ...state, [key]: false }));
    },

    toggle(key: string) {
      update(state => ({ ...state, [key]: !state[key] }));
    },

    isLoading(key: string) {
      return derived({ subscribe }, $state => $state[key] ?? false);
    },

    reset() {
      set({});
    },

    // Helper for async operations
    async withLoading<T>(key: string, fn: () => Promise<T>): Promise<T> {
      this.start(key);
      try {
        return await fn();
      } finally {
        this.stop(key);
      }
    }
  };
}

export const loadingStore = createLoadingStore();

// Derived store for any loading state
export const isAnyLoading = derived(
  loadingStore,
  $state => Object.values($state).some(Boolean)
);

// Global loading state for page transitions
export const pageLoading = writable(false);
```

### Usage Examples

```svelte
<script>
  import {
    Skeleton,
    Spinner,
    Progress,
    CircularProgress,
    LoadingOverlay,
    SkeletonPresets
  } from '@components/ui';
  import { loadingStore, isAnyLoading } from '@stores/loading';

  let loading = true;
  let progress = 0;

  // Simulate loading
  const interval = setInterval(() => {
    progress += 10;
    if (progress >= 100) {
      clearInterval(interval);
      loading = false;
    }
  }, 500);

  // Using loading store
  async function fetchData() {
    await loadingStore.withLoading('users', async () => {
      const response = await fetch('/api/users');
      return response.json();
    });
  }
</script>

<!-- Skeleton loaders -->
<Skeleton variant="text" width="200px" />
<Skeleton variant="circular" width={48} height={48} />
<Skeleton variant="rectangular" height={100} />
<Skeleton variant="text" lines={3} />

<!-- Skeleton presets -->
<SkeletonPresets type="card" count={3} />
<SkeletonPresets type="list-item" count={5} />

<!-- Spinners -->
<Spinner size="sm" />
<Spinner size="md" color="var(--color-primary-500)" />
<Spinner size="lg" speed="slow" />

<!-- Progress bars -->
<Progress value={progress} />
<Progress value={progress} color="success" showValue />
<Progress /> <!-- Indeterminate -->

<!-- Circular progress -->
<CircularProgress value={progress} showValue />
<CircularProgress /> <!-- Indeterminate -->

<!-- Loading overlay -->
<LoadingOverlay loading={loading} text="Loading data..." blur>
  <div class="content">
    Your content here
  </div>
</LoadingOverlay>

<!-- With loading store -->
{#if $loadingStore['users']}
  <Spinner />
{:else}
  <UserList />
{/if}

{#if $isAnyLoading}
  <div class="global-loading-indicator">
    <Spinner size="sm" />
  </div>
{/if}
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/Loading.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, waitFor } from '@testing-library/svelte';
import { get } from 'svelte/store';
import Skeleton from '@components/ui/Loading/Skeleton.svelte';
import Spinner from '@components/ui/Loading/Spinner.svelte';
import Progress from '@components/ui/Loading/Progress.svelte';
import { loadingStore, isAnyLoading } from '@stores/loading';

describe('Skeleton', () => {
  it('should render with text variant by default', () => {
    const { container } = render(Skeleton);
    expect(container.querySelector('.skeleton-text')).toBeInTheDocument();
  });

  it('should render multiple lines', () => {
    const { container } = render(Skeleton, { props: { lines: 3, variant: 'text' } });
    expect(container.querySelectorAll('.skeleton')).toHaveLength(3);
  });

  it('should render circular variant', () => {
    const { container } = render(Skeleton, { props: { variant: 'circular' } });
    expect(container.querySelector('.skeleton-circular')).toBeInTheDocument();
  });

  it('should apply custom dimensions', () => {
    const { container } = render(Skeleton, { props: { width: 200, height: 100 } });
    const skeleton = container.querySelector('.skeleton') as HTMLElement;
    expect(skeleton.style.width).toBe('200px');
    expect(skeleton.style.height).toBe('100px');
  });
});

describe('Spinner', () => {
  it('should render with correct size', () => {
    const { container } = render(Spinner, { props: { size: 'lg' } });
    const svg = container.querySelector('svg');
    expect(svg?.getAttribute('width')).toBe('32');
  });

  it('should have accessible label', () => {
    const { container } = render(Spinner, { props: { label: 'Loading users' } });
    const svg = container.querySelector('svg');
    expect(svg?.getAttribute('aria-label')).toBe('Loading users');
  });
});

describe('Progress', () => {
  it('should render determinate progress', () => {
    const { container } = render(Progress, { props: { value: 50, max: 100 } });
    const progressbar = container.querySelector('[role="progressbar"]');
    expect(progressbar?.getAttribute('aria-valuenow')).toBe('50');
  });

  it('should render indeterminate when no value', () => {
    const { container } = render(Progress);
    expect(container.querySelector('.progress-indeterminate')).toBeInTheDocument();
  });

  it('should show percentage value', () => {
    const { getByText } = render(Progress, { props: { value: 75, showValue: true } });
    expect(getByText('75%')).toBeInTheDocument();
  });
});

describe('loadingStore', () => {
  it('should start and stop loading', () => {
    loadingStore.reset();
    loadingStore.start('test');
    expect(get(loadingStore)['test']).toBe(true);

    loadingStore.stop('test');
    expect(get(loadingStore)['test']).toBe(false);
  });

  it('should track multiple loading states', () => {
    loadingStore.reset();
    loadingStore.start('users');
    loadingStore.start('posts');

    expect(get(isAnyLoading)).toBe(true);

    loadingStore.stop('users');
    expect(get(isAnyLoading)).toBe(true);

    loadingStore.stop('posts');
    expect(get(isAnyLoading)).toBe(false);
  });

  it('should wrap async operations', async () => {
    loadingStore.reset();
    const mockFn = vi.fn().mockResolvedValue('result');

    const result = await loadingStore.withLoading('async', mockFn);

    expect(result).toBe('result');
    expect(get(loadingStore)['async']).toBe(false);
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [203-toast-component.md](./203-toast-component.md) - Toast notifications
- [212-error-boundaries.md](./212-error-boundaries.md) - Error boundaries
