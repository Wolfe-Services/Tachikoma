export type WarningLevel = 'yellow' | 'orange' | 'red';

export interface RedlineWarning {
  level: WarningLevel;
  contextPercent: number;
  message: string;
  recommendations: Recommendation[];
  canDismiss: boolean;
}

export interface Recommendation {
  id: string;
  title: string;
  description: string;
  action: RedlineAction;
  impact: string;
}

export type RedlineAction =
  | 'create_checkpoint'
  | 'summarize_context'
  | 'reboot_mission'
  | 'reduce_specs'
  | 'switch_model';