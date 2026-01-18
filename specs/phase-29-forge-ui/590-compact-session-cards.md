# 590 - Compact Session Cards

## Problem
Session cards in sidebar take too much vertical space, limiting visible sessions.

## Solution
Make session cards more compact while preserving essential info.

## Target File
`web/src/lib/components/forge/SessionSidebar.svelte`

## Design

```
┌─────────────────────────────────┐
│ Session Name (2 lines max)  🟢  │
│ Goal preview...            2p  │
│ ─────────────────────────────  │
│ DRAFTING      17h ago          │
└─────────────────────────────────┘
```

## Acceptance Criteria

- [ ] Session card height reduced by 30%
- [ ] Name limited to 2 lines with `line-clamp: 2`
- [ ] Goal shows 1 line with ellipsis
- [ ] Phase badge and participant count on same row
- [ ] Timestamp right-aligned on badge row
- [ ] Active session has prominent border highlight
