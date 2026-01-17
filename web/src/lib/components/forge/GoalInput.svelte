<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { marked } from 'marked';
  import GoalRefinementChat from './GoalRefinementChat.svelte';

  export let value: string = '';
  export let name: string = '';
  export let errors: string[] = [];

  const dispatch = createEventDispatcher<{
    goalChange: string;
    nameChange: string;
  }>();

  let showPreview = false;
  let showRefinement = false;
  let previewHtml = '';

  function handleGoalInput(event: Event) {
    const target = event.target as HTMLTextAreaElement;
    const newValue = target.value;
    value = newValue;
    dispatch('goalChange', newValue);
    updatePreview(newValue);
  }

  function handleNameInput(event: Event) {
    const target = event.target as HTMLInputElement;
    const newValue = target.value;
    name = newValue;
    dispatch('nameChange', newValue);
  }

  function togglePreview() {
    showPreview = !showPreview;
    if (showPreview) {
      updatePreview(value);
    }
  }

  function toggleRefinement() {
    showRefinement = !showRefinement;
    // Close preview when opening refinement
    if (showRefinement && showPreview) {
      showPreview = false;
    }
  }

  function handleRefinementClose() {
    showRefinement = false;
  }

  function handleApplySuggestions(event: CustomEvent<string>) {
    const newGoal = event.detail;
    value = newGoal;
    dispatch('goalChange', newGoal);
    updatePreview(newGoal);
    showRefinement = false;
  }

  function updatePreview(content: string) {
    if (content.trim()) {
      previewHtml = marked(content);
    } else {
      previewHtml = '<p><em>No content to preview</em></p>';
    }
  }

  // Initialize preview if value exists
  if (value) {
    updatePreview(value);
  }
</script>

<div class="goal-input-step" data-testid="goal-input-step">
  <div class="step-header">
    <h2>Define Session Goal</h2>
    <p class="step-description">
      What do you want to achieve with this deliberation session? Be specific about your objectives, constraints, and success criteria.
    </p>
  </div>

  <div class="form-group">
    <label for="session-name" class="form-label">
      Session Name
      <span class="required">*</span>
    </label>
    <input
      id="session-name"
      type="text"
      class="form-input"
      bind:value={name}
      on:input={handleNameInput}
      placeholder="e.g., Q4 Product Strategy Session"
      data-testid="session-name-input"
    />
  </div>

  <div class="form-group">
    <div class="goal-header">
      <label for="session-goal" class="form-label">
        Goal Description
        <span class="required">*</span>
      </label>
      <div class="goal-controls">
        <button
          type="button"
          class="refine-toggle"
          class:active={showRefinement}
          on:click={toggleRefinement}
          data-testid="refine-toggle"
          title="Refine your goal with AI assistance"
        >
          ✨ Refine with AI
        </button>
        <span class="char-count" class:warning={value.length > 4500}>
          {value.length}/5000
        </span>
        <button
          type="button"
          class="preview-toggle"
          class:active={showPreview}
          on:click={togglePreview}
          data-testid="preview-toggle"
        >
          {showPreview ? 'Edit' : 'Preview'}
        </button>
      </div>
    </div>

    <div class="goal-container">
      {#if showPreview}
        <div 
          class="goal-preview" 
          data-testid="goal-preview"
        >
          {@html previewHtml}
        </div>
      {:else}
        <textarea
          id="session-goal"
          class="goal-textarea"
          bind:value
          on:input={handleGoalInput}
          placeholder="Describe your session goal in detail. You can use markdown formatting:

**Key Questions:**
- What decisions need to be made?
- What constraints should we consider?
- What are the success criteria?

**Background:**
Provide relevant context...

**Expected Outcomes:**
- Specific deliverables
- Action items
- Next steps"
          data-testid="goal-textarea"
        ></textarea>
      {/if}
    </div>

    <div class="markdown-hint">
      <span class="hint-icon">💡</span>
      Supports Markdown formatting: **bold**, *italic*, lists, links, etc.
    </div>
  </div>

  {#if showRefinement}
    <GoalRefinementChat
      currentGoal={value}
      on:close={handleRefinementClose}
      on:applySuggestions={handleApplySuggestions}
    />
  {/if}

  {#if errors.length > 0}
    <div class="error-list" role="alert">
      {#each errors as error}
        <div class="error-item">
          <span class="error-icon">⚠️</span>
          {error}
        </div>
      {/each}
    </div>
  {/if}

  <div class="tips-section">
    <h3>💡 Tips for Writing Effective Goals</h3>
    <ul class="tips-list">
      <li><strong>Be Specific:</strong> Clear, concrete objectives lead to better outcomes</li>
      <li><strong>Include Context:</strong> Provide background information and constraints</li>
      <li><strong>Define Success:</strong> What would a successful session look like?</li>
      <li><strong>Set Scope:</strong> What's in scope vs. out of scope for this session?</li>
    </ul>
  </div>
</div>

<style>
  .goal-input-step {
    max-width: 700px;
    margin: 0 auto;
  }

  .step-header {
    margin-bottom: 2rem;
  }

  .step-header h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
    color: var(--text-primary);
  }

  .step-description {
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .form-group {
    margin-bottom: 1.5rem;
  }

  .form-label {
    display: block;
    font-weight: 500;
    margin-bottom: 0.5rem;
    color: var(--text-primary);
  }

  .required {
    color: var(--error-color, #ef4444);
  }

  .form-input {
    width: 100%;
    padding: 0.875rem 1rem;
    border: 1px solid var(--input-border);
    border-radius: 8px;
    font-size: 1rem;
    background: var(--input-bg);
    color: var(--input-text);
    transition: all 0.2s ease;
  }

  .form-input::placeholder {
    color: var(--text-placeholder);
  }

  .form-input:focus {
    outline: none;
    background: var(--input-bg-focus);
    border-color: var(--border-focus);
    box-shadow: 0 0 0 3px var(--primary-color-alpha);
  }

  .goal-header {
    display: flex;
    justify-content: space-between;
    align-items: end;
    margin-bottom: 0.5rem;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .goal-controls {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .refine-toggle {
    padding: 0.375rem 0.875rem;
    border: 1px solid rgba(78, 205, 196, 0.35);
    border-radius: 6px;
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.12), rgba(78, 205, 196, 0.04));
    color: var(--tachi-cyan, #4ecdc4);
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .refine-toggle:hover {
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.2), rgba(78, 205, 196, 0.08));
    box-shadow: 0 0 12px rgba(78, 205, 196, 0.15);
  }

  .refine-toggle.active {
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.35), rgba(78, 205, 196, 0.15));
    border-color: rgba(78, 205, 196, 0.5);
    box-shadow: 0 0 15px rgba(78, 205, 196, 0.2);
  }

  .char-count {
    font-size: 0.75rem;
    color: var(--text-muted);
    font-family: monospace;
  }

  .char-count.warning {
    color: var(--warning-color, #f59e0b);
  }

  .preview-toggle {
    padding: 0.25rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--secondary-bg);
    color: var(--text-secondary);
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .preview-toggle:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .preview-toggle.active {
    background: var(--primary-color);
    color: white;
    border-color: var(--primary-color);
  }

  .goal-container {
    position: relative;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    overflow: hidden;
  }

  .goal-textarea {
    width: 100%;
    min-height: 300px;
    padding: 1.25rem;
    border: none;
    background: var(--input-bg);
    color: var(--input-text);
    font-family: inherit;
    font-size: 0.95rem;
    line-height: 1.6;
    resize: vertical;
  }

  .goal-textarea::placeholder {
    color: var(--text-placeholder);
  }

  .goal-textarea:focus {
    outline: none;
    background: var(--input-bg-focus);
  }

  .goal-preview {
    min-height: 300px;
    padding: 1rem;
    background: var(--secondary-bg);
    color: var(--text-primary);
    font-size: 0.9rem;
    line-height: 1.5;
    overflow-y: auto;
  }

  :global(.goal-preview h1),
  :global(.goal-preview h2),
  :global(.goal-preview h3) {
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
  }

  :global(.goal-preview h1:first-child),
  :global(.goal-preview h2:first-child),
  :global(.goal-preview h3:first-child) {
    margin-top: 0;
  }

  :global(.goal-preview ul),
  :global(.goal-preview ol) {
    margin: 1rem 0;
    padding-left: 2rem;
  }

  :global(.goal-preview p) {
    margin: 1rem 0;
  }

  :global(.goal-preview p:first-child) {
    margin-top: 0;
  }

  :global(.goal-preview p:last-child) {
    margin-bottom: 0;
  }

  :global(.goal-preview code) {
    background: var(--code-bg, #f3f4f6);
    padding: 0.125rem 0.25rem;
    border-radius: 3px;
    font-size: 0.875rem;
  }

  :global(.goal-preview pre) {
    background: var(--code-bg, #f3f4f6);
    padding: 1rem;
    border-radius: 6px;
    overflow-x: auto;
    margin: 1rem 0;
  }

  .markdown-hint {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.5rem;
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .hint-icon {
    font-size: 1rem;
  }

  .error-list {
    margin-top: 0.5rem;
  }

  .error-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--error-bg, #fef2f2);
    color: var(--error-color, #dc2626);
    border-radius: 4px;
    margin-bottom: 0.5rem;
    font-size: 0.875rem;
  }

  .error-icon {
    flex-shrink: 0;
  }

  .tips-section {
    margin-top: 2rem;
    padding: 1.25rem 1.5rem;
    background: var(--info-bg);
    border-radius: 10px;
    border: 1px solid rgba(14, 165, 233, 0.15);
    border-left: 3px solid var(--info-color);
  }

  .tips-section h3 {
    margin: 0 0 1rem 0;
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .tips-list {
    margin: 0;
    padding-left: 1.25rem;
    list-style-type: none;
  }

  .tips-list li {
    margin-bottom: 0.6rem;
    color: var(--text-secondary);
    line-height: 1.4;
    position: relative;
    font-size: 0.875rem;
  }

  .tips-list li::before {
    content: '→';
    position: absolute;
    left: -1.25rem;
    color: var(--info-color);
    font-weight: bold;
  }

  .tips-list li strong {
    color: var(--text-primary);
  }

  /* Goal container border color */
  .goal-container {
    border-color: var(--input-border);
  }

  .goal-container:focus-within {
    border-color: var(--border-focus);
    box-shadow: 0 0 0 3px var(--primary-color-alpha);
  }

  /* Code blocks in preview */
  :global(.goal-preview code) {
    background: var(--code-bg);
    color: #e879f9;
  }

  :global(.goal-preview pre) {
    background: var(--code-bg);
    border: 1px solid var(--border-color);
  }
</style>
