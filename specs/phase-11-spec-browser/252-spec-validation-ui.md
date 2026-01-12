# 252 - Spec Validation UI

**Phase:** 11 - Spec Browser UI
**Spec ID:** 252
**Status:** Planned
**Dependencies:** 236-spec-browser-layout, 240-spec-editor
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create a validation UI component that displays spec validation results, warnings, errors, and suggestions with inline highlighting and quick-fix actions.

---

## Acceptance Criteria

- [ ] Display validation errors inline
- [ ] Show warnings and suggestions
- [ ] Quick-fix actions for common issues
- [ ] Validate frontmatter completeness
- [ ] Check dependency validity
- [ ] Verify markdown structure
- [ ] Real-time validation feedback
- [ ] Validation summary panel

---

## Implementation Details

### 1. Types (src/lib/types/spec-validation.ts)

```typescript
export type ValidationSeverity = 'error' | 'warning' | 'info' | 'suggestion';

export interface ValidationResult {
  isValid: boolean;
  score: number;
  errors: ValidationIssue[];
  warnings: ValidationIssue[];
  info: ValidationIssue[];
  suggestions: ValidationIssue[];
}

export interface ValidationIssue {
  id: string;
  severity: ValidationSeverity;
  code: string;
  message: string;
  description?: string;
  location?: IssueLocation;
  quickFixes?: QuickFix[];
  relatedIssues?: string[];
}

export interface IssueLocation {
  line: number;
  column?: number;
  endLine?: number;
  endColumn?: number;
  field?: string;
}

export interface QuickFix {
  id: string;
  title: string;
  description: string;
  edits: TextEdit[];
  isPreferred?: boolean;
}

export interface TextEdit {
  range: {
    startLine: number;
    startColumn: number;
    endLine: number;
    endColumn: number;
  };
  newText: string;
}

export interface ValidationRule {
  id: string;
  name: string;
  description: string;
  severity: ValidationSeverity;
  category: ValidationCategory;
  enabled: boolean;
}

export type ValidationCategory =
  | 'frontmatter'
  | 'structure'
  | 'content'
  | 'dependencies'
  | 'links'
  | 'formatting';
```

### 2. Spec Validation Panel Component (src/lib/components/spec-browser/SpecValidationPanel.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type {
    ValidationResult,
    ValidationIssue,
    ValidationSeverity,
    QuickFix,
    ValidationCategory,
  } from '$lib/types/spec-validation';
  import { ipcRenderer } from '$lib/ipc';
  import { slide, fade } from 'svelte/transition';

  export let specId: string;
  export let content: string;

  const dispatch = createEventDispatcher<{
    goToIssue: ValidationIssue;
    applyFix: QuickFix;
    revalidate: void;
  }>();

  let validationResult: ValidationResult | null = null;
  let isValidating = false;
  let selectedCategory: ValidationCategory | 'all' = 'all';
  let expandedIssues: Set<string> = new Set();
  let autoValidate = true;
  let validationTimeout: ReturnType<typeof setTimeout>;

  const severityIcons: Record<ValidationSeverity, string> = {
    error: '❌',
    warning: '⚠️',
    info: 'ℹ️',
    suggestion: '💡',
  };

  const severityOrder: Record<ValidationSeverity, number> = {
    error: 0,
    warning: 1,
    info: 2,
    suggestion: 3,
  };

  const categoryLabels: Record<ValidationCategory, string> = {
    frontmatter: 'Frontmatter',
    structure: 'Structure',
    content: 'Content',
    dependencies: 'Dependencies',
    links: 'Links',
    formatting: 'Formatting',
  };

  async function validate() {
    isValidating = true;
    try {
      validationResult = await ipcRenderer.invoke('spec:validate', {
        specId,
        content,
      });
    } finally {
      isValidating = false;
    }
  }

  function scheduleValidation() {
    if (!autoValidate) return;
    clearTimeout(validationTimeout);
    validationTimeout = setTimeout(validate, 500);
  }

  function getAllIssues(): ValidationIssue[] {
    if (!validationResult) return [];
    return [
      ...validationResult.errors,
      ...validationResult.warnings,
      ...validationResult.info,
      ...validationResult.suggestions,
    ].sort((a, b) => severityOrder[a.severity] - severityOrder[b.severity]);
  }

  function filterIssues(issues: ValidationIssue[]): ValidationIssue[] {
    if (selectedCategory === 'all') return issues;
    return issues.filter(issue => {
      const code = issue.code.split('/')[0] as ValidationCategory;
      return code === selectedCategory;
    });
  }

  function toggleIssue(issueId: string) {
    if (expandedIssues.has(issueId)) {
      expandedIssues.delete(issueId);
    } else {
      expandedIssues.add(issueId);
    }
    expandedIssues = new Set(expandedIssues);
  }

  function goToIssue(issue: ValidationIssue) {
    dispatch('goToIssue', issue);
  }

  async function applyQuickFix(issue: ValidationIssue, fix: QuickFix) {
    try {
      await ipcRenderer.invoke('spec:apply-fix', {
        specId,
        fix,
      });
      dispatch('applyFix', fix);
      // Revalidate after fix
      await validate();
    } catch (error) {
      console.error('Failed to apply fix:', error);
    }
  }

  function getScoreColor(score: number): string {
    if (score >= 90) return 'var(--color-success)';
    if (score >= 70) return 'var(--color-warning)';
    return 'var(--color-error)';
  }

  function getCategoryCount(category: ValidationCategory): number {
    const issues = getAllIssues();
    return issues.filter(i => i.code.startsWith(category)).length;
  }

  onMount(validate);

  $: if (content) scheduleValidation();
  $: allIssues = getAllIssues();
  $: filteredIssues = filterIssues(allIssues);
</script>

<div class="validation-panel">
  <header class="validation-panel__header">
    <div class="header-title">
      <h3>Validation</h3>
      {#if validationResult}
        <div
          class="validation-score"
          style="color: {getScoreColor(validationResult.score)}"
        >
          {validationResult.score}%
        </div>
      {/if}
    </div>

    <div class="header-actions">
      <label class="auto-validate">
        <input type="checkbox" bind:checked={autoValidate} />
        <span>Auto</span>
      </label>
      <button
        class="validate-btn"
        on:click={validate}
        disabled={isValidating}
      >
        {isValidating ? 'Validating...' : 'Validate'}
      </button>
    </div>
  </header>

  {#if validationResult}
    <div class="validation-summary">
      <div class="summary-item summary-item--error">
        <span class="summary-icon">{severityIcons.error}</span>
        <span class="summary-count">{validationResult.errors.length}</span>
        <span class="summary-label">Errors</span>
      </div>
      <div class="summary-item summary-item--warning">
        <span class="summary-icon">{severityIcons.warning}</span>
        <span class="summary-count">{validationResult.warnings.length}</span>
        <span class="summary-label">Warnings</span>
      </div>
      <div class="summary-item summary-item--info">
        <span class="summary-icon">{severityIcons.info}</span>
        <span class="summary-count">{validationResult.info.length}</span>
        <span class="summary-label">Info</span>
      </div>
      <div class="summary-item summary-item--suggestion">
        <span class="summary-icon">{severityIcons.suggestion}</span>
        <span class="summary-count">{validationResult.suggestions.length}</span>
        <span class="summary-label">Suggestions</span>
      </div>
    </div>

    <nav class="category-filter">
      <button
        class="category-btn"
        class:active={selectedCategory === 'all'}
        on:click={() => { selectedCategory = 'all'; }}
      >
        All ({allIssues.length})
      </button>
      {#each Object.entries(categoryLabels) as [category, label]}
        {@const count = getCategoryCount(category)}
        {#if count > 0}
          <button
            class="category-btn"
            class:active={selectedCategory === category}
            on:click={() => { selectedCategory = category; }}
          >
            {label} ({count})
          </button>
        {/if}
      {/each}
    </nav>

    <div class="issues-list">
      {#if filteredIssues.length === 0}
        <div class="no-issues" transition:fade>
          {#if allIssues.length === 0}
            <span class="success-icon">✅</span>
            <p>No issues found</p>
          {:else}
            <p>No issues in this category</p>
          {/if}
        </div>
      {:else}
        {#each filteredIssues as issue (issue.id)}
          <div
            class="issue-item issue-item--{issue.severity}"
            transition:slide={{ duration: 150 }}
          >
            <button
              class="issue-header"
              on:click={() => toggleIssue(issue.id)}
            >
              <span class="issue-icon">{severityIcons[issue.severity]}</span>
              <span class="issue-message">{issue.message}</span>
              {#if issue.location}
                <span class="issue-location">
                  Line {issue.location.line}
                </span>
              {/if}
              <svg
                class="expand-icon"
                class:expanded={expandedIssues.has(issue.id)}
                width="12"
                height="12"
                viewBox="0 0 12 12"
              >
                <path d="M3 4.5l3 3 3-3" stroke="currentColor" stroke-width="1.5" fill="none"/>
              </svg>
            </button>

            {#if expandedIssues.has(issue.id)}
              <div class="issue-details" transition:slide={{ duration: 150 }}>
                {#if issue.description}
                  <p class="issue-description">{issue.description}</p>
                {/if}

                <div class="issue-meta">
                  <span class="issue-code">{issue.code}</span>
                  {#if issue.location?.field}
                    <span class="issue-field">Field: {issue.location.field}</span>
                  {/if}
                </div>

                <div class="issue-actions">
                  {#if issue.location}
                    <button
                      class="action-btn"
                      on:click={() => goToIssue(issue)}
                    >
                      Go to Issue
                    </button>
                  {/if}

                  {#if issue.quickFixes && issue.quickFixes.length > 0}
                    <div class="quick-fixes">
                      <span class="quick-fixes-label">Quick Fixes:</span>
                      {#each issue.quickFixes as fix}
                        <button
                          class="fix-btn"
                          class:preferred={fix.isPreferred}
                          on:click={() => applyQuickFix(issue, fix)}
                          title={fix.description}
                        >
                          {fix.title}
                        </button>
                      {/each}
                    </div>
                  {/if}
                </div>

                {#if issue.relatedIssues && issue.relatedIssues.length > 0}
                  <div class="related-issues">
                    <span class="related-label">Related:</span>
                    {#each issue.relatedIssues as relatedId}
                      <span class="related-id">{relatedId}</span>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
  {:else if isValidating}
    <div class="loading-state">
      <div class="spinner" />
      <p>Validating spec...</p>
    </div>
  {:else}
    <div class="empty-state">
      <p>Click Validate to check this spec</p>
    </div>
  {/if}
</div>

<style>
  .validation-panel {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--color-bg-primary);
  }

  .validation-panel__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .header-title h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .validation-score {
    font-size: 16px;
    font-weight: 700;
    font-family: monospace;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .auto-validate {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .auto-validate input {
    cursor: pointer;
  }

  .validate-btn {
    padding: 6px 14px;
    border: none;
    background: var(--color-primary);
    color: white;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .validate-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .validation-summary {
    display: flex;
    padding: 12px 16px;
    gap: 16px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .summary-item {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
  }

  .summary-count {
    font-weight: 600;
  }

  .summary-label {
    color: var(--color-text-muted);
  }

  .category-filter {
    display: flex;
    gap: 8px;
    padding: 12px 16px;
    overflow-x: auto;
    border-bottom: 1px solid var(--color-border);
  }

  .category-btn {
    padding: 6px 12px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 16px;
    font-size: 12px;
    color: var(--color-text-secondary);
    cursor: pointer;
    white-space: nowrap;
  }

  .category-btn:hover {
    background: var(--color-bg-hover);
  }

  .category-btn.active {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }

  .issues-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .no-issues,
  .loading-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 24px;
    color: var(--color-text-muted);
  }

  .success-icon {
    font-size: 32px;
    margin-bottom: 12px;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .issue-item {
    margin-bottom: 8px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    overflow: hidden;
  }

  .issue-item--error {
    border-left: 3px solid var(--color-error);
  }

  .issue-item--warning {
    border-left: 3px solid var(--color-warning);
  }

  .issue-item--info {
    border-left: 3px solid var(--color-primary);
  }

  .issue-item--suggestion {
    border-left: 3px solid var(--color-text-muted);
  }

  .issue-header {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 10px 12px;
    border: none;
    background: transparent;
    cursor: pointer;
    text-align: left;
  }

  .issue-header:hover {
    background: var(--color-bg-hover);
  }

  .issue-icon {
    font-size: 14px;
  }

  .issue-message {
    flex: 1;
    font-size: 13px;
    color: var(--color-text-primary);
  }

  .issue-location {
    font-family: monospace;
    font-size: 11px;
    color: var(--color-text-muted);
    padding: 2px 6px;
    background: var(--color-bg-secondary);
    border-radius: 4px;
  }

  .expand-icon {
    color: var(--color-text-muted);
    transition: transform 0.15s ease;
  }

  .expand-icon.expanded {
    transform: rotate(180deg);
  }

  .issue-details {
    padding: 12px;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .issue-description {
    margin: 0 0 12px 0;
    font-size: 13px;
    color: var(--color-text-secondary);
    line-height: 1.5;
  }

  .issue-meta {
    display: flex;
    gap: 12px;
    margin-bottom: 12px;
    font-size: 11px;
  }

  .issue-code {
    font-family: monospace;
    padding: 2px 6px;
    background: var(--color-bg-hover);
    border-radius: 4px;
    color: var(--color-text-muted);
  }

  .issue-field {
    color: var(--color-text-muted);
  }

  .issue-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
  }

  .action-btn {
    padding: 6px 12px;
    border: 1px solid var(--color-border);
    background: transparent;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    color: var(--color-text-secondary);
  }

  .action-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .quick-fixes {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .quick-fixes-label {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .fix-btn {
    padding: 4px 10px;
    border: none;
    background: var(--color-primary);
    color: white;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
  }

  .fix-btn.preferred {
    background: var(--color-success);
  }

  .fix-btn:hover {
    filter: brightness(1.1);
  }

  .related-issues {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 12px;
    font-size: 11px;
  }

  .related-label {
    color: var(--color-text-muted);
  }

  .related-id {
    padding: 2px 6px;
    background: var(--color-bg-hover);
    border-radius: 4px;
    font-family: monospace;
  }
</style>
```

---

## Testing Requirements

1. Validation runs correctly
2. Issues display by severity
3. Quick fixes apply correctly
4. Navigation to issue location works
5. Category filtering works
6. Auto-validate triggers appropriately

---

## Related Specs

- Depends on: [240-spec-editor.md](240-spec-editor.md)
- Next: [253-spec-linking.md](253-spec-linking.md)
