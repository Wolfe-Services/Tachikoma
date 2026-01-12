# Spec 240: Spec Templates

## Phase
11 - Spec Browser UI

## Spec ID
240

## Status
Planned

## Dependencies
- Spec 237 (Spec Editor)
- Spec 241 (Spec Creation Form)

## Estimated Context
~8%

---

## Objective

Create a template system for specs that provides pre-built templates for common spec types, supports custom user templates, and enables template variables for dynamic content generation.

---

## Acceptance Criteria

- [ ] Provide built-in templates (UI component, API endpoint, etc.)
- [ ] Support custom user-created templates
- [ ] Template variables with placeholder syntax
- [ ] Template preview before applying
- [ ] Template categories and search
- [ ] Import/export templates
- [ ] Template versioning
- [ ] Share templates between users

---

## Implementation Details

### TemplateSelector.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import type { SpecTemplate, TemplateCategory, TemplateVariable } from '$lib/types/spec';
  import Icon from '$lib/components/Icon.svelte';
  import Button from '$lib/components/Button.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import MarkdownPreview from './MarkdownPreview.svelte';
  import { builtInTemplates } from '$lib/data/spec-templates';

  export let open = false;

  const dispatch = createEventDispatcher<{
    select: { template: SpecTemplate; variables: Record<string, string> };
    close: void;
  }>();

  let customTemplates = writable<SpecTemplate[]>([]);
  let searchQuery = '';
  let selectedCategory: TemplateCategory | 'all' = 'all';
  let selectedTemplate: SpecTemplate | null = null;
  let variableValues: Record<string, string> = {};
  let showPreview = false;

  // Load custom templates from localStorage
  $: {
    const stored = localStorage.getItem('custom-spec-templates');
    if (stored) {
      try {
        customTemplates.set(JSON.parse(stored));
      } catch {
        customTemplates.set([]);
      }
    }
  }

  // Combine built-in and custom templates
  const allTemplates = derived(customTemplates, $custom => [
    ...builtInTemplates,
    ...$custom
  ]);

  // Filter templates
  $: filteredTemplates = $allTemplates.filter(t => {
    const matchesSearch = searchQuery === '' ||
      t.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      t.description.toLowerCase().includes(searchQuery.toLowerCase());

    const matchesCategory = selectedCategory === 'all' ||
      t.category === selectedCategory;

    return matchesSearch && matchesCategory;
  });

  // Get unique categories
  $: categories = Array.from(new Set($allTemplates.map(t => t.category)));

  // Extract variables from template
  function extractVariables(template: SpecTemplate): TemplateVariable[] {
    const regex = /\{\{(\w+)(?:\|([^}]+))?\}\}/g;
    const variables: TemplateVariable[] = [];
    const seen = new Set<string>();

    let match;
    const content = template.content + template.title;

    while ((match = regex.exec(content)) !== null) {
      const name = match[1];
      const defaultValue = match[2] || '';

      if (!seen.has(name)) {
        seen.add(name);
        variables.push({
          name,
          label: formatVariableName(name),
          defaultValue,
          required: !defaultValue
        });
      }
    }

    return variables;
  }

  function formatVariableName(name: string): string {
    return name
      .replace(/([A-Z])/g, ' $1')
      .replace(/_/g, ' ')
      .replace(/^\w/, c => c.toUpperCase())
      .trim();
  }

  // Apply variables to template content
  function applyVariables(content: string, variables: Record<string, string>): string {
    return content.replace(/\{\{(\w+)(?:\|([^}]+))?\}\}/g, (_, name, defaultVal) => {
      return variables[name] || defaultVal || `{{${name}}}`;
    });
  }

  function handleSelectTemplate(template: SpecTemplate) {
    selectedTemplate = template;
    const vars = extractVariables(template);

    // Initialize variable values with defaults
    variableValues = {};
    vars.forEach(v => {
      variableValues[v.name] = v.defaultValue;
    });
  }

  function handleApply() {
    if (!selectedTemplate) return;

    dispatch('select', {
      template: selectedTemplate,
      variables: variableValues
    });

    handleClose();
  }

  function handleClose() {
    selectedTemplate = null;
    variableValues = {};
    showPreview = false;
    dispatch('close');
  }

  function saveAsCustomTemplate(template: SpecTemplate) {
    const customTemplate: SpecTemplate = {
      ...template,
      id: `custom-${Date.now()}`,
      isCustom: true
    };

    customTemplates.update(templates => {
      const updated = [...templates, customTemplate];
      localStorage.setItem('custom-spec-templates', JSON.stringify(updated));
      return updated;
    });
  }

  function deleteCustomTemplate(id: string) {
    customTemplates.update(templates => {
      const updated = templates.filter(t => t.id !== id);
      localStorage.setItem('custom-spec-templates', JSON.stringify(updated));
      return updated;
    });
  }

  $: previewContent = selectedTemplate
    ? applyVariables(selectedTemplate.content, variableValues)
    : '';

  $: templateVariables = selectedTemplate
    ? extractVariables(selectedTemplate)
    : [];
</script>

<Modal {open} on:close={handleClose} size="xl" title="Select Template">
  <div class="template-selector">
    <aside class="template-selector__sidebar">
      <div class="template-selector__search">
        <Icon name="search" size={16} />
        <input
          type="text"
          placeholder="Search templates..."
          bind:value={searchQuery}
        />
      </div>

      <nav class="template-selector__categories">
        <button
          class="template-selector__category"
          class:template-selector__category--active={selectedCategory === 'all'}
          on:click={() => selectedCategory = 'all'}
        >
          <Icon name="grid" size={16} />
          All Templates
        </button>
        {#each categories as category}
          <button
            class="template-selector__category"
            class:template-selector__category--active={selectedCategory === category}
            on:click={() => selectedCategory = category}
          >
            <Icon name={getCategoryIcon(category)} size={16} />
            {category}
          </button>
        {/each}
      </nav>
    </aside>

    <div class="template-selector__main">
      {#if selectedTemplate}
        <div class="template-selector__detail">
          <header class="template-selector__detail-header">
            <button
              class="template-selector__back"
              on:click={() => selectedTemplate = null}
            >
              <Icon name="arrow-left" size={16} />
              Back to templates
            </button>
          </header>

          <div class="template-selector__detail-content">
            <div class="template-selector__info">
              <h2>{selectedTemplate.name}</h2>
              <p>{selectedTemplate.description}</p>

              {#if templateVariables.length > 0}
                <div class="template-selector__variables">
                  <h3>Template Variables</h3>
                  {#each templateVariables as variable}
                    <div class="template-selector__variable">
                      <label for="var-{variable.name}">
                        {variable.label}
                        {#if variable.required}
                          <span class="required">*</span>
                        {/if}
                      </label>
                      <input
                        id="var-{variable.name}"
                        type="text"
                        bind:value={variableValues[variable.name]}
                        placeholder={variable.defaultValue || 'Enter value...'}
                      />
                    </div>
                  {/each}
                </div>
              {/if}
            </div>

            <div class="template-selector__preview">
              <div class="template-selector__preview-header">
                <span>Preview</span>
                <Button
                  variant="ghost"
                  size="sm"
                  on:click={() => showPreview = !showPreview}
                >
                  {showPreview ? 'Show Source' : 'Show Rendered'}
                </Button>
              </div>
              <div class="template-selector__preview-content">
                {#if showPreview}
                  <MarkdownPreview content={previewContent} />
                {:else}
                  <pre><code>{previewContent}</code></pre>
                {/if}
              </div>
            </div>
          </div>

          <footer class="template-selector__detail-footer">
            <Button variant="outline" on:click={() => selectedTemplate = null}>
              Cancel
            </Button>
            <Button variant="primary" on:click={handleApply}>
              <Icon name="check" size={16} />
              Use Template
            </Button>
          </footer>
        </div>
      {:else}
        <div class="template-selector__grid">
          {#each filteredTemplates as template}
            <button
              class="template-selector__card"
              on:click={() => handleSelectTemplate(template)}
            >
              <div class="template-selector__card-icon">
                <Icon name={template.icon || 'file-text'} size={24} />
              </div>
              <h3>{template.name}</h3>
              <p>{template.description}</p>
              <div class="template-selector__card-meta">
                <span class="template-selector__card-category">
                  {template.category}
                </span>
                {#if template.isCustom}
                  <button
                    class="template-selector__delete"
                    on:click|stopPropagation={() => deleteCustomTemplate(template.id)}
                    aria-label="Delete template"
                  >
                    <Icon name="trash" size={14} />
                  </button>
                {/if}
              </div>
            </button>
          {/each}

          {#if filteredTemplates.length === 0}
            <div class="template-selector__empty">
              <Icon name="search" size={32} />
              <p>No templates found</p>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</Modal>

<script context="module" lang="ts">
  function getCategoryIcon(category: string): string {
    const icons: Record<string, string> = {
      'UI Component': 'layout',
      'API Endpoint': 'server',
      'Feature': 'star',
      'Bug Fix': 'bug',
      'Refactoring': 'refresh-cw',
      'Documentation': 'book',
      'Testing': 'flask',
      'Infrastructure': 'cloud'
    };
    return icons[category] || 'file-text';
  }
</script>

<style>
  .template-selector {
    display: flex;
    height: 600px;
  }

  .template-selector__sidebar {
    width: 240px;
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    background: var(--color-surface-subtle);
  }

  .template-selector__search {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px;
    border-bottom: 1px solid var(--color-border);
  }

  .template-selector__search input {
    flex: 1;
    padding: 8px;
    border: none;
    background: none;
    font-size: 0.875rem;
  }

  .template-selector__search input:focus {
    outline: none;
  }

  .template-selector__categories {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .template-selector__category {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 10px 12px;
    text-align: left;
    background: none;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    transition: all 0.15s;
  }

  .template-selector__category:hover {
    background: var(--color-hover);
    color: var(--color-text-primary);
  }

  .template-selector__category--active {
    background: var(--color-primary-subtle);
    color: var(--color-primary);
    font-weight: 500;
  }

  .template-selector__main {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .template-selector__grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 16px;
    padding: 20px;
    overflow-y: auto;
  }

  .template-selector__card {
    display: flex;
    flex-direction: column;
    padding: 20px;
    text-align: left;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .template-selector__card:hover {
    border-color: var(--color-primary);
    box-shadow: var(--shadow-md);
  }

  .template-selector__card-icon {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-primary-subtle);
    color: var(--color-primary);
    border-radius: 8px;
    margin-bottom: 12px;
  }

  .template-selector__card h3 {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 8px;
  }

  .template-selector__card p {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    margin: 0 0 12px;
    flex: 1;
  }

  .template-selector__card-meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .template-selector__card-category {
    font-size: 0.75rem;
    padding: 4px 8px;
    background: var(--color-surface-elevated);
    border-radius: 4px;
    color: var(--color-text-tertiary);
  }

  .template-selector__delete {
    padding: 4px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-tertiary);
    border-radius: 4px;
  }

  .template-selector__delete:hover {
    background: var(--color-danger-subtle);
    color: var(--color-danger);
  }

  .template-selector__empty {
    grid-column: 1 / -1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px;
    color: var(--color-text-tertiary);
  }

  .template-selector__detail {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .template-selector__detail-header {
    padding: 12px 20px;
    border-bottom: 1px solid var(--color-border);
  }

  .template-selector__back {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    border-radius: 6px;
  }

  .template-selector__back:hover {
    background: var(--color-hover);
    color: var(--color-text-primary);
  }

  .template-selector__detail-content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .template-selector__info {
    width: 300px;
    padding: 20px;
    border-right: 1px solid var(--color-border);
    overflow-y: auto;
  }

  .template-selector__info h2 {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0 0 8px;
  }

  .template-selector__info > p {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    margin: 0 0 20px;
  }

  .template-selector__variables h3 {
    font-size: 0.875rem;
    font-weight: 600;
    margin: 0 0 12px;
    color: var(--color-text-secondary);
  }

  .template-selector__variable {
    margin-bottom: 12px;
  }

  .template-selector__variable label {
    display: block;
    font-size: 0.75rem;
    font-weight: 500;
    margin-bottom: 4px;
    color: var(--color-text-secondary);
  }

  .template-selector__variable .required {
    color: var(--color-danger);
  }

  .template-selector__variable input {
    width: 100%;
    padding: 8px 12px;
    font-size: 0.875rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
  }

  .template-selector__preview {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .template-selector__preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    background: var(--color-surface-subtle);
    border-bottom: 1px solid var(--color-border);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .template-selector__preview-content {
    flex: 1;
    padding: 16px;
    overflow-y: auto;
  }

  .template-selector__preview-content pre {
    margin: 0;
    white-space: pre-wrap;
    font-family: var(--font-mono);
    font-size: 0.875rem;
    line-height: 1.6;
  }

  .template-selector__detail-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 16px 20px;
    border-top: 1px solid var(--color-border);
  }
</style>
```

### Built-in Templates

```typescript
// data/spec-templates.ts
import type { SpecTemplate } from '$lib/types/spec';

export const builtInTemplates: SpecTemplate[] = [
  {
    id: 'ui-component',
    name: 'UI Component',
    description: 'Template for creating new UI components with Svelte',
    category: 'UI Component',
    icon: 'layout',
    content: `# Spec {{specId}}: {{componentName}}

## Phase
{{phase}}

## Spec ID
{{specId}}

## Status
Planned

## Dependencies
{{dependencies|None}}

## Estimated Context
{{context|~10%}}

---

## Objective

Create the {{componentName}} component that {{purpose}}.

---

## Acceptance Criteria

- [ ] Component renders correctly
- [ ] Supports required props
- [ ] Handles edge cases gracefully
- [ ] Accessible with proper ARIA attributes
- [ ] Responsive across breakpoints
- [ ] Keyboard navigable

---

## Implementation Details

### {{componentName}}.svelte

\`\`\`svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let {{mainProp|value}}: string = '';

  const dispatch = createEventDispatcher();
</script>

<div class="{{cssClass|component}}">
  <!-- Component content -->
</div>

<style>
  .{{cssClass|component}} {
    /* Styles */
  }
</style>
\`\`\`

---

## Testing Requirements

### Unit Tests

\`\`\`typescript
import { render, screen } from '@testing-library/svelte';
import { describe, it, expect } from 'vitest';
import {{componentName}} from './{{componentName}}.svelte';

describe('{{componentName}}', () => {
  it('renders correctly', () => {
    render({{componentName}});
    // Add assertions
  });
});
\`\`\`

---

## Related Specs

- {{relatedSpecs|None}}
`,
    title: '{{componentName}}',
    variables: [
      { name: 'specId', label: 'Spec ID', required: true },
      { name: 'componentName', label: 'Component Name', required: true },
      { name: 'phase', label: 'Phase', defaultValue: '11' },
      { name: 'purpose', label: 'Purpose/Description', required: true },
      { name: 'mainProp', label: 'Main Prop Name', defaultValue: 'value' },
      { name: 'cssClass', label: 'CSS Class', defaultValue: 'component' }
    ]
  },

  {
    id: 'api-endpoint',
    name: 'API Endpoint',
    description: 'Template for API endpoint specifications',
    category: 'API Endpoint',
    icon: 'server',
    content: `# Spec {{specId}}: {{endpointName}} API

## Phase
{{phase}}

## Spec ID
{{specId}}

## Status
Planned

## Dependencies
{{dependencies|None}}

## Estimated Context
{{context|~8%}}

---

## Objective

Implement the {{method}} {{path}} endpoint that {{purpose}}.

---

## Acceptance Criteria

- [ ] Endpoint accepts valid requests
- [ ] Returns proper response format
- [ ] Handles authentication/authorization
- [ ] Validates input parameters
- [ ] Returns appropriate error codes
- [ ] Logs requests appropriately

---

## API Specification

### Endpoint

\`\`\`
{{method}} {{path}}
\`\`\`

### Request

#### Headers
\`\`\`
Authorization: Bearer <token>
Content-Type: application/json
\`\`\`

#### Body
\`\`\`json
{
  {{requestBody|"field": "value"}}
}
\`\`\`

### Response

#### Success ({{successCode|200}})
\`\`\`json
{
  {{responseBody|"data": {}}}
}
\`\`\`

#### Errors
| Code | Description |
|------|-------------|
| 400 | Bad Request |
| 401 | Unauthorized |
| 404 | Not Found |
| 500 | Server Error |

---

## Implementation Details

\`\`\`typescript
export async function {{handlerName}}(req: Request): Promise<Response> {
  // Implementation
}
\`\`\`

---

## Testing Requirements

- Unit tests for handler logic
- Integration tests with database
- Load testing for performance

---

## Related Specs

- {{relatedSpecs|None}}
`,
    title: '{{endpointName}} API',
    variables: [
      { name: 'specId', label: 'Spec ID', required: true },
      { name: 'endpointName', label: 'Endpoint Name', required: true },
      { name: 'method', label: 'HTTP Method', defaultValue: 'GET' },
      { name: 'path', label: 'Path', defaultValue: '/api/v1/resource' },
      { name: 'purpose', label: 'Purpose', required: true },
      { name: 'handlerName', label: 'Handler Function', defaultValue: 'handleRequest' }
    ]
  },

  {
    id: 'feature',
    name: 'Feature Spec',
    description: 'Template for new feature specifications',
    category: 'Feature',
    icon: 'star',
    content: `# Spec {{specId}}: {{featureName}}

## Phase
{{phase}}

## Spec ID
{{specId}}

## Status
Planned

## Dependencies
{{dependencies|None}}

## Estimated Context
{{context|~12%}}

---

## Objective

{{objective}}

---

## User Story

As a {{userRole|user}}, I want to {{userWant}}, so that {{userBenefit}}.

---

## Acceptance Criteria

- [ ] {{criterion1}}
- [ ] {{criterion2}}
- [ ] {{criterion3}}

---

## Technical Requirements

### Components
- {{component1|Component A}}
- {{component2|Component B}}

### Data Model
\`\`\`typescript
interface {{modelName|Feature}} {
  id: string;
  // Add fields
}
\`\`\`

---

## Implementation Details

<!-- Describe implementation approach -->

---

## Testing Requirements

- Unit tests for business logic
- Integration tests for data flow
- E2E tests for user workflows

---

## Related Specs

- {{relatedSpecs|None}}
`,
    title: '{{featureName}}',
    variables: [
      { name: 'specId', label: 'Spec ID', required: true },
      { name: 'featureName', label: 'Feature Name', required: true },
      { name: 'phase', label: 'Phase', required: true },
      { name: 'objective', label: 'Objective', required: true },
      { name: 'userWant', label: 'User Want', required: true },
      { name: 'userBenefit', label: 'User Benefit', required: true }
    ]
  },

  {
    id: 'blank',
    name: 'Blank Template',
    description: 'Minimal template with just the required structure',
    category: 'Documentation',
    icon: 'file',
    content: `# Spec {{specId}}: {{title}}

## Phase
{{phase}}

## Spec ID
{{specId}}

## Status
Planned

## Dependencies
{{dependencies|None}}

## Estimated Context
{{context|~10%}}

---

## Objective

{{objective}}

---

## Acceptance Criteria

- [ ]

---

## Implementation Details

<!-- Add implementation details -->

---

## Testing Requirements

<!-- Add testing requirements -->

---

## Related Specs

-
`,
    title: '{{title}}',
    variables: [
      { name: 'specId', label: 'Spec ID', required: true },
      { name: 'title', label: 'Title', required: true },
      { name: 'phase', label: 'Phase', required: true },
      { name: 'objective', label: 'Objective', required: true }
    ]
  }
];
```

### Template Types

```typescript
// types/spec.ts additions
export type TemplateCategory =
  | 'UI Component'
  | 'API Endpoint'
  | 'Feature'
  | 'Bug Fix'
  | 'Refactoring'
  | 'Documentation'
  | 'Testing'
  | 'Infrastructure';

export interface TemplateVariable {
  name: string;
  label: string;
  defaultValue?: string;
  required?: boolean;
}

export interface SpecTemplate {
  id: string;
  name: string;
  description: string;
  category: TemplateCategory;
  icon?: string;
  content: string;
  title: string;
  variables?: TemplateVariable[];
  isCustom?: boolean;
}
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import TemplateSelector from './TemplateSelector.svelte';
import { builtInTemplates } from '$lib/data/spec-templates';

describe('TemplateSelector', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('renders built-in templates', () => {
    render(TemplateSelector, { props: { open: true } });

    expect(screen.getByText('UI Component')).toBeInTheDocument();
    expect(screen.getByText('API Endpoint')).toBeInTheDocument();
  });

  it('filters templates by search', async () => {
    render(TemplateSelector, { props: { open: true } });

    const search = screen.getByPlaceholderText('Search templates...');
    await fireEvent.input(search, { target: { value: 'API' } });

    expect(screen.getByText('API Endpoint')).toBeInTheDocument();
    expect(screen.queryByText('UI Component')).not.toBeInTheDocument();
  });

  it('filters templates by category', async () => {
    render(TemplateSelector, { props: { open: true } });

    await fireEvent.click(screen.getByText('Feature'));

    expect(screen.getByText('Feature Spec')).toBeInTheDocument();
  });

  it('shows template detail on click', async () => {
    render(TemplateSelector, { props: { open: true } });

    await fireEvent.click(screen.getByText('UI Component'));

    expect(screen.getByText('Back to templates')).toBeInTheDocument();
    expect(screen.getByText('Template Variables')).toBeInTheDocument();
  });

  it('handles variable input', async () => {
    render(TemplateSelector, { props: { open: true } });

    await fireEvent.click(screen.getByText('UI Component'));

    const specIdInput = screen.getByLabelText(/Spec ID/);
    await fireEvent.input(specIdInput, { target: { value: '250' } });

    // Preview should update
    expect(screen.getByText(/250/)).toBeInTheDocument();
  });

  it('dispatches select event with filled template', async () => {
    const { component } = render(TemplateSelector, { props: { open: true } });

    const selectHandler = vi.fn();
    component.$on('select', selectHandler);

    await fireEvent.click(screen.getByText('Blank Template'));

    const inputs = screen.getAllByRole('textbox');
    for (const input of inputs) {
      await fireEvent.input(input, { target: { value: 'test' } });
    }

    await fireEvent.click(screen.getByText('Use Template'));

    expect(selectHandler).toHaveBeenCalled();
    expect(selectHandler.mock.calls[0][0].detail.template).toBeDefined();
    expect(selectHandler.mock.calls[0][0].detail.variables).toBeDefined();
  });

  it('saves custom templates to localStorage', async () => {
    // Test custom template creation
  });

  it('loads custom templates from localStorage', () => {
    localStorage.setItem('custom-spec-templates', JSON.stringify([
      { id: 'custom-1', name: 'My Template', category: 'Feature' }
    ]));

    render(TemplateSelector, { props: { open: true } });

    expect(screen.getByText('My Template')).toBeInTheDocument();
  });
});
```

---

## Related Specs

- Spec 237: Spec Editor
- Spec 241: Spec Creation Form
- Spec 247: Spec Export (template export)
