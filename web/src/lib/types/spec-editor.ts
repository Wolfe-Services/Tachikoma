/**
 * Types for spec editor functionality with markdown syntax highlighting,
 * auto-completion, and formatting features.
 */

export interface EditorConfig {
  lineNumbers: boolean;
  wordWrap: boolean;
  tabSize: number;
  autoSave: boolean;
  autoSaveDelay: number;
  minimap: boolean;
}

export interface EditorState {
  content: string;
  cursorPosition: { line: number; column: number };
  selection: { start: Position; end: Position } | null;
  history: HistoryEntry[];
  historyIndex: number;
}

export interface Position {
  line: number;
  column: number;
}

export interface HistoryEntry {
  content: string;
  cursor: Position;
  timestamp: number;
}

export interface CompletionItem {
  label: string;
  kind: 'spec' | 'section' | 'template';
  detail: string;
  insertText: string;
}

export interface FindReplaceOptions {
  query: string;
  replacement: string;
  caseSensitive: boolean;
  wholeWord: boolean;
  useRegex: boolean;
}

export interface FindMatch {
  line: number;
  column: number;
  length: number;
  text: string;
}

export interface SpecTemplate {
  id: string;
  name: string;
  description: string;
  trigger: string;
  content: string;
  category: 'structure' | 'formatting' | 'common';
}

export const DEFAULT_EDITOR_CONFIG: EditorConfig = {
  lineNumbers: true,
  wordWrap: true,
  tabSize: 2,
  autoSave: true,
  autoSaveDelay: 2000,
  minimap: false,
};

export const SPEC_TEMPLATES: SpecTemplate[] = [
  {
    id: 'spec-header',
    name: 'Spec Header',
    description: 'Standard spec frontmatter',
    trigger: '/header',
    content: `# [ID] - [Title]

**Phase:** [Phase Number] - [Phase Name]
**Spec ID:** [ID]
**Status:** Planned
**Dependencies:** 
**Estimated Context:** ~[%] of Sonnet window

---

## Objective

[Brief description of what this spec implements]

---

## Acceptance Criteria

- [ ] [Criterion 1]
- [ ] [Criterion 2]

---`,
    category: 'structure',
  },
  {
    id: 'acceptance-criteria',
    name: 'Acceptance Criteria',
    description: 'Acceptance criteria section',
    trigger: '/criteria',
    content: `## Acceptance Criteria

- [ ] [Criterion 1]
- [ ] [Criterion 2]
- [ ] [Criterion 3]`,
    category: 'structure',
  },
  {
    id: 'implementation',
    name: 'Implementation Details',
    description: 'Implementation section with code blocks',
    trigger: '/impl',
    content: `## Implementation Details

### 1. Types (src/lib/types/[name].ts)

\`\`\`typescript
export interface [Name] {
  // Properties
}
\`\`\`

### 2. Component (src/lib/components/[path]/[Component].svelte)

\`\`\`svelte
<script lang="ts">
  // Implementation
</script>

<div class="[component-name]">
  <!-- Template -->
</div>

<style>
  .[component-name] {
    /* Styles */
  }
</style>
\`\`\``,
    category: 'structure',
  },
  {
    id: 'code-block',
    name: 'Code Block',
    description: 'Fenced code block',
    trigger: '/code',
    content: `\`\`\`typescript
// Code here
\`\`\``,
    category: 'formatting',
  },
  {
    id: 'task-list',
    name: 'Task List',
    description: 'Checkbox task list',
    trigger: '/tasks',
    content: `- [ ] Task 1
- [ ] Task 2
- [ ] Task 3`,
    category: 'formatting',
  },
  {
    id: 'table',
    name: 'Table',
    description: 'Markdown table',
    trigger: '/table',
    content: `| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Value 1  | Value 2  | Value 3  |
| Value 4  | Value 5  | Value 6  |`,
    category: 'formatting',
  },
];