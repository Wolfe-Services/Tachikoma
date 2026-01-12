# 314 - Realtime Updates

**Phase:** 14 - Dashboard
**Spec ID:** 314
**Status:** Planned
**Dependencies:** 296-dashboard-layout, 329-websocket-setup
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create real-time update infrastructure for the dashboard using WebSocket connections, enabling live data streaming, instant notifications, and automatic UI updates.

---

## Acceptance Criteria

- [ ] WebSocket connection management
- [ ] Auto-reconnection with backoff
- [ ] Message type handling
- [ ] Store subscription updates
- [ ] Connection status indicator
- [ ] Notification system integration
- [ ] Offline queue for messages
- [ ] Heartbeat/ping-pong

---

## Implementation Details

### 1. WebSocket Store (web/src/lib/stores/websocket.ts)

```typescript
import { writable, derived, get } from 'svelte/store';

export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'reconnecting';

export interface WebSocketMessage {
  type: string;
  payload: any;
  timestamp: string;
}

interface WebSocketState {
  status: ConnectionStatus;
  reconnectAttempts: number;
  lastMessage: WebSocketMessage | null;
  error: string | null;
}

function createWebSocketStore() {
  const { subscribe, set, update } = writable<WebSocketState>({
    status: 'disconnected',
    reconnectAttempts: 0,
    lastMessage: null,
    error: null
  });

  let socket: WebSocket | null = null;
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  let heartbeatTimer: ReturnType<typeof setInterval> | null = null;
  let messageQueue: WebSocketMessage[] = [];
  const messageHandlers = new Map<string, Set<(payload: any) => void>>();

  const maxReconnectAttempts = 10;
  const baseReconnectDelay = 1000;
  const maxReconnectDelay = 30000;
  const heartbeatInterval = 30000;

  function getReconnectDelay(attempts: number): number {
    const delay = baseReconnectDelay * Math.pow(2, attempts);
    return Math.min(delay, maxReconnectDelay);
  }

  function connect(url: string) {
    if (socket?.readyState === WebSocket.OPEN) {
      return;
    }

    update(s => ({ ...s, status: 'connecting', error: null }));

    try {
      socket = new WebSocket(url);

      socket.onopen = () => {
        update(s => ({ ...s, status: 'connected', reconnectAttempts: 0 }));
        startHeartbeat();
        flushMessageQueue();
      };

      socket.onclose = (event) => {
        update(s => ({ ...s, status: 'disconnected' }));
        stopHeartbeat();

        if (!event.wasClean) {
          scheduleReconnect(url);
        }
      };

      socket.onerror = (error) => {
        update(s => ({ ...s, error: 'WebSocket error occurred' }));
      };

      socket.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          handleMessage(message);
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e);
        }
      };
    } catch (error) {
      update(s => ({
        ...s,
        status: 'disconnected',
        error: 'Failed to connect'
      }));
      scheduleReconnect(url);
    }
  }

  function disconnect() {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
    stopHeartbeat();

    if (socket) {
      socket.close(1000, 'Client disconnect');
      socket = null;
    }

    update(s => ({
      ...s,
      status: 'disconnected',
      reconnectAttempts: 0
    }));
  }

  function scheduleReconnect(url: string) {
    const state = get({ subscribe });

    if (state.reconnectAttempts >= maxReconnectAttempts) {
      update(s => ({ ...s, error: 'Max reconnection attempts reached' }));
      return;
    }

    update(s => ({
      ...s,
      status: 'reconnecting',
      reconnectAttempts: s.reconnectAttempts + 1
    }));

    const delay = getReconnectDelay(state.reconnectAttempts);

    reconnectTimer = setTimeout(() => {
      connect(url);
    }, delay);
  }

  function startHeartbeat() {
    stopHeartbeat();
    heartbeatTimer = setInterval(() => {
      if (socket?.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify({ type: 'ping' }));
      }
    }, heartbeatInterval);
  }

  function stopHeartbeat() {
    if (heartbeatTimer) {
      clearInterval(heartbeatTimer);
      heartbeatTimer = null;
    }
  }

  function handleMessage(message: WebSocketMessage) {
    update(s => ({ ...s, lastMessage: message }));

    if (message.type === 'pong') {
      return; // Heartbeat response
    }

    const handlers = messageHandlers.get(message.type);
    if (handlers) {
      handlers.forEach(handler => handler(message.payload));
    }

    // Broadcast to wildcard handlers
    const wildcardHandlers = messageHandlers.get('*');
    if (wildcardHandlers) {
      wildcardHandlers.forEach(handler => handler(message));
    }
  }

  function send(message: WebSocketMessage) {
    if (socket?.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(message));
    } else {
      messageQueue.push(message);
    }
  }

  function flushMessageQueue() {
    while (messageQueue.length > 0 && socket?.readyState === WebSocket.OPEN) {
      const message = messageQueue.shift()!;
      socket.send(JSON.stringify(message));
    }
  }

  function onMessage(type: string, handler: (payload: any) => void) {
    if (!messageHandlers.has(type)) {
      messageHandlers.set(type, new Set());
    }
    messageHandlers.get(type)!.add(handler);

    return () => {
      messageHandlers.get(type)?.delete(handler);
    };
  }

  return {
    subscribe,
    connect,
    disconnect,
    send,
    onMessage
  };
}

export const wsStore = createWebSocketStore();

export const connectionStatus = derived(
  wsStore,
  $ws => $ws.status
);

export const isConnected = derived(
  wsStore,
  $ws => $ws.status === 'connected'
);
```

### 2. Realtime Update Hook (web/src/lib/hooks/useRealtime.ts)

```typescript
import { onMount, onDestroy } from 'svelte';
import { wsStore } from '$lib/stores/websocket';
import type { Writable } from 'svelte/store';

interface RealtimeOptions<T> {
  store: Writable<T>;
  messageType: string;
  transform?: (payload: any, current: T) => T;
  onError?: (error: any) => void;
}

export function useRealtime<T>(options: RealtimeOptions<T>) {
  const { store, messageType, transform, onError } = options;

  let unsubscribe: (() => void) | null = null;

  onMount(() => {
    unsubscribe = wsStore.onMessage(messageType, (payload) => {
      try {
        store.update(current => {
          if (transform) {
            return transform(payload, current);
          }
          return payload as T;
        });
      } catch (error) {
        onError?.(error);
      }
    });
  });

  onDestroy(() => {
    unsubscribe?.();
  });
}

// Specialized hooks for common data types

export function useRealtimeMissions() {
  return useRealtime({
    store: missionStore,
    messageType: 'mission_update',
    transform: (payload, missions) => {
      const index = missions.findIndex(m => m.id === payload.id);
      if (index >= 0) {
        return [...missions.slice(0, index), payload, ...missions.slice(index + 1)];
      }
      return [payload, ...missions];
    }
  });
}

export function useRealtimeMetrics() {
  return useRealtime({
    store: metricsStore,
    messageType: 'metrics_update',
    transform: (payload, metrics) => ({
      ...metrics,
      ...payload
    })
  });
}
```

### 3. Connection Status Component (web/src/lib/components/common/ConnectionStatus.svelte)

```svelte
<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import { connectionStatus, wsStore } from '$lib/stores/websocket';
  import Icon from '$lib/components/common/Icon.svelte';

  export let showDetails: boolean = false;
  export let position: 'inline' | 'fixed' = 'inline';

  $: status = $connectionStatus;
  $: statusConfig = getStatusConfig(status);

  function getStatusConfig(s: typeof status) {
    switch (s) {
      case 'connected':
        return { icon: 'wifi', color: 'var(--green-500)', label: 'Connected' };
      case 'connecting':
        return { icon: 'loader', color: 'var(--blue-500)', label: 'Connecting...' };
      case 'reconnecting':
        return { icon: 'refresh-cw', color: 'var(--yellow-500)', label: 'Reconnecting...' };
      case 'disconnected':
        return { icon: 'wifi-off', color: 'var(--red-500)', label: 'Disconnected' };
      default:
        return { icon: 'help-circle', color: 'var(--gray-500)', label: 'Unknown' };
    }
  }

  function handleReconnect() {
    // Trigger reconnection
    wsStore.connect(import.meta.env.VITE_WS_URL);
  }
</script>

<div
  class="connection-status {position}"
  class:disconnected={status === 'disconnected'}
  title={statusConfig.label}
>
  <span class="status-indicator" style="color: {statusConfig.color}">
    <Icon
      name={statusConfig.icon}
      size={14}
      class={status === 'connecting' || status === 'reconnecting' ? 'spinning' : ''}
    />
  </span>

  {#if showDetails}
    <span class="status-label">{statusConfig.label}</span>
  {/if}

  {#if status === 'disconnected'}
    <button class="reconnect-btn" on:click={handleReconnect}>
      Reconnect
    </button>
  {/if}
</div>

{#if status === 'disconnected' && position === 'fixed'}
  <div class="offline-banner" transition:fly={{ y: -20, duration: 200 }}>
    <Icon name="wifi-off" size={16} />
    <span>You're offline. Some features may be unavailable.</span>
    <button on:click={handleReconnect}>
      Try Again
    </button>
  </div>
{/if}

<style>
  .connection-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .connection-status.fixed {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    padding: 0.5rem 0.75rem;
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.5rem;
    box-shadow: var(--shadow-md);
    z-index: 1000;
  }

  .status-indicator {
    display: flex;
    align-items: center;
  }

  :global(.status-indicator .spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .status-label {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .reconnect-btn {
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--border-color);
    background: transparent;
    border-radius: 0.25rem;
    font-size: 0.6875rem;
    color: var(--text-primary);
    cursor: pointer;
  }

  .reconnect-btn:hover {
    background: var(--bg-hover);
  }

  .offline-banner {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--red-500);
    color: white;
    font-size: 0.875rem;
    z-index: 1001;
  }

  .offline-banner button {
    padding: 0.25rem 0.75rem;
    border: 1px solid rgba(255, 255, 255, 0.3);
    background: transparent;
    border-radius: 0.375rem;
    font-size: 0.75rem;
    color: white;
    cursor: pointer;
  }

  .offline-banner button:hover {
    background: rgba(255, 255, 255, 0.1);
  }
</style>
```

### 4. Realtime Provider Component (web/src/lib/components/providers/RealtimeProvider.svelte)

```svelte
<script lang="ts">
  import { onMount, onDestroy, setContext } from 'svelte';
  import { wsStore, connectionStatus } from '$lib/stores/websocket';

  export let url: string = import.meta.env.VITE_WS_URL || 'ws://localhost:3001/ws';
  export let autoConnect: boolean = true;

  setContext('realtime', {
    send: wsStore.send,
    onMessage: wsStore.onMessage,
    status: connectionStatus
  });

  onMount(() => {
    if (autoConnect) {
      wsStore.connect(url);
    }
  });

  onDestroy(() => {
    wsStore.disconnect();
  });
</script>

<slot />
```

---

## Testing Requirements

1. WebSocket connects successfully
2. Auto-reconnection works with backoff
3. Messages dispatch to handlers
4. Stores update on message receipt
5. Heartbeat keeps connection alive
6. Offline queue sends on reconnect
7. Status indicator reflects state

---

## Related Specs

- Depends on: [296-dashboard-layout.md](296-dashboard-layout.md)
- Related: [329-websocket-setup.md](../phase-15-server/329-websocket-setup.md)
- Next: [315-dashboard-tests.md](315-dashboard-tests.md)
