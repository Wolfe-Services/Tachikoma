# 592 - Sticky Phase Header

## Problem
Phase header scrolls away when reading long messages, losing context.

## Solution
Make the phase header sticky at top of deliberation view.

## Target File
`web/src/lib/components/forge/DeliberationView.svelte`

## Changes

```css
.phase-header {
  position: sticky;
  top: 0;
  z-index: 10;
  backdrop-filter: blur(12px);
  background: rgba(13, 17, 23, 0.85);
}
```

## Acceptance Criteria

- [ ] Phase header stays fixed at top when scrolling messages
- [ ] Header has frosted glass effect (backdrop-filter)
- [ ] Header has z-index above message cards
- [ ] Header casts subtle shadow on scroll
- [ ] Start/Stop/Continue buttons always accessible
