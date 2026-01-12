# Spec 257: Session Creation

## Header
- **Spec ID**: 257
- **Phase**: 12 - Forge UI
- **Component**: Session Creation
- **Dependencies**: Spec 256 (Forge Layout)
- **Status**: Draft

## Objective
Implement a comprehensive session creation workflow that guides users through configuring AI deliberation sessions, including goal definition, participant selection, oracle assignment, and session parameters.

## Acceptance Criteria
1. Multi-step wizard guides users through session configuration
2. Goal input supports rich text with markdown preview
3. Participant selection allows filtering and searching available brains
4. Oracle selection shows compatibility and recommendations
5. Session templates enable quick starts with pre-configured setups
6. Validation prevents invalid configurations before session start
7. Draft sessions can be saved and resumed later
8. Real-time cost estimation based on selected participants

## Implementation

### SessionCreationWizard.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import GoalInput from './GoalInput.svelte';
  import ParticipantSelect from './ParticipantSelect.svelte';
  import OracleSelect from './OracleSelect.svelte';
  import SessionConfig from './SessionConfig.svelte';
  import SessionReview from './SessionReview.svelte';
  import StepIndicator from '$lib/components/StepIndicator.svelte';
  import { forgeSessionStore } from '$lib/stores/forgeSession';
  import { validateSessionConfig } from '$lib/utils/sessionValidation';
  import type {
    SessionDraft,
    SessionTemplate,
    ValidationResult,
    WizardStep
  } from '$lib/types/forge';

  export let template: SessionTemplate | null = null;

  const dispatch = createEventDispatcher<{
    created: { sessionId: string };
    cancelled: void;
    saved: { draftId: string };
  }>();

  const steps: WizardStep[] = [
    { id: 'goal', label: 'Define Goal', icon: 'target' },
    { id: 'participants', label: 'Select Participants', icon: 'users' },
    { id: 'oracle', label: 'Choose Oracle', icon: 'brain' },
    { id: 'config', label: 'Configure Session', icon: 'settings' },
    { id: 'review', label: 'Review & Start', icon: 'check' }
  ];

  let currentStep = writable<number>(0);
  let sessionDraft = writable<SessionDraft>(initializeDraft(template));
  let validationErrors = writable<Map<string, string[]>>(new Map());
  let isSubmitting = writable<boolean>(false);

  function initializeDraft(tmpl: SessionTemplate | null): SessionDraft {
    if (tmpl) {
      return {
        name: `${tmpl.name} - ${new Date().toLocaleDateString()}`,
        goal: tmpl.defaultGoal || '',
        participants: [...tmpl.defaultParticipants],
        oracle: tmpl.defaultOracle,
        config: { ...tmpl.defaultConfig },
        metadata: {
          templateId: tmpl.id,
          createdAt: new Date(),
          lastModified: new Date()
        }
      };
    }

    return {
      name: '',
      goal: '',
      participants: [],
      oracle: null,
      config: {
        maxRounds: 5,
        convergenceThreshold: 0.8,
        allowHumanIntervention: true,
        autoSaveInterval: 30000,
        timeoutMinutes: 60
      },
      metadata: {
        createdAt: new Date(),
        lastModified: new Date()
      }
    };
  }

  const canProceed = derived(
    [currentStep, sessionDraft, validationErrors],
    ([$step, $draft, $errors]) => {
      const stepId = steps[$step].id;
      const stepErrors = $errors.get(stepId) || [];

      switch (stepId) {
        case 'goal':
          return $draft.goal.trim().length >= 10 && stepErrors.length === 0;
        case 'participants':
          return $draft.participants.length >= 2 && stepErrors.length === 0;
        case 'oracle':
          return $draft.oracle !== null && stepErrors.length === 0;
        case 'config':
          return stepErrors.length === 0;
        case 'review':
          return $errors.size === 0;
        default:
          return false;
      }
    }
  );

  const costEstimate = derived(sessionDraft, ($draft) => {
    const participantCost = $draft.participants.reduce((sum, p) => {
      return sum + (p.estimatedCostPerRound || 0);
    }, 0);
    const oracleCost = $draft.oracle?.estimatedCostPerRound || 0;
    const roundCost = participantCost + oracleCost;
    const maxCost = roundCost * ($draft.config.maxRounds || 5);

    return {
      perRound: roundCost,
      estimated: maxCost * 0.7, // Assume average 70% of max rounds
      maximum: maxCost
    };
  });

  async function validateStep(stepIndex: number): Promise<boolean> {
    const stepId = steps[stepIndex].id;
    const result = await validateSessionConfig($sessionDraft, stepId);

    validationErrors.update(errors => {
      if (result.errors.length > 0) {
        errors.set(stepId, result.errors);
      } else {
        errors.delete(stepId);
      }
      return errors;
    });

    return result.valid;
  }

  async function nextStep() {
    const isValid = await validateStep($currentStep);
    if (isValid && $currentStep < steps.length - 1) {
      currentStep.update(n => n + 1);
    }
  }

  function prevStep() {
    if ($currentStep > 0) {
      currentStep.update(n => n - 1);
    }
  }

  function goToStep(index: number) {
    if (index <= $currentStep) {
      currentStep.set(index);
    }
  }

  async function saveDraft() {
    try {
      const draftId = await forgeSessionStore.saveDraft($sessionDraft);
      dispatch('saved', { draftId });
    } catch (error) {
      console.error('Failed to save draft:', error);
    }
  }

  async function createSession() {
    isSubmitting.set(true);

    try {
      // Validate all steps
      for (let i = 0; i < steps.length; i++) {
        const isValid = await validateStep(i);
        if (!isValid) {
          currentStep.set(i);
          isSubmitting.set(false);
          return;
        }
      }

      const sessionId = await forgeSessionStore.createSession($sessionDraft);
      dispatch('created', { sessionId });
    } catch (error) {
      console.error('Failed to create session:', error);
      validationErrors.update(errors => {
        errors.set('submit', [(error as Error).message]);
        return errors;
      });
    } finally {
      isSubmitting.set(false);
    }
  }

  function handleCancel() {
    dispatch('cancelled');
  }

  function updateDraft(field: keyof SessionDraft, value: unknown) {
    sessionDraft.update(draft => {
      draft[field] = value as never;
      draft.metadata.lastModified = new Date();
      return draft;
    });
  }
</script>

<div class="session-wizard" data-testid="session-creation-wizard">
  <header class="wizard-header">
    <h1>Create Forge Session</h1>
    <StepIndicator
      {steps}
      currentStep={$currentStep}
      on:stepClick={(e) => goToStep(e.detail)}
    />
  </header>

  <div class="wizard-content">
    {#if $currentStep === 0}
      <GoalInput
        value={$sessionDraft.goal}
        name={$sessionDraft.name}
        errors={$validationErrors.get('goal') || []}
        on:goalChange={(e) => updateDraft('goal', e.detail)}
        on:nameChange={(e) => updateDraft('name', e.detail)}
      />
    {:else if $currentStep === 1}
      <ParticipantSelect
        selected={$sessionDraft.participants}
        errors={$validationErrors.get('participants') || []}
        on:change={(e) => updateDraft('participants', e.detail)}
      />
    {:else if $currentStep === 2}
      <OracleSelect
        selected={$sessionDraft.oracle}
        participants={$sessionDraft.participants}
        errors={$validationErrors.get('oracle') || []}
        on:change={(e) => updateDraft('oracle', e.detail)}
      />
    {:else if $currentStep === 3}
      <SessionConfig
        config={$sessionDraft.config}
        errors={$validationErrors.get('config') || []}
        on:change={(e) => updateDraft('config', e.detail)}
      />
    {:else if $currentStep === 4}
      <SessionReview
        draft={$sessionDraft}
        costEstimate={$costEstimate}
        errors={$validationErrors}
      />
    {/if}
  </div>

  <footer class="wizard-footer">
    <div class="footer-left">
      <button
        type="button"
        class="btn btn-ghost"
        on:click={handleCancel}
      >
        Cancel
      </button>
      <button
        type="button"
        class="btn btn-secondary"
        on:click={saveDraft}
      >
        Save Draft
      </button>
    </div>

    <div class="cost-estimate" aria-live="polite">
      <span class="cost-label">Estimated Cost:</span>
      <span class="cost-value">${$costEstimate.estimated.toFixed(4)}</span>
      <span class="cost-max">(max: ${$costEstimate.maximum.toFixed(4)})</span>
    </div>

    <div class="footer-right">
      {#if $currentStep > 0}
        <button
          type="button"
          class="btn btn-secondary"
          on:click={prevStep}
        >
          Previous
        </button>
      {/if}

      {#if $currentStep < steps.length - 1}
        <button
          type="button"
          class="btn btn-primary"
          disabled={!$canProceed}
          on:click={nextStep}
        >
          Next
        </button>
      {:else}
        <button
          type="button"
          class="btn btn-success"
          disabled={!$canProceed || $isSubmitting}
          on:click={createSession}
        >
          {$isSubmitting ? 'Creating...' : 'Start Session'}
        </button>
      {/if}
    </div>
  </footer>
</div>

<style>
  .session-wizard {
    display: flex;
    flex-direction: column;
    height: 100%;
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem;
  }

  .wizard-header {
    margin-bottom: 2rem;
  }

  .wizard-header h1 {
    font-size: 1.75rem;
    font-weight: 600;
    margin-bottom: 1.5rem;
    color: var(--text-primary);
  }

  .wizard-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 0;
  }

  .wizard-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 1.5rem;
    border-top: 1px solid var(--border-color);
    margin-top: 1.5rem;
  }

  .footer-left,
  .footer-right {
    display: flex;
    gap: 0.75rem;
  }

  .cost-estimate {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  .cost-label {
    color: var(--text-secondary);
  }

  .cost-value {
    font-weight: 600;
    color: var(--text-primary);
  }

  .cost-max {
    color: var(--text-muted);
    font-size: 0.75rem;
  }

  .btn {
    padding: 0.625rem 1.25rem;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--primary-color);
    color: white;
    border: none;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--primary-hover);
  }

  .btn-secondary {
    background: var(--secondary-bg);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
  }

  .btn-success {
    background: var(--success-color);
    color: white;
    border: none;
  }

  .btn-ghost {
    background: transparent;
    color: var(--text-secondary);
    border: none;
  }

  .btn-ghost:hover {
    color: var(--text-primary);
  }
</style>
```

### Session Validation Utility
```typescript
// lib/utils/sessionValidation.ts
import type { SessionDraft, ValidationResult } from '$lib/types/forge';

export async function validateSessionConfig(
  draft: SessionDraft,
  stepId?: string
): Promise<ValidationResult> {
  const errors: string[] = [];
  const warnings: string[] = [];

  if (!stepId || stepId === 'goal') {
    if (!draft.name || draft.name.trim().length < 3) {
      errors.push('Session name must be at least 3 characters');
    }
    if (!draft.goal || draft.goal.trim().length < 10) {
      errors.push('Goal description must be at least 10 characters');
    }
    if (draft.goal && draft.goal.length > 5000) {
      errors.push('Goal description exceeds maximum length of 5000 characters');
    }
  }

  if (!stepId || stepId === 'participants') {
    if (draft.participants.length < 2) {
      errors.push('At least 2 participants are required');
    }
    if (draft.participants.length > 10) {
      warnings.push('Having more than 10 participants may increase costs significantly');
    }

    const uniqueIds = new Set(draft.participants.map(p => p.id));
    if (uniqueIds.size !== draft.participants.length) {
      errors.push('Duplicate participants are not allowed');
    }
  }

  if (!stepId || stepId === 'oracle') {
    if (!draft.oracle) {
      errors.push('An oracle must be selected');
    }

    if (draft.oracle && draft.participants.some(p => p.id === draft.oracle?.id)) {
      warnings.push('Oracle is also a participant - this may affect deliberation dynamics');
    }
  }

  if (!stepId || stepId === 'config') {
    if (draft.config.maxRounds < 1 || draft.config.maxRounds > 20) {
      errors.push('Maximum rounds must be between 1 and 20');
    }
    if (draft.config.convergenceThreshold < 0.5 || draft.config.convergenceThreshold > 1) {
      errors.push('Convergence threshold must be between 0.5 and 1.0');
    }
    if (draft.config.timeoutMinutes < 5 || draft.config.timeoutMinutes > 480) {
      errors.push('Timeout must be between 5 minutes and 8 hours');
    }
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings
  };
}
```

## Testing Requirements
1. **Unit Tests**: Validate step navigation and form state management
2. **Integration Tests**: Test complete wizard flow from start to session creation
3. **Validation Tests**: Verify all validation rules trigger correctly
4. **Template Tests**: Ensure templates populate draft correctly
5. **Cost Estimation Tests**: Verify cost calculations are accurate

## Related Specs
- Spec 256: Forge Layout
- Spec 258: Goal Input
- Spec 259: Participant Select
- Spec 260: Oracle Select
