# 591 - Improved Message Bubbles

## Problem
Message cards are dense and hard to scan. Participant identity not prominent enough.

## Solution
Redesign message cards as chat bubbles with better visual hierarchy.

## Target File
`web/src/lib/components/forge/DeliberationView.svelte`

## Design

- Left-align AI messages with colored left border per model
- Human messages right-aligned with green accent
- Larger participant avatars (40px)
- Name and role on same line
- Streaming state shows pulsing border

## Color Map
```
Claude: #cc785c (terracotta)
GPT-4: #74aa9c (sage)
Gemini: #8b5cf6 (purple)
Ollama: #4ecdc4 (cyan)
Human: #3fb950 (green)
```

## Acceptance Criteria

- [ ] Message cards have 4px left border with participant color
- [ ] Participant avatar is 40px with initials
- [ ] Name and role display inline: "Claude · Architect"
- [ ] Streaming messages have pulsing glow animation
- [ ] Human messages visually distinct from AI messages
- [ ] Markdown renders with proper spacing
