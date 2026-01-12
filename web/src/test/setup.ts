import '@testing-library/jest-dom/vitest';
import { vi, beforeEach } from 'vitest';

// Mock window.tachikoma for tests
vi.stubGlobal('window', {
  tachikoma: {
    platform: 'darwin',
    invoke: vi.fn().mockResolvedValue({}),
    on: vi.fn(),
    off: vi.fn()
  }
});

// Reset mocks between tests
beforeEach(() => {
  vi.clearAllMocks();
});