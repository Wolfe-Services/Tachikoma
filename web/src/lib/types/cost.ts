export interface CostInfo {
  inputTokens: number;
  outputTokens: number;
  inputCost: number;
  outputCost: number;
  totalCost: number;
  currency: string;
}

export interface CostHistory {
  timestamp: string;
  cost: number;
  tokens: number;
  action: string;
}

export interface CostBudget {
  daily: number;
  weekly: number;
  monthly: number;
  perMission: number;
}

export interface CostProjection {
  estimatedTotal: number;
  remainingBudget: number;
  projectedOverage: number;
  confidence: number;
}