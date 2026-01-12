# Spec 258: Goal Input

## Header
- **Spec ID**: 258
- **Phase**: 12 - Forge UI
- **Component**: Goal Input
- **Dependencies**: Spec 257 (Session Creation)
- **Status**: Draft

## Objective
Create a rich goal input component that allows users to clearly articulate the objective for AI deliberation sessions, with markdown support, templates, and intelligent suggestions.

## Acceptance Criteria
1. Rich text input with markdown preview toggle
2. Session name input with auto-generation option
3. Goal templates for common use cases
4. Character count and length validation
5. AI-powered goal refinement suggestions
6. Context attachment support (files, URLs, code snippets)
7. History of previous goals for quick reuse
8. Real-time validation feedback

## Implementation

### GoalInput.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import MarkdownPreview from '$lib/components/MarkdownPreview.svelte';
  import GoalTemplates from './GoalTemplates.svelte';
  import ContextAttachments from './ContextAttachments.svelte';
  import GoalSuggestions from './GoalSuggestions.svelte';
  import { goalHistoryStore } from '$lib/stores/goalHistory';
  import { debounce } from '$lib/utils/timing';
  import type { GoalContext, GoalTemplate, GoalSuggestion } from '$lib/types/forge';

  export let value: string = '';
  export let name: string = '';
  export let errors: string[] = [];

  const dispatch = createEventDispatcher<{
    goalChange: string;
    nameChange: string;
  }>();

  let showPreview = writable<boolean>(false);
  let showTemplates = writable<boolean>(false);
  let showHistory = writable<boolean>(false);
  let showSuggestions = writable<boolean>(false);
  let contexts = writable<GoalContext[]>([]);
  let suggestions = writable<GoalSuggestion[]>([]);
  let isLoadingSuggestions = writable<boolean>(false);
  let textareaRef: HTMLTextAreaElement;

  const charCount = derived(
    [() => value],
    () => value.length
  );

  const charLimit = 5000;
  const minChars = 10;

  const charStatus = derived(charCount, ($count) => {
    if ($count < minChars) return 'too-short';
    if ($count > charLimit * 0.9) return 'near-limit';
    if ($count > charLimit) return 'exceeded';
    return 'ok';
  });

  function handleGoalInput(event: Event) {
    const target = event.target as HTMLTextAreaElement;
    value = target.value;
    dispatch('goalChange', value);
    debouncedFetchSuggestions();
  }

  function handleNameInput(event: Event) {
    const target = event.target as HTMLInputElement;
    name = target.value;
    dispatch('nameChange', name);
  }

  function generateSessionName() {
    const date = new Date().toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    });
    const goalPreview = value.slice(0, 30).replace(/\s+/g, ' ').trim();
    const generatedName = goalPreview
      ? `${goalPreview}... - ${date}`
      : `New Session - ${date}`;
    name = generatedName;
    dispatch('nameChange', name);
  }

  async function fetchSuggestions() {
    if (value.length < 20) {
      suggestions.set([]);
      return;
    }

    isLoadingSuggestions.set(true);
    try {
      const response = await fetch('/api/goals/suggestions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ goal: value, contexts: $contexts })
      });

      if (response.ok) {
        const data = await response.json();
        suggestions.set(data.suggestions);
      }
    } catch (error) {
      console.error('Failed to fetch suggestions:', error);
    } finally {
      isLoadingSuggestions.set(false);
    }
  }

  const debouncedFetchSuggestions = debounce(fetchSuggestions, 1000);

  function applyTemplate(template: GoalTemplate) {
    value = template.content;
    if (template.suggestedName) {
      name = template.suggestedName;
      dispatch('nameChange', name);
    }
    dispatch('goalChange', value);
    showTemplates.set(false);
  }

  function applyHistoryItem(historicalGoal: string) {
    value = historicalGoal;
    dispatch('goalChange', value);
    showHistory.set(false);
  }

  function applySuggestion(suggestion: GoalSuggestion) {
    if (suggestion.type === 'refinement') {
      value = suggestion.refinedGoal;
    } else if (suggestion.type === 'append') {
      value = `${value}\n\n${suggestion.content}`;
    }
    dispatch('goalChange', value);
    showSuggestions.set(false);
  }

  function addContext(context: GoalContext) {
    contexts.update(ctxs => [...ctxs, context]);
  }

  function removeContext(contextId: string) {
    contexts.update(ctxs => ctxs.filter(c => c.id !== contextId));
  }

  function insertAtCursor(text: string) {
    if (!textareaRef) return;

    const start = textareaRef.selectionStart;
    const end = textareaRef.selectionEnd;
    const before = value.slice(0, start);
    const after = value.slice(end);

    value = `${before}${text}${after}`;
    dispatch('goalChange', value);

    // Restore cursor position
    requestAnimationFrame(() => {
      textareaRef.selectionStart = textareaRef.selectionEnd = start + text.length;
      textareaRef.focus();
    });
  }

  onMount(() => {
    goalHistoryStore.load();
  });
</script>

<div class="goal-input" data-testid="goal-input">
  <div class="name-section">
    <label for="session-name" class="input-label">Session Name</label>
    <div class="name-input-wrapper">
      <input
        id="session-name"
        type="text"
        class="name-input"
        placeholder="Enter session name..."
        value={name}
        on:input={handleNameInput}
        maxlength="100"
      />
      <button
        type="button"
        class="auto-name-btn"
        on:click={generateSessionName}
        title="Auto-generate name from goal"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path d="M12 2L12 6M12 18L12 22M4.93 4.93L7.76 7.76M16.24 16.24L19.07 19.07M2 12L6 12M18 12L22 12M4.93 19.07L7.76 16.24M16.24 7.76L19.07 4.93" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>
  </div>

  <div class="goal-section">
    <div class="goal-header">
      <label for="goal-textarea" class="input-label">Session Goal</label>
      <div class="goal-actions">
        <button
          type="button"
          class="action-btn"
          class:active={$showTemplates}
          on:click={() => showTemplates.update(v => !v)}
        >
          Templates
        </button>
        <button
          type="button"
          class="action-btn"
          class:active={$showHistory}
          on:click={() => showHistory.update(v => !v)}
        >
          History
        </button>
        <button
          type="button"
          class="action-btn"
          class:active={$showPreview}
          on:click={() => showPreview.update(v => !v)}
        >
          {$showPreview ? 'Edit' : 'Preview'}
        </button>
      </div>
    </div>

    {#if $showTemplates}
      <GoalTemplates on:select={(e) => applyTemplate(e.detail)} />
    {/if}

    {#if $showHistory}
      <div class="history-dropdown">
        {#each $goalHistoryStore.slice(0, 5) as historicalGoal}
          <button
            type="button"
            class="history-item"
            on:click={() => applyHistoryItem(historicalGoal)}
          >
            {historicalGoal.slice(0, 100)}...
          </button>
        {/each}
        {#if $goalHistoryStore.length === 0}
          <p class="empty-history">No previous goals</p>
        {/if}
      </div>
    {/if}

    <div class="goal-editor">
      {#if $showPreview}
        <div class="preview-container">
          <MarkdownPreview content={value} />
        </div>
      {:else}
        <textarea
          id="goal-textarea"
          bind:this={textareaRef}
          class="goal-textarea"
          class:has-error={errors.length > 0}
          placeholder="Describe what you want the AI participants to deliberate on...

Example: Design a scalable authentication system that supports multiple OAuth providers, passwordless login, and meets SOC 2 compliance requirements. Consider security, user experience, and implementation complexity."
          value={value}
          on:input={handleGoalInput}
          rows="8"
          aria-describedby="goal-help char-count"
        ></textarea>
      {/if}
    </div>

    <div class="goal-footer">
      <p id="goal-help" class="help-text">
        Markdown formatting is supported. Be specific about constraints and requirements.
      </p>
      <span
        id="char-count"
        class="char-count"
        class:warning={$charStatus === 'near-limit'}
        class:error={$charStatus === 'exceeded' || $charStatus === 'too-short'}
        aria-live="polite"
      >
        {$charCount} / {charLimit}
      </span>
    </div>

    {#if errors.length > 0}
      <div class="error-messages" role="alert">
        {#each errors as error}
          <p class="error-message">{error}</p>
        {/each}
      </div>
    {/if}
  </div>

  <ContextAttachments
    contexts={$contexts}
    on:add={(e) => addContext(e.detail)}
    on:remove={(e) => removeContext(e.detail)}
  />

  {#if $suggestions.length > 0 || $isLoadingSuggestions}
    <GoalSuggestions
      suggestions={$suggestions}
      isLoading={$isLoadingSuggestions}
      on:apply={(e) => applySuggestion(e.detail)}
    />
  {/if}
</div>

<style>
  .goal-input {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .input-label {
    display: block;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-primary);
    margin-bottom: 0.5rem;
  }

  .name-section {
    margin-bottom: 0.5rem;
  }

  .name-input-wrapper {
    display: flex;
    gap: 0.5rem;
  }

  .name-input {
    flex: 1;
    padding: 0.75rem 1rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 1rem;
  }

  .name-input:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 3px var(--primary-alpha);
  }

  .auto-name-btn {
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--secondary-bg);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .auto-name-btn:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .goal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .goal-actions {
    display: flex;
    gap: 0.5rem;
  }

  .action-btn {
    padding: 0.375rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.75rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .action-btn:hover,
  .action-btn.active {
    background: var(--secondary-bg);
    color: var(--text-primary);
  }

  .goal-textarea {
    width: 100%;
    min-height: 200px;
    padding: 1rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.9375rem;
    font-family: inherit;
    line-height: 1.6;
    resize: vertical;
  }

  .goal-textarea:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 3px var(--primary-alpha);
  }

  .goal-textarea.has-error {
    border-color: var(--error-color);
  }

  .goal-textarea::placeholder {
    color: var(--text-muted);
  }

  .preview-container {
    min-height: 200px;
    padding: 1rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg);
  }

  .goal-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 0.5rem;
  }

  .help-text {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .char-count {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .char-count.warning {
    color: var(--warning-color);
  }

  .char-count.error {
    color: var(--error-color);
  }

  .error-messages {
    margin-top: 0.5rem;
  }

  .error-message {
    font-size: 0.875rem;
    color: var(--error-color);
    margin: 0.25rem 0;
  }

  .history-dropdown {
    background: var(--dropdown-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .history-item {
    display: block;
    width: 100%;
    padding: 0.5rem 0.75rem;
    text-align: left;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 0.875rem;
    cursor: pointer;
    border-radius: 4px;
  }

  .history-item:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .empty-history {
    padding: 0.5rem 0.75rem;
    color: var(--text-muted);
    font-size: 0.875rem;
    font-style: italic;
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test character counting, validation states, and event dispatch
2. **Integration Tests**: Verify template application and history functionality
3. **Markdown Tests**: Ensure markdown preview renders correctly
4. **Suggestion Tests**: Test AI suggestion fetching and application
5. **Accessibility Tests**: Validate screen reader announcements and keyboard navigation

## Related Specs
- Spec 257: Session Creation
- Spec 259: Participant Select
- Spec 275: Forge UI Tests
