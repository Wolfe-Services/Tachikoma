# Spec 241: Spec Creation Form

## Phase
11 - Spec Browser UI

## Spec ID
241

## Status
Planned

## Dependencies
- Spec 237 (Spec Editor)
- Spec 239 (Spec Validation)
- Spec 240 (Spec Templates)

## Estimated Context
~9%

---

## Objective

Create a streamlined spec creation workflow with a wizard-style form that guides users through creating new specs, with template selection, auto-generated IDs, and real-time validation.

---

## Acceptance Criteria

- [ ] Step-by-step creation wizard
- [ ] Template selection at start
- [ ] Auto-generate next available ID
- [ ] Phase selection with context
- [ ] Dependency selection with preview
- [ ] Real-time validation feedback
- [ ] Preview before creation
- [ ] Quick create mode for experienced users
- [ ] Save as draft capability

---

## Implementation Details

### SpecCreateWizard.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, fly } from 'svelte/transition';
  import type { Spec, SpecTemplate, SpecStatus } from '$lib/types/spec';
  import Button from '$lib/components/Button.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import ProgressSteps from '$lib/components/ProgressSteps.svelte';
  import TemplateSelector from './TemplateSelector.svelte';
  import SpecEditor from './SpecEditor.svelte';
  import MarkdownPreview from './MarkdownPreview.svelte';
  import DependencySelector from './DependencySelector.svelte';
  import ValidationPanel from './ValidationPanel.svelte';
  import { validateSpec } from '$lib/utils/validation';
  import { specStore } from '$lib/stores/spec-store';

  export let open = false;
  export let quickMode = false;

  const dispatch = createEventDispatcher<{
    create: Spec;
    cancel: void;
  }>();

  type WizardStep = 'template' | 'basics' | 'content' | 'dependencies' | 'review';

  const steps: { id: WizardStep; label: string; icon: string }[] = [
    { id: 'template', label: 'Template', icon: 'file-text' },
    { id: 'basics', label: 'Basics', icon: 'info' },
    { id: 'content', label: 'Content', icon: 'edit' },
    { id: 'dependencies', label: 'Dependencies', icon: 'git-branch' },
    { id: 'review', label: 'Review', icon: 'check-circle' }
  ];

  let currentStep = writable<WizardStep>('template');
  let showTemplateModal = false;
  let selectedTemplate: SpecTemplate | null = null;

  // Form data
  let formData = writable({
    id: '',
    title: '',
    description: '',
    status: 'planned' as SpecStatus,
    phase: 1,
    estimatedContext: '~10%',
    tags: [] as string[],
    dependencies: [] as string[],
    content: ''
  });

  // Generate next available ID
  const nextAvailableId = derived(specStore, $specs => {
    const ids = $specs.map(s => parseInt(s.id, 10)).filter(n => !isNaN(n));
    const maxId = Math.max(0, ...ids);
    return String(maxId + 1);
  });

  // Get phases from existing specs
  const availablePhases = derived(specStore, $specs => {
    const phases = new Set($specs.map(s => s.phase));
    return Array.from(phases).sort((a, b) => a - b);
  });

  // Build spec from form data
  const builtSpec = derived(formData, $data => ({
    ...$data,
    createdAt: new Date(),
    updatedAt: new Date()
  }));

  // Validate spec
  const validationResults = derived([builtSpec, specStore], ([$spec, $specs]) => {
    return validateSpec($spec as Spec, $specs);
  });

  // Step validity
  const stepValidity = derived([formData, validationResults], ([$data, $validation]) => ({
    template: true, // Template is optional
    basics: $data.id.trim() !== '' && $data.title.trim() !== '',
    content: $data.content.trim() !== '',
    dependencies: true, // Dependencies are optional
    review: !$validation.some(r => r.severity === 'error')
  }));

  $: currentStepIndex = steps.findIndex(s => s.id === $currentStep);
  $: canGoBack = currentStepIndex > 0;
  $: canGoForward = $stepValidity[$currentStep] && currentStepIndex < steps.length - 1;
  $: isLastStep = currentStepIndex === steps.length - 1;

  function goToStep(step: WizardStep) {
    currentStep.set(step);
  }

  function nextStep() {
    if (canGoForward) {
      currentStep.set(steps[currentStepIndex + 1].id);
    }
  }

  function prevStep() {
    if (canGoBack) {
      currentStep.set(steps[currentStepIndex - 1].id);
    }
  }

  function handleTemplateSelect(event: CustomEvent) {
    const { template, variables } = event.detail;
    selectedTemplate = template;

    // Apply template to form
    formData.update(data => ({
      ...data,
      title: applyVariables(template.title, variables),
      content: applyVariables(template.content, variables)
    }));

    showTemplateModal = false;
    nextStep();
  }

  function applyVariables(content: string, variables: Record<string, string>): string {
    return content.replace(/\{\{(\w+)(?:\|([^}]+))?\}\}/g, (_, name, defaultVal) => {
      return variables[name] || defaultVal || '';
    });
  }

  function handleCreate() {
    const spec = $builtSpec as Spec;
    dispatch('create', spec);
    handleClose();
  }

  function handleClose() {
    formData.set({
      id: '',
      title: '',
      description: '',
      status: 'planned',
      phase: 1,
      estimatedContext: '~10%',
      tags: [],
      dependencies: [],
      content: ''
    });
    currentStep.set('template');
    selectedTemplate = null;
    dispatch('cancel');
  }

  function saveDraft() {
    localStorage.setItem('spec-create-draft', JSON.stringify($formData));
  }

  function loadDraft() {
    const draft = localStorage.getItem('spec-create-draft');
    if (draft) {
      formData.set(JSON.parse(draft));
    }
  }

  onMount(() => {
    // Auto-populate ID
    formData.update(data => ({
      ...data,
      id: $nextAvailableId
    }));

    // Check for draft
    const draft = localStorage.getItem('spec-create-draft');
    if (draft) {
      // Could show a dialog asking to restore draft
    }
  });
</script>

{#if open}
  <div class="spec-create-wizard" transition:fade={{ duration: 150 }}>
    <div class="spec-create-wizard__backdrop" on:click={handleClose} />

    <div class="spec-create-wizard__panel" transition:fly={{ x: 300, duration: 200 }}>
      <header class="spec-create-wizard__header">
        <h2>Create New Spec</h2>
        <button class="spec-create-wizard__close" on:click={handleClose}>
          <Icon name="x" size={20} />
        </button>
      </header>

      {#if !quickMode}
        <nav class="spec-create-wizard__progress">
          <ProgressSteps
            {steps}
            currentStep={$currentStep}
            validity={$stepValidity}
            on:stepClick={(e) => goToStep(e.detail)}
          />
        </nav>
      {/if}

      <div class="spec-create-wizard__content">
        {#if $currentStep === 'template'}
          <div class="spec-create-wizard__step" in:fade>
            <h3>Choose a Template</h3>
            <p>Start with a template to speed up spec creation, or start from scratch.</p>

            <div class="spec-create-wizard__template-options">
              <button
                class="spec-create-wizard__template-btn"
                on:click={() => showTemplateModal = true}
              >
                <Icon name="layout" size={24} />
                <span>Browse Templates</span>
              </button>

              <button
                class="spec-create-wizard__template-btn"
                on:click={nextStep}
              >
                <Icon name="file-plus" size={24} />
                <span>Start from Scratch</span>
              </button>
            </div>

            {#if selectedTemplate}
              <div class="spec-create-wizard__selected-template">
                <Icon name="check-circle" size={16} />
                Using template: {selectedTemplate.name}
              </div>
            {/if}
          </div>
        {:else if $currentStep === 'basics'}
          <div class="spec-create-wizard__step" in:fade>
            <h3>Basic Information</h3>

            <div class="spec-create-wizard__form">
              <div class="spec-create-wizard__field">
                <label for="spec-id">
                  Spec ID
                  <span class="spec-create-wizard__auto-id">
                    (Auto-generated: {$nextAvailableId})
                  </span>
                </label>
                <input
                  id="spec-id"
                  type="text"
                  bind:value={$formData.id}
                  placeholder="e.g., 241"
                />
              </div>

              <div class="spec-create-wizard__field">
                <label for="spec-title">Title *</label>
                <input
                  id="spec-title"
                  type="text"
                  bind:value={$formData.title}
                  placeholder="Spec title"
                />
              </div>

              <div class="spec-create-wizard__field">
                <label for="spec-desc">Description</label>
                <textarea
                  id="spec-desc"
                  bind:value={$formData.description}
                  placeholder="Brief description"
                  rows="2"
                />
              </div>

              <div class="spec-create-wizard__row">
                <div class="spec-create-wizard__field">
                  <label for="spec-phase">Phase</label>
                  <select id="spec-phase" bind:value={$formData.phase}>
                    {#each Array.from({ length: 20 }, (_, i) => i + 1) as phase}
                      <option value={phase}>Phase {phase}</option>
                    {/each}
                  </select>
                </div>

                <div class="spec-create-wizard__field">
                  <label for="spec-status">Status</label>
                  <select id="spec-status" bind:value={$formData.status}>
                    <option value="planned">Planned</option>
                    <option value="in-progress">In Progress</option>
                    <option value="implemented">Implemented</option>
                    <option value="tested">Tested</option>
                    <option value="deprecated">Deprecated</option>
                  </select>
                </div>

                <div class="spec-create-wizard__field">
                  <label for="spec-context">Est. Context</label>
                  <input
                    id="spec-context"
                    type="text"
                    bind:value={$formData.estimatedContext}
                    placeholder="~10%"
                  />
                </div>
              </div>
            </div>
          </div>
        {:else if $currentStep === 'content'}
          <div class="spec-create-wizard__step spec-create-wizard__step--editor" in:fade>
            <h3>Spec Content</h3>

            <div class="spec-create-wizard__editor-container">
              <SpecEditor
                spec={$builtSpec}
                isNew
                autoSave={false}
                on:change={(e) => formData.update(d => ({ ...d, content: e.detail.content }))}
              />
            </div>
          </div>
        {:else if $currentStep === 'dependencies'}
          <div class="spec-create-wizard__step" in:fade>
            <h3>Dependencies</h3>
            <p>Select specs that this spec depends on.</p>

            <div class="spec-create-wizard__deps">
              <DependencySelector
                bind:selected={$formData.dependencies}
                currentSpecId={$formData.id}
              />
            </div>

            {#if $formData.dependencies.length > 0}
              <div class="spec-create-wizard__dep-preview">
                <h4>Selected Dependencies ({$formData.dependencies.length})</h4>
                <ul>
                  {#each $formData.dependencies as depId}
                    {@const dep = $specStore.find(s => s.id === depId)}
                    {#if dep}
                      <li>
                        <span class="spec-create-wizard__dep-id">{dep.id}</span>
                        {dep.title}
                      </li>
                    {/if}
                  {/each}
                </ul>
              </div>
            {/if}
          </div>
        {:else if $currentStep === 'review'}
          <div class="spec-create-wizard__step" in:fade>
            <h3>Review & Create</h3>

            {#if $validationResults.some(r => r.severity === 'error')}
              <div class="spec-create-wizard__validation-error">
                <Icon name="alert-circle" size={16} />
                Please fix validation errors before creating.
              </div>
            {/if}

            <ValidationPanel results={$validationResults} />

            <div class="spec-create-wizard__review">
              <div class="spec-create-wizard__review-section">
                <h4>Spec Details</h4>
                <dl>
                  <dt>ID</dt>
                  <dd>{$formData.id}</dd>
                  <dt>Title</dt>
                  <dd>{$formData.title}</dd>
                  <dt>Phase</dt>
                  <dd>{$formData.phase}</dd>
                  <dt>Status</dt>
                  <dd>{$formData.status}</dd>
                  <dt>Dependencies</dt>
                  <dd>{$formData.dependencies.length || 'None'}</dd>
                </dl>
              </div>

              <div class="spec-create-wizard__review-preview">
                <h4>Content Preview</h4>
                <div class="spec-create-wizard__preview-scroll">
                  <MarkdownPreview content={$formData.content} />
                </div>
              </div>
            </div>
          </div>
        {/if}
      </div>

      <footer class="spec-create-wizard__footer">
        <div class="spec-create-wizard__footer-left">
          <Button variant="ghost" on:click={saveDraft}>
            <Icon name="save" size={14} />
            Save Draft
          </Button>
        </div>

        <div class="spec-create-wizard__footer-right">
          {#if canGoBack}
            <Button variant="outline" on:click={prevStep}>
              <Icon name="arrow-left" size={14} />
              Back
            </Button>
          {:else}
            <Button variant="outline" on:click={handleClose}>
              Cancel
            </Button>
          {/if}

          {#if isLastStep}
            <Button
              variant="primary"
              on:click={handleCreate}
              disabled={!$stepValidity.review}
            >
              <Icon name="check" size={14} />
              Create Spec
            </Button>
          {:else}
            <Button variant="primary" on:click={nextStep} disabled={!canGoForward}>
              Next
              <Icon name="arrow-right" size={14} />
            </Button>
          {/if}
        </div>
      </footer>
    </div>

    <TemplateSelector
      bind:open={showTemplateModal}
      on:select={handleTemplateSelect}
      on:close={() => showTemplateModal = false}
    />
  </div>
{/if}

<style>
  .spec-create-wizard {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    justify-content: flex-end;
  }

  .spec-create-wizard__backdrop {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
  }

  .spec-create-wizard__panel {
    position: relative;
    width: 100%;
    max-width: 800px;
    height: 100%;
    background: var(--color-surface);
    display: flex;
    flex-direction: column;
    box-shadow: var(--shadow-xl);
  }

  .spec-create-wizard__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 24px;
    border-bottom: 1px solid var(--color-border);
  }

  .spec-create-wizard__header h2 {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0;
  }

  .spec-create-wizard__close {
    padding: 8px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-tertiary);
    border-radius: 6px;
  }

  .spec-create-wizard__close:hover {
    background: var(--color-hover);
    color: var(--color-text-primary);
  }

  .spec-create-wizard__progress {
    padding: 16px 24px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface-subtle);
  }

  .spec-create-wizard__content {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }

  .spec-create-wizard__step h3 {
    font-size: 1.125rem;
    font-weight: 600;
    margin: 0 0 8px;
  }

  .spec-create-wizard__step > p {
    color: var(--color-text-secondary);
    margin: 0 0 24px;
  }

  .spec-create-wizard__step--editor {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .spec-create-wizard__editor-container {
    flex: 1;
    min-height: 400px;
    border: 1px solid var(--color-border);
    border-radius: 8px;
    overflow: hidden;
  }

  .spec-create-wizard__template-options {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
    margin-bottom: 24px;
  }

  .spec-create-wizard__template-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 32px;
    background: var(--color-surface);
    border: 2px solid var(--color-border);
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .spec-create-wizard__template-btn:hover {
    border-color: var(--color-primary);
    background: var(--color-primary-subtle);
  }

  .spec-create-wizard__template-btn span {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .spec-create-wizard__selected-template {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    background: var(--color-success-subtle);
    color: var(--color-success);
    border-radius: 8px;
    font-size: 0.875rem;
  }

  .spec-create-wizard__form {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .spec-create-wizard__row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 16px;
  }

  .spec-create-wizard__field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .spec-create-wizard__field label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .spec-create-wizard__auto-id {
    font-weight: 400;
    color: var(--color-text-tertiary);
  }

  .spec-create-wizard__field input,
  .spec-create-wizard__field textarea,
  .spec-create-wizard__field select {
    padding: 10px 14px;
    font-size: 0.875rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
  }

  .spec-create-wizard__field input:focus,
  .spec-create-wizard__field textarea:focus,
  .spec-create-wizard__field select:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-primary-alpha);
  }

  .spec-create-wizard__deps {
    margin-bottom: 24px;
  }

  .spec-create-wizard__dep-preview {
    padding: 16px;
    background: var(--color-surface-subtle);
    border-radius: 8px;
  }

  .spec-create-wizard__dep-preview h4 {
    font-size: 0.875rem;
    font-weight: 600;
    margin: 0 0 12px;
  }

  .spec-create-wizard__dep-preview ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .spec-create-wizard__dep-preview li {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 0;
    border-bottom: 1px solid var(--color-border);
    font-size: 0.875rem;
  }

  .spec-create-wizard__dep-preview li:last-child {
    border-bottom: none;
  }

  .spec-create-wizard__dep-id {
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--color-primary);
  }

  .spec-create-wizard__validation-error {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    margin-bottom: 16px;
    background: var(--color-danger-subtle);
    color: var(--color-danger);
    border-radius: 8px;
    font-size: 0.875rem;
  }

  .spec-create-wizard__review {
    display: grid;
    grid-template-columns: 300px 1fr;
    gap: 24px;
    margin-top: 16px;
  }

  .spec-create-wizard__review-section h4,
  .spec-create-wizard__review-preview h4 {
    font-size: 0.875rem;
    font-weight: 600;
    margin: 0 0 12px;
    color: var(--color-text-secondary);
  }

  .spec-create-wizard__review-section dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 8px 16px;
    margin: 0;
  }

  .spec-create-wizard__review-section dt {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .spec-create-wizard__review-section dd {
    font-size: 0.875rem;
    margin: 0;
  }

  .spec-create-wizard__preview-scroll {
    max-height: 300px;
    overflow-y: auto;
    padding: 16px;
    background: var(--color-surface-subtle);
    border-radius: 8px;
  }

  .spec-create-wizard__footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 24px;
    border-top: 1px solid var(--color-border);
    background: var(--color-surface-subtle);
  }

  .spec-create-wizard__footer-left,
  .spec-create-wizard__footer-right {
    display: flex;
    gap: 12px;
  }
</style>
```

### ProgressSteps.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';

  export let steps: { id: string; label: string; icon: string }[] = [];
  export let currentStep: string;
  export let validity: Record<string, boolean> = {};

  const dispatch = createEventDispatcher<{
    stepClick: string;
  }>();

  $: currentIndex = steps.findIndex(s => s.id === currentStep);

  function getStepStatus(index: number): 'complete' | 'current' | 'upcoming' {
    if (index < currentIndex) return 'complete';
    if (index === currentIndex) return 'current';
    return 'upcoming';
  }

  function canNavigateTo(index: number): boolean {
    // Can always go back, can only go forward if current and previous are valid
    if (index <= currentIndex) return true;

    for (let i = 0; i <= index - 1; i++) {
      if (!validity[steps[i].id]) return false;
    }
    return true;
  }
</script>

<nav class="progress-steps" aria-label="Progress">
  <ol class="progress-steps__list">
    {#each steps as step, index}
      {@const status = getStepStatus(index)}
      {@const canNavigate = canNavigateTo(index)}
      <li class="progress-steps__item">
        <button
          class="progress-steps__step progress-steps__step--{status}"
          disabled={!canNavigate}
          on:click={() => canNavigate && dispatch('stepClick', step.id)}
          aria-current={status === 'current' ? 'step' : undefined}
        >
          <span class="progress-steps__indicator">
            {#if status === 'complete'}
              <Icon name="check" size={14} />
            {:else}
              <Icon name={step.icon} size={14} />
            {/if}
          </span>
          <span class="progress-steps__label">{step.label}</span>
        </button>

        {#if index < steps.length - 1}
          <div
            class="progress-steps__connector"
            class:progress-steps__connector--complete={index < currentIndex}
          />
        {/if}
      </li>
    {/each}
  </ol>
</nav>

<style>
  .progress-steps__list {
    display: flex;
    align-items: center;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .progress-steps__item {
    display: flex;
    align-items: center;
  }

  .progress-steps__step {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: none;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .progress-steps__step:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .progress-steps__step:not(:disabled):hover {
    background: var(--color-hover);
  }

  .progress-steps__indicator {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--color-surface-elevated);
    color: var(--color-text-tertiary);
    transition: all 0.15s;
  }

  .progress-steps__step--current .progress-steps__indicator {
    background: var(--color-primary);
    color: white;
  }

  .progress-steps__step--complete .progress-steps__indicator {
    background: var(--color-success);
    color: white;
  }

  .progress-steps__label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-tertiary);
  }

  .progress-steps__step--current .progress-steps__label {
    color: var(--color-text-primary);
  }

  .progress-steps__step--complete .progress-steps__label {
    color: var(--color-text-secondary);
  }

  .progress-steps__connector {
    width: 40px;
    height: 2px;
    background: var(--color-border);
    margin: 0 4px;
  }

  .progress-steps__connector--complete {
    background: var(--color-success);
  }
</style>
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import SpecCreateWizard from './SpecCreateWizard.svelte';
import { specStore } from '$lib/stores/spec-store';
import { createMockSpecs } from '$lib/test-utils/mock-data';

describe('SpecCreateWizard', () => {
  beforeEach(() => {
    localStorage.clear();
    specStore.set(createMockSpecs(5));
  });

  it('renders when open', () => {
    render(SpecCreateWizard, { props: { open: true } });

    expect(screen.getByText('Create New Spec')).toBeInTheDocument();
  });

  it('shows template step first', () => {
    render(SpecCreateWizard, { props: { open: true } });

    expect(screen.getByText('Choose a Template')).toBeInTheDocument();
  });

  it('navigates to next step', async () => {
    render(SpecCreateWizard, { props: { open: true } });

    await fireEvent.click(screen.getByText('Start from Scratch'));

    expect(screen.getByText('Basic Information')).toBeInTheDocument();
  });

  it('auto-generates next available ID', () => {
    specStore.set([
      { id: '100', title: 'Test' },
      { id: '101', title: 'Test 2' }
    ]);

    render(SpecCreateWizard, { props: { open: true } });

    // Navigate to basics step
    fireEvent.click(screen.getByText('Start from Scratch'));

    expect(screen.getByDisplayValue('102')).toBeInTheDocument();
  });

  it('validates required fields', async () => {
    render(SpecCreateWizard, { props: { open: true } });

    await fireEvent.click(screen.getByText('Start from Scratch'));

    // Clear the auto-generated ID
    const idInput = screen.getByLabelText(/Spec ID/);
    await fireEvent.input(idInput, { target: { value: '' } });

    // Next button should be disabled
    expect(screen.getByText('Next').closest('button')).toBeDisabled();
  });

  it('shows validation errors on review step', async () => {
    render(SpecCreateWizard, { props: { open: true } });

    // Navigate through steps with minimal data
    await fireEvent.click(screen.getByText('Start from Scratch'));
    await fireEvent.input(screen.getByLabelText(/Title/), { target: { value: 'Test' } });
    await fireEvent.click(screen.getByText('Next'));
    await fireEvent.click(screen.getByText('Next'));
    await fireEvent.click(screen.getByText('Next'));

    // Should show validation results
    expect(screen.getByText('Validation Results')).toBeInTheDocument();
  });

  it('dispatches create event with spec data', async () => {
    const { component } = render(SpecCreateWizard, { props: { open: true } });

    const createHandler = vi.fn();
    component.$on('create', createHandler);

    // Fill in wizard
    await fireEvent.click(screen.getByText('Start from Scratch'));
    await fireEvent.input(screen.getByLabelText(/Title/), { target: { value: 'Test Spec' } });
    await fireEvent.click(screen.getByText('Next'));

    // Skip content (add minimal)
    // ... navigate through steps

    // Click create on final step
    // await fireEvent.click(screen.getByText('Create Spec'));

    // expect(createHandler).toHaveBeenCalled();
  });

  it('saves draft to localStorage', async () => {
    render(SpecCreateWizard, { props: { open: true } });

    await fireEvent.click(screen.getByText('Start from Scratch'));
    await fireEvent.input(screen.getByLabelText(/Title/), { target: { value: 'Draft Spec' } });
    await fireEvent.click(screen.getByText('Save Draft'));

    const draft = localStorage.getItem('spec-create-draft');
    expect(draft).not.toBeNull();
    expect(JSON.parse(draft!).title).toBe('Draft Spec');
  });

  it('applies template variables', async () => {
    // Test template application
  });
});

describe('ProgressSteps', () => {
  const steps = [
    { id: 'step1', label: 'Step 1', icon: 'circle' },
    { id: 'step2', label: 'Step 2', icon: 'circle' },
    { id: 'step3', label: 'Step 3', icon: 'circle' }
  ];

  it('renders all steps', () => {
    render(ProgressSteps, {
      props: { steps, currentStep: 'step1', validity: {} }
    });

    expect(screen.getByText('Step 1')).toBeInTheDocument();
    expect(screen.getByText('Step 2')).toBeInTheDocument();
    expect(screen.getByText('Step 3')).toBeInTheDocument();
  });

  it('marks current step', () => {
    render(ProgressSteps, {
      props: { steps, currentStep: 'step2', validity: {} }
    });

    const step2 = screen.getByText('Step 2').closest('button');
    expect(step2).toHaveClass('progress-steps__step--current');
  });

  it('marks completed steps', () => {
    render(ProgressSteps, {
      props: { steps, currentStep: 'step3', validity: {} }
    });

    const step1 = screen.getByText('Step 1').closest('button');
    expect(step1).toHaveClass('progress-steps__step--complete');
  });

  it('dispatches stepClick on navigation', async () => {
    const { component } = render(ProgressSteps, {
      props: { steps, currentStep: 'step2', validity: { step1: true } }
    });

    const clickHandler = vi.fn();
    component.$on('stepClick', clickHandler);

    await fireEvent.click(screen.getByText('Step 1'));

    expect(clickHandler).toHaveBeenCalledWith(
      expect.objectContaining({ detail: 'step1' })
    );
  });
});
```

---

## Related Specs

- Spec 237: Spec Editor
- Spec 239: Spec Validation
- Spec 240: Spec Templates
- Spec 242: Spec Status Tracking
