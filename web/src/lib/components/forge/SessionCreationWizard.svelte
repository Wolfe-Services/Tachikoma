<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { writable, derived, get } from 'svelte/store';
  import Icon from '$lib/components/common/Icon.svelte';
  import GoalInput from './GoalInput.svelte';
  import ParticipantSelect from './ParticipantSelect.svelte';
  import OracleConfigPanel from './OracleConfigPanel.svelte';
  import SessionReview from './SessionReview.svelte';
  import StepIndicator from '$lib/components/ui/StepIndicator.svelte';
  import { forgeSessionStore } from '$lib/stores/forgeSession';
  import { sessionLoading, sessionError } from '$lib/stores/forgeSession';
  import Spinner from '$lib/components/ui/Spinner/Spinner.svelte';
  import GlassPanel from '$lib/components/ui/GlassPanel.svelte';
  import { validateSessionConfig } from '$lib/utils/sessionValidation';
  import type {
    SessionDraft,
    SessionTemplate,
    WizardStep
  } from '$lib/types/forge';

  export let template: SessionTemplate | null = null;
  export let editSessionId: string | null = null;

  const dispatch = createEventDispatcher<{
    created: { sessionId: string };
    cancelled: void;
    saved: { draftId: string };
  }>();

  const steps: WizardStep[] = [
    { id: 'goal', label: 'Define Goal', icon: 'target' },
    { id: 'participants', label: 'Select Participants', icon: 'users' },
    { id: 'oracle-config', label: 'Oracle & Config', icon: 'sliders' },
    { id: 'review', label: 'Review & Start', icon: 'check' }
  ];

  let currentStep = writable<number>(0);
  let sessionDraft = writable<SessionDraft>(initializeDraft(template, editSessionId));
  let validationErrors = writable<Map<string, string[]>>(new Map());
  let isSubmitting = writable<boolean>(false);
  let isEditMode = editSessionId !== null;

  function initializeDraft(tmpl: SessionTemplate | null, sessionIdToEdit: string | null): SessionDraft {
    // Edit mode: load from existing session
    if (sessionIdToEdit) {
      const state = get(forgeSessionStore);
      const existingSession = state.sessions.find(s => s.id === sessionIdToEdit);
      if (existingSession) {
        return {
          name: existingSession.name,
          goal: existingSession.goal,
          participants: [...existingSession.participants],
          oracle: existingSession.oracle,
          config: existingSession.config || {
            maxRounds: 5,
            convergenceThreshold: 0.8,
            allowHumanIntervention: true,
            autoSaveInterval: 30000,
            timeoutMinutes: 60
          },
          metadata: {
            createdAt: existingSession.createdAt,
            lastModified: new Date()
          }
        };
      }
    }

    // Template mode
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

    // New session
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
        case 'oracle-config':
          return $draft.oracle !== null && stepErrors.length === 0;
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

      let sessionId: string;
      if (isEditMode && editSessionId) {
        // Update existing session
        sessionId = await forgeSessionStore.updateSession(editSessionId, $sessionDraft);
      } else {
        // Create new session
        sessionId = await forgeSessionStore.createSession($sessionDraft);
      }
      dispatch('created', { sessionId });
    } catch (error) {
      console.error('Failed to save session:', error);
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
    sessionDraft.update(draft => ({
      ...draft,
      [field]: value,
      metadata: { ...draft.metadata, lastModified: new Date() }
    }));
  }

  $: submitErrors = $validationErrors.get('submit') || [];
</script>

<GlassPanel
  as="section"
  className="session-wizard"
  accent="cyan"
  subtle
  data-testid="session-creation-wizard"
  aria-busy={$isSubmitting || $sessionLoading}
>
  <header class="wizard-header">
    <div class="wizard-title">
      <div class="title-left">
        <div class="title-icon">
          <Icon name="brain" size={20} glow />
        </div>
        <div class="title-text">
          <div class="title-tag">SPEC FORGE // 公安9課</div>
          <h1>{isEditMode ? 'Edit Session' : 'Create Forge Session'}</h1>
        </div>
      </div>
      <div class="title-right">
        {#if $sessionLoading}
          <div class="title-loading" aria-label="Working">
            <Spinner size={16} color="var(--tachi-cyan, #4ecdc4)" />
            <span>Working…</span>
          </div>
        {/if}
      </div>
    </div>
    <StepIndicator
      {steps}
      currentStep={$currentStep}
      on:stepClick={(e) => goToStep(e.detail)}
    />
  </header>

  {#if submitErrors.length > 0 || $sessionError}
    <div class="submit-banner" role="alert" aria-live="polite">
      <Icon name="alert-triangle" size={18} />
      <div class="submit-text">
        <div class="submit-title">Session creation failed</div>
        <div class="submit-subtitle">{submitErrors[0] ?? $sessionError}</div>
      </div>
      <button type="button" class="btn btn-secondary" on:click={() => forgeSessionStore.clearError()}>
        Dismiss
      </button>
    </div>
  {/if}

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
      <OracleConfigPanel
        selectedOracle={$sessionDraft.oracle}
        participants={$sessionDraft.participants}
        config={$sessionDraft.config}
        errors={$validationErrors.get('oracle-config') || []}
        on:oracleChange={(e) => updateDraft('oracle', e.detail)}
        on:configChange={(e) => updateDraft('config', e.detail)}
      />
    {:else if $currentStep === 3}
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
          {#if $isSubmitting}
            {isEditMode ? 'Saving...' : 'Creating...'}
          {:else}
            {isEditMode ? 'Save Changes' : 'Start Session'}
          {/if}
        </button>
      {/if}
    </div>
  </footer>
</GlassPanel>

<style>
  .session-wizard {
    display: flex;
    flex-direction: column;
    height: 100%;
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem 2.5rem;
  }

  .wizard-header {
    margin-bottom: 2rem;
  }

  .wizard-title {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1.25rem;
  }

  .title-left {
    display: flex;
    align-items: flex-start;
    gap: 0.9rem;
  }

  .title-icon {
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 12px;
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.18), rgba(78, 205, 196, 0.05));
    border: 1px solid rgba(78, 205, 196, 0.28);
    color: var(--tachi-cyan, #4ecdc4);
    flex-shrink: 0;
  }

  .title-text {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .title-tag {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.65rem;
    letter-spacing: 2.5px;
    text-transform: uppercase;
    color: rgba(78, 205, 196, 0.9);
  }

  .wizard-header h1 {
    font-size: 1.6rem;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 1.25px;
    text-transform: uppercase;
    text-shadow: 0 0 18px rgba(78, 205, 196, 0.25);
  }

  .title-loading {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.45rem 0.65rem;
    border-radius: 999px;
    background: rgba(13, 17, 23, 0.35);
    border: 1px solid rgba(78, 205, 196, 0.14);
    color: rgba(230, 237, 243, 0.65);
    font-size: 0.85rem;
  }

  .wizard-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 0;
  }

  .submit-banner {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 0.9rem;
    border-radius: 12px;
    background: rgba(255, 107, 107, 0.08);
    border: 1px solid rgba(255, 107, 107, 0.25);
    color: rgba(230, 237, 243, 0.85);
    margin: 0 0 1rem;
  }

  .submit-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
  }

  .submit-title {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    letter-spacing: 1px;
    text-transform: uppercase;
    font-size: 0.75rem;
    color: rgba(230, 237, 243, 0.9);
  }

  .submit-subtitle {
    font-size: 0.9rem;
    color: rgba(230, 237, 243, 0.65);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .wizard-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 1.5rem;
    border-top: 1px solid rgba(78, 205, 196, 0.14);
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
    border-radius: 10px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: linear-gradient(135deg, var(--tachi-cyan-dark, #2d7a7a), var(--tachi-cyan, #4ecdc4));
    color: var(--bg-primary, #0d1117);
    border: 1px solid rgba(78, 205, 196, 0.5);
  }

  .btn-primary:hover:not(:disabled) {
    background: linear-gradient(135deg, var(--tachi-cyan, #4ecdc4), var(--tachi-cyan-bright, #6ee7df));
  }

  .btn-secondary {
    background: rgba(13, 17, 23, 0.25);
    color: rgba(230, 237, 243, 0.85);
    border: 1px solid rgba(78, 205, 196, 0.16);
  }

  .btn-success {
    background: rgba(63, 185, 80, 0.2);
    color: var(--success-color, #3fb950);
    border: 1px solid rgba(63, 185, 80, 0.4);
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