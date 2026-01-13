export interface ImplementationPlan {
  specId: string;
  title: string;
  sections: PlanSection[];
  progress: PlanProgress;
}

export interface PlanSection {
  id: string;
  title: string;
  items: PlanItem[];
}

export interface PlanItem {
  id: string;
  text: string;
  completed: boolean;
  lineNumber: number;
  subItems?: PlanItem[];
}

export interface PlanProgress {
  completed: number;
  total: number;
  percentage: number;
}