# 589 - Session List Tooltips

## Problem
Session names and goals are truncated in the sidebar with no way to read the full text.

## Solution
Add tooltips that show full session name and goal preview on hover.

## Target File
`web/src/lib/components/forge/SessionSidebar.svelte`

## Changes

1. Add `title` attribute with full session name to session cards
2. Show goal preview (first 100 chars) in tooltip
3. Ensure session cards have proper text truncation CSS

## Acceptance Criteria

- [ ] Each session card has a `title` attribute with full name
- [ ] Tooltip shows session goal preview (truncated to 100 chars with "...")
- [ ] Session name displays 2 lines max with ellipsis
- [ ] Goal preview shows 1 line with ellipsis
- [ ] Phase badge and timestamp always visible
