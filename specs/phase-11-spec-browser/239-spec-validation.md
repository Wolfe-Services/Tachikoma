# Spec 239: Spec Validation

## Phase
11 - Spec Browser UI

## Spec ID
239

## Status
Planned

## Dependencies
- Spec 237 (Spec Editor)
- Phase 10 (Core UI Components)

## Estimated Context
~8%

---

## Objective

Implement a comprehensive validation system for specs that checks for required fields, format compliance, circular dependencies, broken references, and content quality. Provide real-time feedback with actionable suggestions.

---

## Acceptance Criteria

- [ ] Validate required fields (ID, title, status, phase)
- [ ] Check ID format and uniqueness
- [ ] Detect circular dependencies
- [ ] Validate dependency references exist
- [ ] Check markdown content structure
- [ ] Validate acceptance criteria format
- [ ] Provide severity levels (error, warning, info)
- [ ] Show inline validation markers
- [ ] Offer auto-fix suggestions
- [ ] Support custom validation rules

---

## Implementation Details

### ValidationPanel.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { ValidationResult, ValidationSeverity } from '$lib/types/spec';
  import Icon from '$lib/components/Icon.svelte';
  import Button from '$lib/components/Button.svelte';

  export let results: ValidationResult[] = [];
  export let showFixes = true;

  const dispatch = createEventDispatcher<{
    fix: ValidationResult;
    focus: { field: string; line?: number };
  }>();

  $: groupedResults = groupBySeverity(results);
  $: errorCount = groupedResults.error?.length ?? 0;
  $: warningCount = groupedResults.warning?.length ?? 0;
  $: infoCount = groupedResults.info?.length ?? 0;

  function groupBySeverity(
    results: ValidationResult[]
  ): Record<ValidationSeverity, ValidationResult[]> {
    return results.reduce((acc, result) => {
      if (!acc[result.severity]) {
        acc[result.severity] = [];
      }
      acc[result.severity].push(result);
      return acc;
    }, {} as Record<ValidationSeverity, ValidationResult[]>);
  }

  function getSeverityIcon(severity: ValidationSeverity): string {
    return {
      error: 'x-circle',
      warning: 'alert-triangle',
      info: 'info'
    }[severity];
  }

  function getSeverityColor(severity: ValidationSeverity): string {
    return {
      error: 'var(--color-danger)',
      warning: 'var(--color-warning)',
      info: 'var(--color-info)'
    }[severity];
  }

  function handleFocus(result: ValidationResult) {
    dispatch('focus', { field: result.field, line: result.line });
  }

  function handleFix(result: ValidationResult) {
    dispatch('fix', result);
  }
</script>

<div class="validation-panel">
  <header class="validation-panel__header">
    <h3>Validation Results</h3>
    <div class="validation-panel__counts">
      {#if errorCount > 0}
        <span class="validation-panel__count validation-panel__count--error">
          <Icon name="x-circle" size={14} />
          {errorCount}
        </span>
      {/if}
      {#if warningCount > 0}
        <span class="validation-panel__count validation-panel__count--warning">
          <Icon name="alert-triangle" size={14} />
          {warningCount}
        </span>
      {/if}
      {#if infoCount > 0}
        <span class="validation-panel__count validation-panel__count--info">
          <Icon name="info" size={14} />
          {infoCount}
        </span>
      {/if}
    </div>
  </header>

  <div class="validation-panel__body">
    {#if results.length === 0}
      <div class="validation-panel__empty">
        <Icon name="check-circle" size={24} />
        <span>No validation issues</span>
      </div>
    {:else}
      {#each ['error', 'warning', 'info'] as severity}
        {#if groupedResults[severity]?.length > 0}
          <section class="validation-panel__section">
            <h4 class="validation-panel__section-title" style:color={getSeverityColor(severity)}>
              <Icon name={getSeverityIcon(severity)} size={14} />
              {severity === 'error' ? 'Errors' : severity === 'warning' ? 'Warnings' : 'Information'}
            </h4>
            <ul class="validation-panel__list">
              {#each groupedResults[severity] as result}
                <li class="validation-panel__item">
                  <div class="validation-panel__item-content">
                    <button
                      class="validation-panel__item-message"
                      on:click={() => handleFocus(result)}
                    >
                      {result.message}
                    </button>
                    {#if result.field}
                      <span class="validation-panel__item-field">
                        {result.field}
                        {#if result.line}
                          :L{result.line}
                        {/if}
                      </span>
                    {/if}
                  </div>
                  {#if showFixes && result.fix}
                    <Button
                      variant="ghost"
                      size="sm"
                      on:click={() => handleFix(result)}
                    >
                      <Icon name="wand" size={12} />
                      Fix
                    </Button>
                  {/if}
                </li>
              {/each}
            </ul>
          </section>
        {/if}
      {/each}
    {/if}
  </div>
</div>

<style>
  .validation-panel {
    background: var(--color-surface);
    border-top: 1px solid var(--color-border);
  }

  .validation-panel__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    background: var(--color-surface-subtle);
    border-bottom: 1px solid var(--color-border);
  }

  .validation-panel__header h3 {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    margin: 0;
  }

  .validation-panel__counts {
    display: flex;
    gap: 12px;
  }

  .validation-panel__count {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .validation-panel__count--error { color: var(--color-danger); }
  .validation-panel__count--warning { color: var(--color-warning); }
  .validation-panel__count--info { color: var(--color-info); }

  .validation-panel__body {
    max-height: 180px;
    overflow-y: auto;
  }

  .validation-panel__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 24px;
    color: var(--color-success);
    font-size: 0.875rem;
  }

  .validation-panel__section {
    padding: 8px 0;
  }

  .validation-panel__section:not(:last-child) {
    border-bottom: 1px solid var(--color-border);
  }

  .validation-panel__section-title {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 16px;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    margin: 0;
  }

  .validation-panel__list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .validation-panel__item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 16px;
  }

  .validation-panel__item:hover {
    background: var(--color-hover);
  }

  .validation-panel__item-content {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
  }

  .validation-panel__item-message {
    flex: 1;
    padding: 0;
    background: none;
    border: none;
    text-align: left;
    font-size: 0.875rem;
    color: var(--color-text-primary);
    cursor: pointer;
  }

  .validation-panel__item-message:hover {
    text-decoration: underline;
  }

  .validation-panel__item-field {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }
</style>
```

### Validation Utilities

```typescript
// utils/validation.ts
import type {
  Spec,
  ValidationResult,
  ValidationSeverity,
  ValidationRule
} from '$lib/types/spec';

export function validateSpec(spec: Spec, allSpecs: Spec[]): ValidationResult[] {
  const results: ValidationResult[] = [];

  // Run all validation rules
  const rules = getValidationRules();

  for (const rule of rules) {
    const ruleResults = rule.validate(spec, allSpecs);
    results.push(...ruleResults);
  }

  return results.sort((a, b) => {
    const severityOrder = { error: 0, warning: 1, info: 2 };
    return severityOrder[a.severity] - severityOrder[b.severity];
  });
}

function getValidationRules(): ValidationRule[] {
  return [
    // Required fields
    {
      id: 'required-id',
      validate: (spec) => {
        if (!spec.id?.trim()) {
          return [{
            id: 'required-id',
            severity: 'error',
            message: 'Spec ID is required',
            field: 'id',
            fix: null
          }];
        }
        return [];
      }
    },

    // ID format
    {
      id: 'id-format',
      validate: (spec) => {
        if (spec.id && !/^\d{1,4}$/.test(spec.id)) {
          return [{
            id: 'id-format',
            severity: 'warning',
            message: 'Spec ID should be a 1-4 digit number',
            field: 'id',
            fix: {
              description: 'Extract numeric ID',
              apply: (s) => ({ ...s, id: s.id.replace(/\D/g, '').slice(0, 4) })
            }
          }];
        }
        return [];
      }
    },

    // Unique ID
    {
      id: 'unique-id',
      validate: (spec, allSpecs) => {
        const duplicates = allSpecs.filter(s => s.id === spec.id && s !== spec);
        if (duplicates.length > 0) {
          return [{
            id: 'unique-id',
            severity: 'error',
            message: `Spec ID "${spec.id}" is already in use`,
            field: 'id',
            fix: null
          }];
        }
        return [];
      }
    },

    // Required title
    {
      id: 'required-title',
      validate: (spec) => {
        if (!spec.title?.trim()) {
          return [{
            id: 'required-title',
            severity: 'error',
            message: 'Spec title is required',
            field: 'title',
            fix: null
          }];
        }
        return [];
      }
    },

    // Title length
    {
      id: 'title-length',
      validate: (spec) => {
        if (spec.title && spec.title.length > 100) {
          return [{
            id: 'title-length',
            severity: 'warning',
            message: 'Title is too long (max 100 characters)',
            field: 'title',
            fix: {
              description: 'Truncate title',
              apply: (s) => ({ ...s, title: s.title.slice(0, 97) + '...' })
            }
          }];
        }
        return [];
      }
    },

    // Valid status
    {
      id: 'valid-status',
      validate: (spec) => {
        const validStatuses = ['planned', 'in-progress', 'implemented', 'tested', 'deprecated'];
        if (!validStatuses.includes(spec.status)) {
          return [{
            id: 'valid-status',
            severity: 'error',
            message: `Invalid status "${spec.status}"`,
            field: 'status',
            fix: {
              description: 'Set to "planned"',
              apply: (s) => ({ ...s, status: 'planned' })
            }
          }];
        }
        return [];
      }
    },

    // Valid phase
    {
      id: 'valid-phase',
      validate: (spec) => {
        if (spec.phase < 1 || spec.phase > 99) {
          return [{
            id: 'valid-phase',
            severity: 'error',
            message: 'Phase must be between 1 and 99',
            field: 'phase',
            fix: {
              description: 'Set to valid range',
              apply: (s) => ({ ...s, phase: Math.max(1, Math.min(99, s.phase)) })
            }
          }];
        }
        return [];
      }
    },

    // Dependencies exist
    {
      id: 'dependencies-exist',
      validate: (spec, allSpecs) => {
        const results: ValidationResult[] = [];
        const specIds = new Set(allSpecs.map(s => s.id));

        for (const depId of spec.dependencies ?? []) {
          if (!specIds.has(depId)) {
            results.push({
              id: 'dependencies-exist',
              severity: 'error',
              message: `Dependency "${depId}" does not exist`,
              field: 'dependencies',
              fix: {
                description: 'Remove invalid dependency',
                apply: (s) => ({
                  ...s,
                  dependencies: s.dependencies?.filter(d => d !== depId)
                })
              }
            });
          }
        }

        return results;
      }
    },

    // Circular dependencies
    {
      id: 'circular-dependencies',
      validate: (spec, allSpecs) => {
        const visited = new Set<string>();
        const path: string[] = [];

        function hasCycle(id: string): string[] | null {
          if (path.includes(id)) {
            return [...path.slice(path.indexOf(id)), id];
          }
          if (visited.has(id)) {
            return null;
          }

          visited.add(id);
          path.push(id);

          const current = allSpecs.find(s => s.id === id);
          for (const depId of current?.dependencies ?? []) {
            const cycle = hasCycle(depId);
            if (cycle) return cycle;
          }

          path.pop();
          return null;
        }

        const cycle = hasCycle(spec.id);
        if (cycle) {
          return [{
            id: 'circular-dependencies',
            severity: 'error',
            message: `Circular dependency detected: ${cycle.join(' -> ')}`,
            field: 'dependencies',
            fix: null
          }];
        }

        return [];
      }
    },

    // Self dependency
    {
      id: 'self-dependency',
      validate: (spec) => {
        if (spec.dependencies?.includes(spec.id)) {
          return [{
            id: 'self-dependency',
            severity: 'error',
            message: 'Spec cannot depend on itself',
            field: 'dependencies',
            fix: {
              description: 'Remove self-reference',
              apply: (s) => ({
                ...s,
                dependencies: s.dependencies?.filter(d => d !== s.id)
              })
            }
          }];
        }
        return [];
      }
    },

    // Estimated context format
    {
      id: 'context-format',
      validate: (spec) => {
        if (spec.estimatedContext && !/^~?\d{1,3}%?$/.test(spec.estimatedContext)) {
          return [{
            id: 'context-format',
            severity: 'warning',
            message: 'Estimated context should be in format like "~10%" or "8%"',
            field: 'estimatedContext',
            fix: null
          }];
        }
        return [];
      }
    },

    // Content has required sections
    {
      id: 'required-sections',
      validate: (spec) => {
        const results: ValidationResult[] = [];
        const content = spec.content.toLowerCase();

        const requiredSections = [
          { name: 'Objective', pattern: /##\s*objective/i },
          { name: 'Acceptance Criteria', pattern: /##\s*acceptance\s*criteria/i }
        ];

        for (const section of requiredSections) {
          if (!section.pattern.test(spec.content)) {
            results.push({
              id: 'required-sections',
              severity: 'warning',
              message: `Missing "${section.name}" section`,
              field: 'content',
              fix: {
                description: `Add ${section.name} section`,
                apply: (s) => ({
                  ...s,
                  content: s.content + `\n\n## ${section.name}\n\n`
                })
              }
            });
          }
        }

        return results;
      }
    },

    // Acceptance criteria format
    {
      id: 'criteria-format',
      validate: (spec) => {
        const criteriaMatch = spec.content.match(
          /##\s*acceptance\s*criteria\s*\n+([\s\S]*?)(?=\n##|$)/i
        );

        if (criteriaMatch) {
          const criteriaContent = criteriaMatch[1];
          const hasCheckboxes = /^\s*-\s*\[[x ]\]/im.test(criteriaContent);

          if (!hasCheckboxes && criteriaContent.trim()) {
            return [{
              id: 'criteria-format',
              severity: 'info',
              message: 'Acceptance criteria should use checkbox format (- [ ] item)',
              field: 'content',
              fix: null
            }];
          }
        }

        return [];
      }
    },

    // Empty content
    {
      id: 'empty-content',
      validate: (spec) => {
        if (!spec.content?.trim()) {
          return [{
            id: 'empty-content',
            severity: 'warning',
            message: 'Spec has no content',
            field: 'content',
            fix: null
          }];
        }
        return [];
      }
    },

    // Content length
    {
      id: 'content-length',
      validate: (spec) => {
        const wordCount = spec.content.split(/\s+/).filter(Boolean).length;

        if (wordCount < 50) {
          return [{
            id: 'content-length',
            severity: 'info',
            message: 'Spec content is quite short. Consider adding more details.',
            field: 'content',
            fix: null
          }];
        }

        return [];
      }
    }
  ];
}

// Utility to get dependency chain for a spec
export function getDependencyChain(
  specId: string,
  allSpecs: Spec[],
  maxDepth = 10
): string[][] {
  const chains: string[][] = [];

  function traverse(id: string, path: string[], depth: number) {
    if (depth > maxDepth) return;

    const spec = allSpecs.find(s => s.id === id);
    if (!spec?.dependencies?.length) {
      if (path.length > 0) {
        chains.push([...path, id]);
      }
      return;
    }

    for (const depId of spec.dependencies) {
      traverse(depId, [...path, id], depth + 1);
    }
  }

  traverse(specId, [], 0);
  return chains;
}
```

### Validation Types

```typescript
// types/spec.ts additions
export type ValidationSeverity = 'error' | 'warning' | 'info';

export interface ValidationResult {
  id: string;
  severity: ValidationSeverity;
  message: string;
  field?: string;
  line?: number;
  fix?: ValidationFix | null;
}

export interface ValidationFix {
  description: string;
  apply: (spec: Spec) => Spec;
}

export interface ValidationRule {
  id: string;
  validate: (spec: Spec, allSpecs: Spec[]) => ValidationResult[];
}
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import ValidationPanel from './ValidationPanel.svelte';
import { validateSpec, getDependencyChain } from '$lib/utils/validation';
import { createMockSpec, createMockSpecs } from '$lib/test-utils/mock-data';

describe('ValidationPanel', () => {
  it('displays validation results grouped by severity', () => {
    const results = [
      { id: '1', severity: 'error', message: 'Error message', field: 'id' },
      { id: '2', severity: 'warning', message: 'Warning message', field: 'title' },
      { id: '3', severity: 'info', message: 'Info message', field: 'content' }
    ];

    render(ValidationPanel, { props: { results } });

    expect(screen.getByText('Errors')).toBeInTheDocument();
    expect(screen.getByText('Warnings')).toBeInTheDocument();
    expect(screen.getByText('Information')).toBeInTheDocument();
  });

  it('shows empty state when no issues', () => {
    render(ValidationPanel, { props: { results: [] } });

    expect(screen.getByText('No validation issues')).toBeInTheDocument();
  });

  it('displays counts in header', () => {
    const results = [
      { id: '1', severity: 'error', message: 'E1' },
      { id: '2', severity: 'error', message: 'E2' },
      { id: '3', severity: 'warning', message: 'W1' }
    ];

    render(ValidationPanel, { props: { results } });

    // Should show 2 errors, 1 warning
    expect(screen.getByText('2')).toBeInTheDocument();
    expect(screen.getByText('1')).toBeInTheDocument();
  });

  it('dispatches focus event on message click', async () => {
    const results = [
      { id: '1', severity: 'error', message: 'Error', field: 'id', line: 5 }
    ];

    const { component } = render(ValidationPanel, { props: { results } });

    const focusHandler = vi.fn();
    component.$on('focus', focusHandler);

    await fireEvent.click(screen.getByText('Error'));

    expect(focusHandler).toHaveBeenCalledWith(
      expect.objectContaining({
        detail: { field: 'id', line: 5 }
      })
    );
  });

  it('shows fix button when fix available', () => {
    const results = [{
      id: '1',
      severity: 'warning',
      message: 'Warning',
      fix: { description: 'Auto fix', apply: () => {} }
    }];

    render(ValidationPanel, { props: { results, showFixes: true } });

    expect(screen.getByText('Fix')).toBeInTheDocument();
  });
});

describe('validateSpec', () => {
  const allSpecs = createMockSpecs(5);

  it('validates required fields', () => {
    const spec = createMockSpec({ id: '', title: '' });
    const results = validateSpec(spec, allSpecs);

    expect(results.some(r => r.id === 'required-id')).toBe(true);
    expect(results.some(r => r.id === 'required-title')).toBe(true);
  });

  it('validates ID format', () => {
    const spec = createMockSpec({ id: 'invalid-id' });
    const results = validateSpec(spec, allSpecs);

    expect(results.some(r => r.id === 'id-format')).toBe(true);
  });

  it('detects duplicate IDs', () => {
    const specs = [
      createMockSpec({ id: '100' }),
      createMockSpec({ id: '100' })
    ];
    const results = validateSpec(specs[1], specs);

    expect(results.some(r => r.id === 'unique-id')).toBe(true);
  });

  it('validates dependency existence', () => {
    const spec = createMockSpec({ dependencies: ['999'] });
    const results = validateSpec(spec, allSpecs);

    expect(results.some(r => r.id === 'dependencies-exist')).toBe(true);
  });

  it('detects circular dependencies', () => {
    const specs = [
      createMockSpec({ id: '1', dependencies: ['2'] }),
      createMockSpec({ id: '2', dependencies: ['3'] }),
      createMockSpec({ id: '3', dependencies: ['1'] })
    ];

    const results = validateSpec(specs[0], specs);

    expect(results.some(r => r.id === 'circular-dependencies')).toBe(true);
  });

  it('detects self-dependency', () => {
    const spec = createMockSpec({ id: '100', dependencies: ['100'] });
    const results = validateSpec(spec, allSpecs);

    expect(results.some(r => r.id === 'self-dependency')).toBe(true);
  });

  it('validates estimated context format', () => {
    const spec = createMockSpec({ estimatedContext: 'invalid' });
    const results = validateSpec(spec, allSpecs);

    expect(results.some(r => r.id === 'context-format')).toBe(true);
  });

  it('checks for required content sections', () => {
    const spec = createMockSpec({ content: 'No sections here' });
    const results = validateSpec(spec, allSpecs);

    expect(results.some(r =>
      r.id === 'required-sections' && r.message.includes('Objective')
    )).toBe(true);
  });

  it('applies fix correctly', () => {
    const spec = createMockSpec({ title: 'A'.repeat(150) });
    const results = validateSpec(spec, allSpecs);

    const titleResult = results.find(r => r.id === 'title-length');
    expect(titleResult?.fix).toBeDefined();

    if (titleResult?.fix) {
      const fixed = titleResult.fix.apply(spec);
      expect(fixed.title.length).toBeLessThanOrEqual(100);
    }
  });
});

describe('getDependencyChain', () => {
  it('returns empty for spec with no dependencies', () => {
    const specs = [createMockSpec({ id: '1', dependencies: [] })];
    const chains = getDependencyChain('1', specs);

    expect(chains).toEqual([]);
  });

  it('returns linear chain', () => {
    const specs = [
      createMockSpec({ id: '1', dependencies: ['2'] }),
      createMockSpec({ id: '2', dependencies: ['3'] }),
      createMockSpec({ id: '3', dependencies: [] })
    ];

    const chains = getDependencyChain('1', specs);

    expect(chains).toContainEqual(['1', '2', '3']);
  });

  it('handles branching dependencies', () => {
    const specs = [
      createMockSpec({ id: '1', dependencies: ['2', '3'] }),
      createMockSpec({ id: '2', dependencies: [] }),
      createMockSpec({ id: '3', dependencies: [] })
    ];

    const chains = getDependencyChain('1', specs);

    expect(chains.length).toBe(2);
  });
});
```

---

## Related Specs

- Spec 237: Spec Editor
- Spec 240: Spec Templates
- Spec 241: Spec Creation Form
- Spec 243: Dependency Visualization
