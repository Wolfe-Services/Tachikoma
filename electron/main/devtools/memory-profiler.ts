import { BrowserWindow, webContents } from 'electron';
import { Logger } from '../logger';

const logger = new Logger('memory-profiler');

interface MemoryProfile {
  timestamp: number;
  heap: {
    total: number;
    used: number;
    limit: number;
    external: number;
  };
  process: {
    rss: number;
    heapTotal: number;
    heapUsed: number;
    external: number;
    arrayBuffers: number;
  };
  gc?: {
    type: string;
    before: number;
    after: number;
    duration: number;
  };
}

interface MemoryLeak {
  id: string;
  detected: number;
  growthRate: number; // bytes per second
  suspectedCause: string;
  stackTrace?: string;
}

class MemoryProfiler {
  private isEnabled = false;
  private profiles: MemoryProfile[] = [];
  private maxProfiles = 500;
  private subscribers: Set<BrowserWindow> = new Set();
  private intervalId: NodeJS.Timeout | null = null;
  private leakDetectionThreshold = 10 * 1024 * 1024; // 10MB
  private detectedLeaks: MemoryLeak[] = [];

  enable(intervalMs: number = 5000): void {
    if (this.isEnabled) return;

    this.isEnabled = true;
    this.startProfiling(intervalMs);
    logger.info('Memory profiling enabled');
  }

  disable(): void {
    if (!this.isEnabled) return;

    this.isEnabled = false;
    this.stopProfiling();
    logger.info('Memory profiling disabled');
  }

  subscribe(window: BrowserWindow): void {
    this.subscribers.add(window);

    window.on('closed', () => {
      this.subscribers.delete(window);
    });
  }

  unsubscribe(window: BrowserWindow): void {
    this.subscribers.delete(window);
  }

  private startProfiling(intervalMs: number): void {
    this.intervalId = setInterval(() => {
      this.takeSnapshot();
    }, intervalMs);
  }

  private stopProfiling(): void {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  private takeSnapshot(): void {
    const memoryUsage = process.memoryUsage();
    
    const profile: MemoryProfile = {
      timestamp: Date.now(),
      heap: {
        total: memoryUsage.heapTotal,
        used: memoryUsage.heapUsed,
        limit: 0, // Not directly available in Node.js
        external: memoryUsage.external,
      },
      process: {
        rss: memoryUsage.rss,
        heapTotal: memoryUsage.heapTotal,
        heapUsed: memoryUsage.heapUsed,
        external: memoryUsage.external,
        arrayBuffers: memoryUsage.arrayBuffers,
      },
    };

    this.profiles.push(profile);
    if (this.profiles.length > this.maxProfiles) {
      this.profiles.shift();
    }

    // Check for memory leaks
    this.checkForLeaks();

    // Broadcast to subscribers
    for (const window of this.subscribers) {
      if (!window.isDestroyed()) {
        window.webContents.send('memory:profile', profile);
      }
    }
  }

  private checkForLeaks(): void {
    if (this.profiles.length < 10) return; // Need enough data points

    const recent = this.profiles.slice(-10);
    const oldest = recent[0];
    const newest = recent[recent.length - 1];
    
    const timeDiff = newest.timestamp - oldest.timestamp;
    const memoryDiff = newest.heap.used - oldest.heap.used;
    const growthRate = (memoryDiff / timeDiff) * 1000; // bytes per second

    if (growthRate > this.leakDetectionThreshold / 60) { // 10MB per minute
      const leakId = `leak_${Date.now()}`;
      const leak: MemoryLeak = {
        id: leakId,
        detected: Date.now(),
        growthRate,
        suspectedCause: this.analyzeLeak(recent),
      };

      this.detectedLeaks.push(leak);
      logger.warn('Memory leak detected', {
        growthRate: `${(growthRate / 1024 / 1024).toFixed(2)} MB/s`,
        suspectedCause: leak.suspectedCause,
      });

      // Broadcast leak detection
      for (const window of this.subscribers) {
        if (!window.isDestroyed()) {
          window.webContents.send('memory:leak-detected', leak);
        }
      }
    }
  }

  private analyzeLeak(profiles: MemoryProfile[]): string {
    // Simple heuristic analysis
    const heapGrowth = profiles[profiles.length - 1].heap.used - profiles[0].heap.used;
    const externalGrowth = profiles[profiles.length - 1].heap.external - profiles[0].heap.external;
    
    if (externalGrowth > heapGrowth) {
      return 'External memory (likely native modules or buffers)';
    } else {
      return 'JavaScript heap (likely object retention)';
    }
  }

  forceGC(): void {
    if (global.gc) {
      const before = process.memoryUsage().heapUsed;
      const startTime = Date.now();
      
      global.gc();
      
      const after = process.memoryUsage().heapUsed;
      const duration = Date.now() - startTime;
      
      logger.info('Manual garbage collection', {
        before: `${(before / 1024 / 1024).toFixed(2)} MB`,
        after: `${(after / 1024 / 1024).toFixed(2)} MB`,
        freed: `${((before - after) / 1024 / 1024).toFixed(2)} MB`,
        duration: `${duration}ms`,
      });

      // Add GC info to latest profile
      if (this.profiles.length > 0) {
        this.profiles[this.profiles.length - 1].gc = {
          type: 'manual',
          before,
          after,
          duration,
        };
      }
    } else {
      logger.warn('Garbage collection not available (not started with --expose-gc)');
    }
  }

  getProfiles(count?: number): MemoryProfile[] {
    if (count) {
      return this.profiles.slice(-count);
    }
    return [...this.profiles];
  }

  getLatestProfile(): MemoryProfile | null {
    return this.profiles.length > 0 ? this.profiles[this.profiles.length - 1] : null;
  }

  getDetectedLeaks(): MemoryLeak[] {
    return [...this.detectedLeaks];
  }

  clearProfiles(): void {
    this.profiles = [];
    this.detectedLeaks = [];
    logger.debug('Memory profiles cleared');
  }

  generateReport(): string {
    const latest = this.getLatestProfile();
    const leaks = this.getDetectedLeaks();
    
    let report = '# Memory Profile Report\n\n';
    
    if (latest) {
      report += `## Current Memory Usage\n`;
      report += `- RSS: ${(latest.process.rss / 1024 / 1024).toFixed(2)} MB\n`;
      report += `- Heap Used: ${(latest.heap.used / 1024 / 1024).toFixed(2)} MB\n`;
      report += `- Heap Total: ${(latest.heap.total / 1024 / 1024).toFixed(2)} MB\n`;
      report += `- External: ${(latest.heap.external / 1024 / 1024).toFixed(2)} MB\n\n`;
    }

    if (leaks.length > 0) {
      report += `## Detected Memory Leaks\n`;
      for (const leak of leaks) {
        report += `- **${leak.id}**: ${(leak.growthRate / 1024).toFixed(2)} KB/s (${leak.suspectedCause})\n`;
      }
      report += '\n';
    }

    report += `## Profile History\n`;
    report += `- Total snapshots: ${this.profiles.length}\n`;
    if (this.profiles.length > 1) {
      const oldest = this.profiles[0];
      const newest = this.profiles[this.profiles.length - 1];
      const growth = newest.heap.used - oldest.heap.used;
      report += `- Memory growth: ${(growth / 1024 / 1024).toFixed(2)} MB\n`;
    }

    return report;
  }
}

export const memoryProfiler = new MemoryProfiler();