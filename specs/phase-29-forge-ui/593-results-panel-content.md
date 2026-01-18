# 593 - Results Panel Content

## Problem
Results panel shows "No results yet" even after deliberation completes.

## Solution
Display actual session messages and export options in results panel.

## Target File
`web/src/lib/components/forge/ResultPanel.svelte`

## Changes

1. Show message count and participant summary
2. Display condensed message list (collapsed by default)
3. Keep export buttons functional
4. Show session statistics (rounds, tokens, cost estimate)

## Acceptance Criteria

- [ ] Results panel shows message count when session has messages
- [ ] Shows list of participants who contributed
- [ ] Export buttons (Markdown, JSON, etc.) are visible and functional
- [ ] Shows "Expand to view messages" toggle
- [ ] Displays round count and estimated token usage
- [ ] Empty state only shown when truly no results exist
