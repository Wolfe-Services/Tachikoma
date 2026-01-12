import { app, BrowserWindow, ipcMain, webContents } from 'electron';
import { Logger } from '../logger';

const logger = new Logger('performance');

interface PerformanceMetrics {
  timestamp: number;
  cpu: {
    percentCPUUsage: number;
    idleWakeupsPerSecond: number;
  };
  memory: {
    workingSetSize: number;
    peakWorkingSetSize: number;
    privateBytes: number;
  };
  renderer: {
    frameCount: number;
    fps: number;
  };
  heap: {
    totalHeapSize: number;
    usedHeapSize: number;
    heapSizeLimit: number;
  };
}

class PerformanceMonitor {
  private isMonitoring = false;
  private intervalId: NodeJS.Timeout | null = null;
  private metrics: PerformanceMetrics[] = [];
  private maxMetrics = 1000;
  private subscribers: Set<BrowserWindow> = new Set();

  start(intervalMs: number = 1000): void {
    if (this.isMonitoring) {
      return;
    }

    this.isMonitoring = true;
    logger.info('Performance monitoring started');

    this.intervalId = setInterval(() => {
      this.collectMetrics();
    }, intervalMs);
  }

  stop(): void {
    if (!this.isMonitoring) {
      return;
    }

    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }

    this.isMonitoring = false;
    logger.info('Performance monitoring stopped');
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

  private async collectMetrics(): Promise<void> {
    const cpuUsage = process.getCPUUsage();
    const memoryUsage = process.memoryUsage();
    const appMetrics = app.getAppMetrics();

    // Calculate total CPU and memory from all processes
    let totalCPU = 0;
    let totalMemory = 0;

    for (const metric of appMetrics) {
      totalCPU += metric.cpu.percentCPUUsage;
      totalMemory += metric.memory.workingSetSize;
    }

    const metrics: PerformanceMetrics = {
      timestamp: Date.now(),
      cpu: {
        percentCPUUsage: totalCPU,
        idleWakeupsPerSecond: cpuUsage.idleWakeupsPerSecond,
      },
      memory: {
        workingSetSize: totalMemory,
        peakWorkingSetSize: Math.max(
          ...appMetrics.map((m) => m.memory.peakWorkingSetSize)
        ),
        privateBytes: memoryUsage.heapUsed,
      },
      renderer: {
        frameCount: 0,
        fps: 0,
      },
      heap: {
        totalHeapSize: memoryUsage.heapTotal,
        usedHeapSize: memoryUsage.heapUsed,
        heapSizeLimit: 0,
      },
    };

    // Store metrics
    this.metrics.push(metrics);
    if (this.metrics.length > this.maxMetrics) {
      this.metrics.shift();
    }

    // Broadcast to subscribers
    for (const window of this.subscribers) {
      if (!window.isDestroyed()) {
        window.webContents.send('performance:metrics', metrics);
      }
    }
  }

  getMetrics(count?: number): PerformanceMetrics[] {
    if (count) {
      return this.metrics.slice(-count);
    }
    return [...this.metrics];
  }

  getLatestMetrics(): PerformanceMetrics | null {
    return this.metrics.length > 0 ? this.metrics[this.metrics.length - 1] : null;
  }

  clearMetrics(): void {
    this.metrics = [];
  }
}

export const performanceMonitor = new PerformanceMonitor();