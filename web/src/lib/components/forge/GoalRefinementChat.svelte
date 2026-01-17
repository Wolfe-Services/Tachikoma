<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { marked } from 'marked';
  import { goalRefinementStore } from '$lib/services/goalRefinement';
  import type { RefinementMessage } from '$lib/services/goalRefinement';
  import Icon from '$lib/components/common/Icon.svelte';

  export let currentGoal: string = '';

  const dispatch = createEventDispatcher<{
    close: void;
    applySuggestions: string;
  }>();

  let inputValue = '';
  let messagesContainer: HTMLDivElement;
  let inputElement: HTMLInputElement;

  // Subscribe to stores
  let messages: RefinementMessage[] = [];
  let refinementState = {
    isActive: false,
    isStreaming: false,
    initialGoal: '',
    contextGaps: [] as { category: string; description: string; question: string; priority: string; filled: boolean }[],
    currentGapIndex: 0,
    refinedGoal: '',
    error: null as string | null
  };

  const unsubMessages = goalRefinementStore.messages.subscribe(m => {
    messages = m;
    setTimeout(scrollToBottom, 50);
  });

  const unsubState = goalRefinementStore.state.subscribe(s => {
    refinementState = s;
  });

  onMount(() => {
    goalRefinementStore.startRefinement(currentGoal);
    inputElement?.focus();
  });

  onDestroy(() => {
    unsubMessages();
    unsubState();
    goalRefinementStore.reset();
  });

  function scrollToBottom() {
    if (messagesContainer) {
      messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }
  }

  function handleSubmit() {
    if (!inputValue.trim() || refinementState.isStreaming) return;
    goalRefinementStore.submitAnswer(inputValue.trim());
    inputValue = '';
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      handleSubmit();
    }
  }

  function handleClose() {
    goalRefinementStore.stopRefinement();
    dispatch('close');
  }

  function handleApplySuggestions() {
    const markdown = goalRefinementStore.getRefinedGoalMarkdown();
    dispatch('applySuggestions', markdown);
  }

  function renderMarkdown(content: string): string {
    try {
      return marked(content) as string;
    } catch {
      return content;
    }
  }

  $: hasRefinedGoal = refinementState.refinedGoal.length > 0;
  $: filledCount = refinementState.contextGaps.filter(g => g.filled).length;
  $: totalGaps = refinementState.contextGaps.length;
</script>

<div class="refinement-panel" data-testid="goal-refinement-chat">
  <!-- Panel Header -->
  <header class="panel-header">
    <div class="header-title">
      <Icon name="message-circle" size={16} />
      <span>Q&A REFINEMENT</span>
    </div>
    {#if totalGaps > 0}
      <div class="progress-badge">
        {filledCount}/{totalGaps}
      </div>
    {/if}
    <button 
      type="button" 
      class="close-btn" 
      on:click={handleClose}
      aria-label="Close"
    >
      <Icon name="x" size={14} />
    </button>
  </header>

  <!-- Progress Track -->
  {#if totalGaps > 0}
    <div class="progress-track">
      <div class="progress-fill" style="width: {(filledCount / totalGaps) * 100}%"></div>
    </div>
  {/if}

  <!-- Messages Area -->
  <div class="messages-area" bind:this={messagesContainer}>
    {#each messages as message (message.id)}
      <div class="message" class:is-user={message.role === 'user'}>
        <div class="message-indicator">
          {#if message.role === 'assistant'}
            <Icon name="cpu" size={14} />
          {:else}
            <Icon name="user" size={14} />
          {/if}
        </div>
        <div class="message-content">
          {#if message.status === 'pending'}
            <div class="typing-dots">
              <span></span><span></span><span></span>
            </div>
          {:else if message.content}
            <div class="message-text">
              {@html renderMarkdown(message.content)}
            </div>
          {/if}
        </div>
      </div>
    {/each}
  </div>

  <!-- Input Area -->
  <div class="input-area">
    <input
      bind:this={inputElement}
      bind:value={inputValue}
      type="text"
      class="chat-input"
      placeholder="Type your response..."
      disabled={refinementState.isStreaming}
      on:keydown={handleKeydown}
      data-testid="refinement-input"
    />
    <button
      type="button"
      class="send-btn"
      disabled={!inputValue.trim() || refinementState.isStreaming}
      on:click={handleSubmit}
      aria-label="Send"
    >
      <Icon name="send" size={14} />
    </button>
  </div>

  <!-- Actions -->
  {#if hasRefinedGoal}
    <div class="actions-bar">
      <button type="button" class="btn-secondary" on:click={() => inputElement?.focus()}>
        Continue
      </button>
      <button type="button" class="btn-primary" on:click={handleApplySuggestions}>
        <Icon name="check" size={14} />
        Apply to Goal
      </button>
    </div>
  {/if}
</div>

<style>
  .refinement-panel {
    display: flex;
    flex-direction: column;
    background: rgba(22, 27, 34, 0.75);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    border-radius: 12px;
    overflow: hidden;
    margin-top: 1rem;
    max-height: 480px;
    backdrop-filter: blur(8px);
  }

  /* Header */
  .panel-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    background: linear-gradient(90deg, rgba(78, 205, 196, 0.08), transparent);
    border-bottom: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    font-weight: 600;
    color: var(--tachi-cyan, #4ecdc4);
    letter-spacing: 1.5px;
  }

  .progress-badge {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.6rem;
    font-weight: 600;
    padding: 0.2rem 0.5rem;
    background: rgba(78, 205, 196, 0.1);
    border: 1px solid rgba(78, 205, 196, 0.2);
    border-radius: 4px;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    letter-spacing: 0.5px;
  }

  .close-btn {
    margin-left: auto;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: transparent;
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    border-radius: 4px;
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .close-btn:hover {
    background: rgba(255, 107, 107, 0.1);
    border-color: rgba(255, 107, 107, 0.3);
    color: var(--tachi-red, #ff6b6b);
  }

  /* Progress Track */
  .progress-track {
    height: 2px;
    background: var(--bg-tertiary, #1c2128);
  }

  .progress-fill {
    height: 100%;
    background: var(--tachi-cyan, #4ecdc4);
    transition: width 0.4s ease;
  }

  /* Messages */
  .messages-area {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    min-height: 200px;
    max-height: 320px;
  }

  .message {
    display: flex;
    align-items: flex-start;
    gap: 0.625rem;
  }

  .message.is-user {
    flex-direction: row-reverse;
  }

  .message-indicator {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    flex-shrink: 0;
    background: var(--bg-tertiary, #1c2128);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
    color: var(--text-muted, rgba(230, 237, 243, 0.5));
  }

  .message.is-user .message-indicator {
    background: rgba(78, 205, 196, 0.1);
    border-color: rgba(78, 205, 196, 0.2);
    color: var(--tachi-cyan, #4ecdc4);
  }

  .message-content {
    max-width: 85%;
    padding: 0.625rem 0.875rem;
    border-radius: 8px;
    font-size: 0.85rem;
    line-height: 1.5;
    background: var(--bg-tertiary, #1c2128);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
    color: var(--text-secondary, rgba(230, 237, 243, 0.8));
  }

  .message.is-user .message-content {
    background: rgba(78, 205, 196, 0.08);
    border-color: rgba(78, 205, 196, 0.2);
    color: var(--text-primary, #e6edf3);
  }

  .message-text :global(p) {
    margin: 0 0 0.5rem 0;
  }

  .message-text :global(p:last-child) {
    margin-bottom: 0;
  }

  .message-text :global(h2) {
    margin: 0.625rem 0 0.375rem 0;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--tachi-cyan, #4ecdc4);
  }

  .message-text :global(h2:first-child) {
    margin-top: 0;
  }

  .message-text :global(ul),
  .message-text :global(ol) {
    margin: 0.375rem 0;
    padding-left: 1.125rem;
  }

  .message-text :global(li) {
    margin-bottom: 0.125rem;
  }

  .message-text :global(strong) {
    color: var(--text-primary, #e6edf3);
  }

  .message-text :global(hr) {
    border: none;
    border-top: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    margin: 0.625rem 0;
  }

  /* Typing Animation */
  .typing-dots {
    display: flex;
    gap: 4px;
    padding: 0.25rem 0;
  }

  .typing-dots span {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--tachi-cyan, #4ecdc4);
    animation: dotPulse 1.2s infinite ease-in-out;
  }

  .typing-dots span:nth-child(2) { animation-delay: 0.15s; }
  .typing-dots span:nth-child(3) { animation-delay: 0.3s; }

  @keyframes dotPulse {
    0%, 80%, 100% { opacity: 0.25; transform: scale(0.85); }
    40% { opacity: 1; transform: scale(1); }
  }

  /* Input Area */
  .input-area {
    display: flex;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-top: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
    background: var(--bg-tertiary, #1c2128);
  }

  .chat-input {
    flex: 1;
    padding: 0.5rem 0.75rem;
    background: var(--bg-secondary, #161b22);
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
    border-radius: 6px;
    color: var(--text-primary, #e6edf3);
    font-size: 0.85rem;
    transition: all 0.15s ease;
  }

  .chat-input::placeholder {
    color: var(--text-muted, rgba(230, 237, 243, 0.35));
  }

  .chat-input:focus {
    outline: none;
    border-color: var(--tachi-cyan, #4ecdc4);
    box-shadow: 0 0 0 2px rgba(78, 205, 196, 0.1);
  }

  .chat-input:disabled {
    opacity: 0.5;
  }

  .send-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    background: rgba(78, 205, 196, 0.1);
    border: 1px solid rgba(78, 205, 196, 0.25);
    border-radius: 6px;
    color: var(--tachi-cyan, #4ecdc4);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .send-btn:hover:not(:disabled) {
    background: rgba(78, 205, 196, 0.2);
    border-color: var(--tachi-cyan, #4ecdc4);
  }

  .send-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  /* Actions Bar */
  .actions-bar {
    display: flex;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    border-top: 1px solid var(--border-color, rgba(78, 205, 196, 0.1));
    background: linear-gradient(90deg, rgba(78, 205, 196, 0.04), transparent);
  }

  .btn-primary,
  .btn-secondary {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.375rem;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.5px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-primary {
    background: linear-gradient(135deg, var(--tachi-cyan-dark, #2d7a7a), var(--tachi-cyan, #4ecdc4));
    color: var(--bg-primary, #0d1117);
    border: 1px solid rgba(78, 205, 196, 0.5);
  }

  .btn-primary:hover {
    box-shadow: 0 0 12px rgba(78, 205, 196, 0.25);
  }

  .btn-secondary {
    background: var(--bg-tertiary, #1c2128);
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.15));
  }

  .btn-secondary:hover {
    background: rgba(78, 205, 196, 0.08);
    color: var(--text-primary, #e6edf3);
  }
</style>
