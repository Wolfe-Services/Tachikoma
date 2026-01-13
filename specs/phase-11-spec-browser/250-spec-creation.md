# 250 - Spec Creation

**Phase:** 11 - Spec Browser UI
**Spec ID:** 250
**Status:** Planned
**Dependencies:** 236-spec-browser-layout, 251-template-selection
**Estimated Context:** ~11% of Sonnet window

---

## Objective

Create a spec creation wizard component that guides users through creating new specifications with templates, validation, and automatic numbering.

---

## Acceptance Criteria

- [x] Multi-step creation wizard
- [x] Template selection
- [x] Auto-generate spec number
- [x] Phase selection
- [x] Frontmatter editor
- [x] Initial content scaffolding
- [x] Dependency selection
- [x] Preview before creation
- [x] Validation at each step

---

## Implementation Details

### 1. Types (src/lib/types/spec-creation.ts)

```typescript
export interface NewSpecData {
  title: string;
  phase: number;
  specId?: string;
  templateId: string;
  frontmatter: SpecFrontmatter;
  initialContent: string;
  dependencies: string[];
}

export interface SpecFrontmatter {
  specId: string;
  phase: number;
  status: 'Planned' | 'In Progress' | 'Complete';
  dependencies: string;
  estimatedContext: string;
  tags?: string[];
}

export interface SpecTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  frontmatterTemplate: Partial<SpecFrontmatter>;
  contentTemplate: string;
  sections: string[];
}

export interface CreationStep {
  id: string;
  title: string;
  description: string;
  isComplete: boolean;
  isValid: boolean;
  errors: string[];
}
```

### 2. Spec Creation Wizard Component (src/lib/components/spec-browser/SpecCreationWizard.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type { NewSpecData, SpecTemplate, CreationStep, SpecFrontmatter } from '$lib/types/spec-creation';
  import { ipcRenderer } from '$lib/ipc';
  import { fade, slide } from 'svelte/transition';
  import TemplateSelector from './TemplateSelector.svelte';

  export let open = false;
  export let initialPhase: number | null = null;

  const dispatch = createEventDispatcher<{
    create: NewSpecData;
    close: void;
  }>();

  let currentStep = 0;
  let templates: SpecTemplate[] = [];
  let phases: { number: number; name: string; specCount: number }[] = [];
  let nextSpecNumber = '';

  let specData: NewSpecData = {
    title: '',
    phase: initialPhase || 1,
    templateId: '',
    frontmatter: {
      specId: '',
      phase: initialPhase || 1,
      status: 'Planned',
      dependencies: '',
      estimatedContext: '~5% of Sonnet window',
    },
    initialContent: '',
    dependencies: [],
  };

  const steps: CreationStep[] = [
    {
      id: 'template',
      title: 'Choose Template',
      description: 'Select a template for your new spec',
      isComplete: false,
      isValid: false,
      errors: [],
    },
    {
      id: 'details',
      title: 'Basic Details',
      description: 'Enter spec title and phase',
      isComplete: false,
      isValid: false,
      errors: [],
    },
    {
      id: 'frontmatter',
      title: 'Metadata',
      description: 'Configure spec metadata',
      isComplete: false,
      isValid: false,
      errors: [],
    },
    {
      id: 'dependencies',
      title: 'Dependencies',
      description: 'Select spec dependencies',
      isComplete: false,
      isValid: false,
      errors: [],
    },
    {
      id: 'preview',
      title: 'Preview',
      description: 'Review and create',
      isComplete: false,
      isValid: true,
      errors: [],
    },
  ];

  async function loadTemplates() {
    templates = await ipcRenderer.invoke('spec:get-templates');
  }

  async function loadPhases() {
    phases = await ipcRenderer.invoke('spec:get-phases');
  }

  async function generateSpecNumber() {
    nextSpecNumber = await ipcRenderer.invoke('spec:next-number', specData.phase);
    specData.frontmatter.specId = nextSpecNumber;
  }

  function validateStep(stepIndex: number): boolean {
    const step = steps[stepIndex];
    step.errors = [];

    switch (step.id) {
      case 'template':
        if (!specData.templateId) {
          step.errors.push('Please select a template');
        }
        break;
      case 'details':
        if (!specData.title.trim()) {
          step.errors.push('Title is required');
        }
        if (specData.title.length > 100) {
          step.errors.push('Title must be less than 100 characters');
        }
        if (!specData.phase) {
          step.errors.push('Phase is required');
        }
        break;
      case 'frontmatter':
        if (!specData.frontmatter.estimatedContext) {
          step.errors.push('Estimated context is required');
        }
        break;
      case 'dependencies':
        // Dependencies are optional
        break;
      case 'preview':
        // Final validation
        break;
    }

    step.isValid = step.errors.length === 0;
    step.isComplete = step.isValid;
    steps[stepIndex] = step;
    return step.isValid;
  }

  function nextStep() {
    if (validateStep(currentStep) && currentStep < steps.length - 1) {
      currentStep++;
      if (currentStep === 1 && specData.templateId) {
        applyTemplate();
      }
      if (currentStep === 2) {
        generateSpecNumber();
      }
    }
  }

  function prevStep() {
    if (currentStep > 0) {
      currentStep--;
    }
  }

  function goToStep(index: number) {
    // Only allow going to completed steps or the next one
    if (index <= currentStep || (index === currentStep + 1 && validateStep(currentStep))) {
      currentStep = index;
    }
  }

  function applyTemplate() {
    const template = templates.find(t => t.id === specData.templateId);
    if (template) {
      specData.initialContent = template.contentTemplate;
      specData.frontmatter = {
        ...specData.frontmatter,
        ...template.frontmatterTemplate,
      };
    }
  }

  function selectTemplate(templateId: string) {
    specData.templateId = templateId;
    validateStep(0);
  }

  async function createSpec() {
    // Generate final content
    const fullContent = generateFullContent();

    try {
      const result = await ipcRenderer.invoke('spec:create', {
        ...specData,
        content: fullContent,
      });

      dispatch('create', result);
      close();
    } catch (error) {
      console.error('Failed to create spec:', error);
    }
  }

  function generateFullContent(): string {
    const fm = specData.frontmatter;
    const deps = specData.dependencies.length > 0
      ? specData.dependencies.join(', ')
      : 'None';

    return `# ${fm.specId} - ${specData.title}

**Phase:** ${fm.phase}
**Spec ID:** ${fm.specId}
**Status:** ${fm.status}
**Dependencies:** ${deps}
**Estimated Context:** ${fm.estimatedContext}

---

${specData.initialContent}
`;
  }

  function close() {
    open = false;
    dispatch('close');
    resetForm();
  }

  function resetForm() {
    currentStep = 0;
    specData = {
      title: '',
      phase: initialPhase || 1,
      templateId: '',
      frontmatter: {
        specId: '',
        phase: initialPhase || 1,
        status: 'Planned',
        dependencies: '',
        estimatedContext: '~5% of Sonnet window',
      },
      initialContent: '',
      dependencies: [],
    };
    steps.forEach(step => {
      step.isComplete = false;
      step.isValid = false;
      step.errors = [];
    });
  }

  onMount(() => {
    loadTemplates();
    loadPhases();
  });

  $: if (open && initialPhase) {
    specData.phase = initialPhase;
    specData.frontmatter.phase = initialPhase;
  }
</script>

{#if open}
  <div
    class="wizard-overlay"
    on:click={close}
    transition:fade={{ duration: 150 }}
  >
    <div
      class="wizard"
      on:click|stopPropagation
      transition:slide={{ duration: 200 }}
    >
      <header class="wizard__header">
        <h2>Create New Spec</h2>
        <button class="close-btn" on:click={close}>
          <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
            <path d="M5.293 5.293a1 1 0 011.414 0L10 8.586l3.293-3.293a1 1 0 111.414 1.414L11.414 10l3.293 3.293a1 1 0 01-1.414 1.414L10 11.414l-3.293 3.293a1 1 0 01-1.414-1.414L8.586 10 5.293 6.707a1 1 0 010-1.414z"/>
          </svg>
        </button>
      </header>

      <nav class="wizard__steps">
        {#each steps as step, index}
          <button
            class="step-indicator"
            class:active={index === currentStep}
            class:complete={step.isComplete}
            class:error={step.errors.length > 0 && index < currentStep}
            on:click={() => goToStep(index)}
            disabled={index > currentStep + 1}
          >
            <span class="step-number">
              {#if step.isComplete}
                <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
                  <path d="M5.5 10.5l-3-3 1-1 2 2 5-5 1 1-6 6z"/>
                </svg>
              {:else}
                {index + 1}
              {/if}
            </span>
            <span class="step-title">{step.title}</span>
          </button>
        {/each}
      </nav>

      <div class="wizard__content">
        {#if currentStep === 0}
          <!-- Template Selection -->
          <div class="step-content" transition:fade={{ duration: 150 }}>
            <h3>{steps[0].title}</h3>
            <p class="step-description">{steps[0].description}</p>

            <TemplateSelector
              {templates}
              selectedId={specData.templateId}
              on:select={(e) => selectTemplate(e.detail)}
            />

            {#if steps[0].errors.length > 0}
              <div class="step-errors">
                {#each steps[0].errors as error}
                  <p class="error">{error}</p>
                {/each}
              </div>
            {/if}
          </div>
        {:else if currentStep === 1}
          <!-- Basic Details -->
          <div class="step-content" transition:fade={{ duration: 150 }}>
            <h3>{steps[1].title}</h3>
            <p class="step-description">{steps[1].description}</p>

            <div class="form-group">
              <label for="spec-title">Spec Title</label>
              <input
                id="spec-title"
                type="text"
                bind:value={specData.title}
                placeholder="e.g., User Authentication Flow"
                class:error={steps[1].errors.some(e => e.includes('Title'))}
              />
            </div>

            <div class="form-group">
              <label for="spec-phase">Phase</label>
              <select
                id="spec-phase"
                bind:value={specData.phase}
                on:change={() => { specData.frontmatter.phase = specData.phase; }}
              >
                {#each phases as phase}
                  <option value={phase.number}>
                    Phase {phase.number}: {phase.name} ({phase.specCount} specs)
                  </option>
                {/each}
              </select>
            </div>

            {#if steps[1].errors.length > 0}
              <div class="step-errors">
                {#each steps[1].errors as error}
                  <p class="error">{error}</p>
                {/each}
              </div>
            {/if}
          </div>
        {:else if currentStep === 2}
          <!-- Frontmatter -->
          <div class="step-content" transition:fade={{ duration: 150 }}>
            <h3>{steps[2].title}</h3>
            <p class="step-description">{steps[2].description}</p>

            <div class="form-group">
              <label for="spec-id">Spec ID</label>
              <input
                id="spec-id"
                type="text"
                value={specData.frontmatter.specId}
                disabled
                class="disabled"
              />
              <span class="help-text">Auto-generated based on phase</span>
            </div>

            <div class="form-group">
              <label for="spec-status">Status</label>
              <select
                id="spec-status"
                bind:value={specData.frontmatter.status}
              >
                <option value="Planned">Planned</option>
                <option value="In Progress">In Progress</option>
                <option value="Complete">Complete</option>
              </select>
            </div>

            <div class="form-group">
              <label for="estimated-context">Estimated Context</label>
              <select
                id="estimated-context"
                bind:value={specData.frontmatter.estimatedContext}
              >
                <option value="~3% of Sonnet window">~3% (Small)</option>
                <option value="~5% of Sonnet window">~5% (Medium)</option>
                <option value="~8% of Sonnet window">~8% (Large)</option>
                <option value="~10% of Sonnet window">~10% (Very Large)</option>
                <option value="~15% of Sonnet window">~15% (Complex)</option>
              </select>
            </div>
          </div>
        {:else if currentStep === 3}
          <!-- Dependencies -->
          <div class="step-content" transition:fade={{ duration: 150 }}>
            <h3>{steps[3].title}</h3>
            <p class="step-description">{steps[3].description}</p>

            <div class="dependency-selector">
              <p class="help-text">
                Select specs that this spec depends on. Dependencies help establish
                the order of implementation.
              </p>

              <!-- This would integrate with spec tree/search -->
              <div class="selected-dependencies">
                {#if specData.dependencies.length === 0}
                  <p class="empty-deps">No dependencies selected</p>
                {:else}
                  {#each specData.dependencies as depId}
                    <span class="dependency-tag">
                      {depId}
                      <button
                        class="remove-dep"
                        on:click={() => {
                          specData.dependencies = specData.dependencies.filter(d => d !== depId);
                        }}
                      >
                        x
                      </button>
                    </span>
                  {/each}
                {/if}
              </div>

              <button
                class="add-dependency-btn"
                on:click={() => {
                  // Would open spec picker
                }}
              >
                + Add Dependency
              </button>
            </div>
          </div>
        {:else if currentStep === 4}
          <!-- Preview -->
          <div class="step-content preview-step" transition:fade={{ duration: 150 }}>
            <h3>{steps[4].title}</h3>
            <p class="step-description">{steps[4].description}</p>

            <div class="preview-container">
              <div class="preview-header">
                <span class="preview-filename">
                  {specData.frontmatter.specId}-{specData.title.toLowerCase().replace(/\s+/g, '-')}.md
                </span>
              </div>
              <pre class="preview-content">{generateFullContent()}</pre>
            </div>
          </div>
        {/if}
      </div>

      <footer class="wizard__footer">
        <button
          class="btn btn--secondary"
          on:click={prevStep}
          disabled={currentStep === 0}
        >
          Back
        </button>

        <div class="footer-spacer" />

        {#if currentStep < steps.length - 1}
          <button
            class="btn btn--primary"
            on:click={nextStep}
          >
            Next
          </button>
        {:else}
          <button
            class="btn btn--primary btn--create"
            on:click={createSpec}
          >
            Create Spec
          </button>
        {/if}
      </footer>
    </div>
  </div>
{/if}

<style>
  .wizard-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .wizard {
    width: 90%;
    max-width: 700px;
    max-height: 90vh;
    background: var(--color-bg-primary);
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .wizard__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 24px;
    border-bottom: 1px solid var(--color-border);
  }

  .wizard__header h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
  }

  .close-btn {
    padding: 8px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    border-radius: 6px;
    cursor: pointer;
  }

  .close-btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .wizard__steps {
    display: flex;
    padding: 16px 24px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
    overflow-x: auto;
  }

  .step-indicator {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    white-space: nowrap;
  }

  .step-indicator:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .step-indicator.active {
    color: var(--color-primary);
  }

  .step-indicator.complete {
    color: var(--color-success);
  }

  .step-indicator.error {
    color: var(--color-error);
  }

  .step-number {
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

  .step-indicator.active .step-number {
    background: var(--color-primary);
    color: white;
  }

  .step-indicator.complete .step-number {
    background: var(--color-success);
    color: white;
  }

  .step-title {
    font-size: 13px;
  }

  .wizard__content {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }

  .step-content h3 {
    margin: 0 0 8px 0;
    font-size: 18px;
  }

  .step-description {
    margin: 0 0 24px 0;
    color: var(--color-text-muted);
    font-size: 14px;
  }

  .form-group {
    margin-bottom: 20px;
  }

  .form-group label {
    display: block;
    margin-bottom: 8px;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .form-group input,
  .form-group select {
    width: 100%;
    padding: 10px 14px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
    font-size: 14px;
  }

  .form-group input:focus,
  .form-group select:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .form-group input.disabled {
    background: var(--color-bg-hover);
    color: var(--color-text-muted);
  }

  .form-group input.error {
    border-color: var(--color-error);
  }

  .help-text {
    display: block;
    margin-top: 6px;
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .step-errors {
    margin-top: 16px;
    padding: 12px;
    background: rgba(244, 67, 54, 0.1);
    border-radius: 6px;
  }

  .step-errors .error {
    margin: 0;
    color: var(--color-error);
    font-size: 13px;
  }

  .dependency-selector {
    background: var(--color-bg-secondary);
    border-radius: 8px;
    padding: 16px;
  }

  .selected-dependencies {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin: 16px 0;
  }

  .empty-deps {
    color: var(--color-text-muted);
    font-style: italic;
    font-size: 13px;
  }

  .dependency-tag {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    background: var(--color-bg-hover);
    border-radius: 4px;
    font-size: 13px;
    font-family: monospace;
  }

  .remove-dep {
    padding: 2px 4px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    font-size: 12px;
  }

  .remove-dep:hover {
    color: var(--color-error);
  }

  .add-dependency-btn {
    padding: 8px 16px;
    border: 1px dashed var(--color-border);
    background: transparent;
    border-radius: 6px;
    color: var(--color-text-secondary);
    font-size: 13px;
    cursor: pointer;
  }

  .add-dependency-btn:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .preview-container {
    border: 1px solid var(--color-border);
    border-radius: 8px;
    overflow: hidden;
  }

  .preview-header {
    padding: 10px 16px;
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
  }

  .preview-filename {
    font-family: monospace;
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .preview-content {
    margin: 0;
    padding: 16px;
    background: var(--color-bg-primary);
    font-family: monospace;
    font-size: 12px;
    line-height: 1.5;
    white-space: pre-wrap;
    overflow-x: auto;
    max-height: 300px;
    overflow-y: auto;
  }

  .wizard__footer {
    display: flex;
    align-items: center;
    padding: 16px 24px;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-secondary);
  }

  .footer-spacer {
    flex: 1;
  }

  .btn {
    padding: 10px 20px;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
  }

  .btn--secondary {
    background: transparent;
    border: 1px solid var(--color-border);
    color: var(--color-text-secondary);
  }

  .btn--secondary:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .btn--secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn--primary {
    background: var(--color-primary);
    color: white;
  }

  .btn--primary:hover {
    filter: brightness(1.1);
  }

  .btn--create {
    background: var(--color-success);
  }
</style>
```

---

## Testing Requirements

1. Wizard navigation works
2. Template selection applies content
3. Spec number auto-generates
4. Validation prevents invalid data
5. Preview shows correct content
6. Spec creates successfully

---

## Related Specs

- Depends on: [251-template-selection.md](251-template-selection.md)
- Next: [252-spec-validation-ui.md](252-spec-validation-ui.md)
