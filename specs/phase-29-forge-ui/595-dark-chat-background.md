# 595 - Dark Chat Background

## Problem
Chat area blends into surrounding UI, reducing focus on messages.

## Solution
Darken the chat background to create visual depth and focus.

## Target File
`web/src/lib/components/forge/DeliberationView.svelte`

## Changes

```css
.deliberation-view {
  background: #0a0c0f; /* Near black */
  border: 1px solid rgba(78, 205, 196, 0.1);
}

.messages-container {
  background: linear-gradient(
    180deg,
    rgba(10, 12, 15, 0.95) 0%,
    rgba(13, 17, 23, 0.98) 100%
  );
}
```

## Acceptance Criteria

- [ ] Chat background is darker than surrounding panels (#0a0c0f)
- [ ] Subtle gradient adds depth
- [ ] Message cards have slight elevation/glow
- [ ] Border reduced to 1px with low opacity
- [ ] Empty state icon visible against dark background
