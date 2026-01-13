/**
 * Types for spec template functionality including template management,
 * categorization, and customization.
 */

export interface SpecTemplate {
  id: string;
  name: string;
  description: string;
  category: TemplateCategory;
  icon: string;
  tags: string[];
  sections: TemplateSection[];
  frontmatterDefaults: Partial<SpecFrontmatter>;
  contentTemplate: string;
  isBuiltin: boolean;
  isCustom: boolean;
  createdAt?: string;
  usageCount: number;
}

export type TemplateCategory =
  | 'component'
  | 'feature'
  | 'api'
  | 'integration'
  | 'test'
  | 'documentation'
  | 'infrastructure'
  | 'custom';

export interface TemplateSection {
  id: string;
  title: string;
  content: string;
  isOptional: boolean;
}

export interface TemplatePreview {
  template: SpecTemplate;
  renderedContent: string;
}

export interface SpecFrontmatter {
  title: string;
  phase: string;
  specId: string;
  status: 'Planned' | 'In Progress' | 'Review' | 'Complete' | 'Blocked';
  dependencies: string[];
  estimatedContext: string;
}

// Default templates
export const BUILTIN_TEMPLATES: SpecTemplate[] = [
  {
    id: 'component-template',
    name: 'UI Component',
    description: 'Template for creating reusable UI components',
    category: 'component',
    icon: '🧩',
    tags: ['svelte', 'component', 'ui'],
    sections: [
      {
        id: 'objective',
        title: 'Objective',
        content: 'Brief description of the component purpose',
        isOptional: false,
      },
      {
        id: 'criteria',
        title: 'Acceptance Criteria',
        content: 'List of functional requirements',
        isOptional: false,
      },
      {
        id: 'implementation',
        title: 'Implementation Details',
        content: 'Technical implementation details',
        isOptional: false,
      },
      {
        id: 'testing',
        title: 'Testing Requirements',
        content: 'Testing strategy and requirements',
        isOptional: true,
      },
    ],
    frontmatterDefaults: {
      status: 'Planned',
    },
    contentTemplate: `# [ID] - [Component Name]

**Phase:** [Phase Number] - [Phase Name]
**Spec ID:** [ID]
**Status:** Planned
**Dependencies:** 
**Estimated Context:** ~[%] of Sonnet window

---

## Objective

Create a reusable [component name] component for [use case].

---

## Acceptance Criteria

- [ ] Component renders correctly
- [ ] Props are properly typed
- [ ] Events are emitted correctly
- [ ] Accessible markup
- [ ] Responsive design
- [ ] Dark theme support

---

## Implementation Details

### 1. Types (src/lib/types/[name].ts)

\`\`\`typescript
export interface [ComponentName]Props {
  // Properties
}
\`\`\`

### 2. Component (src/lib/components/[path]/[ComponentName].svelte)

\`\`\`svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  export let prop1: string = '';
  
  const dispatch = createEventDispatcher();
</script>

<div class="[component-name]">
  <!-- Template -->
</div>

<style>
  .[component-name] {
    /* Styles */
  }
</style>
\`\`\`

---

## Testing Requirements

1. Component renders without errors
2. Props work as expected
3. Events are dispatched correctly
4. Accessibility tests pass

---`,
    isBuiltin: true,
    isCustom: false,
    usageCount: 0,
  },
  {
    id: 'feature-template',
    name: 'Feature Spec',
    description: 'Template for larger feature implementations',
    category: 'feature',
    icon: '⭐',
    tags: ['feature', 'user-story'],
    sections: [
      {
        id: 'objective',
        title: 'Objective',
        content: 'Feature overview and goals',
        isOptional: false,
      },
      {
        id: 'user-story',
        title: 'User Story',
        content: 'User perspective and requirements',
        isOptional: false,
      },
      {
        id: 'criteria',
        title: 'Acceptance Criteria',
        content: 'Detailed functional requirements',
        isOptional: false,
      },
      {
        id: 'implementation',
        title: 'Implementation Details',
        content: 'Technical implementation approach',
        isOptional: false,
      },
    ],
    frontmatterDefaults: {
      status: 'Planned',
    },
    contentTemplate: `# [ID] - [Feature Name]

**Phase:** [Phase Number] - [Phase Name]
**Spec ID:** [ID]
**Status:** Planned
**Dependencies:** 
**Estimated Context:** ~[%] of Sonnet window

---

## Objective

Implement [feature name] to [achieve goal].

---

## User Story

As a [user type], I want [functionality] so that [benefit].

---

## Acceptance Criteria

- [ ] [Core functionality 1]
- [ ] [Core functionality 2]
- [ ] [Edge case handling]
- [ ] [Error handling]
- [ ] [Performance requirements]
- [ ] [Accessibility requirements]

---

## Implementation Details

### 1. Architecture Overview

[Brief architecture description]

### 2. Components

[List of components to be created/modified]

### 3. Data Flow

[Description of data flow]

### 4. API Changes

[Any API modifications needed]

---

## Testing Requirements

1. Unit tests for all components
2. Integration tests for feature flow
3. E2E tests for user scenarios

---`,
    isBuiltin: true,
    isCustom: false,
    usageCount: 0,
  },
  {
    id: 'api-template',
    name: 'API Endpoint',
    description: 'Template for API endpoint specifications',
    category: 'api',
    icon: '🔌',
    tags: ['api', 'backend', 'endpoint'],
    sections: [
      {
        id: 'objective',
        title: 'Objective',
        content: 'API endpoint purpose',
        isOptional: false,
      },
      {
        id: 'specification',
        title: 'API Specification',
        content: 'Request/response details',
        isOptional: false,
      },
      {
        id: 'implementation',
        title: 'Implementation',
        content: 'Backend implementation details',
        isOptional: false,
      },
      {
        id: 'testing',
        title: 'Testing',
        content: 'API testing strategy',
        isOptional: true,
      },
    ],
    frontmatterDefaults: {
      status: 'Planned',
    },
    contentTemplate: `# [ID] - [Endpoint Name]

**Phase:** [Phase Number] - [Phase Name]
**Spec ID:** [ID]
**Status:** Planned
**Dependencies:** 
**Estimated Context:** ~[%] of Sonnet window

---

## Objective

Create [HTTP method] endpoint for [purpose].

---

## API Specification

### Endpoint
\`[METHOD] /api/[path]\`

### Request
\`\`\`typescript
interface [Request]Body {
  // Request properties
}
\`\`\`

### Response
\`\`\`typescript
interface [Response]Body {
  // Response properties
}
\`\`\`

### Status Codes
- \`200\` - Success
- \`400\` - Bad Request
- \`401\` - Unauthorized
- \`404\` - Not Found
- \`500\` - Server Error

---

## Implementation Details

### 1. Route Handler

\`\`\`rust
// Implementation details
\`\`\`

### 2. Validation

[Input validation requirements]

### 3. Business Logic

[Core logic description]

---

## Testing Requirements

1. Unit tests for handlers
2. Integration tests for full flow
3. Error case testing

---`,
    isBuiltin: true,
    isCustom: false,
    usageCount: 0,
  },
  {
    id: 'test-template',
    name: 'Test Specification',
    description: 'Template for test suite specifications',
    category: 'test',
    icon: '✅',
    tags: ['testing', 'quality', 'automation'],
    sections: [
      {
        id: 'objective',
        title: 'Objective',
        content: 'Testing scope and goals',
        isOptional: false,
      },
      {
        id: 'test-cases',
        title: 'Test Cases',
        content: 'Detailed test scenarios',
        isOptional: false,
      },
      {
        id: 'automation',
        title: 'Test Automation',
        content: 'Automation strategy',
        isOptional: true,
      },
    ],
    frontmatterDefaults: {
      status: 'Planned',
    },
    contentTemplate: `# [ID] - [Test Suite Name]

**Phase:** [Phase Number] - [Phase Name]
**Spec ID:** [ID]
**Status:** Planned
**Dependencies:** 
**Estimated Context:** ~[%] of Sonnet window

---

## Objective

Test [functionality/feature] to ensure [quality criteria].

---

## Test Cases

### Unit Tests
- [ ] [Test case 1]
- [ ] [Test case 2]

### Integration Tests  
- [ ] [Integration scenario 1]
- [ ] [Integration scenario 2]

### E2E Tests
- [ ] [User flow 1]
- [ ] [User flow 2]

### Edge Cases
- [ ] [Edge case 1]
- [ ] [Edge case 2]

---

## Test Automation

### Framework
[Testing framework and tools]

### Implementation
\`\`\`typescript
// Test implementation
\`\`\`

---`,
    isBuiltin: true,
    isCustom: false,
    usageCount: 0,
  },
  {
    id: 'documentation-template',
    name: 'Documentation',
    description: 'Template for documentation specifications',
    category: 'documentation',
    icon: '📄',
    tags: ['docs', 'guide', 'reference'],
    sections: [
      {
        id: 'objective',
        title: 'Objective',
        content: 'Documentation purpose and audience',
        isOptional: false,
      },
      {
        id: 'structure',
        title: 'Content Structure',
        content: 'Documentation outline',
        isOptional: false,
      },
      {
        id: 'maintenance',
        title: 'Maintenance',
        content: 'Update and review process',
        isOptional: true,
      },
    ],
    frontmatterDefaults: {
      status: 'Planned',
    },
    contentTemplate: `# [ID] - [Documentation Title]

**Phase:** [Phase Number] - [Phase Name]
**Spec ID:** [ID]
**Status:** Planned
**Dependencies:** 
**Estimated Context:** ~[%] of Sonnet window

---

## Objective

Create [documentation type] for [audience] covering [topic].

---

## Content Structure

### Sections
1. [Section 1]
2. [Section 2] 
3. [Section 3]

### Key Topics
- [ ] [Topic 1]
- [ ] [Topic 2]
- [ ] [Topic 3]

---

## Implementation Details

### 1. Content Creation

[Writing approach and style]

### 2. Examples

[Code examples and screenshots needed]

### 3. Review Process

[Review and approval workflow]

---`,
    isBuiltin: true,
    isCustom: false,
    usageCount: 0,
  },
];