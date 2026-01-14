import { onMount, onDestroy } from 'svelte';
import { wsStore } from '$lib/stores/websocket';
import { missionStore } from '$lib/stores/mission';
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
    transform: (payload, current) => {
      // Update the current mission status
      if (payload.id === current.current?.id) {
        return {
          ...current,
          current: payload
        };
      }
      return current;
    }
  });
}

// Create a simple metrics store for demonstration
import { writable } from 'svelte/store';

interface MetricsData {
  success_rate?: number;
  error_rate?: number;
  response_time?: number;
  active_missions?: number;
  [key: string]: any;
}

const metricsStore = writable<MetricsData>({});

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

export { metricsStore };