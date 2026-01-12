# Spec 200: Checkbox and Toggle Components

## Phase
Phase 9: UI Foundation

## Spec ID
200

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 191-195: Design System

## Estimated Context
~8%

---

## Objective

Implement Checkbox and Toggle (Switch) components for Tachikoma with support for indeterminate states, groups, labels, and full accessibility compliance following the Tachikoma blue theme.

---

## Acceptance Criteria

- [ ] Checkbox with checked, unchecked, and indeterminate states
- [ ] Toggle/Switch component
- [ ] Multiple sizes (sm, md, lg)
- [ ] Label and description support
- [ ] Checkbox group for multiple selections
- [ ] Disabled state
- [ ] Keyboard accessible (Space to toggle)
- [ ] Animated transitions
- [ ] Focus visible states

---

## Implementation Details

### src/lib/components/ui/Checkbox/Checkbox.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { generateId, cn } from '@utils/component';
  import Icon from '../Icon/Icon.svelte';

  type CheckboxSize = 'sm' | 'md' | 'lg';

  export let checked: boolean = false;
  export let indeterminate: boolean = false;
  export let size: CheckboxSize = 'md';
  export let disabled: boolean = false;
  export let name: string = '';
  export let value: string = '';
  export let label: string | undefined = undefined;
  export let description: string | undefined = undefined;
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    change: { checked: boolean; value: string };
  }>();

  const id = generateId('checkbox');
  let inputElement: HTMLInputElement;

  $: {
    if (inputElement) {
      inputElement.indeterminate = indeterminate;
    }
  }

  function handleChange(event: Event) {
    const target = event.target as HTMLInputElement;
    checked = target.checked;
    indeterminate = false;
    dispatch('change', { checked, value });
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === ' ') {
      event.preventDefault();
      if (!disabled) {
        checked = !checked;
        indeterminate = false;
        dispatch('change', { checked, value });
      }
    }
  }

  $: wrapperClasses = cn(
    'checkbox-wrapper',
    `checkbox-${size}`,
    disabled && 'checkbox-disabled',
    className
  );
</script>

<label class={wrapperClasses}>
  <span class="checkbox-control">
    <input
      bind:this={inputElement}
      type="checkbox"
      {id}
      {name}
      {value}
      {checked}
      {disabled}
      class="checkbox-input"
      on:change={handleChange}
      on:keydown={handleKeyDown}
      aria-checked={indeterminate ? 'mixed' : checked}
      aria-describedby={description ? `${id}-description` : undefined}
    />
    <span class="checkbox-box" class:checked class:indeterminate>
      {#if checked}
        <Icon name="check" size={size === 'sm' ? 12 : size === 'lg' ? 18 : 14} />
      {:else if indeterminate}
        <span class="checkbox-indeterminate-icon"></span>
      {/if}
    </span>
  </span>

  {#if label || description}
    <span class="checkbox-content">
      {#if label}
        <span class="checkbox-label">{label}</span>
      {/if}
      {#if description}
        <span id="{id}-description" class="checkbox-description">
          {description}
        </span>
      {/if}
    </span>
  {/if}
</label>

<style>
  .checkbox-wrapper {
    display: inline-flex;
    align-items: flex-start;
    gap: var(--spacing-2);
    cursor: pointer;
    user-select: none;
  }

  .checkbox-wrapper.checkbox-disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .checkbox-control {
    position: relative;
    flex-shrink: 0;
  }

  .checkbox-input {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  .checkbox-box {
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-bg-input);
    border: 2px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    color: transparent;
    transition:
      background-color var(--duration-150) var(--ease-out),
      border-color var(--duration-150) var(--ease-out),
      color var(--duration-150) var(--ease-out),
      box-shadow var(--duration-150) var(--ease-out);
  }

  /* Sizes */
  .checkbox-sm .checkbox-box {
    width: 16px;
    height: 16px;
  }

  .checkbox-md .checkbox-box {
    width: 20px;
    height: 20px;
  }

  .checkbox-lg .checkbox-box {
    width: 24px;
    height: 24px;
  }

  /* Hover state */
  .checkbox-wrapper:not(.checkbox-disabled):hover .checkbox-box {
    border-color: var(--color-border-strong);
  }

  /* Focus state */
  .checkbox-input:focus-visible + .checkbox-box {
    box-shadow: var(--focus-ring);
  }

  /* Checked state */
  .checkbox-box.checked {
    background-color: var(--tachikoma-500);
    border-color: var(--tachikoma-500);
    color: white;
  }

  .checkbox-box.checked:hover {
    background-color: var(--tachikoma-400);
    border-color: var(--tachikoma-400);
  }

  /* Indeterminate state */
  .checkbox-box.indeterminate {
    background-color: var(--tachikoma-500);
    border-color: var(--tachikoma-500);
  }

  .checkbox-indeterminate-icon {
    width: 60%;
    height: 2px;
    background-color: white;
    border-radius: 1px;
  }

  /* Content */
  .checkbox-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-0-5);
    padding-top: 1px;
  }

  .checkbox-label {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-fg-default);
    line-height: var(--leading-tight);
  }

  .checkbox-description {
    font-size: var(--text-xs);
    color: var(--color-fg-muted);
    line-height: var(--leading-normal);
  }

  .checkbox-sm .checkbox-label {
    font-size: var(--text-xs);
  }

  .checkbox-lg .checkbox-label {
    font-size: var(--text-base);
  }
</style>
```

### src/lib/components/ui/Toggle/Toggle.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { generateId, cn } from '@utils/component';

  type ToggleSize = 'sm' | 'md' | 'lg';

  export let checked: boolean = false;
  export let size: ToggleSize = 'md';
  export let disabled: boolean = false;
  export let name: string = '';
  export let label: string | undefined = undefined;
  export let description: string | undefined = undefined;
  export let labelPosition: 'left' | 'right' = 'right';
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    change: boolean;
  }>();

  const id = generateId('toggle');

  function handleChange() {
    if (disabled) return;
    checked = !checked;
    dispatch('change', checked);
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === ' ' || event.key === 'Enter') {
      event.preventDefault();
      handleChange();
    }
  }

  $: wrapperClasses = cn(
    'toggle-wrapper',
    `toggle-${size}`,
    labelPosition === 'left' && 'toggle-label-left',
    disabled && 'toggle-disabled',
    className
  );

  // Size configurations
  const sizes = {
    sm: { width: 32, height: 18, knob: 14, translate: 14 },
    md: { width: 44, height: 24, knob: 20, translate: 20 },
    lg: { width: 56, height: 30, knob: 26, translate: 26 }
  };

  $: sizeConfig = sizes[size];
</script>

<label class={wrapperClasses}>
  {#if (label || description) && labelPosition === 'left'}
    <span class="toggle-content">
      {#if label}
        <span class="toggle-label">{label}</span>
      {/if}
      {#if description}
        <span id="{id}-description" class="toggle-description">
          {description}
        </span>
      {/if}
    </span>
  {/if}

  <button
    type="button"
    role="switch"
    {id}
    {disabled}
    aria-checked={checked}
    aria-describedby={description ? `${id}-description` : undefined}
    class="toggle-track"
    class:checked
    style="
      width: {sizeConfig.width}px;
      height: {sizeConfig.height}px;
    "
    on:click={handleChange}
    on:keydown={handleKeyDown}
  >
    <span
      class="toggle-knob"
      style="
        width: {sizeConfig.knob}px;
        height: {sizeConfig.knob}px;
        transform: translateX({checked ? sizeConfig.translate : 0}px);
      "
    ></span>

    <input
      type="checkbox"
      {name}
      {checked}
      {disabled}
      class="toggle-input"
      tabindex="-1"
      aria-hidden="true"
    />
  </button>

  {#if (label || description) && labelPosition === 'right'}
    <span class="toggle-content">
      {#if label}
        <span class="toggle-label">{label}</span>
      {/if}
      {#if description}
        <span id="{id}-description" class="toggle-description">
          {description}
        </span>
      {/if}
    </span>
  {/if}
</label>

<style>
  .toggle-wrapper {
    display: inline-flex;
    align-items: flex-start;
    gap: var(--spacing-3);
    cursor: pointer;
    user-select: none;
  }

  .toggle-wrapper.toggle-label-left {
    flex-direction: row;
  }

  .toggle-wrapper.toggle-disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .toggle-input {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  .toggle-track {
    position: relative;
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
    padding: 2px;
    background-color: var(--color-bg-muted);
    border: none;
    border-radius: 9999px;
    cursor: pointer;
    transition:
      background-color var(--duration-200) var(--ease-out),
      box-shadow var(--duration-150) var(--ease-out);
  }

  .toggle-track:hover:not(:disabled) {
    background-color: var(--color-bg-subtle);
  }

  .toggle-track:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
  }

  .toggle-track.checked {
    background-color: var(--tachikoma-500);
  }

  .toggle-track.checked:hover:not(:disabled) {
    background-color: var(--tachikoma-400);
  }

  .toggle-track:disabled {
    cursor: not-allowed;
  }

  .toggle-knob {
    display: block;
    background-color: white;
    border-radius: 50%;
    box-shadow: var(--shadow-sm);
    transition: transform var(--duration-200) var(--ease-out);
  }

  .toggle-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-0-5);
    padding-top: 2px;
  }

  .toggle-label {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-fg-default);
    line-height: var(--leading-tight);
  }

  .toggle-description {
    font-size: var(--text-xs);
    color: var(--color-fg-muted);
    line-height: var(--leading-normal);
  }

  .toggle-sm .toggle-label {
    font-size: var(--text-xs);
  }

  .toggle-lg .toggle-label {
    font-size: var(--text-base);
  }
</style>
```

### src/lib/components/ui/Checkbox/CheckboxGroup.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher, setContext } from 'svelte';
  import { writable } from 'svelte/store';
  import { generateId, cn } from '@utils/component';
  import Checkbox from './Checkbox.svelte';

  export interface CheckboxOption {
    value: string;
    label: string;
    description?: string;
    disabled?: boolean;
  }

  export let options: CheckboxOption[] = [];
  export let value: string[] = [];
  export let name: string = '';
  export let label: string | undefined = undefined;
  export let required: boolean = false;
  export let disabled: boolean = false;
  export let orientation: 'horizontal' | 'vertical' = 'vertical';
  export let error: string | undefined = undefined;
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    change: string[];
  }>();

  const id = generateId('checkbox-group');
  const valueStore = writable<string[]>(value);

  $: valueStore.set(value);

  setContext('checkbox-group', {
    name,
    disabled,
    value: valueStore
  });

  function handleChange(event: CustomEvent<{ checked: boolean; value: string }>) {
    const { checked, value: optionValue } = event.detail;

    if (checked) {
      value = [...value, optionValue];
    } else {
      value = value.filter(v => v !== optionValue);
    }

    dispatch('change', value);
  }

  $: wrapperClasses = cn(
    'checkbox-group',
    `checkbox-group-${orientation}`,
    error && 'checkbox-group-error',
    className
  );
</script>

<fieldset class={wrapperClasses} aria-describedby={error ? `${id}-error` : undefined}>
  {#if label}
    <legend class="checkbox-group-label">
      {label}
      {#if required}
        <span class="checkbox-group-required" aria-hidden="true">*</span>
      {/if}
    </legend>
  {/if}

  <div class="checkbox-group-options">
    {#each options as option}
      <Checkbox
        {name}
        value={option.value}
        label={option.label}
        description={option.description}
        checked={value.includes(option.value)}
        disabled={disabled || option.disabled}
        on:change={handleChange}
      />
    {/each}
    <slot />
  </div>

  {#if error}
    <span id="{id}-error" class="checkbox-group-error-message" role="alert">
      {error}
    </span>
  {/if}
</fieldset>

<style>
  .checkbox-group {
    border: none;
    padding: 0;
    margin: 0;
  }

  .checkbox-group-label {
    display: block;
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-fg-default);
    margin-bottom: var(--spacing-3);
  }

  .checkbox-group-required {
    color: var(--color-error-fg);
    margin-left: var(--spacing-1);
  }

  .checkbox-group-options {
    display: flex;
    gap: var(--spacing-3);
  }

  .checkbox-group-vertical .checkbox-group-options {
    flex-direction: column;
  }

  .checkbox-group-horizontal .checkbox-group-options {
    flex-direction: row;
    flex-wrap: wrap;
  }

  .checkbox-group-error-message {
    display: block;
    font-size: var(--text-xs);
    color: var(--color-error-fg);
    margin-top: var(--spacing-2);
  }
</style>
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/Checkbox.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import Checkbox from '@components/ui/Checkbox/Checkbox.svelte';
import Toggle from '@components/ui/Toggle/Toggle.svelte';

describe('Checkbox', () => {
  it('should render unchecked by default', () => {
    const { getByRole } = render(Checkbox);
    expect(getByRole('checkbox')).not.toBeChecked();
  });

  it('should render checked when checked prop is true', () => {
    const { getByRole } = render(Checkbox, { props: { checked: true } });
    expect(getByRole('checkbox')).toBeChecked();
  });

  it('should toggle on click', async () => {
    const handleChange = vi.fn();
    const { getByRole, component } = render(Checkbox);

    component.$on('change', handleChange);
    await fireEvent.click(getByRole('checkbox'));

    expect(handleChange).toHaveBeenCalledWith(
      expect.objectContaining({ detail: { checked: true, value: '' } })
    );
  });

  it('should render with label', () => {
    const { getByText } = render(Checkbox, {
      props: { label: 'Accept terms' }
    });

    expect(getByText('Accept terms')).toBeInTheDocument();
  });

  it('should render indeterminate state', () => {
    const { getByRole } = render(Checkbox, {
      props: { indeterminate: true }
    });

    expect(getByRole('checkbox')).toHaveAttribute('aria-checked', 'mixed');
  });

  it('should be disabled', () => {
    const { getByRole } = render(Checkbox, {
      props: { disabled: true }
    });

    expect(getByRole('checkbox')).toBeDisabled();
  });
});

describe('Toggle', () => {
  it('should render unchecked by default', () => {
    const { getByRole } = render(Toggle);
    expect(getByRole('switch')).toHaveAttribute('aria-checked', 'false');
  });

  it('should toggle on click', async () => {
    const handleChange = vi.fn();
    const { getByRole, component } = render(Toggle);

    component.$on('change', handleChange);
    await fireEvent.click(getByRole('switch'));

    expect(handleChange).toHaveBeenCalledWith(
      expect.objectContaining({ detail: true })
    );
  });

  it('should toggle on space key', async () => {
    const handleChange = vi.fn();
    const { getByRole, component } = render(Toggle);

    component.$on('change', handleChange);
    await fireEvent.keyDown(getByRole('switch'), { key: ' ' });

    expect(handleChange).toHaveBeenCalled();
  });

  it('should render label on right by default', () => {
    const { container } = render(Toggle, {
      props: { label: 'Enable notifications' }
    });

    const wrapper = container.querySelector('.toggle-wrapper');
    expect(wrapper).not.toHaveClass('toggle-label-left');
  });

  it('should render label on left when specified', () => {
    const { container } = render(Toggle, {
      props: { label: 'Enable notifications', labelPosition: 'left' }
    });

    const wrapper = container.querySelector('.toggle-wrapper');
    expect(wrapper).toHaveClass('toggle-label-left');
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [199-select-component.md](./199-select-component.md) - Select component
- [213-form-validation.md](./213-form-validation.md) - Form validation
