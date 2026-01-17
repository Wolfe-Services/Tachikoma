<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Icon from '$lib/components/common/Icon.svelte';
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
            <Icon
              name={getStepIcon(step, index)}
              size={18}
              glow={getStepStatus(index) !== 'upcoming'}
            />
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
    padding: 0.75rem 0.9rem;
    background: transparent;
    border: 1px solid transparent;
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
    background: var(--hover-bg, rgba(78, 205, 196, 0.08));
    border-color: rgba(78, 205, 196, 0.18);
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
    background: rgba(13, 17, 23, 0.55);
    border: 1px solid rgba(78, 205, 196, 0.15);
    color: var(--tachi-cyan, #4ecdc4);
  }

  .step-label {
    font-size: 0.875rem;
    font-weight: 500;
    transition: color 0.2s ease;
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
  }

  .step-connector {
    flex: 1;
    height: 2px;
    margin: 0 0.5rem;
    transition: background-color 0.2s ease;
    background: rgba(78, 205, 196, 0.12);
  }

  /* Completed state */
  .step-item.completed .step-icon {
    background: rgba(63, 185, 80, 0.12);
    border-color: rgba(63, 185, 80, 0.5);
    color: var(--success-color, #3fb950);
    box-shadow: 0 0 0 3px rgba(63, 185, 80, 0.12);
  }

  .step-item.completed .step-label {
    color: var(--success-color, #3fb950);
  }

  .step-item.completed .step-connector {
    background: rgba(63, 185, 80, 0.6);
  }

  /* Current state */
  .step-item.current .step-icon {
    background: rgba(78, 205, 196, 0.14);
    border-color: rgba(78, 205, 196, 0.65);
    color: var(--tachi-cyan, #4ecdc4);
    box-shadow: 0 0 0 3px rgba(78, 205, 196, 0.16);
  }

  .step-item.current .step-label {
    color: var(--tachi-cyan, #4ecdc4);
    font-weight: 600;
  }

  .step-item.current .step-connector {
    background: rgba(78, 205, 196, 0.18);
  }

  /* Upcoming state */
  .step-item.upcoming .step-icon {
    background: rgba(13, 17, 23, 0.35);
    color: var(--text-muted, rgba(230, 237, 243, 0.35));
    border: 1px solid rgba(78, 205, 196, 0.1);
    opacity: 0.6;
  }

  .step-item.upcoming .step-label {
    color: var(--text-muted, rgba(230, 237, 243, 0.45));
  }

  .step-item.upcoming .step-connector {
    background: rgba(78, 205, 196, 0.08);
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