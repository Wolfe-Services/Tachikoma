<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { WizardStep } from '$lib/types/forge';

  export let steps: WizardStep[] = [];
  export let currentStep: number = 0;

  const dispatch = createEventDispatcher<{
    stepClick: number;
  }>();

  function handleStepClick(index: number) {
    // Only allow clicking on completed or current steps
    if (index <= currentStep) {
      dispatch('stepClick', index);
    }
  }

  function getStepStatus(index: number): 'completed' | 'current' | 'upcoming' {
    if (index < currentStep) return 'completed';
    if (index === currentStep) return 'current';
    return 'upcoming';
  }

  function getStepIcon(step: WizardStep, index: number): string {
    const status = getStepStatus(index);
    if (status === 'completed') return 'check';
    return step.icon;
  }
</script>

<nav class="step-indicator" aria-label="Progress through wizard steps">
  <ol class="step-list">
    {#each steps as step, index}
      <li class="step-item" class:completed={getStepStatus(index) === 'completed'} 
          class:current={getStepStatus(index) === 'current'}
          class:upcoming={getStepStatus(index) === 'upcoming'}>
        <button
          type="button"
          class="step-button"
          disabled={index > currentStep}
          aria-current={index === currentStep ? 'step' : false}
          on:click={() => handleStepClick(index)}
          data-testid="step-{index}"
        >
          <span class="step-icon" aria-hidden="true">
            {#if getStepIcon(step, index) === 'check'}
              ✓
            {:else if step.icon === 'target'}
              🎯
            {:else if step.icon === 'users'}
              👥
            {:else if step.icon === 'brain'}
              🧠
            {:else if step.icon === 'settings'}
              ⚙️
            {:else}
              ✓
            {/if}
          </span>
          <span class="step-label">{step.label}</span>
        </button>
        {#if index < steps.length - 1}
          <div class="step-connector" aria-hidden="true"></div>
        {/if}
      </li>
    {/each}
  </ol>
</nav>

<style>
  .step-indicator {
    width: 100%;
    margin-bottom: 2rem;
  }

  .step-list {
    display: flex;
    list-style: none;
    margin: 0;
    padding: 0;
    align-items: center;
  }

  .step-item {
    display: flex;
    align-items: center;
    flex: 1;
    position: relative;
  }

  .step-button {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem;
    background: none;
    border: none;
    cursor: pointer;
    transition: all 0.2s ease;
    border-radius: 8px;
    text-align: center;
    min-width: 120px;
  }

  .step-button:disabled {
    cursor: not-allowed;
  }

  .step-button:hover:not(:disabled) {
    background: var(--hover-bg, rgba(255, 255, 255, 0.05));
  }

  .step-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2.5rem;
    height: 2.5rem;
    border-radius: 50%;
    font-size: 1.25rem;
    transition: all 0.2s ease;
  }

  .step-label {
    font-size: 0.875rem;
    font-weight: 500;
    transition: color 0.2s ease;
  }

  .step-connector {
    flex: 1;
    height: 2px;
    margin: 0 0.5rem;
    transition: background-color 0.2s ease;
  }

  /* Completed state */
  .step-item.completed .step-icon {
    background: var(--success-color, #10b981);
    color: white;
  }

  .step-item.completed .step-label {
    color: var(--success-color, #10b981);
  }

  .step-item.completed .step-connector {
    background: var(--success-color, #10b981);
  }

  /* Current state */
  .step-item.current .step-icon {
    background: var(--primary-color, #3b82f6);
    color: white;
    box-shadow: 0 0 0 3px var(--primary-color-alpha, rgba(59, 130, 246, 0.2));
  }

  .step-item.current .step-label {
    color: var(--primary-color, #3b82f6);
    font-weight: 600;
  }

  .step-item.current .step-connector {
    background: var(--border-color, #e5e7eb);
  }

  /* Upcoming state */
  .step-item.upcoming .step-icon {
    background: var(--secondary-bg, #f3f4f6);
    color: var(--text-muted, #6b7280);
    border: 2px solid var(--border-color, #e5e7eb);
  }

  .step-item.upcoming .step-label {
    color: var(--text-muted, #6b7280);
  }

  .step-item.upcoming .step-connector {
    background: var(--border-color, #e5e7eb);
  }

  /* Remove connector from last item */
  .step-item:last-child .step-connector {
    display: none;
  }

  /* Responsive design */
  @media (max-width: 768px) {
    .step-button {
      min-width: 80px;
      padding: 0.75rem 0.5rem;
    }

    .step-icon {
      width: 2rem;
      height: 2rem;
      font-size: 1rem;
    }

    .step-label {
      font-size: 0.75rem;
    }
  }
</style>