# Spec 552: Conversational Spec Creation

**Phase:** 27 - Ralph Integration  
**Status:** Planned  
**Priority:** P1 - High  
**Dependencies:** 551 (Application Shell)
**Inspired By:** Ralph TUI's `ralph-tui create-prd --chat`

## Overview

Add a conversational "interview" mode where the user describes what they want in natural language, and Tachikoma generates a structured spec through back-and-forth Q&A.

## Problem Statement

Currently, spec creation requires:
1. Understanding the spec template format
2. Writing markdown by hand
3. Knowing what sections to include

Ralph TUI solves this with an interactive chat that asks questions and builds the spec for you.

## Acceptance Criteria

- [ ] Add `tachikoma chat` CLI command for conversational spec creation
- [ ] Implement interview flow with structured questions
- [ ] Generate spec markdown from conversation
- [ ] Create acceptance criteria checkboxes automatically
- [ ] Support both CLI and GUI modes
- [ ] Add "quick spec" button in UI that launches interview modal
- [ ] Save conversation history for reference
- [ ] Option to send generated spec to Forge for multi-model refinement

## Interview Flow

### Phase 1: Goal Gathering
```
Tachikoma: What would you like to build? Describe it in your own words.
User: I want to add OAuth login with GitHub

Tachikoma: Got it! Let me ask a few clarifying questions...
```

### Phase 2: Clarifying Questions
```
Tachikoma: Will this be the only auth method, or should it support multiple providers?
User: Multiple providers eventually, but GitHub first

Tachikoma: Should users be able to link multiple accounts?
User: Yes

Tachikoma: What should happen if the OAuth flow fails?
User: Show an error and let them retry
```

### Phase 3: Technical Probing
```
Tachikoma: I noticed you have existing auth patterns in the codebase. Should I follow those?
User: Yes, use the same session management

Tachikoma: Do you need any specific scopes from GitHub (email, repos, etc)?
User: Just email for identity
```

### Phase 4: Spec Generation
```
Tachikoma: Here's the generated spec:

# GitHub OAuth Integration

## Overview
Add GitHub as an OAuth provider, following existing auth patterns...

## Acceptance Criteria
- [ ] GitHub OAuth app configuration
- [ ] OAuth callback handler
- [ ] Session creation on successful auth
- [ ] Error handling with retry option
- [ ] Account linking support

Would you like to:
1. Edit this spec
2. Send to Forge for multi-model review
3. Save and start implementation
```

## CLI Interface

```bash
# Interactive chat mode
tachikoma chat --create-spec

# With initial prompt
tachikoma chat --create-spec "Add GitHub OAuth login"

# Non-interactive (use AI to generate questions/answers)
tachikoma chat --create-spec "Add GitHub OAuth" --auto

# Output options
tachikoma chat --create-spec --output specs/auth/github-oauth.md
```

## GUI Interface

1. **Quick Spec Button** in header or command palette (Cmd+K)
2. **Chat Modal** with conversation history
3. **Spec Preview Pane** showing generated markdown
4. **Action Buttons**: Edit, Forge, Save, Implement

## Implementation Details

### Prompt Engineering

System prompt for the interviewer:
```
You are helping a user create a software specification. Your job is to:

1. Understand their goal in plain language
2. Ask clarifying questions (max 5-7 questions)
3. Probe for edge cases they might not have considered
4. Reference existing patterns in their codebase
5. Generate a structured spec with acceptance criteria

Be conversational and friendly. Don't overwhelm with too many questions at once.
```

### Files to Create/Modify

1. **`crates/tachikoma-cli/src/commands/chat.rs`** - CLI command
2. **`web/src/lib/components/spec-browser/QuickSpecModal.svelte`** - GUI modal
3. **`crates/tachikoma-spec/src/generator.rs`** - Spec generation logic
4. **`crates/tachikoma-spec/src/interview.rs`** - Interview question logic

### Integration with Forge

After generating a spec, offer to:
1. Send directly to implementation
2. Send to Forge for multi-model critique and refinement
3. Save as draft for manual editing

## Testing

- Test CLI flow end-to-end
- Test GUI modal with various inputs
- Verify generated specs are valid and parseable
- Test integration with Forge

## References

- Ralph TUI's `create-prd --chat` command
- Existing spec template: `specs/phase-XX/XXX-template.md`
- Forge integration: `crates/tachikoma-forge/`
