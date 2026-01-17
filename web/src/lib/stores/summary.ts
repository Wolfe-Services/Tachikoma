import { writable, derived } from 'svelte/store';
import type { MissionStatus } from '$lib/ipc/types';

interface MissionStats {
  total: number;
  running: number;
  completed: number;
  failed: number;
  avgDuration: string;
  successRate: number;
}

interface SystemHealthService {
  name: string;
  status: 'healthy' | 'degraded' | 'unhealthy';
  latency: number;
}

interface SystemHealth {
  status: 'healthy' | 'degraded' | 'unhealthy';
  services: SystemHealthService[];
  lastCheck: string;
}

interface ActivityItem {
  id: string;
  type: 'mission_started' | 'mission_completed' | 'mission_failed' | 'spec_created' | 'config_changed';
  message: string;
  timestamp: Date;
}

// Mock data stores for now - these would be populated from real APIs
const mockMissionStats: MissionStats = {
  total: 24,
  running: 2,
  completed: 20,
  failed: 2,
  avgDuration: '4.2m',
  successRate: 83
};

const mockSystemHealth: SystemHealth = {
  status: 'healthy',
  services: [
    { name: 'API Gateway', status: 'healthy', latency: 45 },
    { name: 'Database', status: 'healthy', latency: 12 },
    { name: 'Message Queue', status: 'degraded', latency: 120 },
    { name: 'File Storage', status: 'healthy', latency: 23 }
  ],
  lastCheck: new Date().toLocaleTimeString()
};

const mockRecentActivity: ActivityItem[] = [
  {
    id: '1',
    type: 'mission_completed',
    message: 'Mission "Update README" completed successfully',
    timestamp: new Date(Date.now() - 5 * 60 * 1000) // 5 minutes ago
  },
  {
    id: '2',
    type: 'spec_created',
    message: 'Created new spec: 310-export-reports.md',
    timestamp: new Date(Date.now() - 15 * 60 * 1000) // 15 minutes ago
  },
  {
    id: '3',
    type: 'mission_started',
    message: 'Started mission "Implement summaries"',
    timestamp: new Date(Date.now() - 30 * 60 * 1000) // 30 minutes ago
  },
  {
    id: '4',
    type: 'config_changed',
    message: 'Updated system configuration',
    timestamp: new Date(Date.now() - 45 * 60 * 1000) // 45 minutes ago
  },
  {
    id: '5',
    type: 'mission_failed',
    message: 'Mission "Deploy staging" failed',
    timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000) // 2 hours ago
  }
];

export const missionStats = writable<MissionStats>(mockMissionStats);
export const systemHealth = writable<SystemHealth>(mockSystemHealth);
export const recentActivity = writable<ActivityItem[]>(mockRecentActivity);

// Functions to update the stores (would connect to real APIs)
export function refreshMissionStats() {
  // This would fetch real data from the API
  missionStats.update(stats => ({
    ...stats,
    lastUpdated: new Date()
  }));
}

export function refreshSystemHealth() {
  systemHealth.update(health => ({
    ...health,
    lastCheck: new Date().toLocaleTimeString()
  }));
}

export function addActivity(activity: Omit<ActivityItem, 'id'>) {
  recentActivity.update(activities => [
    { ...activity, id: Date.now().toString() },
    ...activities.slice(0, 19) // Keep last 20 items
  ]);
}