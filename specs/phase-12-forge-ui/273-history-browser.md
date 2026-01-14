# Spec 273: History Browser

## Header
- **Spec ID**: 273
- **Phase**: 12 - Forge UI
- **Component**: History Browser
- **Dependencies**: Spec 256 (Forge Layout)
- **Status**: Draft

## Objective
Create a comprehensive history browser for viewing past forge sessions, including search, filtering, session replay, and analytics across historical deliberation data.

## Acceptance Criteria
- [x] Display paginated list of past sessions with key metrics
- [x] Provide advanced search and filtering capabilities
- [x] Support session replay with round-by-round navigation
- [x] Show session comparison features
- [x] Display aggregate analytics and trends
- [x] Enable session archival and restoration
- [x] Export session data in multiple formats
- [x] Track session lineage and relationships

## Implementation

### HistoryBrowser.svelte
```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, fly } from 'svelte/transition';
  import SessionCard from './SessionCard.svelte';
  import SessionDetail from './SessionDetail.svelte';
  import SessionReplay from './SessionReplay.svelte';
  import HistoryFilters from './HistoryFilters.svelte';
  import HistoryAnalytics from './HistoryAnalytics.svelte';
  import SessionComparison from './SessionComparison.svelte';
  import { sessionHistoryStore } from '$lib/stores/sessionHistory';
  import type {
    SessionSummary,
    HistoryFilters as Filters,
    SessionAnalytics
  } from '$lib/types/forge';

  let searchQuery = writable<string>('');
  let filters = writable<Filters>({
    dateRange: { start: null, end: null },
    status: [],
    participants: [],
    minRounds: null,
    maxRounds: null,
    hasResult: null,
    tags: []
  });
  let sortBy = writable<'date' | 'name' | 'rounds' | 'duration'>('date');
  let sortOrder = writable<'asc' | 'desc'>('desc');
  let viewMode = writable<'grid' | 'list' | 'analytics'>('list');
  let selectedSessionId = writable<string | null>(null);
  let comparisonSessionIds = writable<string[]>([]);
  let showReplay = writable<boolean>(false);
  let page = writable<number>(1);
  let pageSize = 20;

  const sessions = derived(sessionHistoryStore, ($store) => $store.sessions);

  const filteredSessions = derived(
    [sessions, searchQuery, filters],
    ([$sessions, $query, $filters]) => {
      return $sessions.filter(session => {
        // Search query
        if ($query) {
          const query = $query.toLowerCase();
          const matchesQuery =
            session.name.toLowerCase().includes(query) ||
            session.goal.toLowerCase().includes(query) ||
            session.participants.some(p => p.name.toLowerCase().includes(query)) ||
            session.tags?.some(t => t.toLowerCase().includes(query));
          if (!matchesQuery) return false;
        }

        // Date range filter
        if ($filters.dateRange.start) {
          const start = new Date($filters.dateRange.start);
          if (new Date(session.createdAt) < start) return false;
        }
        if ($filters.dateRange.end) {
          const end = new Date($filters.dateRange.end);
          if (new Date(session.createdAt) > end) return false;
        }

        // Status filter
        if ($filters.status.length > 0 && !$filters.status.includes(session.status)) {
          return false;
        }

        // Participants filter
        if ($filters.participants.length > 0) {
          const sessionParticipantIds = session.participants.map(p => p.id);
          if (!$filters.participants.some(id => sessionParticipantIds.includes(id))) {
            return false;
          }
        }

        // Rounds filter
        if ($filters.minRounds !== null && session.totalRounds < $filters.minRounds) {
          return false;
        }
        if ($filters.maxRounds !== null && session.totalRounds > $filters.maxRounds) {
          return false;
        }

        // Has result filter
        if ($filters.hasResult !== null && session.hasResult !== $filters.hasResult) {
          return false;
        }

        // Tags filter
        if ($filters.tags.length > 0) {
          if (!$filters.tags.some(tag => session.tags?.includes(tag))) {
            return false;
          }
        }

        return true;
      });
    }
  );

  const sortedSessions = derived(
    [filteredSessions, sortBy, sortOrder],
    ([$sessions, $sortBy, $order]) => {
      const sorted = [...$sessions].sort((a, b) => {
        let comparison = 0;

        switch ($sortBy) {
          case 'date':
            comparison = new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
            break;
          case 'name':
            comparison = a.name.localeCompare(b.name);
            break;
          case 'rounds':
            comparison = a.totalRounds - b.totalRounds;
            break;
          case 'duration':
            comparison = (a.durationMs || 0) - (b.durationMs || 0);
            break;
        }

        return $order === 'asc' ? comparison : -comparison;
      });

      return sorted;
    }
  );

  const paginatedSessions = derived(
    [sortedSessions, page],
    ([$sessions, $page]) => {
      const start = ($page - 1) * pageSize;
      return $sessions.slice(start, start + pageSize);
    }
  );

  const totalPages = derived(sortedSessions, ($sessions) =>
    Math.ceil($sessions.length / pageSize)
  );

  const analytics = derived(filteredSessions, ($sessions) => {
    const total = $sessions.length;
    const completed = $sessions.filter(s => s.status === 'completed').length;
    const avgRounds = total > 0
      ? $sessions.reduce((sum, s) => sum + s.totalRounds, 0) / total
      : 0;
    const avgDuration = total > 0
      ? $sessions.reduce((sum, s) => sum + (s.durationMs || 0), 0) / total
      : 0;
    const withResult = $sessions.filter(s => s.hasResult).length;

    // Sessions over time
    const byMonth = new Map<string, number>();
    for (const session of $sessions) {
      const month = new Date(session.createdAt).toISOString().slice(0, 7);
      byMonth.set(month, (byMonth.get(month) || 0) + 1);
    }

    return {
      total,
      completed,
      completionRate: total > 0 ? completed / total : 0,
      avgRounds,
      avgDuration,
      withResult,
      resultRate: total > 0 ? withResult / total : 0,
      byMonth: Array.from(byMonth.entries()).sort((a, b) => a[0].localeCompare(b[0]))
    };
  });

  const selectedSession = derived(
    [sessions, selectedSessionId],
    ([$sessions, $id]) => $sessions.find(s => s.id === $id) || null
  );

  function selectSession(id: string) {
    selectedSessionId.set(id);
    showReplay.set(false);
  }

  function toggleComparison(id: string) {
    comparisonSessionIds.update(ids => {
      if (ids.includes(id)) {
        return ids.filter(i => i !== id);
      }
      if (ids.length < 3) {
        return [...ids, id];
      }
      return ids;
    });
  }

  function startReplay() {
    if ($selectedSession) {
      showReplay.set(true);
    }
  }

  async function archiveSession(id: string) {
    await sessionHistoryStore.archive(id);
  }

  async function exportSession(id: string, format: string) {
    await sessionHistoryStore.export(id, format);
  }

  function formatDuration(ms: number): string {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);

    if (hours > 0) {
      return `${hours}h ${minutes % 60}m`;
    }
    return `${minutes}m ${seconds % 60}s`;
  }

  onMount(() => {
    sessionHistoryStore.load();
  });
</script>

<div class="history-browser" data-testid="history-browser">
  <header class="browser-header">
    <div class="header-title">
      <h2>Session History</h2>
      <span class="session-count">{$filteredSessions.length} sessions</span>
    </div>

    <div class="header-controls">
      <div class="search-box">
        <input
          type="search"
          placeholder="Search sessions..."
          bind:value={$searchQuery}
          class="search-input"
        />
      </div>

      <div class="view-toggle">
        <button
          class:active={$viewMode === 'list'}
          on:click={() => viewMode.set('list')}
        >
          List
        </button>
        <button
          class:active={$viewMode === 'grid'}
          on:click={() => viewMode.set('grid')}
        >
          Grid
        </button>
        <button
          class:active={$viewMode === 'analytics'}
          on:click={() => viewMode.set('analytics')}
        >
          Analytics
        </button>
      </div>
    </div>
  </header>

  <HistoryFilters
    bind:filters={$filters}
    bind:sortBy={$sortBy}
    bind:sortOrder={$sortOrder}
  />

  {#if $comparisonSessionIds.length > 0}
    <div class="comparison-bar" transition:slide>
      <span>Comparing {$comparisonSessionIds.length} sessions</span>
      <button class="compare-btn" disabled={$comparisonSessionIds.length < 2}>
        Compare
      </button>
      <button class="clear-btn" on:click={() => comparisonSessionIds.set([])}>
        Clear
      </button>
    </div>
  {/if}

  <div class="browser-content">
    {#if $viewMode === 'analytics'}
      <HistoryAnalytics analytics={$analytics} />
    {:else}
      <div class="sessions-panel">
        {#if $paginatedSessions.length === 0}
          <div class="empty-state" transition:fade>
            <p>No sessions found</p>
            <p class="hint">
              {$searchQuery || Object.values($filters).some(v => v && (Array.isArray(v) ? v.length > 0 : true))
                ? 'Try adjusting your search or filters'
                : 'Create a new session to get started'}
            </p>
          </div>
        {:else}
          <div class="session-{$viewMode}">
            {#each $paginatedSessions as session (session.id)}
              <SessionCard
                {session}
                selected={$selectedSessionId === session.id}
                comparing={$comparisonSessionIds.includes(session.id)}
                viewMode={$viewMode}
                on:click={() => selectSession(session.id)}
                on:compare={() => toggleComparison(session.id)}
                on:archive={() => archiveSession(session.id)}
                on:export={(e) => exportSession(session.id, e.detail)}
              />
            {/each}
          </div>

          {#if $totalPages > 1}
            <div class="pagination">
              <button
                class="page-btn"
                disabled={$page === 1}
                on:click={() => page.update(p => p - 1)}
              >
                Previous
              </button>
              <span class="page-info">
                Page {$page} of {$totalPages}
              </span>
              <button
                class="page-btn"
                disabled={$page === $totalPages}
                on:click={() => page.update(p => p + 1)}
              >
                Next
              </button>
            </div>
          {/if}
        {/if}
      </div>

      {#if $selectedSession && !$showReplay}
        <aside class="detail-panel" transition:fly={{ x: 300, duration: 200 }}>
          <SessionDetail
            session={$selectedSession}
            on:close={() => selectedSessionId.set(null)}
            on:replay={startReplay}
            on:export={(e) => exportSession($selectedSession.id, e.detail)}
          />
        </aside>
      {/if}
    {/if}
  </div>

  {#if $showReplay && $selectedSession}
    <div class="replay-overlay" transition:fade>
      <SessionReplay
        session={$selectedSession}
        on:close={() => showReplay.set(false)}
      />
    </div>
  {/if}
</div>

<style>
  .history-browser {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--panel-bg);
  }

  .browser-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.25rem 1.5rem;
    border-bottom: 1px solid var(--border-color);
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .header-title h2 {
    font-size: 1.25rem;
    font-weight: 600;
  }

  .session-count {
    padding: 0.25rem 0.5rem;
    background: var(--secondary-bg);
    border-radius: 4px;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .header-controls {
    display: flex;
    gap: 1rem;
    align-items: center;
  }

  .search-box {
    width: 300px;
  }

  .search-input {
    width: 100%;
    padding: 0.5rem 1rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg);
    color: var(--text-primary);
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
    font-size: 0.8125rem;
    cursor: pointer;
  }

  .view-toggle button.active {
    background: var(--primary-color);
    color: white;
  }

  .comparison-bar {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 1.5rem;
    background: var(--info-alpha);
    border-bottom: 1px solid var(--info-color);
  }

  .compare-btn {
    padding: 0.375rem 0.75rem;
    background: var(--info-color);
    border: none;
    border-radius: 4px;
    color: white;
    font-size: 0.8125rem;
    cursor: pointer;
  }

  .compare-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .clear-btn {
    padding: 0.375rem 0.75rem;
    background: transparent;
    border: 1px solid var(--info-color);
    border-radius: 4px;
    color: var(--info-color);
    font-size: 0.8125rem;
    cursor: pointer;
  }

  .browser-content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .sessions-panel {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
  }

  .session-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .session-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1rem;
  }

  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1rem;
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color);
  }

  .page-btn {
    padding: 0.5rem 1rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    color: var(--text-primary);
    font-size: 0.875rem;
    cursor: pointer;
  }

  .page-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .page-info {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .detail-panel {
    width: 450px;
    border-left: 1px solid var(--border-color);
    background: var(--card-bg);
    overflow-y: auto;
  }

  .empty-state {
    text-align: center;
    padding: 3rem;
    color: var(--text-muted);
  }

  .hint {
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }

  .replay-overlay {
    position: fixed;
    inset: 0;
    background: var(--panel-bg);
    z-index: 100;
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test filtering, sorting, and pagination logic
2. **Integration Tests**: Verify session loading and display
3. **Search Tests**: Test search functionality accuracy
4. **Replay Tests**: Validate session replay navigation
5. **Export Tests**: Test export in all formats

## Related Specs
- Spec 256: Forge Layout
- Spec 274: Comparison View
- Spec 275: Forge UI Tests
