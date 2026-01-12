import '@testing-library/jest-dom/vitest';
import { vi, beforeEach } from 'vitest';

// Mock performance.now for Svelte animations
vi.stubGlobal('performance', {
  now: vi.fn(() => Date.now())
});

// Mock window.tachikoma for tests
vi.stubGlobal('window', {
  tachikoma: {
    platform: 'darwin',
    invoke: vi.fn().mockResolvedValue({}),
    on: vi.fn(),
    off: vi.fn()
  },
  performance: {
    now: vi.fn(() => Date.now())
  }
});

// Reset mocks between tests
beforeEach(() => {
  vi.clearAllMocks();
});