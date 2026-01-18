# 587 - Chat-First Layout

## Problem
DeliberationView is inside a scrolling container, pushing chat below the fold. Users can't see AI responses.

## Solution
Restructure MainContentArea to make chat the primary content with session info collapsed.

## Target File
`web/src/lib/components/forge/MainContentArea.svelte`

## Changes

1. Move goal-section and info-grid into a collapsible drawer
2. Give DeliberationView proper flex sizing to fill available space
3. Only show collapsed session info during active deliberation

## Acceptance Criteria

- [ ] DeliberationView has `flex: 1` and `min-height: 0` instead of nested inside scroll
- [ ] During drafting/critiquing/converging phases, goal and info-grid are hidden
- [ ] Session header remains visible with name and phase badge
- [ ] Chat area fills available vertical space (no clipping)
- [ ] When phase is 'configuring' or 'completed', show full goal and info sections
