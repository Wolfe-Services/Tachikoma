# 588 - Deliberation View Flex Fix

## Problem
DeliberationView has `height: 100%` which doesn't work inside a flex container with overflow.

## Solution
Fix CSS to properly fill available space using flexbox.

## Target File
`web/src/lib/components/forge/DeliberationView.svelte`

## Changes

Update the `.deliberation-view` styles:

```css
.deliberation-view {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0; /* Critical for flex overflow */
  /* remove height: 100% */
}

.messages-container {
  flex: 1;
  min-height: 0; /* Allow shrinking */
  overflow-y: auto;
}
```

## Acceptance Criteria

- [ ] Remove `height: 100%` from `.deliberation-view`
- [ ] Add `flex: 1` and `min-height: 0` to `.deliberation-view`
- [ ] Add `min-height: 0` to `.messages-container`
- [ ] Messages scroll within their container without clipping
- [ ] Phase header stays sticky at top of deliberation area
