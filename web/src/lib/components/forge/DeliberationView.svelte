<script lang="ts">
  import { marked } from 'marked';
  import Icon from '$lib/components/common/Icon.svelte';
  import Spinner from '$lib/components/ui/Spinner/Spinner.svelte';
  import { deliberationStore } from '$lib/services/deliberation';
  import { forgeService } from '$lib/services/forgeService';
  import type { ForgeSession, SessionPhase } from '$lib/types/forge';

  export let session: ForgeSession | null;
  export let phase: SessionPhase;

  let messagesContainer: HTMLElement;
  let prompt = '';
  let selectedHumanId: string | null = null;
  let isSending = false;

  let artifactLoading = false;
  let artifactError: string | null = null;
  let artifactMarkdown: string | null = null;
  let latestDraft: string | null = null;

  let viewMode: 'feed' | 'artifacts' = 'feed';

  // Subscribe to the nested stores
  const deliberationState = deliberationStore.state;
  const messagesStore = deliberationStore.messages;

  // Always keep a safe array here; reactive assignments can run before store values settle.
  let messages: any[] = [];

  $: isRunning = $deliberationState.isRunning;
  $: activeParticipantId = $deliberationState.activeParticipantId;
  $: error = $deliberationState.error;
  $: useMockMode = $deliberationState.useMockMode;
  $: messages = Array.isArray($messagesStore) ? $messagesStore : [];

  $: aiCount = session?.participants?.filter(p => p.type === 'ai')?.length ?? 0;
  $: humans = session?.participants?.filter(p => p.type === 'human') ?? [];
  $: if (humans.length > 0 && !selectedHumanId) selectedHumanId = humans[0].id;

  function handleStartDeliberation() {
    if (!session) return;
    if (aiCount === 0) return;

    deliberationStore.clearMessages();
    // In mock mode, this will run a structured multi-phase pipeline; in IPC mode, the backend emits rounds.
    deliberationStore.startDeliberation({ ...session, phase });
  }

  function handleStop() {
    deliberationStore.stopDeliberation(session?.id);
  }

  async function handleSendPrompt() {
    if (!session) return;
    if (!prompt.trim()) return;
    if (!selectedHumanId) return;
    isSending = true;
    try {
      await forgeService.submitMessage(session.id, prompt.trim(), selectedHumanId);
      prompt = '';
    } finally {
      isSending = false;
    }
  }

  function getLatestDraft(): string | null {
    if (!Array.isArray(messages)) return null;
    const candidates = [...messages]
      .filter(m => m.status === 'complete' && (m.type === 'proposal' || m.type === 'synthesis'))
      .reverse();
    return candidates[0]?.content ?? null;
  }

  $: latestDraft = getLatestDraft();

  async function refreshArtifacts() {
    if (!session) return;
    artifactLoading = true;
    artifactError = null;
    try {
      const out = await forgeService.generateOutput({
        sessionId: session.id,
        format: 'markdown',
        includeMetadata: false,
      });
      artifactMarkdown = out.content;
    } catch (e) {
      artifactError = e instanceof Error ? e.message : String(e);
    } finally {
      artifactLoading = false;
    }
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

  const brokenAvatarIds = new Set<string>();

  function parseAvatar(avatar?: string): { src?: string; emoji?: string } {
    if (!avatar) return {};
    if (avatar.startsWith('asset:')) {
      const raw = avatar.slice('asset:'.length);
      const [src, emoji] = raw.split('|');
      return { src, emoji };
    }
    if (avatar.startsWith('/') || avatar.startsWith('http')) {
      return { src: avatar };
    }
    return { emoji: avatar };
  }

  function getParticipantAvatar(message: any): { src?: string; emoji?: string } {
    const byId = session?.participants?.find(p => p.id === message.participantId);
    const byName = session?.participants?.find(
      p => (p.name || '').toLowerCase() === (message.participantName || '').toLowerCase()
    );
    const p = byId || byName;
    return parseAvatar(p?.avatar);
  }

  function markAvatarBroken(id: string) {
    brokenAvatarIds.add(id);
  }
</script>

<div class="deliberation-view">
  <div class="topbar">
    <div class="topbar-left">
      <div class="title">
        <span class="title-kicker">Deliberation</span>
        <span class="title-main">Workspace</span>
      </div>
      <div class="badges">
        {#if useMockMode}
          <span class="chip chip-warn">Mock</span>
        {:else}
          <span class="chip chip-ok">IPC</span>
        {/if}
        <span class="chip chip-run">{phase}</span>
        {#if isRunning}
          <span class="chip chip-live">Running</span>
        {/if}
      </div>
    </div>

    <div class="topbar-right">
      {#if isRunning}
        <button type="button" class="btn btn-danger" on:click={handleStop}>
          <Icon name="square" size={16} />
          Stop
        </button>
      {:else if session && aiCount === 0}
        <button type="button" class="btn btn-secondary" disabled title="Add at least one AI agent to run a round">
          <Icon name="user-plus" size={16} />
          Add AI agents
        </button>
      {:else}
        <button
          type="button"
          class="btn btn-primary"
          disabled={!session || aiCount === 0}
          on:click={handleStartDeliberation}
        >
          <Icon name="zap" size={16} />
          Start deliberation
        </button>
      {/if}
    </div>
  </div>

  {#if error}
    <div class="error-banner" role="alert">
      <Icon name="alert-triangle" size={16} />
      <div class="error-text">
        <div class="error-title">Deliberation error</div>
        <div class="error-msg">{error}</div>
      </div>
    </div>
  {/if}

  <div class="workspace">
    <div class="workspace-header">
      <div class="workspace-tabs" role="tablist" aria-label="Workspace view">
        <button type="button" class="wt" class:active={viewMode === 'feed'} on:click={() => (viewMode = 'feed')}>
          <Icon name="message-circle" size={14} />
          Feed
        </button>
        <button type="button" class="wt" class:active={viewMode === 'artifacts'} on:click={() => (viewMode = 'artifacts')}>
          <Icon name="file-text" size={14} />
          Artifacts
        </button>
      </div>
      <div class="workspace-actions">
        <button type="button" class="btn btn-secondary" on:click={refreshArtifacts} disabled={!session || artifactLoading}>
          {#if artifactLoading}
            <Spinner size={14} color="var(--tachi-cyan)" />
          {:else}
            <Icon name="refresh-cw" size={14} />
          {/if}
          Refresh artifacts
        </button>
      </div>
    </div>

    {#if viewMode === 'feed'}
      <div class="messages-container" bind:this={messagesContainer}>
        {#if messages.length === 0 && isRunning}
          <div class="starting-state" aria-live="polite">
            <Spinner size={18} color="var(--tachi-cyan)" />
            <div class="starting-text">
              <div class="starting-title">Starting round…</div>
              <div class="starting-subtitle">
                {#if useMockMode}
                  Mock mode: simulating events.
                {:else}
                  Waiting for the first streaming event from the backend.
                {/if}
              </div>
            </div>
          </div>
        {:else if messages.length === 0 && session && aiCount === 0}
          <div class="empty-state">
            <div class="empty-icon">
              <Icon name="users" size={48} />
            </div>
            <p>This session has no AI agents.</p>
            <small>Add one or more AI participants to generate drafts.</small>
          </div>
        {:else if messages.length === 0}
          <div class="empty-state">
            <div class="empty-icon">
              <Icon name="message-circle" size={48} />
            </div>
            <p>Click “Start deliberation” to let the agents run.</p>
            <small>Use the prompt box below to add constraints or ask follow-ups.</small>
          </div>
        {:else}
          {#each messages as message (message.id)}
            {@const a = getParticipantAvatar(message)}
            <div
              class="message-card"
              class:streaming={message.status === 'streaming'}
              class:thinking={message.type === 'thinking'}
              data-participant-type={message.participantType}
            >
              <div class="message-header">
                <div class="participant-avatar" class:ai={message.participantType === 'ai'}>
                  {#if a.src && !brokenAvatarIds.has(message.participantId)}
                    <img
                      src={a.src}
                      alt="{message.participantName} avatar"
                      on:error={() => markAvatarBroken(message.participantId)}
                    />
                  {:else if a.emoji}
                    <span class="avatar-emoji" aria-hidden="true">{a.emoji}</span>
                  {:else}
                    {message.participantName.charAt(0).toUpperCase()}
                  {/if}
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
                    <span>Streaming…</span>
                  {:else if message.status === 'complete'}
                    <Icon name="check-circle" size={14} />
                  {:else}
                    <Spinner size={14} color="var(--tachi-cyan)" />
                    <span>Thinking…</span>
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
              <span>Participant is responding…</span>
            </div>
          {/if}
        {/if}
      </div>
    {:else}
      <div class="artifacts-panel">
        {#if artifactError}
          <div class="artifact-error">
            <Icon name="alert-triangle" size={14} />
            <span>{artifactError}</span>
          </div>
        {/if}

        <section class="artifact-card">
          <div class="artifact-head">
            <div class="artifact-title">
              <Icon name="file-text" size={16} />
              <span>Latest draft (from feed)</span>
            </div>
          </div>
          <div class="artifact-body">
            {#if latestDraft}
              {@html renderMarkdown(latestDraft)}
            {:else}
              <div class="artifact-empty">No completed draft yet.</div>
            {/if}
          </div>
        </section>

        <section class="artifact-card">
          <div class="artifact-head">
            <div class="artifact-title">
              <Icon name="package" size={16} />
              <span>Generated output (markdown)</span>
            </div>
          </div>
          <div class="artifact-body">
            {#if artifactMarkdown}
              {@html renderMarkdown(artifactMarkdown)}
            {:else}
              <div class="artifact-empty">Click “Refresh artifacts” to generate output.</div>
            {/if}
          </div>
        </section>
      </div>
    {/if}

    <div class="prompt-bar" role="group" aria-label="Prompt input">
      <div class="prompt-left">
        <div class="prompt-label">
          <Icon name="corner-down-right" size={14} />
          <span>Prompt</span>
        </div>
        {#if humans.length > 0}
          <select class="prompt-select" bind:value={selectedHumanId} aria-label="Select human participant">
            {#each humans as h}
              <option value={h.id}>{h.name}</option>
            {/each}
          </select>
        {:else}
          <span class="prompt-hint">No human participant in this session.</span>
        {/if}
      </div>
      <div class="prompt-right">
        <textarea
          class="prompt-input"
          bind:value={prompt}
          rows="2"
          placeholder="Ask a question, add constraints, or provide feedback…"
        />
        <button
          type="button"
          class="btn btn-primary"
          disabled={!session || !selectedHumanId || !prompt.trim() || isSending}
          on:click={handleSendPrompt}
        >
          {#if isSending}
            <Spinner size={14} color="var(--bg-primary, #0d1117)" />
          {:else}
            <Icon name="send" size={14} />
          {/if}
          Send
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .deliberation-view {
    display: flex;
    flex-direction: column;
    min-height: 560px;
    background: rgba(13, 17, 23, 0.35);
    border: 1px solid rgba(78, 205, 196, 0.18);
    border-radius: 16px;
    overflow: hidden;
  }

  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.9rem 1.1rem;
    background: rgba(78, 205, 196, 0.06);
    border-bottom: 1px solid rgba(78, 205, 196, 0.14);
  }

  .topbar-left {
    display: flex;
    align-items: baseline;
    gap: 1rem;
    min-width: 0;
  }

  .title {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    min-width: 0;
  }

  .title-kicker {
    font-family: var(--font-display, "Orbitron", sans-serif);
    letter-spacing: 1px;
    text-transform: uppercase;
    color: rgba(78, 205, 196, 0.85);
    font-size: 0.85rem;
    font-weight: 700;
  }

  .title-main {
    font-family: var(--font-display, "Orbitron", sans-serif);
    letter-spacing: 1px;
    text-transform: uppercase;
    color: rgba(230, 237, 243, 0.9);
    font-size: 0.85rem;
    font-weight: 600;
    opacity: 0.9;
  }

  .badges {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }

  .chip {
    font-size: 0.65rem;
    font-weight: 600;
    padding: 0.15rem 0.45rem;
    border-radius: 999px;
    border: 1px solid transparent;
    letter-spacing: 0.4px;
    text-transform: uppercase;
  }

  .chip-ok {
    color: rgba(63, 185, 80, 0.95);
    background: rgba(63, 185, 80, 0.12);
    border-color: rgba(63, 185, 80, 0.22);
  }

  .chip-warn {
    color: rgba(243, 156, 18, 0.95);
    background: rgba(243, 156, 18, 0.12);
    border-color: rgba(243, 156, 18, 0.22);
  }

  .chip-run {
    color: rgba(78, 205, 196, 0.95);
    background: rgba(78, 205, 196, 0.12);
    border-color: rgba(78, 205, 196, 0.22);
  }

  .chip-live {
    color: rgba(88, 166, 255, 0.95);
    background: rgba(88, 166, 255, 0.12);
    border-color: rgba(88, 166, 255, 0.22);
  }

  .topbar-right {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
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

  .btn-secondary {
    background: rgba(13, 17, 23, 0.35);
    color: rgba(230, 237, 243, 0.85);
    border-color: rgba(78, 205, 196, 0.18);
  }

  .btn-secondary:hover:enabled {
    background: rgba(78, 205, 196, 0.1);
    border-color: rgba(78, 205, 196, 0.35);
  }

  .btn-danger {
    background: rgba(255, 107, 107, 0.15);
    color: rgba(255, 107, 107, 0.95);
    border-color: rgba(255, 107, 107, 0.35);
  }

  .btn-danger:hover {
    background: rgba(255, 107, 107, 0.25);
  }

  .round-tabs {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid rgba(78, 205, 196, 0.12);
    background: rgba(13, 17, 23, 0.25);
  }

  .tab {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.45rem 0.7rem;
    border-radius: 999px;
    border: 1px solid rgba(78, 205, 196, 0.16);
    background: rgba(13, 17, 23, 0.35);
    color: rgba(230, 237, 243, 0.85);
    cursor: pointer;
    transition: all 0.15s ease;
    font-size: 0.8rem;
  }

  .tab:hover:enabled {
    border-color: rgba(78, 205, 196, 0.35);
    background: rgba(78, 205, 196, 0.08);
  }

  .tab:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tab.active {
    border-color: rgba(78, 205, 196, 0.55);
    background: rgba(78, 205, 196, 0.12);
    color: var(--tachi-cyan, #4ecdc4);
  }

  .workspace {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .workspace-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.65rem 1rem;
    border-bottom: 1px solid rgba(78, 205, 196, 0.12);
    background: rgba(13, 17, 23, 0.22);
  }

  .workspace-tabs {
    display: inline-flex;
    gap: 0.4rem;
  }

  .wt {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.45rem 0.75rem;
    border-radius: 10px;
    border: 1px solid rgba(78, 205, 196, 0.14);
    background: rgba(13, 17, 23, 0.35);
    color: rgba(230, 237, 243, 0.85);
    cursor: pointer;
  }

  .wt.active {
    border-color: rgba(78, 205, 196, 0.5);
    background: rgba(78, 205, 196, 0.12);
    color: var(--tachi-cyan, #4ecdc4);
  }

  .workspace-actions {
    display: inline-flex;
    gap: 0.4rem;
    align-items: center;
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-height: 260px;
  }

  .error-banner {
    display: flex;
    gap: 0.75rem;
    align-items: flex-start;
    padding: 0.75rem 1rem;
    background: rgba(255, 107, 107, 0.12);
    border-bottom: 1px solid rgba(255, 107, 107, 0.25);
    color: rgba(255, 107, 107, 0.95);
  }

  .error-text {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .error-title {
    font-weight: 600;
    font-size: 0.85rem;
  }

  .error-msg {
    font-size: 0.8rem;
    opacity: 0.9;
  }

  .starting-state {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    border-radius: 12px;
    background: rgba(78, 205, 196, 0.06);
    border: 1px solid rgba(78, 205, 196, 0.14);
    color: rgba(230, 237, 243, 0.85);
    min-height: 96px;
  }

  .starting-text {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .starting-title {
    font-weight: 600;
    letter-spacing: 0.2px;
  }

  .starting-subtitle {
    font-size: 0.85rem;
    color: rgba(230, 237, 243, 0.6);
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
    overflow: hidden;
  }

  .participant-avatar.ai {
    background: linear-gradient(135deg, rgba(88, 166, 255, 0.25), rgba(88, 166, 255, 0.08));
    border-color: rgba(88, 166, 255, 0.35);
    color: rgba(88, 166, 255, 0.95);
  }

  .participant-avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar-emoji {
    line-height: 1;
    font-size: 1rem;
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

  .artifacts-panel {
    flex: 1;
    overflow: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  .artifact-card {
    background: rgba(22, 27, 34, 0.55);
    border: 1px solid rgba(78, 205, 196, 0.12);
    border-radius: 12px;
    overflow: hidden;
  }

  .artifact-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.7rem 0.9rem;
    border-bottom: 1px solid rgba(78, 205, 196, 0.1);
    background: rgba(78, 205, 196, 0.04);
  }

  .artifact-title {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--tachi-cyan, #4ecdc4);
    font-weight: 600;
    font-size: 0.8rem;
    letter-spacing: 0.4px;
    text-transform: uppercase;
  }

  .artifact-body {
    padding: 0.9rem 1rem;
    color: rgba(230, 237, 243, 0.9);
    line-height: 1.65;
    font-size: 0.9rem;
  }

  .artifact-empty {
    color: rgba(230, 237, 243, 0.5);
    font-style: italic;
  }

  .artifact-error {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    padding: 0.6rem 0.75rem;
    border: 1px solid rgba(255, 107, 107, 0.2);
    background: rgba(255, 107, 107, 0.08);
    border-radius: 10px;
    color: rgba(255, 107, 107, 0.95);
    font-size: 0.85rem;
  }

  .prompt-bar {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-top: 1px solid rgba(78, 205, 196, 0.14);
    background: rgba(13, 17, 23, 0.55);
  }

  .prompt-left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    justify-content: space-between;
  }

  .prompt-label {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    color: rgba(230, 237, 243, 0.75);
    font-weight: 600;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .prompt-select {
    background: rgba(13, 17, 23, 0.35);
    border: 1px solid rgba(78, 205, 196, 0.18);
    color: rgba(230, 237, 243, 0.85);
    border-radius: 10px;
    padding: 0.35rem 0.6rem;
    font-size: 0.85rem;
  }

  .prompt-hint {
    font-size: 0.8rem;
    color: rgba(230, 237, 243, 0.5);
  }

  .prompt-right {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .prompt-input {
    width: 100%;
    resize: vertical;
    min-height: 56px;
    background: rgba(13, 17, 23, 0.35);
    border: 1px solid rgba(78, 205, 196, 0.18);
    border-radius: 12px;
    padding: 0.6rem 0.75rem;
    color: rgba(230, 237, 243, 0.9);
    font-size: 0.9rem;
    line-height: 1.5;
  }

  @media (max-width: 860px) {
    .prompt-right {
      grid-template-columns: 1fr;
    }
  }
</style>
