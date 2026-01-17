# Spec 568: Application Shell Integration

**Phase:** 27 - Ralph Integration  
**Status:** Planned  
**Priority:** P0 - Critical  
**Dependencies:** 186-235 (UI Foundation + Mission Panel specs)

## Overview

Wire up all existing UI components into a cohesive application shell. Currently the main page (`+page.svelte`) is a placeholder test page, but all the components exist in `lib/components/`.

## Problem Statement

The Ralph loop completed UI specs by building individual components, but never integrated them into the main application. The result is a working component library with no usable app.

## Acceptance Criteria

- [x] Replace placeholder `+page.svelte` with Mission Panel layout
- [x] Add sidebar navigation with routes (Mission, Specs, Forge, Settings, Dashboard)
- [x] Wire up existing components: `MissionComparison`, `ContextMeter`, `CostTracking`, etc.
- [x] Implement responsive layout with collapsible sidebar
- [x] Add header with app title, status indicator, and user menu
- [x] Fix the `crypto is not defined` error in mission:start handler
- [x] Verify all IPC handlers work with real UI interactions

## Implementation Details

### Layout Structure

```
┌─────────────────────────────────────────────────────────────┐
│  Header: Tachikoma | Status: Ready | [Settings] [User]      │
├──────────┬──────────────────────────────────────────────────┤
│          │                                                  │
│  Sidebar │              Main Content Area                   │
│          │                                                  │
│  Mission │  ┌─────────────────────────────────────────────┐ │
│  Specs   │  │        Active Mission / Dashboard            │ │
│  Forge   │  │                                              │ │
│  History │  └─────────────────────────────────────────────┘ │
│  Settings│                                                  │
│          │                                                  │
└──────────┴──────────────────────────────────────────────────┘
```

### File Changes

1. **`web/src/routes/+page.svelte`** - Replace with dashboard/mission panel
2. **`web/src/routes/+layout.svelte`** - Add sidebar navigation
3. **`web/src/lib/components/layout/AppShell.svelte`** - Create app shell wrapper
4. **`web/src/lib/components/layout/Sidebar.svelte`** - Navigation sidebar
5. **`web/src/lib/components/layout/Header.svelte`** - Top header bar
6. **`electron/main/native.ts`** - Fix crypto import for mission:start

### Crypto Fix

Replace:
```typescript
crypto.randomUUID()
```

With:
```typescript
import { randomUUID } from 'crypto';
randomUUID()
```

Or use Electron's built-in:
```typescript
import { randomBytes } from 'crypto';
const id = randomBytes(16).toString('hex');
```

## Testing

- Launch app and verify sidebar navigation works
- Click through all routes (Mission, Specs, Forge, Settings)
- Verify components render without console errors
- Test mission:start now works without crypto error

## References

- Pattern: `web/src/lib/components/mission/` (existing components)
- Pattern: `web/src/lib/stores/mission.ts` (state management)
