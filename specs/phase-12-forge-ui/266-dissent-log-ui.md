# Spec 266: Dissent Log UI

## Header
- **Spec ID**: 266
- **Phase**: 12 - Forge UI
- **Component**: Dissent Log UI
- **Dependencies**: Spec 265 (Decision Log UI)
- **Status**: Draft

## Objective
Create a dedicated interface for tracking and displaying dissenting opinions from AI participants, ensuring minority viewpoints are recorded, analyzed, and considered in the deliberation process.

## Acceptance Criteria
1. Display all recorded dissents with full attribution
2. Categorize dissents by type and severity
3. Track dissent persistence across rounds
4. Show dissent rationale and supporting arguments
5. Visualize dissent patterns and clustering
6. Enable dissent acknowledgment and response
7. Generate dissent summary reports
8. Link dissents to related decisions and conflicts

## Implementation

### DissentLogUI.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, slide } from 'svelte/transition';
  import DissentCard from './DissentCard.svelte';
  import DissentDetail from './DissentDetail.svelte';
  import DissentChart from './DissentChart.svelte';
  import DissentPatterns from './DissentPatterns.svelte';
  import { dissentLogStore } from '$lib/stores/dissentLog';
  import type {
    Dissent,
    DissentType,
    DissentStatus,
    DissentPattern
  } from '$lib/types/forge';

  export let sessionId: string;

  const dispatch = createEventDispatcher<{
    acknowledge: { dissentId: string; response: string };
    escalate: { dissentId: string };
    link: { dissentId: string; relatedId: string; relationType: string };
  }>();

  let selectedDissentId = writable<string | null>(null);
  let filterType = writable<DissentType | 'all'>('all');
  let filterStatus = writable<DissentStatus | 'all'>('all');
  let filterParticipant = writable<string | 'all'>('all');
  let viewMode = writable<'list' | 'patterns' | 'evolution'>('list');
  let showAcknowledged = writable<boolean>(true);

  const dissents = derived(dissentLogStore, ($store) =>
    $store.dissents.filter(d => d.sessionId === sessionId)
  );

  const filteredDissents = derived(
    [dissents, filterType, filterStatus, filterParticipant, showAcknowledged],
    ([$dissents, $type, $status, $participant, $showAck]) => {
      return $dissents.filter(dissent => {
        if ($type !== 'all' && dissent.type !== $type) return false;
        if ($status !== 'all' && dissent.status !== $status) return false;
        if ($participant !== 'all' && dissent.participantId !== $participant) return false;
        if (!$showAck && dissent.status === 'acknowledged') return false;
        return true;
      });
    }
  );

  const participants = derived(dissents, ($dissents) => {
    const participantMap = new Map<string, string>();
    for (const dissent of $dissents) {
      participantMap.set(dissent.participantId, dissent.participantName);
    }
    return Array.from(participantMap.entries()).map(([id, name]) => ({ id, name }));
  });

  const dissentStats = derived(dissents, ($dissents) => {
    const byParticipant = new Map<string, number>();
    const byType = new Map<DissentType, number>();
    const byRound = new Map<number, number>();

    for (const dissent of $dissents) {
      byParticipant.set(
        dissent.participantId,
        (byParticipant.get(dissent.participantId) || 0) + 1
      );
      byType.set(dissent.type, (byType.get(dissent.type) || 0) + 1);
      byRound.set(dissent.roundNumber, (byRound.get(dissent.roundNumber) || 0) + 1);
    }

    return {
      total: $dissents.length,
      pending: $dissents.filter(d => d.status === 'pending').length,
      acknowledged: $dissents.filter(d => d.status === 'acknowledged').length,
      persistent: $dissents.filter(d => d.persistedRounds && d.persistedRounds > 1).length,
      byParticipant,
      byType,
      byRound
    };
  });

  const dissentPatterns = derived(dissents, ($dissents) => {
    const patterns: DissentPattern[] = [];

    // Group by topic/theme
    const topicGroups = new Map<string, Dissent[]>();
    for (const dissent of $dissents) {
      for (const topic of dissent.topics || []) {
        if (!topicGroups.has(topic)) {
          topicGroups.set(topic, []);
        }
        topicGroups.get(topic)!.push(dissent);
      }
    }

    for (const [topic, topicDissents] of topicGroups) {
      if (topicDissents.length >= 2) {
        patterns.push({
          type: 'topic_cluster',
          topic,
          dissents: topicDissents,
          participants: [...new Set(topicDissents.map(d => d.participantId))],
          significance: topicDissents.length / $dissents.length
        });
      }
    }

    // Identify persistent dissenters
    const participantCounts = new Map<string, number>();
    for (const dissent of $dissents) {
      participantCounts.set(
        dissent.participantId,
        (participantCounts.get(dissent.participantId) || 0) + 1
      );
    }

    for (const [participantId, count] of participantCounts) {
      if (count >= 3) {
        const participantDissents = $dissents.filter(d => d.participantId === participantId);
        patterns.push({
          type: 'persistent_dissenter',
          participantId,
          participantName: participantDissents[0]?.participantName,
          dissents: participantDissents,
          count,
          significance: count / $dissents.length
        });
      }
    }

    return patterns.sort((a, b) => b.significance - a.significance);
  });

  const selectedDissent = derived(
    [dissents, selectedDissentId],
    ([$dissents, $id]) => $dissents.find(d => d.id === $id) || null
  );

  function selectDissent(id: string) {
    selectedDissentId.set(id);
  }

  function handleAcknowledge(dissentId: string, response: string) {
    dispatch('acknowledge', { dissentId, response });
    dissentLogStore.acknowledge(dissentId, response);
  }

  function handleEscalate(dissentId: string) {
    dispatch('escalate', { dissentId });
    dissentLogStore.escalate(dissentId);
  }

  function getSeverityColor(severity: string): string {
    switch (severity) {
      case 'critical': return 'var(--error-color)';
      case 'significant': return 'var(--warning-color)';
      case 'minor': return 'var(--info-color)';
      default: return 'var(--text-muted)';
    }
  }

  onMount(() => {
    dissentLogStore.loadForSession(sessionId);
  });
</script>

<div class="dissent-log-ui" data-testid="dissent-log-ui">
  <header class="log-header">
    <div class="header-title">
      <h3>Dissent Log</h3>
      <span class="dissent-count">{$dissentStats.total} dissents</span>
      {#if $dissentStats.pending > 0}
        <span class="pending-badge">{$dissentStats.pending} pending</span>
      {/if}
    </div>

    <p class="header-description">
      Recording minority viewpoints and alternative perspectives
    </p>
  </header>

  <div class="stats-bar">
    <div class="stat-item">
      <div class="stat-ring" style="--progress: {($dissentStats.acknowledged / $dissentStats.total) * 100}%">
        <span class="stat-value">{$dissentStats.acknowledged}</span>
      </div>
      <span class="stat-label">Acknowledged</span>
    </div>
    <div class="stat-item">
      <div class="stat-ring warning" style="--progress: {($dissentStats.pending / $dissentStats.total) * 100}%">
        <span class="stat-value">{$dissentStats.pending}</span>
      </div>
      <span class="stat-label">Pending</span>
    </div>
    <div class="stat-item">
      <div class="stat-ring info" style="--progress: {($dissentStats.persistent / $dissentStats.total) * 100}%">
        <span class="stat-value">{$dissentStats.persistent}</span>
      </div>
      <span class="stat-label">Persistent</span>
    </div>
    <div class="participant-breakdown">
      <span class="breakdown-label">By Participant:</span>
      <div class="breakdown-bars">
        {#each $participants as participant}
          {@const count = $dissentStats.byParticipant.get(participant.id) || 0}
          <div
            class="breakdown-bar"
            style="width: {(count / $dissentStats.total) * 100}%"
            title="{participant.name}: {count}"
          ></div>
        {/each}
      </div>
    </div>
  </div>

  <div class="toolbar">
    <div class="filters">
      <select bind:value={$filterType} class="filter-select">
        <option value="all">All Types</option>
        <option value="factual">Factual</option>
        <option value="methodological">Methodological</option>
        <option value="ethical">Ethical</option>
        <option value="strategic">Strategic</option>
        <option value="scope">Scope</option>
      </select>

      <select bind:value={$filterStatus} class="filter-select">
        <option value="all">All Status</option>
        <option value="pending">Pending</option>
        <option value="acknowledged">Acknowledged</option>
        <option value="escalated">Escalated</option>
        <option value="resolved">Resolved</option>
      </select>

      <select bind:value={$filterParticipant} class="filter-select">
        <option value="all">All Participants</option>
        {#each $participants as participant}
          <option value={participant.id}>{participant.name}</option>
        {/each}
      </select>

      <label class="show-acknowledged">
        <input type="checkbox" bind:checked={$showAcknowledged} />
        Show acknowledged
      </label>
    </div>

    <div class="view-toggle">
      <button
        class:active={$viewMode === 'list'}
        on:click={() => viewMode.set('list')}
      >
        List
      </button>
      <button
        class:active={$viewMode === 'patterns'}
        on:click={() => viewMode.set('patterns')}
      >
        Patterns
      </button>
      <button
        class:active={$viewMode === 'evolution'}
        on:click={() => viewMode.set('evolution')}
      >
        Evolution
      </button>
    </div>
  </div>

  <div class="content-area">
    {#if $filteredDissents.length === 0}
      <div class="empty-state" transition:fade>
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" stroke-width="2"/>
        </svg>
        <p>No dissents recorded</p>
        <p class="hint">
          {$dissents.length === 0
            ? 'Dissenting opinions will appear here when participants disagree'
            : 'No dissents match your current filters'}
        </p>
      </div>
    {:else if $viewMode === 'list'}
      <div class="dissent-list">
        {#each $filteredDissents as dissent (dissent.id)}
          <DissentCard
            {dissent}
            selected={$selectedDissentId === dissent.id}
            on:click={() => selectDissent(dissent.id)}
            on:acknowledge={(e) => handleAcknowledge(dissent.id, e.detail)}
          />
        {/each}
      </div>
    {:else if $viewMode === 'patterns'}
      <DissentPatterns
        patterns={$dissentPatterns}
        on:selectDissent={(e) => selectDissent(e.detail)}
      />
    {:else if $viewMode === 'evolution'}
      <DissentChart
        dissents={$filteredDissents}
        stats={$dissentStats}
        on:select={(e) => selectDissent(e.detail)}
      />
    {/if}

    {#if $selectedDissent}
      <aside class="dissent-detail" transition:slide={{ axis: 'x' }}>
        <DissentDetail
          dissent={$selectedDissent}
          on:close={() => selectedDissentId.set(null)}
          on:acknowledge={(e) => handleAcknowledge($selectedDissent.id, e.detail)}
          on:escalate={() => handleEscalate($selectedDissent.id)}
        />
      </aside>
    {/if}
  </div>
</div>

<style>
  .dissent-log-ui {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--panel-bg);
    border-radius: 8px;
    overflow: hidden;
  }

  .log-header {
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }

  .header-title h3 {
    font-size: 1rem;
    font-weight: 600;
  }

  .dissent-count {
    padding: 0.25rem 0.5rem;
    background: var(--secondary-bg);
    border-radius: 4px;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .pending-badge {
    padding: 0.25rem 0.5rem;
    background: var(--warning-alpha);
    color: var(--warning-color);
    border-radius: 4px;
    font-size: 0.75rem;
  }

  .header-description {
    font-size: 0.8125rem;
    color: var(--text-muted);
  }

  .stats-bar {
    display: flex;
    gap: 1.5rem;
    align-items: center;
    padding: 1rem 1.25rem;
    background: var(--secondary-bg);
    border-bottom: 1px solid var(--border-color);
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }

  .stat-ring {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: conic-gradient(
      var(--success-color) var(--progress),
      var(--border-color) var(--progress)
    );
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
  }

  .stat-ring::before {
    content: '';
    position: absolute;
    width: 36px;
    height: 36px;
    background: var(--panel-bg);
    border-radius: 50%;
  }

  .stat-ring.warning {
    background: conic-gradient(
      var(--warning-color) var(--progress),
      var(--border-color) var(--progress)
    );
  }

  .stat-ring.info {
    background: conic-gradient(
      var(--info-color) var(--progress),
      var(--border-color) var(--progress)
    );
  }

  .stat-ring .stat-value {
    position: relative;
    z-index: 1;
    font-weight: 600;
    font-size: 0.875rem;
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .participant-breakdown {
    flex: 1;
    margin-left: 1rem;
    padding-left: 1rem;
    border-left: 1px solid var(--border-color);
  }

  .breakdown-label {
    font-size: 0.75rem;
    color: var(--text-muted);
    display: block;
    margin-bottom: 0.5rem;
  }

  .breakdown-bars {
    display: flex;
    height: 8px;
    background: var(--border-color);
    border-radius: 4px;
    overflow: hidden;
  }

  .breakdown-bar {
    height: 100%;
    background: var(--primary-color);
    opacity: 0.8;
  }

  .breakdown-bar:nth-child(2n) {
    background: var(--info-color);
  }

  .breakdown-bar:nth-child(3n) {
    background: var(--warning-color);
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .filters {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .filter-select {
    padding: 0.375rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.8125rem;
  }

  .show-acknowledged {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    cursor: pointer;
    margin-left: 0.5rem;
  }

  .view-toggle {
    display: flex;
    background: var(--secondary-bg);
    border-radius: 4px;
    overflow: hidden;
  }

  .view-toggle button {
    padding: 0.5rem 0.75rem;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 0.75rem;
    cursor: pointer;
  }

  .view-toggle button.active {
    background: var(--primary-color);
    color: white;
  }

  .content-area {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .dissent-list {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .dissent-detail {
    width: 400px;
    border-left: 1px solid var(--border-color);
    background: var(--card-bg);
    overflow-y: auto;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    color: var(--text-muted);
  }

  .empty-state svg {
    margin-bottom: 1rem;
    color: var(--success-color);
  }

  .hint {
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test filtering and statistics calculations
2. **Integration Tests**: Verify dissent logging during deliberation
3. **Pattern Tests**: Validate pattern detection algorithm
4. **Acknowledgment Tests**: Test acknowledgment workflow
5. **Evolution Tests**: Verify round-over-round tracking

## Related Specs
- Spec 265: Decision Log UI
- Spec 263: Critique Viewer
- Spec 267: Convergence Indicator
