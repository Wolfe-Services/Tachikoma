# 222 - Mode Toggle Component

**Phase:** 10 - Mission Panel UI
**Spec ID:** 222
**Status:** Planned
**Dependencies:** 216-mission-layout, 217-mission-state
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create a mode toggle component that allows users to switch between agentic (autonomous) and interactive (step-by-step) execution modes, with clear explanations of each mode's behavior and implications.

---

## Acceptance Criteria

- [x] Visual toggle between agentic and interactive modes
- [x] Clear mode descriptions and implications
- [x] Mode-specific option configurations
- [x] Animated transition between modes
- [x] Keyboard accessible toggle
- [x] Mode recommendation based on task type

---

## Implementation Details

### 1. Types (src/lib/types/execution-mode.ts)

```typescript
/**
 * Types for execution mode configuration.
 */

export type ExecutionMode = 'agentic' | 'interactive';

export interface ModeConfig {
  mode: ExecutionMode;
  agenticOptions: AgenticOptions;
  interactiveOptions: InteractiveOptions;
}

export interface AgenticOptions {
  autoApproveFileChanges: boolean;
  autoApproveCommands: boolean;
  maxIterations: number;
  pauseOnError: boolean;
  pauseOnRedline: boolean;
  checkpointFrequency: 'never' | 'on_change' | 'on_step' | 'always';
}

export interface InteractiveOptions {
  requireApprovalFor: ApprovalRequirement[];
  showDiffPreview: boolean;
  allowBatching: boolean;
  autoScrollToAction: boolean;
}

export type ApprovalRequirement =
  | 'file_create'
  | 'file_modify'
  | 'file_delete'
  | 'command_run'
  | 'network_request'
  | 'install_dependency';

export interface ModeInfo {
  mode: ExecutionMode;
  title: string;
  description: string;
  icon: string;
  benefits: string[];
  considerations: string[];
  recommendedFor: string[];
}

export const MODE_INFO: Record<ExecutionMode, ModeInfo> = {
  agentic: {
    mode: 'agentic',
    title: 'Agentic Mode',
    description: 'The AI works autonomously, making decisions and taking actions without requiring approval for each step.',
    icon: 'robot',
    benefits: [
      'Faster completion for complex tasks',
      'No interruptions during execution',
      'Optimal for well-defined specifications',
    ],
    considerations: [
      'Less control over individual decisions',
      'May need checkpoints for rollback',
      'Higher token consumption per session',
    ],
    recommendedFor: [
      'Implementing new features from specs',
      'Refactoring with clear goals',
      'Generating boilerplate code',
    ],
  },
  interactive: {
    mode: 'interactive',
    title: 'Interactive Mode',
    description: 'The AI proposes actions and waits for your approval before proceeding, giving you full control.',
    icon: 'user-check',
    benefits: [
      'Full control over every action',
      'Learn as the AI explains its reasoning',
      'Safer for sensitive operations',
    ],
    considerations: [
      'Slower overall completion',
      'Requires active participation',
      'May interrupt your workflow',
    ],
    recommendedFor: [
      'Debugging unfamiliar code',
      'Security-sensitive changes',
      'Learning new patterns',
    ],
  },
};

export const DEFAULT_AGENTIC_OPTIONS: AgenticOptions = {
  autoApproveFileChanges: true,
  autoApproveCommands: false,
  maxIterations: 50,
  pauseOnError: true,
  pauseOnRedline: true,
  checkpointFrequency: 'on_change',
};

export const DEFAULT_INTERACTIVE_OPTIONS: InteractiveOptions = {
  requireApprovalFor: ['file_modify', 'file_delete', 'command_run'],
  showDiffPreview: true,
  allowBatching: true,
  autoScrollToAction: true,
};
```

### 2. Mode Toggle Component (src/lib/components/mission/ModeToggle.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { slide, fade } from 'svelte/transition';
  import type { ExecutionMode, AgenticOptions, InteractiveOptions } from '$lib/types/execution-mode';
  import {
    MODE_INFO,
    DEFAULT_AGENTIC_OPTIONS,
    DEFAULT_INTERACTIVE_OPTIONS,
  } from '$lib/types/execution-mode';
  import ModeCard from './ModeCard.svelte';
  import ModeOptions from './ModeOptions.svelte';

  export let mode: ExecutionMode = 'agentic';
  export let agenticOptions: AgenticOptions = { ...DEFAULT_AGENTIC_OPTIONS };
  export let interactiveOptions: InteractiveOptions = { ...DEFAULT_INTERACTIVE_OPTIONS };
  export let showOptions = false;
  export let disabled = false;

  const dispatch = createEventDispatcher<{
    change: ExecutionMode;
    optionsChange: { agenticOptions: AgenticOptions; interactiveOptions: InteractiveOptions };
  }>();

  let optionsExpanded = false;

  function selectMode(newMode: ExecutionMode) {
    if (disabled || newMode === mode) return;
    mode = newMode;
    dispatch('change', mode);
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (disabled) return;

    if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
      event.preventDefault();
      selectMode(mode === 'agentic' ? 'interactive' : 'agentic');
    }
  }

  function updateAgenticOptions(updates: Partial<AgenticOptions>) {
    agenticOptions = { ...agenticOptions, ...updates };
    dispatch('optionsChange', { agenticOptions, interactiveOptions });
  }

  function updateInteractiveOptions(updates: Partial<InteractiveOptions>) {
    interactiveOptions = { ...interactiveOptions, ...updates };
    dispatch('optionsChange', { agenticOptions, interactiveOptions });
  }
</script>

<div
  class="mode-toggle"
  class:mode-toggle--disabled={disabled}
  role="radiogroup"
  aria-label="Execution mode"
  on:keydown={handleKeyDown}
>
  <!-- Mode Cards -->
  <div class="mode-toggle__cards">
    {#each Object.values(MODE_INFO) as modeInfo}
      <ModeCard
        {modeInfo}
        selected={mode === modeInfo.mode}
        {disabled}
        on:select={() => selectMode(modeInfo.mode)}
      />
    {/each}
  </div>

  <!-- Toggle Switch (Alternative compact view) -->
  <div class="mode-toggle__switch">
    <span
      class="mode-toggle__label"
      class:mode-toggle__label--active={mode === 'agentic'}
    >
      Agentic
    </span>

    <button
      class="toggle-switch"
      class:toggle-switch--interactive={mode === 'interactive'}
      role="switch"
      aria-checked={mode === 'interactive'}
      {disabled}
      on:click={() => selectMode(mode === 'agentic' ? 'interactive' : 'agentic')}
    >
      <span class="toggle-switch__thumb"></span>
    </button>

    <span
      class="mode-toggle__label"
      class:mode-toggle__label--active={mode === 'interactive'}
    >
      Interactive
    </span>
  </div>

  <!-- Current Mode Description -->
  <div class="mode-toggle__description" transition:fade={{ duration: 150 }}>
    <p>{MODE_INFO[mode].description}</p>
  </div>

  <!-- Options Expander -->
  {#if showOptions}
    <button
      class="mode-toggle__options-toggle"
      on:click={() => { optionsExpanded = !optionsExpanded; }}
      aria-expanded={optionsExpanded}
    >
      <span>Advanced Options</span>
      <svg
        width="12"
        height="12"
        viewBox="0 0 12 12"
        class:rotated={optionsExpanded}
      >
        <path fill="currentColor" d="M2 4l4 4 4-4"/>
      </svg>
    </button>

    {#if optionsExpanded}
      <div class="mode-toggle__options" transition:slide={{ duration: 200 }}>
        <ModeOptions
          {mode}
          {agenticOptions}
          {interactiveOptions}
          on:agenticChange={(e) => updateAgenticOptions(e.detail)}
          on:interactiveChange={(e) => updateInteractiveOptions(e.detail)}
        />
      </div>
    {/if}
  {/if}
</div>

<style>
  .mode-toggle {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .mode-toggle--disabled {
    opacity: 0.6;
    pointer-events: none;
  }

  .mode-toggle__cards {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }

  .mode-toggle__switch {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 12px 0;
  }

  .mode-toggle__label {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-secondary);
    transition: color 0.15s ease;
  }

  .mode-toggle__label--active {
    color: var(--color-primary);
  }

  .toggle-switch {
    position: relative;
    width: 52px;
    height: 28px;
    padding: 2px;
    border: none;
    border-radius: 14px;
    background: var(--color-bg-hover);
    cursor: pointer;
    transition: background-color 0.2s ease;
  }

  .toggle-switch:focus {
    outline: none;
    box-shadow: 0 0 0 3px var(--color-focus-ring);
  }

  .toggle-switch--interactive {
    background: var(--color-primary);
  }

  .toggle-switch__thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 24px;
    height: 24px;
    background: white;
    border-radius: 50%;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    transition: transform 0.2s ease;
  }

  .toggle-switch--interactive .toggle-switch__thumb {
    transform: translateX(24px);
  }

  .mode-toggle__description {
    padding: 12px;
    background: var(--color-bg-secondary);
    border-radius: 8px;
    text-align: center;
  }

  .mode-toggle__description p {
    margin: 0;
    font-size: 13px;
    color: var(--color-text-secondary);
    line-height: 1.5;
  }

  .mode-toggle__options-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 8px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 13px;
    cursor: pointer;
  }

  .mode-toggle__options-toggle:hover {
    color: var(--color-text-primary);
  }

  .mode-toggle__options-toggle svg {
    transition: transform 0.15s ease;
  }

  .mode-toggle__options-toggle svg.rotated {
    transform: rotate(180deg);
  }

  .mode-toggle__options {
    padding: 16px;
    background: var(--color-bg-secondary);
    border-radius: 8px;
  }
</style>
```

### 3. Mode Card Component (src/lib/components/mission/ModeCard.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { ModeInfo } from '$lib/types/execution-mode';

  export let modeInfo: ModeInfo;
  export let selected = false;
  export let disabled = false;

  const dispatch = createEventDispatcher<{ select: void }>();

  const icons: Record<string, string> = {
    robot: '🤖',
    'user-check': '👤',
  };
</script>

<button
  class="mode-card"
  class:mode-card--selected={selected}
  role="radio"
  aria-checked={selected}
  {disabled}
  on:click={() => dispatch('select')}
>
  <span class="mode-card__icon">{icons[modeInfo.icon]}</span>
  <h4 class="mode-card__title">{modeInfo.title}</h4>

  <ul class="mode-card__benefits">
    {#each modeInfo.benefits.slice(0, 2) as benefit}
      <li>{benefit}</li>
    {/each}
  </ul>

  {#if selected}
    <div class="mode-card__selected-badge">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
        <path d="M12.03 3.97a.75.75 0 010 1.06l-6.25 6.25a.75.75 0 01-1.06 0L2.47 9.03a.75.75 0 011.06-1.06l1.72 1.72 5.72-5.72a.75.75 0 011.06 0z"/>
      </svg>
      Selected
    </div>
  {/if}
</button>

<style>
  .mode-card {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 20px 16px;
    border: 2px solid var(--color-border);
    border-radius: 12px;
    background: var(--color-bg-primary);
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: center;
  }

  .mode-card:hover:not(:disabled) {
    border-color: var(--color-primary);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .mode-card:focus {
    outline: none;
    box-shadow: 0 0 0 3px var(--color-focus-ring);
  }

  .mode-card--selected {
    border-color: var(--color-primary);
    background: var(--color-bg-active);
  }

  .mode-card:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .mode-card__icon {
    font-size: 32px;
    margin-bottom: 12px;
  }

  .mode-card__title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 12px 0;
  }

  .mode-card__benefits {
    list-style: none;
    padding: 0;
    margin: 0;
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .mode-card__benefits li {
    padding: 4px 0;
  }

  .mode-card__selected-badge {
    position: absolute;
    top: 8px;
    right: 8px;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    background: var(--color-primary);
    color: white;
    font-size: 11px;
    font-weight: 500;
    border-radius: 12px;
  }
</style>
```

### 4. Mode Options Component (src/lib/components/mission/ModeOptions.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { ExecutionMode, AgenticOptions, InteractiveOptions, ApprovalRequirement } from '$lib/types/execution-mode';

  export let mode: ExecutionMode;
  export let agenticOptions: AgenticOptions;
  export let interactiveOptions: InteractiveOptions;

  const dispatch = createEventDispatcher<{
    agenticChange: Partial<AgenticOptions>;
    interactiveChange: Partial<InteractiveOptions>;
  }>();

  const approvalOptions: { value: ApprovalRequirement; label: string }[] = [
    { value: 'file_create', label: 'Create files' },
    { value: 'file_modify', label: 'Modify files' },
    { value: 'file_delete', label: 'Delete files' },
    { value: 'command_run', label: 'Run commands' },
    { value: 'network_request', label: 'Network requests' },
    { value: 'install_dependency', label: 'Install dependencies' },
  ];

  const checkpointOptions = [
    { value: 'never', label: 'Never' },
    { value: 'on_change', label: 'On file changes' },
    { value: 'on_step', label: 'Every step' },
    { value: 'always', label: 'Always' },
  ];

  function toggleApproval(requirement: ApprovalRequirement) {
    const current = interactiveOptions.requireApprovalFor;
    const updated = current.includes(requirement)
      ? current.filter(r => r !== requirement)
      : [...current, requirement];
    dispatch('interactiveChange', { requireApprovalFor: updated });
  }
</script>

<div class="mode-options">
  {#if mode === 'agentic'}
    <div class="options-section">
      <h5>Auto-Approval</h5>

      <label class="option-row">
        <input
          type="checkbox"
          checked={agenticOptions.autoApproveFileChanges}
          on:change={(e) => dispatch('agenticChange', { autoApproveFileChanges: e.currentTarget.checked })}
        />
        <span>Auto-approve file changes</span>
      </label>

      <label class="option-row">
        <input
          type="checkbox"
          checked={agenticOptions.autoApproveCommands}
          on:change={(e) => dispatch('agenticChange', { autoApproveCommands: e.currentTarget.checked })}
        />
        <span>Auto-approve shell commands</span>
      </label>
    </div>

    <div class="options-section">
      <h5>Safety</h5>

      <label class="option-row">
        <input
          type="checkbox"
          checked={agenticOptions.pauseOnError}
          on:change={(e) => dispatch('agenticChange', { pauseOnError: e.currentTarget.checked })}
        />
        <span>Pause on errors</span>
      </label>

      <label class="option-row">
        <input
          type="checkbox"
          checked={agenticOptions.pauseOnRedline}
          on:change={(e) => dispatch('agenticChange', { pauseOnRedline: e.currentTarget.checked })}
        />
        <span>Pause when context redlines</span>
      </label>

      <label class="option-row option-row--inline">
        <span>Max iterations:</span>
        <input
          type="number"
          min="1"
          max="200"
          value={agenticOptions.maxIterations}
          on:change={(e) => dispatch('agenticChange', { maxIterations: parseInt(e.currentTarget.value) })}
        />
      </label>
    </div>

    <div class="options-section">
      <h5>Checkpoints</h5>

      <div class="radio-group">
        {#each checkpointOptions as option}
          <label class="radio-option">
            <input
              type="radio"
              name="checkpoint-frequency"
              value={option.value}
              checked={agenticOptions.checkpointFrequency === option.value}
              on:change={() => dispatch('agenticChange', { checkpointFrequency: option.value })}
            />
            <span>{option.label}</span>
          </label>
        {/each}
      </div>
    </div>
  {:else}
    <div class="options-section">
      <h5>Require Approval For</h5>

      <div class="checkbox-grid">
        {#each approvalOptions as option}
          <label class="checkbox-option">
            <input
              type="checkbox"
              checked={interactiveOptions.requireApprovalFor.includes(option.value)}
              on:change={() => toggleApproval(option.value)}
            />
            <span>{option.label}</span>
          </label>
        {/each}
      </div>
    </div>

    <div class="options-section">
      <h5>Display</h5>

      <label class="option-row">
        <input
          type="checkbox"
          checked={interactiveOptions.showDiffPreview}
          on:change={(e) => dispatch('interactiveChange', { showDiffPreview: e.currentTarget.checked })}
        />
        <span>Show diff preview for file changes</span>
      </label>

      <label class="option-row">
        <input
          type="checkbox"
          checked={interactiveOptions.autoScrollToAction}
          on:change={(e) => dispatch('interactiveChange', { autoScrollToAction: e.currentTarget.checked })}
        />
        <span>Auto-scroll to pending actions</span>
      </label>

      <label class="option-row">
        <input
          type="checkbox"
          checked={interactiveOptions.allowBatching}
          on:change={(e) => dispatch('interactiveChange', { allowBatching: e.currentTarget.checked })}
        />
        <span>Allow batch approval of similar actions</span>
      </label>
    </div>
  {/if}
</div>

<style>
  .mode-options {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .options-section h5 {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    margin: 0 0 12px 0;
  }

  .option-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 0;
    cursor: pointer;
  }

  .option-row input[type="checkbox"],
  .option-row input[type="radio"] {
    width: 16px;
    height: 16px;
  }

  .option-row span {
    font-size: 14px;
    color: var(--color-text-primary);
  }

  .option-row--inline {
    justify-content: space-between;
  }

  .option-row--inline input[type="number"] {
    width: 80px;
    padding: 6px 10px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 14px;
  }

  .radio-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .radio-option {
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
  }

  .radio-option span {
    font-size: 14px;
    color: var(--color-text-primary);
  }

  .checkbox-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }

  .checkbox-option {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .checkbox-option input {
    width: 14px;
    height: 14px;
  }

  .checkbox-option span {
    font-size: 13px;
    color: var(--color-text-primary);
  }
</style>
```

---

## Testing Requirements

1. Mode toggle switches correctly
2. Selection emits change event
3. Mode descriptions update on switch
4. Options expand/collapse properly
5. Option changes emit events
6. Keyboard navigation works
7. Disabled state prevents interaction

### Test File (src/lib/components/mission/__tests__/ModeToggle.test.ts)

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import ModeToggle from '../ModeToggle.svelte';

describe('ModeToggle', () => {
  it('renders with default agentic mode', () => {
    render(ModeToggle);

    expect(screen.getByText('Agentic Mode')).toBeInTheDocument();
    expect(screen.getByText('Interactive Mode')).toBeInTheDocument();
  });

  it('switches mode on click', async () => {
    const { component } = render(ModeToggle, { mode: 'agentic' });
    const handler = vi.fn();
    component.$on('change', handler);

    const interactiveCard = screen.getByText('Interactive Mode').closest('button');
    await fireEvent.click(interactiveCard!);

    expect(handler).toHaveBeenCalledWith(expect.objectContaining({ detail: 'interactive' }));
  });

  it('shows correct description for selected mode', () => {
    render(ModeToggle, { mode: 'interactive' });

    expect(screen.getByText(/proposes actions and waits for your approval/)).toBeInTheDocument();
  });

  it('expands options on click', async () => {
    render(ModeToggle, { showOptions: true });

    const optionsToggle = screen.getByText('Advanced Options');
    await fireEvent.click(optionsToggle);

    expect(screen.getByText('Auto-Approval')).toBeInTheDocument();
  });

  it('disables toggle when disabled prop is true', async () => {
    render(ModeToggle, { disabled: true });

    const toggleSwitch = screen.getByRole('switch');
    expect(toggleSwitch).toBeDisabled();
  });
});
```

---

## Related Specs

- Depends on: [216-mission-layout.md](216-mission-layout.md)
- Depends on: [217-mission-state.md](217-mission-state.md)
- Next: [223-mission-controls.md](223-mission-controls.md)
- Used by: [218-mission-creation.md](218-mission-creation.md)
