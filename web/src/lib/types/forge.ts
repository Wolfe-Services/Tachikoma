export interface ForgeLayoutConfig {
  leftSidebarWidth: number;
  rightPanelWidth: number;
  leftSidebarVisible: boolean;
  rightPanelVisible: boolean;
  bottomPanelVisible: boolean;
  bottomPanelHeight: number;
}

export interface PanelState {
  hasActiveSession: boolean;
  showParticipants: boolean;
  showResults: boolean;
  sessionPhase: SessionPhase;
}

export type SessionPhase =
  | 'idle'
  | 'configuring'
  | 'drafting'
  | 'critiquing'
  | 'deliberating'
  | 'converging'
  | 'completed'
  | 'paused'
  | 'error';

export interface Participant {
  id: string;
  name: string;
  type: 'human' | 'ai';
  role: string;
  status: 'active' | 'inactive' | 'thinking' | 'contributing';
  avatar?: string;
}

export interface Oracle {
  id: string;
  name: string;
  type: string;
  config: Record<string, any>;
}

export interface Round {
  id: string;
  number: number;
  phase: SessionPhase;
  contributions: Contribution[];
  critiques: Critique[];
  startTime: Date;
  endTime?: Date;
  status: 'active' | 'completed' | 'paused';
}

export interface Contribution {
  id: string;
  participantId: string;
  content: string;
  timestamp: Date;
  type: 'proposal' | 'refinement' | 'alternative';
}

export interface Critique {
  id: string;
  participantId: string;
  targetId: string; // contribution ID
  content: string;
  timestamp: Date;
  severity: 'info' | 'suggestion' | 'concern' | 'critical';
}

export interface ForgeSession {
  id: string;
  name: string;
  goal: string;
  phase: SessionPhase;
  participants: Participant[];
  oracle: Oracle | null;
  rounds: Round[];
  hasResults: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface ForgeSessionState {
  activeSession: ForgeSession | null;
  sessions: ForgeSession[];
  loading: boolean;
  error: string | null;
}