/**
 * Types for prompt editor functionality.
 */

export interface PromptVariable {
  name: string;
  value: string;
  description: string;
  source: 'system' | 'user' | 'spec';
}

export interface PromptSnippet {
  id: string;
  trigger: string;
  name: string;
  description: string;
  content: string;
  category: string;
}

export interface PromptEditorState {
  value: string;
  cursorPosition: number;
  selectionStart: number;
  selectionEnd: number;
  history: HistoryEntry[];
  historyIndex: number;
  showPreview: boolean;
  showVariables: boolean;
  showSnippets: boolean;
}

export interface HistoryEntry {
  value: string;
  cursorPosition: number;
  timestamp: number;
}

export interface TokenEstimate {
  inputTokens: number;
  estimatedCost: number;
  modelContext: number;
  usagePercent: number;
}

export const DEFAULT_SNIPPETS: PromptSnippet[] = [
  {
    id: 'implement',
    trigger: '/implement',
    name: 'Implementation Request',
    description: 'Request to implement a feature',
    content: `Implement the following:\n\n**Feature**: \n**Requirements**:\n- \n\n**Constraints**:\n- `,
    category: 'feature',
  },
  {
    id: 'fix',
    trigger: '/fix',
    name: 'Bug Fix Request',
    description: 'Request to fix a bug',
    content: `Fix the following bug:\n\n**Description**: \n**Steps to Reproduce**:\n1. \n\n**Expected Behavior**: \n**Actual Behavior**: `,
    category: 'bugfix',
  },
  {
    id: 'refactor',
    trigger: '/refactor',
    name: 'Refactoring Request',
    description: 'Request to refactor code',
    content: `Refactor the following:\n\n**Target**: \n**Goals**:\n- \n\n**Keep unchanged**:\n- `,
    category: 'refactor',
  },
  {
    id: 'test',
    trigger: '/test',
    name: 'Test Writing Request',
    description: 'Request to write tests',
    content: `Write tests for:\n\n**Target**: \n**Test Types**:\n- [ ] Unit tests\n- [ ] Integration tests\n\n**Coverage Goals**: `,
    category: 'test',
  },
  {
    id: 'review',
    trigger: '/review',
    name: 'Code Review Request',
    description: 'Request for code review',
    content: `Review the following code:\n\n**Focus Areas**:\n- Correctness\n- Performance\n- Security\n- Maintainability\n\n**Code**:\n\`\`\`\n\n\`\`\``,
    category: 'review',
  },
];

export const SYSTEM_VARIABLES: PromptVariable[] = [
  { name: 'project_name', value: '', description: 'Current project name', source: 'system' },
  { name: 'current_date', value: '', description: 'Current date', source: 'system' },
  { name: 'selected_files', value: '', description: 'Currently selected files', source: 'system' },
  { name: 'git_branch', value: '', description: 'Current git branch', source: 'system' },
];