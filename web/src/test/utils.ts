import { render, type RenderResult } from '@testing-library/svelte';
import type { ComponentProps, SvelteComponent } from 'svelte';
import { vi } from 'vitest';

/**
 * Render a Svelte component with default test providers.
 */
export function renderWithProviders<T extends SvelteComponent>(
  component: new (...args: any[]) => T,
  props?: ComponentProps<T>
): RenderResult<T> {
  return render(component, { props });
}

/**
 * Wait for next tick.
 */
export function tick(): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, 0));
}

/**
 * Mock IPC invoke response.
 */
export function mockIpcInvoke(channel: string, response: unknown): void {
  (window.tachikoma.invoke as ReturnType<typeof vi.fn>).mockImplementation(
    async (ch: string) => {
      if (ch === channel) return response;
      return {};
    }
  );
}