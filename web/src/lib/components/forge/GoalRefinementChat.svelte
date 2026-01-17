<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { marked } from 'marked';
  import { goalRefinementStore } from '$lib/services/goalRefinement';
  import type { RefinementMessage } from '$lib/services/goalRefinement';
  import Spinner from '$lib/components/ui/Spinner/Spinner.svelte';

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
    currentQuestionIndex: 0,
    structuredGoal: { objective: '', context: '', constraints: '', successCriteria: '' },
    error: null as string | null
  };

  const unsubMessages = goalRefinementStore.messages.subscribe(m => {
    messages = m;
    // Auto-scroll on new messages
    setTimeout(scrollToBottom, 50);
  });

  const unsubState = goalRefinementStore.state.subscribe(s => {
    refinementState = s;
  });

  onMount(() => {
    goalRefinementStore.startRefinement();
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
    const markdown = goalRefinementStore.getStructuredGoalMarkdown();
    const newGoal = currentGoal 
      ? `${currentGoal}\n\n---\n\n${markdown}`
      : markdown;
    dispatch('applySuggestions', newGoal);
  }

  function renderMarkdown(content: string): string {
    try {
      return marked(content) as string;
    } catch {
      return content;
    }
  }

  $: hasStructuredGoal = refinementState.structuredGoal.objective.length > 0;
  $: allQuestionsAnswered = refinementState.currentQuestionIndex >= 3 && !refinementState.isStreaming;
</script>

<div class="refinement-chat" data-testid="goal-refinement-chat">
  <header class="chat-header">
    <div class="header-left">
      <span class="header-icon">🤖</span>
      <div class="header-text">
        <h3>AI Refinement Assistant</h3>
        <p class="header-subtitle">Let's clarify your session goal</p>
      </div>
    </div>
    <button 
      type="button" 
      class="close-btn" 
      on:click={handleClose}
      aria-label="Close refinement chat"
    >
      ×
    </button>
  </header>

  <div class="messages-container" bind:this={messagesContainer}>
    {#each messages as message (message.id)}
      <div 
        class="message" 
        class:assistant={message.role === 'assistant'}
        class:user={message.role === 'user'}
        class:streaming={message.status === 'streaming'}
      >
        <div class="message-avatar">
          {#if message.role === 'assistant'}
            🤖
          {:else}
            👤
          {/if}
        </div>
        <div class="message-bubble">
          {#if message.status === 'pending'}
            <div class="thinking-indicator">
              <span class="dot"></span>
              <span class="dot"></span>
              <span class="dot"></span>
            </div>
          {:else if message.content}
            <div class="message-content">
              {@html renderMarkdown(message.content)}
            </div>
          {/if}
        </div>
      </div>
    {/each}

    {#if refinementState.isStreaming && messages.length > 0}
      <div class="typing-indicator">
        <Spinner size={12} color="var(--tachi-cyan, #4ecdc4)" />
        <span>Assistant is typing...</span>
      </div>
    {/if}
  </div>

  <div class="input-area">
    <div class="input-wrapper">
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
        aria-label="Send message"
      >
        ▶
      </button>
    </div>
  </div>

  <footer class="chat-footer">
    <button
      type="button"
      class="btn btn-primary"
      disabled={!hasStructuredGoal}
      on:click={handleApplySuggestions}
    >
      Apply Suggestions to Goal
    </button>
    <button
      type="button"
      class="btn btn-secondary"
      on:click={() => inputElement?.focus()}
    >
      Continue Refining
    </button>
  </footer>
</div>

<style>
  .refinement-chat {
    display: flex;
    flex-direction: column;
    background: rgba(13, 17, 23, 0.6);
    border: 1px solid rgba(78, 205, 196, 0.25);
    border-radius: 12px;
    overflow: hidden;
    max-height: 500px;
    margin-top: 1rem;
    animation: slideIn 0.3s ease;
  }

  @keyframes slideIn {
    from { 
      opacity: 0; 
      transform: translateY(-10px); 
    }
    to { 
      opacity: 1; 
      transform: translateY(0); 
    }
  }

  .chat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.875rem 1rem;
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.12), rgba(78, 205, 196, 0.04));
    border-bottom: 1px solid rgba(78, 205, 196, 0.18);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .header-icon {
    font-size: 1.5rem;
  }

  .header-text h3 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--tachi-cyan, #4ecdc4);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .header-subtitle {
    margin: 0.15rem 0 0 0;
    font-size: 0.75rem;
    color: rgba(230, 237, 243, 0.5);
  }

  .close-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 107, 107, 0.1);
    border: 1px solid rgba(255, 107, 107, 0.25);
    border-radius: 6px;
    color: rgba(255, 107, 107, 0.8);
    font-size: 1.25rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .close-btn:hover {
    background: rgba(255, 107, 107, 0.2);
    color: rgba(255, 107, 107, 1);
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.875rem;
    min-height: 200px;
    max-height: 280px;
  }

  .message {
    display: flex;
    align-items: flex-start;
    gap: 0.625rem;
    animation: fadeIn 0.25s ease;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(5px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .message.user {
    flex-direction: row-reverse;
  }

  .message-avatar {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    font-size: 1rem;
    flex-shrink: 0;
  }

  .message.assistant .message-avatar {
    background: linear-gradient(135deg, rgba(88, 166, 255, 0.25), rgba(88, 166, 255, 0.08));
    border: 1px solid rgba(88, 166, 255, 0.35);
  }

  .message.user .message-avatar {
    background: linear-gradient(135deg, rgba(63, 185, 80, 0.25), rgba(63, 185, 80, 0.08));
    border: 1px solid rgba(63, 185, 80, 0.35);
  }

  .message-bubble {
    max-width: 85%;
    padding: 0.75rem 1rem;
    border-radius: 12px;
    font-size: 0.875rem;
    line-height: 1.55;
  }

  .message.assistant .message-bubble {
    background: rgba(22, 27, 34, 0.8);
    border: 1px solid rgba(78, 205, 196, 0.15);
    color: rgba(230, 237, 243, 0.9);
    border-bottom-left-radius: 4px;
  }

  .message.user .message-bubble {
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.18), rgba(78, 205, 196, 0.08));
    border: 1px solid rgba(78, 205, 196, 0.3);
    color: rgba(230, 237, 243, 0.95);
    border-bottom-right-radius: 4px;
  }

  .message.streaming .message-bubble {
    border-color: rgba(78, 205, 196, 0.4);
    box-shadow: 0 0 12px rgba(78, 205, 196, 0.1);
  }

  .message-content :global(p) {
    margin: 0 0 0.5rem 0;
  }

  .message-content :global(p:last-child) {
    margin-bottom: 0;
  }

  .message-content :global(h2) {
    margin: 0.75rem 0 0.5rem 0;
    font-size: 0.95rem;
    color: var(--tachi-cyan, #4ecdc4);
  }

  .message-content :global(h2:first-child) {
    margin-top: 0;
  }

  .message-content :global(ul),
  .message-content :global(ol) {
    margin: 0.5rem 0;
    padding-left: 1.25rem;
  }

  .message-content :global(li) {
    margin-bottom: 0.25rem;
  }

  .message-content :global(em) {
    color: rgba(230, 237, 243, 0.6);
  }

  .message-content :global(strong) {
    color: var(--text-primary, #e6edf3);
  }

  .thinking-indicator {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.25rem 0;
  }

  .thinking-indicator .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--tachi-cyan, #4ecdc4);
    animation: thinking 1.4s infinite ease-in-out;
  }

  .thinking-indicator .dot:nth-child(2) {
    animation-delay: 0.2s;
  }

  .thinking-indicator .dot:nth-child(3) {
    animation-delay: 0.4s;
  }

  @keyframes thinking {
    0%, 80%, 100% { opacity: 0.3; transform: scale(0.8); }
    40% { opacity: 1; transform: scale(1); }
  }

  .typing-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: rgba(78, 205, 196, 0.06);
    border: 1px solid rgba(78, 205, 196, 0.12);
    border-radius: 8px;
    color: var(--tachi-cyan, #4ecdc4);
    font-size: 0.75rem;
    align-self: flex-start;
  }

  .input-area {
    padding: 0.875rem 1rem;
    border-top: 1px solid rgba(78, 205, 196, 0.12);
    background: rgba(13, 17, 23, 0.4);
  }

  .input-wrapper {
    display: flex;
    gap: 0.5rem;
  }

  .chat-input {
    flex: 1;
    padding: 0.625rem 1rem;
    background: rgba(22, 27, 34, 0.6);
    border: 1px solid rgba(78, 205, 196, 0.18);
    border-radius: 8px;
    color: rgba(230, 237, 243, 0.9);
    font-size: 0.875rem;
    transition: all 0.15s ease;
  }

  .chat-input::placeholder {
    color: rgba(230, 237, 243, 0.35);
  }

  .chat-input:focus {
    outline: none;
    border-color: rgba(78, 205, 196, 0.45);
    box-shadow: 0 0 0 2px rgba(78, 205, 196, 0.1);
  }

  .chat-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .send-btn {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.25), rgba(78, 205, 196, 0.1));
    border: 1px solid rgba(78, 205, 196, 0.35);
    border-radius: 8px;
    color: var(--tachi-cyan, #4ecdc4);
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .send-btn:hover:not(:disabled) {
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.35), rgba(78, 205, 196, 0.15));
    box-shadow: 0 0 12px rgba(78, 205, 196, 0.2);
  }

  .send-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .chat-footer {
    display: flex;
    gap: 0.75rem;
    padding: 0.875rem 1rem;
    border-top: 1px solid rgba(78, 205, 196, 0.12);
    background: rgba(78, 205, 196, 0.03);
  }

  .btn {
    flex: 1;
    padding: 0.625rem 1rem;
    border-radius: 8px;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-primary {
    background: linear-gradient(135deg, var(--tachi-cyan-dark, #2d7a7a), var(--tachi-cyan, #4ecdc4));
    color: var(--bg-primary, #0d1117);
    border: 1px solid rgba(78, 205, 196, 0.5);
  }

  .btn-primary:hover:not(:disabled) {
    background: linear-gradient(135deg, var(--tachi-cyan, #4ecdc4), var(--tachi-cyan-bright, #6ee7df));
    box-shadow: 0 0 15px rgba(78, 205, 196, 0.25);
  }

  .btn-secondary {
    background: rgba(13, 17, 23, 0.4);
    color: rgba(230, 237, 243, 0.75);
    border: 1px solid rgba(78, 205, 196, 0.16);
  }

  .btn-secondary:hover:not(:disabled) {
    background: rgba(78, 205, 196, 0.08);
    color: rgba(230, 237, 243, 0.9);
  }
</style>
