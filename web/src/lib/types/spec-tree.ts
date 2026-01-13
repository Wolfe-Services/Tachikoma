export interface SpecTreeNode {
  id: string;
  type: 'phase' | 'spec';
  label: string;
  number: number;
  status?: SpecStatus;
  children?: SpecTreeNode[];
  isExpanded?: boolean;
  path?: string;
}

export type SpecStatus = 'planned' | 'in_progress' | 'complete' | 'blocked';

export interface TreeState {
  nodes: SpecTreeNode[];
  expandedIds: Set<string>;
  selectedId: string | null;
  focusedId: string | null;
  dragState: DragState | null;
}

export interface DragState {
  nodeId: string;
  targetId: string | null;
  position: 'before' | 'after' | 'inside';
}