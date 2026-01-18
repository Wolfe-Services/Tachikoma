# 594 - Human Input Bar

## Problem
Humans can't easily inject messages into the deliberation.

## Solution
Add an input bar at the bottom of the deliberation view for human contributions.

## Target File
`web/src/lib/components/forge/DeliberationView.svelte`

## Design

```
┌──────────────────────────────────────────────────┐
│ 💬 [Type your contribution...]          [Send]  │
└──────────────────────────────────────────────────┘
```

## Changes

1. Add textarea at bottom of deliberation view
2. Send button submits message to deliberation
3. Disable input when AI is actively generating
4. Show character count and "as Human" indicator

## Acceptance Criteria

- [ ] Input bar fixed at bottom of deliberation view
- [ ] Textarea with placeholder "Add your perspective..."
- [ ] Send button with icon, disabled when empty
- [ ] Input disabled during AI streaming (with visual indicator)
- [ ] Submitted messages appear in chat as human contributions
- [ ] Ctrl+Enter keyboard shortcut to send
