# Spec 586: Frontend-Backend Wire

**Priority:** P0  
**Status:** planned  
**Depends on:** 581, 585  
**Estimated Effort:** 3 hours  
**Target Files:**
- `web/src/lib/services/forgeService.ts` (update)
- `web/src/lib/services/deliberation.ts` (update)
- `web/src/lib/ipc/types.ts` (update if needed)

---

## Overview

Wire the frontend to receive real streaming events from the Rust backend via IPC. Remove mock mode fallback when real backend is connected. Display actual LLM responses in real-time.

---

## Acceptance Criteria

- [ ] Update `forgeService.ts` to detect when NAPI bindings are available
- [ ] When backend is connected, `useMockMode` should be false
- [ ] Streaming tokens from backend display in UI in real-time
- [ ] Participant name and model are shown with each response
- [ ] Error messages from backend display in UI
- [ ] Add loading/thinking indicators per participant
- [ ] Handle backend disconnection gracefully (show reconnect message)
- [ ] Remove console.warn spam when in real mode
- [ ] Verify deliberation works end-to-end with real LLM calls

---

## Implementation Notes

The key changes are:

1. **forgeService.ts** - Check if `window.electron.nativeBindingsAvailable` is true
2. **deliberation.ts** - When IPC events arrive, update the messages store immediately
3. **DeliberationView.svelte** - Show streaming tokens as they arrive, not just complete messages

```typescript
// web/src/lib/services/forgeService.ts - key changes

// Check if real bindings are available
function checkBackendConnection(): boolean {
  if (typeof window !== 'undefined' && window.electron) {
    // Check if native bindings loaded successfully
    return window.electron.nativeBindingsAvailable === true;
  }
  return false;
}

// In createForgeService():
const ipcAvailable = isIpcAvailable();
const backendConnected = checkBackendConnection();

if (ipcAvailable && backendConnected) {
  state.update(s => ({ ...s, isConnected: true, useMockMode: false }));
  console.log('Forge: Connected to Rust backend');
} else {
  console.log('Forge: Running in mock mode (backend not available)');
}
```

```typescript
// web/src/lib/services/deliberation.ts - key changes

// Handle streaming tokens
forgeService.messages.subscribe(forgeMessages => {
  // Don't filter - show all messages including deltas
  const converted: DeliberationMessage[] = forgeMessages.map(fm => ({
    id: fm.messageId,
    participantId: fm.participantId,
    participantName: fm.participantName,
    participantType: fm.participantType,
    content: fm.content,
    timestamp: new Date(fm.timestamp),
    type: fm.type,  // 'delta' or 'complete'
    status: fm.status
  }));
  
  // Merge streaming tokens into existing messages
  messages.update(existing => {
    const result = [...existing];
    for (const msg of converted) {
      if (msg.type === 'delta') {
        // Find existing message for this participant and append
        const idx = result.findIndex(m => 
          m.participantId === msg.participantId && m.status === 'streaming'
        );
        if (idx >= 0) {
          result[idx].content += msg.content;
        } else {
          result.push({ ...msg, content: msg.content, status: 'streaming' });
        }
      } else if (msg.type === 'complete') {
        // Replace streaming message with complete version
        const idx = result.findIndex(m => 
          m.participantId === msg.participantId && m.status === 'streaming'
        );
        if (idx >= 0) {
          result[idx] = msg;
        } else {
          result.push(msg);
        }
      }
    }
    return result;
  });
});
```

```svelte
<!-- web/src/lib/components/forge/DeliberationView.svelte - streaming indicator -->

{#each messages as message}
  <div class="message" class:streaming={message.status === 'streaming'}>
    <div class="message-header">
      <span class="participant-name">{message.participantName}</span>
      {#if message.status === 'streaming'}
        <span class="typing-indicator">typing...</span>
      {/if}
    </div>
    <div class="message-content">
      {message.content}
      {#if message.status === 'streaming'}
        <span class="cursor">▊</span>
      {/if}
    </div>
  </div>
{/each}

<style>
  .streaming {
    border-left: 2px solid var(--tachi-cyan);
  }
  
  .typing-indicator {
    font-size: 0.75rem;
    color: var(--tachi-cyan);
    animation: pulse 1s infinite;
  }
  
  .cursor {
    animation: blink 0.8s infinite;
  }
  
  @keyframes blink {
    0%, 50% { opacity: 1; }
    51%, 100% { opacity: 0; }
  }
</style>
```
