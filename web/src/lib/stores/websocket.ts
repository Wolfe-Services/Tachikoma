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