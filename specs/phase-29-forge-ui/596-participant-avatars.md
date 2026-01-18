# 596 - Participant Avatar Row

## Problem
During deliberation, participant context is lost without looking at sidebar.

## Solution
Add a compact participant avatar row below the phase header.

## Target File
`web/src/lib/components/forge/DeliberationView.svelte`

## Design

```
┌────────────────────────────────────────┐
│ ✏️ DRAFTING PHASE    [Stop] [Continue] │
├────────────────────────────────────────┤
│ 👤 👤 🤖 🤖 🤖  ← 2 humans, 3 AI       │
└────────────────────────────────────────┘
```

## Changes

1. Add avatar row below phase header
2. Show participant avatars with colored rings
3. Active participant has animated ring
4. Tooltip shows participant name and role

## Acceptance Criteria

- [ ] Avatar row shows all session participants
- [ ] Avatars are 28px with 2px colored border
- [ ] Active participant has pulsing border animation
- [ ] Hover shows participant name and role
- [ ] Human avatars green, AI avatars colored by model
- [ ] "+N more" shown if more than 6 participants
