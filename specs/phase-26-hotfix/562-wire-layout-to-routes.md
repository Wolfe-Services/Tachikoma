# Spec 562: Wire AppShell to Root Layout

## Header
- **Spec ID**: 562
- **Phase**: 26 - Hotfix Critical UI
- **Priority**: P0 - CRITICAL
- **Dependencies**: 561
- **Estimated Time**: 15 minutes

## Objective
Update the root layout (`+layout.svelte`) to use the AppShell component so ALL routes get the sidebar navigation.

## Acceptance Criteria
- [ ] `web/src/routes/+layout.svelte` imports and uses AppShell
- [ ] All child routes render inside the AppShell
- [ ] Theme variables are defined in root layout CSS
- [ ] Body has dark theme by default

## Implementation

### Update +layout.svelte
```svelte
<script lang="ts">
  import AppShell from '$lib/components/layout/AppShell.svelte';
  import '../app.css';
</script>

<AppShell>
  <slot />
</AppShell>

<style>
  :global(:root) {
    --bg-primary: #0f0f1a;
    --bg-secondary: #1a1a2e;
    --bg-tertiary: #252542;
    --text-primary: #ffffff;
    --text-secondary: rgba(255, 255, 255, 0.7);
    --text-muted: rgba(255, 255, 255, 0.5);
    --accent-primary: #3b82f6;
    --accent-success: #22c55e;
    --accent-warning: #f59e0b;
    --accent-error: #ef4444;
    --sidebar-bg: #1a1a2e;
    --border-color: rgba(255, 255, 255, 0.1);
  }
  
  :global(body) {
    margin: 0;
    padding: 0;
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: system-ui, -apple-system, sans-serif;
  }
  
  :global(*) {
    box-sizing: border-box;
  }
</style>
```

## Verification
1. Run `npm run dev`
2. Open http://localhost:1420
3. Sidebar should be visible on the left
4. Navigation links should be clickable
