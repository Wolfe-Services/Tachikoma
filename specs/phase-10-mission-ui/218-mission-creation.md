# 218 - Mission Creation Dialog

**Phase:** 10 - Mission Panel UI
**Spec ID:** 218
**Status:** Planned
**Dependencies:** 217-mission-state, 219-prompt-editor, 220-spec-selector, 221-backend-selector
**Estimated Context:** ~14% of Sonnet window

---

## Objective

Create a comprehensive mission creation dialog that allows users to configure all aspects of a new mission, including prompt input, spec selection, backend choice, and execution mode settings.

---

## Acceptance Criteria

- [ ] Modal dialog for mission creation
- [ ] Multi-step wizard for complex configuration
- [ ] Form validation with inline error messages
- [ ] Template selection for common mission types
- [ ] Preview of mission configuration before creation
- [ ] Keyboard navigation through form fields
- [ ] Save as draft functionality

---

## Implementation Details

### 1. Types (src/lib/types/mission-creation.ts)

```typescript
/**
 * Types for mission creation workflow.
 */

export interface MissionTemplate {
  id: string;
  name: string;
  description: string;
  icon: string;
  defaultPrompt: string;
  suggestedSpecs: string[];
  suggestedMode: 'agentic' | 'interactive';
  category: TemplateCategory;
}

export type TemplateCategory =
  | 'feature'
  | 'bugfix'
  | 'refactor'
  | 'test'
  | 'documentation'
  | 'custom';

export interface MissionDraft {
  id: string;
  title: string;
  prompt: string;
  specIds: string[];
  backendId: string;
  mode: 'agentic' | 'interactive';
  tags: string[];
  templateId?: string;
  createdAt: string;
  updatedAt: string;
}

export interface MissionCreationStep {
  id: string;
  title: string;
  description: string;
  isComplete: boolean;
  isValid: boolean;
  errors: string[];
}

export interface MissionCreationState {
  currentStep: number;
  steps: MissionCreationStep[];
  draft: MissionDraft;
  isSubmitting: boolean;
  submitError: string | null;
}

export interface ValidationResult {
  isValid: boolean;
  errors: Record<string, string[]>;
}

export const DEFAULT_TEMPLATES: MissionTemplate[] = [
  {
    id: 'new-feature',
    name: 'New Feature',
    description: 'Implement a new feature from scratch',
    icon: 'sparkles',
    defaultPrompt: 'Implement the following feature:\n\n',
    suggestedSpecs: [],
    suggestedMode: 'agentic',
    category: 'feature',
  },
  {
    id: 'bug-fix',
    name: 'Bug Fix',
    description: 'Fix an existing bug or issue',
    icon: 'bug',
    defaultPrompt: 'Fix the following bug:\n\nDescription:\nSteps to reproduce:\nExpected behavior:\n',
    suggestedSpecs: [],
    suggestedMode: 'interactive',
    category: 'bugfix',
  },
  {
    id: 'refactor',
    name: 'Refactor',
    description: 'Improve existing code structure',
    icon: 'wrench',
    defaultPrompt: 'Refactor the following code:\n\nGoals:\n- \n\nConstraints:\n- \n',
    suggestedSpecs: [],
    suggestedMode: 'agentic',
    category: 'refactor',
  },
  {
    id: 'write-tests',
    name: 'Write Tests',
    description: 'Add tests for existing code',
    icon: 'beaker',
    defaultPrompt: 'Write tests for:\n\nCoverage goals:\n- Unit tests\n- Integration tests\n',
    suggestedSpecs: [],
    suggestedMode: 'agentic',
    category: 'test',
  },
];
```

### 2. Creation Store (src/lib/stores/mission-creation-store.ts)

```typescript
import { writable, derived, get } from 'svelte/store';
import type {
  MissionCreationState,
  MissionDraft,
  MissionCreationStep,
  ValidationResult,
  MissionTemplate,
} from '$lib/types/mission-creation';
import { missionStore } from './mission-store';

function createEmptyDraft(): MissionDraft {
  return {
    id: `draft-${Date.now()}`,
    title: '',
    prompt: '',
    specIds: [],
    backendId: '',
    mode: 'agentic',
    tags: [],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
}

function createSteps(): MissionCreationStep[] {
  return [
    {
      id: 'template',
      title: 'Choose Template',
      description: 'Select a template or start from scratch',
      isComplete: false,
      isValid: true,
      errors: [],
    },
    {
      id: 'prompt',
      title: 'Define Mission',
      description: 'Enter mission title and prompt',
      isComplete: false,
      isValid: false,
      errors: [],
    },
    {
      id: 'specs',
      title: 'Select Specs',
      description: 'Choose relevant specifications',
      isComplete: false,
      isValid: true,
      errors: [],
    },
    {
      id: 'config',
      title: 'Configure',
      description: 'Set backend and execution mode',
      isComplete: false,
      isValid: false,
      errors: [],
    },
    {
      id: 'review',
      title: 'Review',
      description: 'Review and confirm mission',
      isComplete: false,
      isValid: true,
      errors: [],
    },
  ];
}

function createMissionCreationStore() {
  const initialState: MissionCreationState = {
    currentStep: 0,
    steps: createSteps(),
    draft: createEmptyDraft(),
    isSubmitting: false,
    submitError: null,
  };

  const { subscribe, set, update } = writable<MissionCreationState>(initialState);

  function validateStep(stepId: string, draft: MissionDraft): ValidationResult {
    const errors: Record<string, string[]> = {};

    switch (stepId) {
      case 'prompt':
        if (!draft.title.trim()) {
          errors.title = ['Title is required'];
        } else if (draft.title.length < 3) {
          errors.title = ['Title must be at least 3 characters'];
        }
        if (!draft.prompt.trim()) {
          errors.prompt = ['Prompt is required'];
        } else if (draft.prompt.length < 10) {
          errors.prompt = ['Prompt must be at least 10 characters'];
        }
        break;

      case 'config':
        if (!draft.backendId) {
          errors.backendId = ['Please select a backend'];
        }
        break;
    }

    return {
      isValid: Object.keys(errors).length === 0,
      errors,
    };
  }

  return {
    subscribe,

    reset() {
      set(initialState);
    },

    applyTemplate(template: MissionTemplate) {
      update(state => ({
        ...state,
        draft: {
          ...state.draft,
          prompt: template.defaultPrompt,
          specIds: template.suggestedSpecs,
          mode: template.suggestedMode,
          templateId: template.id,
          updatedAt: new Date().toISOString(),
        },
        steps: state.steps.map((step, i) =>
          i === 0 ? { ...step, isComplete: true } : step
        ),
        currentStep: 1,
      }));
    },

    skipTemplate() {
      update(state => ({
        ...state,
        steps: state.steps.map((step, i) =>
          i === 0 ? { ...step, isComplete: true } : step
        ),
        currentStep: 1,
      }));
    },

    updateDraft(updates: Partial<MissionDraft>) {
      update(state => {
        const draft = {
          ...state.draft,
          ...updates,
          updatedAt: new Date().toISOString(),
        };

        // Re-validate current step
        const currentStepId = state.steps[state.currentStep].id;
        const validation = validateStep(currentStepId, draft);
        const steps = state.steps.map((step, i) =>
          i === state.currentStep
            ? { ...step, isValid: validation.isValid, errors: Object.values(validation.errors).flat() }
            : step
        );

        return { ...state, draft, steps };
      });
    },

    nextStep() {
      update(state => {
        const currentStepId = state.steps[state.currentStep].id;
        const validation = validateStep(currentStepId, state.draft);

        if (!validation.isValid) {
          const steps = state.steps.map((step, i) =>
            i === state.currentStep
              ? { ...step, isValid: false, errors: Object.values(validation.errors).flat() }
              : step
          );
          return { ...state, steps };
        }

        const steps = state.steps.map((step, i) =>
          i === state.currentStep ? { ...step, isComplete: true, isValid: true, errors: [] } : step
        );

        return {
          ...state,
          steps,
          currentStep: Math.min(state.currentStep + 1, state.steps.length - 1),
        };
      });
    },

    previousStep() {
      update(state => ({
        ...state,
        currentStep: Math.max(state.currentStep - 1, 0),
      }));
    },

    goToStep(stepIndex: number) {
      update(state => {
        // Can only go to completed steps or next incomplete step
        const canGo = stepIndex <= state.steps.findIndex(s => !s.isComplete) ||
                      state.steps.every(s => s.isComplete);
        if (!canGo) return state;
        return { ...state, currentStep: stepIndex };
      });
    },

    async submit(): Promise<boolean> {
      const state = get({ subscribe });

      // Validate all steps
      for (const step of state.steps.slice(0, -1)) {
        const validation = validateStep(step.id, state.draft);
        if (!validation.isValid) {
          update(s => ({ ...s, submitError: `Please complete step: ${step.title}` }));
          return false;
        }
      }

      update(s => ({ ...s, isSubmitting: true, submitError: null }));

      try {
        const mission = await missionStore.createMission({
          title: state.draft.title,
          prompt: state.draft.prompt,
          specIds: state.draft.specIds,
          backendId: state.draft.backendId,
          mode: state.draft.mode,
          tags: state.draft.tags,
        });

        if (mission) {
          set(initialState);
          return true;
        } else {
          update(s => ({ ...s, isSubmitting: false, submitError: 'Failed to create mission' }));
          return false;
        }
      } catch (error) {
        update(s => ({
          ...s,
          isSubmitting: false,
          submitError: error instanceof Error ? error.message : 'Unknown error',
        }));
        return false;
      }
    },

    saveDraft() {
      const state = get({ subscribe });
      localStorage.setItem(`mission-draft-${state.draft.id}`, JSON.stringify(state.draft));
    },

    loadDraft(draftId: string) {
      const saved = localStorage.getItem(`mission-draft-${draftId}`);
      if (saved) {
        const draft = JSON.parse(saved) as MissionDraft;
        update(state => ({ ...state, draft }));
      }
    },
  };
}

export const missionCreationStore = createMissionCreationStore();

export const isCreationValid = derived(missionCreationStore, $state =>
  $state.steps.every(step => step.isValid)
);

export const currentStep = derived(missionCreationStore, $state =>
  $state.steps[$state.currentStep]
);
```

### 3. Mission Creation Dialog (src/lib/components/mission/MissionCreationDialog.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { missionCreationStore, currentStep, isCreationValid } from '$lib/stores/mission-creation-store';
  import { DEFAULT_TEMPLATES } from '$lib/types/mission-creation';
  import TemplateSelector from './TemplateSelector.svelte';
  import PromptEditor from './PromptEditor.svelte';
  import SpecSelector from './SpecSelector.svelte';
  import BackendSelector from './BackendSelector.svelte';
  import ModeToggle from './ModeToggle.svelte';
  import MissionReview from './MissionReview.svelte';

  export let open = false;

  const dispatch = createEventDispatcher<{
    close: void;
    created: { missionId: string };
  }>();

  function handleClose() {
    missionCreationStore.saveDraft();
    dispatch('close');
  }

  async function handleSubmit() {
    const success = await missionCreationStore.submit();
    if (success) {
      dispatch('close');
    }
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleClose();
    }
  }

  $: stepId = $currentStep?.id;
</script>

<svelte:window on:keydown={handleKeyDown} />

{#if open}
  <div class="dialog-overlay" transition:fade={{ duration: 150 }} on:click={handleClose}>
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="dialog-title"
      transition:fly={{ y: 20, duration: 200 }}
      on:click|stopPropagation
    >
      <!-- Header -->
      <header class="dialog__header">
        <h2 id="dialog-title" class="dialog__title">Create Mission</h2>
        <button
          class="dialog__close"
          on:click={handleClose}
          aria-label="Close dialog"
        >
          <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
            <path d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" />
          </svg>
        </button>
      </header>

      <!-- Step Indicator -->
      <nav class="dialog__steps" aria-label="Creation steps">
        {#each $missionCreationStore.steps as step, index}
          <button
            class="step-indicator"
            class:step-indicator--active={index === $missionCreationStore.currentStep}
            class:step-indicator--complete={step.isComplete}
            class:step-indicator--error={!step.isValid && step.errors.length > 0}
            disabled={index > $missionCreationStore.steps.findIndex(s => !s.isComplete)}
            on:click={() => missionCreationStore.goToStep(index)}
            aria-current={index === $missionCreationStore.currentStep ? 'step' : undefined}
          >
            <span class="step-indicator__number">{index + 1}</span>
            <span class="step-indicator__title">{step.title}</span>
          </button>
        {/each}
      </nav>

      <!-- Content -->
      <div class="dialog__content">
        {#if stepId === 'template'}
          <TemplateSelector
            templates={DEFAULT_TEMPLATES}
            on:select={(e) => missionCreationStore.applyTemplate(e.detail)}
            on:skip={() => missionCreationStore.skipTemplate()}
          />
        {:else if stepId === 'prompt'}
          <div class="form-group">
            <label for="mission-title" class="form-label">Mission Title</label>
            <input
              id="mission-title"
              type="text"
              class="form-input"
              placeholder="Enter a descriptive title"
              value={$missionCreationStore.draft.title}
              on:input={(e) => missionCreationStore.updateDraft({ title: e.currentTarget.value })}
            />
          </div>
          <div class="form-group">
            <label for="mission-prompt" class="form-label">Mission Prompt</label>
            <PromptEditor
              value={$missionCreationStore.draft.prompt}
              on:change={(e) => missionCreationStore.updateDraft({ prompt: e.detail })}
            />
          </div>
        {:else if stepId === 'specs'}
          <SpecSelector
            selectedIds={$missionCreationStore.draft.specIds}
            on:change={(e) => missionCreationStore.updateDraft({ specIds: e.detail })}
          />
        {:else if stepId === 'config'}
          <div class="config-section">
            <h3 class="config-section__title">Backend</h3>
            <BackendSelector
              selectedId={$missionCreationStore.draft.backendId}
              on:change={(e) => missionCreationStore.updateDraft({ backendId: e.detail })}
            />
          </div>
          <div class="config-section">
            <h3 class="config-section__title">Execution Mode</h3>
            <ModeToggle
              mode={$missionCreationStore.draft.mode}
              on:change={(e) => missionCreationStore.updateDraft({ mode: e.detail })}
            />
          </div>
          <div class="config-section">
            <h3 class="config-section__title">Tags</h3>
            <input
              type="text"
              class="form-input"
              placeholder="Add tags (comma separated)"
              value={$missionCreationStore.draft.tags.join(', ')}
              on:blur={(e) => {
                const tags = e.currentTarget.value
                  .split(',')
                  .map(t => t.trim())
                  .filter(Boolean);
                missionCreationStore.updateDraft({ tags });
              }}
            />
          </div>
        {:else if stepId === 'review'}
          <MissionReview draft={$missionCreationStore.draft} />
        {/if}

        <!-- Errors -->
        {#if $currentStep?.errors.length}
          <div class="form-errors" role="alert">
            {#each $currentStep.errors as error}
              <p class="form-error">{error}</p>
            {/each}
          </div>
        {/if}

        {#if $missionCreationStore.submitError}
          <div class="form-errors" role="alert">
            <p class="form-error">{$missionCreationStore.submitError}</p>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <footer class="dialog__footer">
        <button
          class="btn btn--secondary"
          on:click={() => missionCreationStore.saveDraft()}
        >
          Save Draft
        </button>
        <div class="dialog__footer-actions">
          {#if $missionCreationStore.currentStep > 0}
            <button
              class="btn btn--ghost"
              on:click={() => missionCreationStore.previousStep()}
            >
              Back
            </button>
          {/if}
          {#if $missionCreationStore.currentStep < $missionCreationStore.steps.length - 1}
            <button
              class="btn btn--primary"
              on:click={() => missionCreationStore.nextStep()}
            >
              Next
            </button>
          {:else}
            <button
              class="btn btn--primary"
              disabled={!$isCreationValid || $missionCreationStore.isSubmitting}
              on:click={handleSubmit}
            >
              {#if $missionCreationStore.isSubmitting}
                Creating...
              {:else}
                Create Mission
              {/if}
            </button>
          {/if}
        </div>
      </footer>
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: var(--color-bg-primary);
    border-radius: 12px;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
    width: 90%;
    max-width: 720px;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .dialog__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--color-border);
  }

  .dialog__title {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .dialog__close {
    padding: 8px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    cursor: pointer;
    border-radius: 6px;
    transition: all 0.15s ease;
  }

  .dialog__close:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .dialog__steps {
    display: flex;
    padding: 16px 20px;
    gap: 8px;
    border-bottom: 1px solid var(--color-border);
    overflow-x: auto;
  }

  .step-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 13px;
    cursor: pointer;
    border-radius: 6px;
    transition: all 0.15s ease;
    white-space: nowrap;
  }

  .step-indicator:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .step-indicator:not(:disabled):hover {
    background: var(--color-bg-hover);
  }

  .step-indicator--active {
    background: var(--color-bg-active);
    color: var(--color-primary);
  }

  .step-indicator--complete .step-indicator__number {
    background: var(--color-success);
    color: white;
  }

  .step-indicator--error .step-indicator__number {
    background: var(--color-error);
    color: white;
  }

  .step-indicator__number {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: var(--color-bg-hover);
    font-size: 12px;
    font-weight: 600;
  }

  .dialog__content {
    flex: 1;
    padding: 20px;
    overflow-y: auto;
  }

  .dialog__footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .dialog__footer-actions {
    display: flex;
    gap: 8px;
  }

  .form-group {
    margin-bottom: 20px;
  }

  .form-label {
    display: block;
    margin-bottom: 6px;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .form-input {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 14px;
    transition: border-color 0.15s ease;
  }

  .form-input:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px var(--color-focus-ring);
  }

  .form-errors {
    margin-top: 16px;
    padding: 12px;
    background: rgba(244, 67, 54, 0.1);
    border-radius: 6px;
  }

  .form-error {
    color: var(--color-error);
    font-size: 13px;
    margin: 0;
  }

  .config-section {
    margin-bottom: 24px;
  }

  .config-section__title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 12px 0;
  }

  .btn {
    padding: 10px 16px;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn--primary {
    background: var(--color-primary);
    color: white;
  }

  .btn--primary:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn--primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn--secondary {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .btn--ghost {
    background: transparent;
    color: var(--color-text-secondary);
  }

  .btn--ghost:hover {
    background: var(--color-bg-hover);
  }
</style>
```

### 4. Template Selector (src/lib/components/mission/TemplateSelector.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { MissionTemplate } from '$lib/types/mission-creation';

  export let templates: MissionTemplate[];

  const dispatch = createEventDispatcher<{
    select: MissionTemplate;
    skip: void;
  }>();

  const icons: Record<string, string> = {
    sparkles: '✨',
    bug: '🐛',
    wrench: '🔧',
    beaker: '🧪',
    document: '📄',
  };
</script>

<div class="template-selector">
  <p class="template-selector__hint">Choose a template to get started or skip to create from scratch.</p>

  <div class="template-grid">
    {#each templates as template}
      <button
        class="template-card"
        on:click={() => dispatch('select', template)}
      >
        <span class="template-card__icon">{icons[template.icon] || '📋'}</span>
        <h3 class="template-card__name">{template.name}</h3>
        <p class="template-card__description">{template.description}</p>
        <span class="template-card__mode">{template.suggestedMode}</span>
      </button>
    {/each}
  </div>

  <button class="skip-button" on:click={() => dispatch('skip')}>
    Skip and start from scratch
  </button>
</div>

<style>
  .template-selector {
    text-align: center;
  }

  .template-selector__hint {
    color: var(--color-text-secondary);
    margin-bottom: 24px;
  }

  .template-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 16px;
    margin-bottom: 24px;
  }

  .template-card {
    padding: 20px;
    border: 1px solid var(--color-border);
    border-radius: 8px;
    background: var(--color-bg-primary);
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
  }

  .template-card:hover {
    border-color: var(--color-primary);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .template-card__icon {
    font-size: 24px;
    display: block;
    margin-bottom: 12px;
  }

  .template-card__name {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 8px 0;
  }

  .template-card__description {
    font-size: 13px;
    color: var(--color-text-secondary);
    margin: 0 0 12px 0;
  }

  .template-card__mode {
    display: inline-block;
    padding: 4px 8px;
    background: var(--color-bg-hover);
    border-radius: 4px;
    font-size: 11px;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .skip-button {
    padding: 10px 20px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 14px;
    cursor: pointer;
    text-decoration: underline;
  }

  .skip-button:hover {
    color: var(--color-primary);
  }
</style>
```

---

## Testing Requirements

1. Dialog opens and closes correctly
2. Step navigation works properly
3. Form validation prevents progression with invalid data
4. Template selection populates draft correctly
5. Draft saving and loading works
6. Mission creation calls store correctly
7. Error states display appropriately

### Test File (src/lib/components/mission/__tests__/MissionCreationDialog.test.ts)

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import MissionCreationDialog from '../MissionCreationDialog.svelte';
import { missionCreationStore } from '$lib/stores/mission-creation-store';

describe('MissionCreationDialog', () => {
  beforeEach(() => {
    missionCreationStore.reset();
  });

  it('renders when open', () => {
    render(MissionCreationDialog, { open: true });
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByText('Create Mission')).toBeInTheDocument();
  });

  it('does not render when closed', () => {
    render(MissionCreationDialog, { open: false });
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('closes on escape key', async () => {
    const { component } = render(MissionCreationDialog, { open: true });
    const handler = vi.fn();
    component.$on('close', handler);

    await fireEvent.keyDown(window, { key: 'Escape' });
    expect(handler).toHaveBeenCalled();
  });

  it('navigates through steps', async () => {
    render(MissionCreationDialog, { open: true });

    // Skip template
    await fireEvent.click(screen.getByText('Skip and start from scratch'));

    // Fill prompt step
    const titleInput = screen.getByLabelText('Mission Title');
    await fireEvent.input(titleInput, { target: { value: 'Test Mission' } });

    // Should now be on prompt step
    expect(screen.getByText('Define Mission')).toBeInTheDocument();
  });

  it('validates required fields', async () => {
    render(MissionCreationDialog, { open: true });

    // Skip template
    await fireEvent.click(screen.getByText('Skip and start from scratch'));

    // Try to proceed without filling required fields
    await fireEvent.click(screen.getByText('Next'));

    // Should show validation errors
    await waitFor(() => {
      expect(screen.getByText('Title is required')).toBeInTheDocument();
    });
  });
});
```

---

## Related Specs

- Depends on: [217-mission-state.md](217-mission-state.md)
- Depends on: [219-prompt-editor.md](219-prompt-editor.md)
- Depends on: [220-spec-selector.md](220-spec-selector.md)
- Depends on: [221-backend-selector.md](221-backend-selector.md)
- Next: [219-prompt-editor.md](219-prompt-editor.md)
- Used by: [216-mission-layout.md](216-mission-layout.md)
