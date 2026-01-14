export interface DeploymentData {
  id: string;
  version: string;
  environment: string;
  status: 'success' | 'failed' | 'rolled_back' | 'in_progress';
  duration: number;
  timestamp: string;
  deployedBy: string;
  commitSha?: string;
  rollbackOf?: string;
}

export interface PipelineStage {
  name: string;
  avgDuration: number;
  successRate: number;
}

export interface DeploymentSummary {
  total: number;
  successful: number;
  failed: number;
  rollbacks: number;
  change: number;
  meanDeployTime: number;
  deployTimeTrend: number[];
  pipelineStages: PipelineStage[];
}