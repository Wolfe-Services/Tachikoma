<script lang="ts">
  import { marked } from 'marked';
  import Icon from '$lib/components/common/Icon.svelte';
  import Spinner from '$lib/components/ui/Spinner/Spinner.svelte';
  import { deliberationStore } from '$lib/services/deliberation';
  import { forgeSessionStore } from '$lib/stores/forgeSession';
  import type { ForgeSession, SessionPhase } from '$lib/types/forge';

  export let session: ForgeSession | null;
  export let phase: SessionPhase;

  let messagesContainer: HTMLElement;

  // Subscribe to the nested stores
  const deliberationState = deliberationStore.state;
  const messagesStore = deliberationStore.messages;
  
  $: isRunning = $deliberationState.isRunning;
  $: activeParticipantId = $deliberationState.activeParticipantId;
  $: messages = $messagesStore;
  $: canStart = session && !isRunning && messages.length === 0;

  function handleStart() {
    if (session) {
      deliberationStore.startDeliberation(session);
    }
  }

  function handleStop() {
    deliberationStore.stopDeliberation(session?.id);
  }

  function handleContinue() {
    if (session) {
      // Move to next phase and continue
      const nextPhase = getNextPhase(phase);
      if (nextPhase) {
        forgeSessionStore.updateSessionPhase(session.id, nextPhase);
        deliberationStore.clearMessages();
        // Auto-start the next phase after a brief delay
        setTimeout(() => {
          const updatedSession = { ...session!, phase: nextPhase };
          deliberationStore.startDeliberation(updatedSession);
        }, 500);
      }
    }
  }

  function getNextPhase(current: SessionPhase): SessionPhase | null {
    const flow: SessionPhase[] = ['drafting', 'critiquing', 'converging', 'completed'];
    const idx = flow.indexOf(current);
    return idx >= 0 && idx < flow.length - 1 ? flow[idx + 1] : null;
  }

  function getPhaseLabel(phase: SessionPhase): string {
    const labels: Record<SessionPhase, string> = {
      idle: 'Ready',
      configuring: 'Configuring',
      drafting: 'Initial Drafts',
      critiquing: 'Critique Round',
      deliberating: 'Deliberation',
      converging: 'Synthesizing',
      completed: 'Complete',
      paused: 'Paused',
      error: 'Error'
    };
    return labels[phase] || phase;
  }

  // Auto-scroll to bottom when new messages arrive
  $: if (messages.length && messagesContainer) {
    setTimeout(() => {
      messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }, 50);
  }

  function renderMarkdown(content: string): string {
    return marked(content) as string;
  }
</script>

<div class="deliberation-view" data-phase={phase}>
  <!-- Phase Header -->
  <div class="phase-header">
    <div class="phase-info">
      <div class="phase-icon">
        {#if phase === 'drafting'}✏️
        {:else if phase === 'critiquing'}🔍
        {:else if phase === 'converging'}🎯
        {:else if phase === 'completed'}✅
        {:else}💬{/if}
      </div>
      <div class="phase-text">
        <h3 class="phase-title">{getPhaseLabel(phase)} Phase</h3>
        <p class="phase-description">
          {#if phase === 'drafting'}
            Participants generate initial proposals and ideas
          {:else if phase === 'critiquing'}
            Review and provide feedback on proposals
          {:else if phase === 'converging'}
            Synthesize feedback into refined solutions
          {:else if phase === 'completed'}
            Deliberation complete - review results
          {:else}
            Structured deliberation in progress
          {/if}
        </p>
      </div>
    </div>
    <div class="phase-actions">
      {#if isRunning}
        <button type="button" class="btn btn-danger" on:click={handleStop}>
          <Icon name="square" size={16} />
          Stop
        </button>
      {:else if canStart}
        <button type="button" class="btn btn-primary" on:click={handleStart}>
          <Icon name="play" size={16} />
          Begin {getPhaseLabel(phase)}
        </button>
      {:else if messages.length > 0 && getNextPhase(phase)}
        {@const nextPhase = getNextPhase(phase)}
        {#if nextPhase}
          <button type="button" class="btn btn-primary" on:click={handleContinue}>
            <Icon name="arrow-right" size={16} />
            Continue to {getPhaseLabel(nextPhase)}
          </button>
        {/if}
      {/if}
    </div>
  </div>

  <!-- Messages Stream -->
  <div class="messages-container" bind:this={messagesContainer}>
    {#if messages.length === 0 && !isRunning}
      <div class="empty-state">
        <div class="empty-icon">
          <Icon name="message-circle" size={48} />
        </div>
        <p>Click "Begin {getPhaseLabel(phase)}" to start the deliberation</p>
      </div>
    {:else}
      {#each messages as message (message.id)}
        <div 
          class="message-card" 
          class:streaming={message.status === 'streaming'}
          class:thinking={message.type === 'thinking'}
          data-participant-type={message.participantType}
        >
          <div class="message-header">
            <div class="participant-avatar" class:ai={message.participantType === 'ai'}>
              {message.participantName.charAt(0).toUpperCase()}
            </div>
            <div class="participant-info">
              <span class="participant-name">{message.participantName}</span>
              <span class="message-time">
                {message.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
              </span>
            </div>
            <div class="message-status">
              {#if message.status === 'streaming'}
                <Spinner size={14} color="var(--tachi-cyan)" />
                <span>Generating...</span>
              {:else if message.status === 'complete'}
                <Icon name="check-circle" size={14} />
              {:else}
                <Spinner size={14} color="var(--tachi-cyan)" />
                <span>Thinking...</span>
              {/if}
            </div>
          </div>
          
          <div class="message-content">
            {#if message.content}
              {@html renderMarkdown(message.content)}
            {:else}
              <div class="thinking-indicator">
                <span class="dot"></span>
                <span class="dot"></span>
                <span class="dot"></span>
              </div>
            {/if}
          </div>
        </div>
      {/each}

      {#if isRunning && activeParticipantId}
        <div class="typing-indicator">
          <Spinner size={12} color="var(--tachi-cyan)" />
          <span>Participant is responding...</span>
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .deliberation-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: rgba(13, 17, 23, 0.35);
    border: 1px solid rgba(78, 205, 196, 0.18);
    border-radius: 16px;
    overflow: hidden;
  }

  .phase-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    background: rgba(78, 205, 196, 0.06);
    border-bottom: 1px solid rgba(78, 205, 196, 0.14);
  }

  .phase-info {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .phase-icon {
    font-size: 1.5rem;
  }

  .phase-text {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .phase-title {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--tachi-cyan, #4ecdc4);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .phase-description {
    margin: 0;
    font-size: 0.8rem;
    color: rgba(230, 237, 243, 0.6);
  }

  .phase-actions {
    display: flex;
    gap: 0.5rem;
  }

  .btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.9rem;
    border-radius: 8px;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    border: 1px solid transparent;
  }

  .btn-primary {
    background: linear-gradient(135deg, var(--tachi-cyan-dark, #2d7a7a), var(--tachi-cyan, #4ecdc4));
    color: var(--bg-primary, #0d1117);
    border-color: rgba(78, 205, 196, 0.5);
  }

  .btn-primary:hover {
    box-shadow: 0 0 20px rgba(78, 205, 196, 0.3);
  }

  .btn-danger {
    background: rgba(255, 107, 107, 0.15);
    color: rgba(255, 107, 107, 0.95);
    border-color: rgba(255, 107, 107, 0.35);
  }

  .btn-danger:hover {
    background: rgba(255, 107, 107, 0.25);
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 1rem;
    color: rgba(230, 237, 243, 0.4);
  }

  .empty-icon {
    opacity: 0.4;
  }

  .empty-state p {
    margin: 0;
    font-size: 0.9rem;
  }

  .message-card {
    background: rgba(22, 27, 34, 0.6);
    border: 1px solid rgba(78, 205, 196, 0.12);
    border-radius: 12px;
    overflow: hidden;
    animation: fadeIn 0.3s ease;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .message-card.streaming {
    border-color: rgba(78, 205, 196, 0.35);
    box-shadow: 0 0 15px rgba(78, 205, 196, 0.1);
  }

  .message-card.thinking .message-content {
    padding: 1rem;
  }

  .message-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    background: rgba(78, 205, 196, 0.04);
    border-bottom: 1px solid rgba(78, 205, 196, 0.08);
  }

  .participant-avatar {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: linear-gradient(135deg, rgba(63, 185, 80, 0.25), rgba(63, 185, 80, 0.08));
    border: 1px solid rgba(63, 185, 80, 0.35);
    color: rgba(63, 185, 80, 0.95);
    font-weight: 600;
    font-size: 0.85rem;
  }

  .participant-avatar.ai {
    background: linear-gradient(135deg, rgba(88, 166, 255, 0.25), rgba(88, 166, 255, 0.08));
    border-color: rgba(88, 166, 255, 0.35);
    color: rgba(88, 166, 255, 0.95);
  }

  .participant-info {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .participant-name {
    font-weight: 600;
    color: rgba(230, 237, 243, 0.9);
    font-size: 0.85rem;
  }

  .message-time {
    font-size: 0.75rem;
    color: rgba(230, 237, 243, 0.4);
  }

  .message-status {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.7rem;
    color: var(--tachi-cyan, #4ecdc4);
  }

  .message-content {
    padding: 1rem 1.25rem;
    color: rgba(230, 237, 243, 0.9);
    font-size: 0.9rem;
    line-height: 1.65;
  }

  :global(.message-content p) {
    margin: 0 0 0.75rem 0;
  }

  :global(.message-content p:last-child) {
    margin-bottom: 0;
  }

  :global(.message-content h1),
  :global(.message-content h2),
  :global(.message-content h3),
  :global(.message-content h4) {
    margin: 1rem 0 0.5rem 0;
    color: var(--text-primary, #e6edf3);
    font-weight: 600;
  }

  :global(.message-content h1:first-child),
  :global(.message-content h2:first-child),
  :global(.message-content h3:first-child) {
    margin-top: 0;
  }

  :global(.message-content strong) {
    color: var(--text-primary, #e6edf3);
    font-weight: 600;
  }

  :global(.message-content ul),
  :global(.message-content ol) {
    margin: 0.5rem 0;
    padding-left: 1.5rem;
  }

  :global(.message-content li) {
    margin-bottom: 0.35rem;
  }

  :global(.message-content code) {
    background: rgba(78, 205, 196, 0.12);
    padding: 0.15rem 0.35rem;
    border-radius: 4px;
    font-size: 0.85rem;
    font-family: 'JetBrains Mono', monospace;
    color: var(--tachi-cyan, #4ecdc4);
  }

  :global(.message-content pre) {
    background: rgba(13, 17, 23, 0.6);
    padding: 0.9rem 1rem;
    border-radius: 8px;
    overflow-x: auto;
    margin: 0.75rem 0;
    border: 1px solid rgba(78, 205, 196, 0.14);
  }

  :global(.message-content pre code) {
    background: none;
    padding: 0;
    color: rgba(230, 237, 243, 0.85);
  }

  :global(.message-content blockquote) {
    margin: 0.75rem 0;
    padding-left: 1rem;
    border-left: 3px solid rgba(78, 205, 196, 0.4);
    color: rgba(230, 237, 243, 0.7);
    font-style: italic;
  }

  .thinking-indicator {
    display: flex;
    align-items: center;
    gap: 0.3rem;
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
    padding: 0.6rem 1rem;
    background: rgba(78, 205, 196, 0.08);
    border: 1px solid rgba(78, 205, 196, 0.15);
    border-radius: 8px;
    color: var(--tachi-cyan, #4ecdc4);
    font-size: 0.8rem;
  }
</style>
