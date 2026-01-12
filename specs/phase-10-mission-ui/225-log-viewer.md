# 225 - Log Viewer Component

**Phase:** 10 - Mission Panel UI
**Spec ID:** 225
**Status:** Planned
**Dependencies:** 216-mission-layout, 217-mission-state
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Create a log viewer component that displays real-time mission execution logs with filtering, search, log level highlighting, and virtual scrolling for performance with large log volumes.

---

## Acceptance Criteria

- [x] Real-time log streaming display
- [x] Log level filtering (trace, debug, info, warn, error)
- [x] Text search within logs
- [x] Virtual scrolling for performance
- [x] Auto-scroll with manual override
- [x] Log entry expansion for details
- [x] Copy log entries to clipboard
- [x] Export logs functionality

---

## Implementation Details

### 1. Types (src/lib/types/log-viewer.ts)

```typescript
/**
 * Types for log viewer functionality.
 */

export type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error';

export interface LogEntry {
  id: string;
  timestamp: string;
  level: LogLevel;
  message: string;
  source: string;
  metadata?: Record<string, unknown>;
  stackTrace?: string;
}

export interface LogFilter {
  levels: LogLevel[];
  search: string;
  source: string | null;
  timeRange: {
    from: string | null;
    to: string | null;
  };
}

export interface LogViewerConfig {
  maxEntries: number;
  autoScroll: boolean;
  showTimestamps: boolean;
  showSource: boolean;
  wrapLines: boolean;
  fontSize: 'small' | 'medium' | 'large';
}

export const LOG_LEVEL_COLORS: Record<LogLevel, string> = {
  trace: '#6B7280',
  debug: '#60A5FA',
  info: '#34D399',
  warn: '#FBBF24',
  error: '#F87171',
};

export const LOG_LEVEL_ICONS: Record<LogLevel, string> = {
  trace: '...',
  debug: 'D',
  info: 'i',
  warn: '!',
  error: 'x',
};

export const DEFAULT_LOG_FILTER: LogFilter = {
  levels: ['info', 'warn', 'error'],
  search: '',
  source: null,
  timeRange: { from: null, to: null },
};

export const DEFAULT_LOG_CONFIG: LogViewerConfig = {
  maxEntries: 10000,
  autoScroll: true,
  showTimestamps: true,
  showSource: false,
  wrapLines: true,
  fontSize: 'medium',
};
```

### 2. Log Store (src/lib/stores/log-store.ts)

```typescript
import { writable, derived } from 'svelte/store';
import type { LogEntry, LogFilter, LogLevel } from '$lib/types/log-viewer';
import { DEFAULT_LOG_FILTER } from '$lib/types/log-viewer';
import { ipcRenderer } from '$lib/ipc';

interface LogStoreState {
  entries: LogEntry[];
  filter: LogFilter;
  isStreaming: boolean;
  missionId: string | null;
}

function createLogStore() {
  const initialState: LogStoreState = {
    entries: [],
    filter: DEFAULT_LOG_FILTER,
    isStreaming: false,
    missionId: null,
  };

  const { subscribe, set, update } = writable<LogStoreState>(initialState);

  // IPC listener for log events
  ipcRenderer.on('mission:log', (_event, entry: LogEntry) => {
    update(state => {
      if (state.missionId && entry.metadata?.missionId === state.missionId) {
        const entries = [...state.entries, entry];
        // Limit entries to prevent memory issues
        if (entries.length > 10000) {
          entries.splice(0, entries.length - 10000);
        }
        return { ...state, entries };
      }
      return state;
    });
  });

  return {
    subscribe,

    startStreaming(missionId: string) {
      update(state => ({
        ...state,
        missionId,
        isStreaming: true,
        entries: [],
      }));
      ipcRenderer.invoke('log:subscribe', missionId);
    },

    stopStreaming() {
      update(state => {
        if (state.missionId) {
          ipcRenderer.invoke('log:unsubscribe', state.missionId);
        }
        return { ...state, isStreaming: false };
      });
    },

    async loadHistory(missionId: string, limit = 1000): Promise<void> {
      const entries = await ipcRenderer.invoke('log:history', { missionId, limit });
      update(state => ({ ...state, entries, missionId }));
    },

    addEntry(entry: LogEntry) {
      update(state => ({
        ...state,
        entries: [...state.entries, entry],
      }));
    },

    clear() {
      update(state => ({ ...state, entries: [] }));
    },

    setFilter(filter: Partial<LogFilter>) {
      update(state => ({
        ...state,
        filter: { ...state.filter, ...filter },
      }));
    },

    toggleLevel(level: LogLevel) {
      update(state => {
        const levels = state.filter.levels.includes(level)
          ? state.filter.levels.filter(l => l !== level)
          : [...state.filter.levels, level];
        return { ...state, filter: { ...state.filter, levels } };
      });
    },

    setSearch(search: string) {
      update(state => ({
        ...state,
        filter: { ...state.filter, search },
      }));
    },

    reset() {
      set(initialState);
    },
  };
}

export const logStore = createLogStore();

export const filteredLogs = derived(logStore, $state => {
  let entries = $state.entries;
  const { levels, search, source, timeRange } = $state.filter;

  // Filter by level
  if (levels.length > 0) {
    entries = entries.filter(e => levels.includes(e.level));
  }

  // Filter by search
  if (search) {
    const searchLower = search.toLowerCase();
    entries = entries.filter(e =>
      e.message.toLowerCase().includes(searchLower) ||
      e.source.toLowerCase().includes(searchLower)
    );
  }

  // Filter by source
  if (source) {
    entries = entries.filter(e => e.source === source);
  }

  // Filter by time range
  if (timeRange.from) {
    entries = entries.filter(e => e.timestamp >= timeRange.from!);
  }
  if (timeRange.to) {
    entries = entries.filter(e => e.timestamp <= timeRange.to!);
  }

  return entries;
});

export const logSources = derived(logStore, $state => {
  const sources = new Set($ state.entries.map(e => e.source));
  return Array.from(sources).sort();
});

export const logStats = derived(logStore, $state => {
  const counts: Record<LogLevel, number> = {
    trace: 0,
    debug: 0,
    info: 0,
    warn: 0,
    error: 0,
  };

  $state.entries.forEach(e => {
    counts[e.level]++;
  });

  return counts;
});
```

### 3. Log Viewer Component (src/lib/components/mission/LogViewer.svelte)

```svelte
<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { logStore, filteredLogs, logStats } from '$lib/stores/log-store';
  import type { LogEntry, LogLevel, LogViewerConfig } from '$lib/types/log-viewer';
  import { LOG_LEVEL_COLORS, LOG_LEVEL_ICONS, DEFAULT_LOG_CONFIG } from '$lib/types/log-viewer';
  import LogEntryRow from './LogEntryRow.svelte';
  import VirtualList from '$lib/components/common/VirtualList.svelte';

  export let missionId: string;
  export let config: LogViewerConfig = DEFAULT_LOG_CONFIG;

  let containerRef: HTMLElement;
  let autoScroll = config.autoScroll;
  let searchInput: HTMLInputElement;
  let expandedEntryId: string | null = null;

  const levels: LogLevel[] = ['trace', 'debug', 'info', 'warn', 'error'];

  function formatTimestamp(timestamp: string): string {
    return new Date(timestamp).toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      fractionalSecondDigits: 3,
    });
  }

  function handleScroll(event: Event) {
    const target = event.target as HTMLElement;
    const atBottom = target.scrollHeight - target.scrollTop - target.clientHeight < 50;
    autoScroll = atBottom;
  }

  async function scrollToBottom() {
    autoScroll = true;
    await tick();
    if (containerRef) {
      containerRef.scrollTop = containerRef.scrollHeight;
    }
  }

  function copyEntry(entry: LogEntry) {
    const text = `[${entry.timestamp}] [${entry.level.toUpperCase()}] ${entry.message}`;
    navigator.clipboard.writeText(text);
  }

  function copyAllFiltered() {
    const text = $filteredLogs
      .map(e => `[${e.timestamp}] [${e.level.toUpperCase()}] ${e.message}`)
      .join('\n');
    navigator.clipboard.writeText(text);
  }

  function exportLogs() {
    const text = $filteredLogs
      .map(e => JSON.stringify(e))
      .join('\n');
    const blob = new Blob([text], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `mission-${missionId}-logs.jsonl`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function handleKeyDown(event: KeyboardEvent) {
    // Focus search: Cmd+F
    if ((event.metaKey || event.ctrlKey) && event.key === 'f') {
      event.preventDefault();
      searchInput?.focus();
    }

    // Clear search: Escape
    if (event.key === 'Escape' && $logStore.filter.search) {
      logStore.setSearch('');
    }

    // Scroll to bottom: Cmd+End
    if ((event.metaKey || event.ctrlKey) && event.key === 'End') {
      event.preventDefault();
      scrollToBottom();
    }
  }

  // Auto-scroll effect
  $: if (autoScroll && $filteredLogs.length > 0) {
    tick().then(() => {
      if (containerRef) {
        containerRef.scrollTop = containerRef.scrollHeight;
      }
    });
  }

  onMount(() => {
    logStore.startStreaming(missionId);
    logStore.loadHistory(missionId);
    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    logStore.stopStreaming();
    window.removeEventListener('keydown', handleKeyDown);
  });
</script>

<div class="log-viewer">
  <!-- Toolbar -->
  <div class="log-viewer__toolbar">
    <!-- Level Filters -->
    <div class="level-filters">
      {#each levels as level}
        <button
          class="level-filter"
          class:level-filter--active={$logStore.filter.levels.includes(level)}
          style="--level-color: {LOG_LEVEL_COLORS[level]}"
          on:click={() => logStore.toggleLevel(level)}
          title="{level} ({$logStats[level]})"
        >
          <span class="level-filter__icon">{LOG_LEVEL_ICONS[level]}</span>
          <span class="level-filter__count">{$logStats[level]}</span>
        </button>
      {/each}
    </div>

    <!-- Search -->
    <div class="log-viewer__search">
      <input
        bind:this={searchInput}
        type="text"
        placeholder="Search logs..."
        value={$logStore.filter.search}
        on:input={(e) => logStore.setSearch(e.currentTarget.value)}
      />
      {#if $logStore.filter.search}
        <button
          class="search-clear"
          on:click={() => logStore.setSearch('')}
        >
          Clear
        </button>
      {/if}
    </div>

    <!-- Actions -->
    <div class="log-viewer__actions">
      <button
        class="action-btn"
        class:action-btn--active={autoScroll}
        on:click={() => { autoScroll = !autoScroll; }}
        title="Auto-scroll"
      >
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <path d="M7 1v10M3 8l4 4 4-4"/>
        </svg>
      </button>

      <button
        class="action-btn"
        on:click={copyAllFiltered}
        title="Copy all"
      >
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <path d="M4 2a1 1 0 00-1 1v7a1 1 0 001 1h6a1 1 0 001-1V3a1 1 0 00-1-1H4zm5 10H3a1 1 0 01-1-1V4h1v7h6v1z"/>
        </svg>
      </button>

      <button
        class="action-btn"
        on:click={exportLogs}
        title="Export logs"
      >
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <path d="M7 1v8M3 6l4 4 4-4M2 12h10"/>
        </svg>
      </button>

      <button
        class="action-btn"
        on:click={() => logStore.clear()}
        title="Clear logs"
      >
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <path d="M3 3h8M5 3V2h4v1M4 5v7h6V5"/>
        </svg>
      </button>
    </div>
  </div>

  <!-- Log Entries -->
  <div
    bind:this={containerRef}
    class="log-viewer__content"
    class:log-viewer__content--wrap={config.wrapLines}
    style="font-size: {config.fontSize === 'small' ? '11px' : config.fontSize === 'large' ? '14px' : '12px'}"
    on:scroll={handleScroll}
  >
    {#if $filteredLogs.length === 0}
      <div class="log-viewer__empty">
        {#if $logStore.entries.length === 0}
          No logs yet. Logs will appear here when the mission starts.
        {:else}
          No logs match the current filters.
        {/if}
      </div>
    {:else}
      <VirtualList items={$filteredLogs} itemHeight={24} let:item>
        <LogEntryRow
          entry={item}
          showTimestamp={config.showTimestamps}
          showSource={config.showSource}
          expanded={expandedEntryId === item.id}
          on:toggle={() => { expandedEntryId = expandedEntryId === item.id ? null : item.id; }}
          on:copy={() => copyEntry(item)}
        />
      </VirtualList>
    {/if}
  </div>

  <!-- Status Bar -->
  <div class="log-viewer__status">
    <span>{$filteredLogs.length} of {$logStore.entries.length} entries</span>
    {#if $logStore.isStreaming}
      <span class="streaming-indicator">
        <span class="streaming-dot"></span>
        Live
      </span>
    {/if}
    {#if !autoScroll}
      <button class="scroll-to-bottom" on:click={scrollToBottom}>
        Scroll to bottom
      </button>
    {/if}
  </div>
</div>

<style>
  .log-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg-primary);
  }

  .log-viewer__toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .level-filters {
    display: flex;
    gap: 4px;
  }

  .level-filter {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 12px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .level-filter:hover {
    background: var(--color-bg-hover);
  }

  .level-filter--active {
    background: var(--color-bg-active);
    color: var(--level-color);
  }

  .level-filter__icon {
    font-weight: 600;
  }

  .level-filter__count {
    opacity: 0.7;
  }

  .log-viewer__search {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .log-viewer__search input {
    flex: 1;
    padding: 6px 10px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 13px;
  }

  .search-clear {
    padding: 4px 8px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 12px;
    cursor: pointer;
  }

  .log-viewer__actions {
    display: flex;
    gap: 4px;
  }

  .action-btn {
    padding: 6px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    border-radius: 4px;
    cursor: pointer;
  }

  .action-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .action-btn--active {
    color: var(--color-primary);
  }

  .log-viewer__content {
    flex: 1;
    overflow-y: auto;
    font-family: 'SF Mono', 'Monaco', 'Consolas', monospace;
  }

  .log-viewer__content--wrap {
    white-space: pre-wrap;
    word-break: break-word;
  }

  .log-viewer__empty {
    padding: 32px;
    text-align: center;
    color: var(--color-text-muted);
    font-size: 13px;
  }

  .log-viewer__status {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .streaming-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--color-success);
  }

  .streaming-dot {
    width: 6px;
    height: 6px;
    background: var(--color-success);
    border-radius: 50%;
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  .scroll-to-bottom {
    margin-left: auto;
    padding: 4px 8px;
    border: none;
    background: var(--color-primary);
    color: white;
    font-size: 11px;
    border-radius: 4px;
    cursor: pointer;
  }
</style>
```

### 4. Log Entry Row Component (src/lib/components/mission/LogEntryRow.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { LogEntry } from '$lib/types/log-viewer';
  import { LOG_LEVEL_COLORS } from '$lib/types/log-viewer';

  export let entry: LogEntry;
  export let showTimestamp = true;
  export let showSource = false;
  export let expanded = false;

  const dispatch = createEventDispatcher<{
    toggle: void;
    copy: void;
  }>();

  function formatTime(timestamp: string): string {
    return new Date(timestamp).toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  }
</script>

<div
  class="log-entry"
  class:log-entry--expanded={expanded}
  class:log-entry--error={entry.level === 'error'}
  class:log-entry--warn={entry.level === 'warn'}
  on:click={() => dispatch('toggle')}
  on:dblclick={() => dispatch('copy')}
>
  {#if showTimestamp}
    <span class="log-entry__time">{formatTime(entry.timestamp)}</span>
  {/if}

  <span
    class="log-entry__level"
    style="color: {LOG_LEVEL_COLORS[entry.level]}"
  >
    [{entry.level.toUpperCase()}]
  </span>

  {#if showSource}
    <span class="log-entry__source">{entry.source}</span>
  {/if}

  <span class="log-entry__message">{entry.message}</span>

  {#if entry.metadata || entry.stackTrace}
    <button
      class="log-entry__expand"
      aria-expanded={expanded}
    >
      {expanded ? '-' : '+'}
    </button>
  {/if}
</div>

{#if expanded && (entry.metadata || entry.stackTrace)}
  <div class="log-entry__details">
    {#if entry.metadata}
      <pre class="log-entry__metadata">{JSON.stringify(entry.metadata, null, 2)}</pre>
    {/if}
    {#if entry.stackTrace}
      <pre class="log-entry__stack">{entry.stackTrace}</pre>
    {/if}
  </div>
{/if}

<style>
  .log-entry {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 2px 12px;
    cursor: pointer;
    transition: background-color 0.1s ease;
  }

  .log-entry:hover {
    background: var(--color-bg-hover);
  }

  .log-entry--expanded {
    background: var(--color-bg-secondary);
  }

  .log-entry--error {
    background: rgba(248, 113, 113, 0.1);
  }

  .log-entry--warn {
    background: rgba(251, 191, 36, 0.05);
  }

  .log-entry__time {
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .log-entry__level {
    flex-shrink: 0;
    font-weight: 500;
  }

  .log-entry__source {
    color: var(--color-text-secondary);
    flex-shrink: 0;
  }

  .log-entry__message {
    flex: 1;
    color: var(--color-text-primary);
  }

  .log-entry__expand {
    padding: 0 4px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-family: monospace;
    cursor: pointer;
  }

  .log-entry__details {
    padding: 8px 12px 8px 80px;
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
  }

  .log-entry__metadata,
  .log-entry__stack {
    margin: 0;
    padding: 8px;
    background: var(--color-bg-primary);
    border-radius: 4px;
    font-size: 11px;
    overflow-x: auto;
  }

  .log-entry__stack {
    color: var(--color-error);
  }
</style>
```

---

## Testing Requirements

1. Logs display in real-time
2. Level filtering works correctly
3. Search filters log entries
4. Virtual scrolling performs well
5. Auto-scroll toggles properly
6. Copy and export work
7. Expanded details show metadata

### Test File (src/lib/components/mission/__tests__/LogViewer.test.ts)

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import LogViewer from '../LogViewer.svelte';
import { logStore } from '$lib/stores/log-store';

vi.mock('$lib/ipc', () => ({
  ipcRenderer: {
    invoke: vi.fn().mockResolvedValue([]),
    on: vi.fn(),
  },
}));

describe('LogViewer', () => {
  it('shows empty state initially', () => {
    render(LogViewer, { missionId: 'test-1' });
    expect(screen.getByText(/No logs yet/)).toBeInTheDocument();
  });

  it('displays log entries', async () => {
    render(LogViewer, { missionId: 'test-1' });

    logStore.addEntry({
      id: '1',
      timestamp: new Date().toISOString(),
      level: 'info',
      message: 'Test log message',
      source: 'test',
    });

    expect(screen.getByText('Test log message')).toBeInTheDocument();
  });

  it('filters by level', async () => {
    render(LogViewer, { missionId: 'test-1' });

    logStore.addEntry({ id: '1', timestamp: new Date().toISOString(), level: 'info', message: 'Info message', source: 'test' });
    logStore.addEntry({ id: '2', timestamp: new Date().toISOString(), level: 'error', message: 'Error message', source: 'test' });

    // Toggle off info level
    const infoFilter = screen.getByTitle(/info/);
    await fireEvent.click(infoFilter);

    expect(screen.queryByText('Info message')).not.toBeInTheDocument();
    expect(screen.getByText('Error message')).toBeInTheDocument();
  });

  it('searches within logs', async () => {
    render(LogViewer, { missionId: 'test-1' });

    logStore.addEntry({ id: '1', timestamp: new Date().toISOString(), level: 'info', message: 'Hello world', source: 'test' });
    logStore.addEntry({ id: '2', timestamp: new Date().toISOString(), level: 'info', message: 'Goodbye world', source: 'test' });

    const searchInput = screen.getByPlaceholderText('Search logs...');
    await fireEvent.input(searchInput, { target: { value: 'Hello' } });

    expect(screen.getByText('Hello world')).toBeInTheDocument();
    expect(screen.queryByText('Goodbye world')).not.toBeInTheDocument();
  });
});
```

---

## Related Specs

- Depends on: [216-mission-layout.md](216-mission-layout.md)
- Depends on: [217-mission-state.md](217-mission-state.md)
- Next: [226-checkpoint-display.md](226-checkpoint-display.md)
- Used by: [216-mission-layout.md](216-mission-layout.md)
