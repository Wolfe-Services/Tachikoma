# Spec 188b: Wire Up Main Application Layout

## Header
- **Spec ID**: 188b
- **Phase**: 09 - UI Foundation
- **Component**: Main Layout Integration
- **Dependencies**: 188, 216, 296
- **Status**: CRITICAL - Main page is showing test placeholder instead of actual UI
- **Priority**: P0 - Blocking user experience

## Objective
Replace the test placeholder page with the actual application layout. The components exist (52+ Svelte components) but the main page still shows a "Test IPC Bridge" button instead of the dashboard/mission UI.

## Problem Statement
After 85%+ spec completion, the main page (`web/src/routes/+page.svelte`) is still showing a test/placeholder page instead of the actual application UI. Components like MissionPanel, SpecBrowser, ForgeUI, Dashboard exist in `$lib/components/` but are not integrated into the routes.

## Acceptance Criteria
- [ ] Main layout component shows sidebar navigation
- [ ] Default route shows Dashboard or Mission Panel
- [ ] Navigation works between Dashboard, Specs, Forge, Settings
- [ ] Sidebar can collapse/expand
- [ ] Theme (dark/light) applies consistently
- [ ] All existing components are accessible via navigation

## Implementation

### Update +page.svelte
Replace the test page with actual dashboard:

```svelte
<script lang="ts">
  import DashboardLayout from '$lib/components/dashboard/DashboardLayout.svelte';
  import MissionOverview from '$lib/components/mission/MissionOverview.svelte';
  import QuickActions from '$lib/components/dashboard/QuickActions.svelte';
</script>

<DashboardLayout>
  <MissionOverview />
  <QuickActions />
</DashboardLayout>
```

### Create App Shell Layout (+layout.svelte)
```svelte
<script lang="ts">
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import Header from '$lib/components/layout/Header.svelte';
  import { themeStore } from '$lib/stores/theme';
</script>

<div class="app-shell" data-theme={$themeStore}>
  <Sidebar />
  <main class="main-content">
    <Header />
    <slot />
  </main>
</div>
```

### Create Missing Layout Components
If not exist, create:
- `$lib/components/layout/Sidebar.svelte`
- `$lib/components/layout/Header.svelte`
- `$lib/components/layout/AppShell.svelte`

## Testing
1. Run `npm run dev` and verify dashboard shows instead of test page
2. Click navigation items and verify routing works
3. Verify all 52 existing components are accessible

## Notes
This is a P0 critical fix - the app is 85%+ complete but users see only a test page.
