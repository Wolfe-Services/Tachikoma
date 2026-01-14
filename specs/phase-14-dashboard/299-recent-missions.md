# 299 - Recent Missions

**Phase:** 14 - Dashboard
**Spec ID:** 299
**Status:** Planned
**Dependencies:** 297-mission-cards
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Create a recent missions component that displays a chronologically ordered list of recently executed missions with quick status overview, filtering, and navigation capabilities.

---

## Acceptance Criteria

- [x] `RecentMissions.svelte` component created
- [x] Chronological mission list
- [x] Status filtering (all, success, failed, etc.)
- [x] Pagination or infinite scroll
- [x] Quick actions (view, rerun, archive)
- [x] Time-based grouping (today, yesterday, this week)
- [x] Search within recent missions
- [x] Empty state handling

---

## Implementation Details

### 1. Recent Missions Component (web/src/lib/components/missions/RecentMissions.svelte)

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { fly, fade } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import type { Mission, MissionState } from '$lib/types/mission';
  import { recentMissionsStore, loadMoreMissions } from '$lib/stores/missions';
  import Icon from '$lib/components/common/Icon.svelte';
  import MissionListItem from './MissionListItem.svelte';
  import FilterPills from '$lib/components/common/FilterPills.svelte';
  import SearchInput from '$lib/components/common/SearchInput.svelte';

  export let limit: number = 20;
  export let showHeader: boolean = true;
  export let showSearch: boolean = true;

  let searchQuery = '';
  let selectedFilter: MissionState | 'all' = 'all';
  let loading = false;
  let hasMore = true;

  const filterOptions: Array<{ value: MissionState | 'all'; label: string; icon: string }> = [
    { value: 'all', label: 'All', icon: 'layers' },
    { value: 'complete', label: 'Completed', icon: 'check-circle' },
    { value: 'error', label: 'Failed', icon: 'x-circle' },
    { value: 'running', label: 'Running', icon: 'play' },
    { value: 'paused', label: 'Paused', icon: 'pause' }
  ];

  $: filteredMissions = filterMissions($recentMissionsStore, selectedFilter, searchQuery);
  $: groupedMissions = groupByDate(filteredMissions);

  function filterMissions(
    missions: Mission[],
    filter: MissionState | 'all',
    query: string
  ): Mission[] {
    let result = missions;

    if (filter !== 'all') {
      result = result.filter(m => m.state === filter);
    }

    if (query.trim()) {
      const lowerQuery = query.toLowerCase();
      result = result.filter(m =>
        m.title.toLowerCase().includes(lowerQuery) ||
        m.specId.toLowerCase().includes(lowerQuery)
      );
    }

    return result;
  }

  function groupByDate(missions: Mission[]): Map<string, Mission[]> {
    const groups = new Map<string, Mission[]>();
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const yesterday = new Date(today.getTime() - 86400000);
    const weekAgo = new Date(today.getTime() - 604800000);

    for (const mission of missions) {
      const date = new Date(mission.completedAt || mission.updatedAt);
      let group: string;

      if (date >= today) {
        group = 'Today';
      } else if (date >= yesterday) {
        group = 'Yesterday';
      } else if (date >= weekAgo) {
        group = 'This Week';
      } else {
        group = 'Older';
      }

      if (!groups.has(group)) {
        groups.set(group, []);
      }
      groups.get(group)!.push(mission);
    }

    return groups;
  }

  async function handleLoadMore() {
    if (loading || !hasMore) return;
    loading = true;

    try {
      const loaded = await loadMoreMissions(limit);
      hasMore = loaded === limit;
    } finally {
      loading = false;
    }
  }

  function handleAction(event: CustomEvent<{ action: string; missionId: string }>) {
    const { action, missionId } = event.detail;

    switch (action) {
      case 'view':
        window.location.href = `/missions/${missionId}`;
        break;
      case 'rerun':
        // Trigger rerun flow
        break;
      case 'archive':
        // Archive mission
        break;
    }
  }
</script>

<section class="recent-missions">
  {#if showHeader}
    <header class="section-header">
      <h2 class="section-title">
        <Icon name="history" size={20} />
        Recent Missions
      </h2>

      <a href="/missions" class="view-all-link">
        View All
        <Icon name="arrow-right" size={14} />
      </a>
    </header>
  {/if}

  <div class="filters-row">
    {#if showSearch}
      <SearchInput
        bind:value={searchQuery}
        placeholder="Search missions..."
      />
    {/if}

    <FilterPills
      options={filterOptions}
      bind:selected={selectedFilter}
    />
  </div>

  <div class="missions-container">
    {#if filteredMissions.length === 0}
      <div class="empty-state" in:fade>
        {#if searchQuery || selectedFilter !== 'all'}
          <Icon name="search-x" size={48} />
          <p>No missions match your filters</p>
          <button class="clear-filters" on:click={() => { searchQuery = ''; selectedFilter = 'all'; }}>
            Clear Filters
          </button>
        {:else}
          <Icon name="inbox" size={48} />
          <p>No recent missions</p>
          <a href="/missions/new" class="start-mission">Start a Mission</a>
        {/if}
      </div>
    {:else}
      {#each [...groupedMissions] as [groupName, missions] (groupName)}
        <div class="mission-group" in:fly={{ y: 10, duration: 200 }}>
          <h3 class="group-header">{groupName}</h3>

          <ul class="mission-list">
            {#each missions as mission (mission.id)}
              <li animate:flip={{ duration: 200 }}>
                <MissionListItem
                  {mission}
                  on:action={handleAction}
                />
              </li>
            {/each}
          </ul>
        </div>
      {/each}

      {#if hasMore}
        <div class="load-more">
          <button
            class="load-more-btn"
            on:click={handleLoadMore}
            disabled={loading}
          >
            {#if loading}
              <Icon name="loader" size={16} class="spinning" />
              Loading...
            {:else}
              Load More
            {/if}
          </button>
        </div>
      {/if}
    {/if}
  </div>
</section>

<style>
  .recent-missions {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.75rem;
    overflow: hidden;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .view-all-link {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.875rem;
    color: var(--accent-color);
    text-decoration: none;
  }

  .view-all-link:hover {
    text-decoration: underline;
  }

  .filters-row {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  .missions-container {
    max-height: 600px;
    overflow-y: auto;
  }

  .mission-group {
    padding: 0.5rem 0;
  }

  .group-header {
    padding: 0.5rem 1.25rem;
    margin: 0;
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-tertiary);
  }

  .mission-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 3rem;
    color: var(--text-tertiary);
    text-align: center;
  }

  .empty-state p {
    margin: 0;
    font-size: 0.9375rem;
  }

  .clear-filters,
  .start-mission {
    padding: 0.5rem 1rem;
    border: 1px solid var(--border-color);
    background: var(--bg-primary);
    border-radius: 0.5rem;
    font-size: 0.875rem;
    color: var(--text-primary);
    text-decoration: none;
    cursor: pointer;
  }

  .clear-filters:hover,
  .start-mission:hover {
    background: var(--bg-hover);
  }

  .load-more {
    padding: 1rem;
    text-align: center;
  }

  .load-more-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1.25rem;
    border: 1px solid var(--border-color);
    background: var(--bg-primary);
    border-radius: 0.5rem;
    font-size: 0.875rem;
    color: var(--text-primary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .load-more-btn:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .load-more-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
```

### 2. Mission List Item Component (web/src/lib/components/missions/MissionListItem.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Mission } from '$lib/types/mission';
  import Icon from '$lib/components/common/Icon.svelte';
  import RelativeTime from '$lib/components/common/RelativeTime.svelte';
  import ContextMenu from '$lib/components/common/ContextMenu.svelte';

  export let mission: Mission;

  const dispatch = createEventDispatcher<{
    action: { action: string; missionId: string };
  }>();

  let showContextMenu = false;
  let contextMenuPosition = { x: 0, y: 0 };

  const stateIcons: Record<string, string> = {
    idle: 'circle',
    running: 'play',
    paused: 'pause',
    complete: 'check-circle',
    error: 'x-circle',
    redlined: 'alert-triangle'
  };

  const stateColors: Record<string, string> = {
    idle: 'var(--gray-500)',
    running: 'var(--blue-500)',
    paused: 'var(--yellow-500)',
    complete: 'var(--green-500)',
    error: 'var(--red-500)',
    redlined: 'var(--orange-500)'
  };

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    contextMenuPosition = { x: e.clientX, y: e.clientY };
    showContextMenu = true;
  }

  function handleAction(action: string) {
    dispatch('action', { action, missionId: mission.id });
    showContextMenu = false;
  }
</script>

<article
  class="mission-item"
  on:contextmenu={handleContextMenu}
  role="listitem"
>
  <div class="item-icon" style="color: {stateColors[mission.state]}">
    <Icon name={stateIcons[mission.state]} size={18} />
  </div>

  <div class="item-content">
    <a href="/missions/{mission.id}" class="item-title">
      {mission.title}
    </a>
    <div class="item-meta">
      <span class="spec-id">{mission.specId}</span>
      <span class="separator">-</span>
      <RelativeTime date={mission.completedAt || mission.updatedAt} />
    </div>
  </div>

  <div class="item-stats">
    <span class="stat" title="Token usage">
      <Icon name="coins" size={14} />
      {(mission.tokenUsage.total / 1000).toFixed(1)}k
    </span>
    {#if mission.state === 'complete'}
      <span class="stat success" title="Completed successfully">
        <Icon name="check" size={14} />
      </span>
    {:else if mission.state === 'error'}
      <span class="stat error" title="Failed">
        <Icon name="x" size={14} />
      </span>
    {/if}
  </div>

  <div class="item-actions">
    <button
      class="action-btn"
      on:click={() => handleAction('view')}
      title="View details"
    >
      <Icon name="eye" size={16} />
    </button>
    <button
      class="action-btn"
      on:click={() => handleAction('rerun')}
      title="Rerun mission"
    >
      <Icon name="refresh-cw" size={16} />
    </button>
    <button
      class="action-btn"
      on:click|stopPropagation={() => showContextMenu = true}
      title="More actions"
    >
      <Icon name="more-vertical" size={16} />
    </button>
  </div>
</article>

{#if showContextMenu}
  <ContextMenu
    position={contextMenuPosition}
    on:close={() => showContextMenu = false}
  >
    <button on:click={() => handleAction('view')}>
      <Icon name="eye" size={14} />
      View Details
    </button>
    <button on:click={() => handleAction('rerun')}>
      <Icon name="refresh-cw" size={14} />
      Rerun Mission
    </button>
    <button on:click={() => handleAction('duplicate')}>
      <Icon name="copy" size={14} />
      Duplicate
    </button>
    <hr />
    <button on:click={() => handleAction('archive')} class="danger">
      <Icon name="archive" size={14} />
      Archive
    </button>
  </ContextMenu>
{/if}

<style>
  .mission-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1.25rem;
    transition: background 0.15s ease;
  }

  .mission-item:hover {
    background: var(--bg-hover);
  }

  .item-icon {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .item-content {
    flex: 1;
    min-width: 0;
  }

  .item-title {
    display: block;
    font-weight: 500;
    color: var(--text-primary);
    text-decoration: none;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-title:hover {
    color: var(--accent-color);
  }

  .item-meta {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    margin-top: 0.125rem;
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .separator {
    opacity: 0.5;
  }

  .item-stats {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .stat {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .stat.success {
    color: var(--green-500);
  }

  .stat.error {
    color: var(--red-500);
  }

  .item-actions {
    display: flex;
    gap: 0.25rem;
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .mission-item:hover .item-actions {
    opacity: 1;
  }

  .action-btn {
    padding: 0.375rem;
    border: none;
    background: transparent;
    border-radius: 0.375rem;
    cursor: pointer;
    color: var(--text-secondary);
  }

  .action-btn:hover {
    background: var(--bg-secondary);
    color: var(--text-primary);
  }
</style>
```

---

## Testing Requirements

1. Missions display in correct chronological order
2. Filtering works for all states
3. Search filters by title and spec ID
4. Date grouping categorizes correctly
5. Load more fetches additional missions
6. Context menu actions dispatch correctly
7. Empty states render appropriately

---

## Related Specs

- Depends on: [297-mission-cards.md](297-mission-cards.md)
- Next: [300-cost-summary.md](300-cost-summary.md)
- Used by: Dashboard overview, mission history view
