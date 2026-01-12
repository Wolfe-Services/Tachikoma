# Spec 198: Input Component

## Phase
Phase 9: UI Foundation

## Spec ID
198

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 191-195: Design System

## Estimated Context
~10%

---

## Objective

Implement a flexible Input component for Tachikoma supporting various input types, validation states, adornments (icons, prefixes, suffixes), and seamless form integration with full accessibility compliance.

---

## Acceptance Criteria

- [x] Multiple input types (text, email, password, number, search, url)
- [x] Three sizes (sm, md, lg)
- [x] Validation states (error, success)
- [x] Helper text and error messages
- [x] Left and right adornments (icons, text)
- [x] Password visibility toggle
- [x] Clearable input
- [x] Character count
- [x] Disabled and readonly states
- [x] Full keyboard and screen reader support

---

## Implementation Details

### src/lib/components/ui/Input/Input.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { HTMLInputAttributes } from 'svelte/elements';
  import { generateId, cn } from '@utils/component';
  import { getAriaDescribedBy } from '@utils/a11y';
  import Icon from '../Icon/Icon.svelte';

  type InputSize = 'sm' | 'md' | 'lg';
  type InputType = 'text' | 'email' | 'password' | 'number' | 'search' | 'url' | 'tel';

  interface $$Props extends Omit<HTMLInputAttributes, 'size'> {
    type?: InputType;
    size?: InputSize;
    value?: string;
    label?: string;
    placeholder?: string;
    helperText?: string;
    error?: string | boolean;
    success?: boolean;
    disabled?: boolean;
    readonly?: boolean;
    required?: boolean;
    clearable?: boolean;
    showPasswordToggle?: boolean;
    maxLength?: number;
    showCharacterCount?: boolean;
    class?: string;
  }

  export let type: InputType = 'text';
  export let size: InputSize = 'md';
  export let value: string = '';
  export let label: string | undefined = undefined;
  export let placeholder: string = '';
  export let helperText: string | undefined = undefined;
  export let error: string | boolean = false;
  export let success: boolean = false;
  export let disabled: boolean = false;
  export let readonly: boolean = false;
  export let required: boolean = false;
  export let clearable: boolean = false;
  export let showPasswordToggle: boolean = false;
  export let maxLength: number | undefined = undefined;
  export let showCharacterCount: boolean = false;
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    input: Event;
    change: Event;
    focus: FocusEvent;
    blur: FocusEvent;
    clear: void;
  }>();

  const id = generateId('input');
  let inputElement: HTMLInputElement;
  let isFocused = false;
  let showPassword = false;

  $: actualType = type === 'password' && showPassword ? 'text' : type;
  $: hasError = !!error;
  $: errorMessage = typeof error === 'string' ? error : undefined;
  $: hasLeftAdornment = $$slots.leftAdornment;
  $: hasRightAdornment = $$slots.rightAdornment || clearable || (type === 'password' && showPasswordToggle);
  $: characterCount = value?.length ?? 0;

  $: wrapperClasses = cn(
    'input-wrapper',
    `input-${size}`,
    isFocused && 'input-focused',
    hasError && 'input-error',
    success && !hasError && 'input-success',
    disabled && 'input-disabled',
    readonly && 'input-readonly',
    hasLeftAdornment && 'has-left-adornment',
    hasRightAdornment && 'has-right-adornment',
    className
  );

  function handleInput(event: Event) {
    value = (event.target as HTMLInputElement).value;
    dispatch('input', event);
  }

  function handleChange(event: Event) {
    dispatch('change', event);
  }

  function handleFocus(event: FocusEvent) {
    isFocused = true;
    dispatch('focus', event);
  }

  function handleBlur(event: FocusEvent) {
    isFocused = false;
    dispatch('blur', event);
  }

  function handleClear() {
    value = '';
    dispatch('clear');
    inputElement?.focus();
  }

  function togglePassword() {
    showPassword = !showPassword;
  }

  export function focus() {
    inputElement?.focus();
  }

  export function blur() {
    inputElement?.blur();
  }

  export function select() {
    inputElement?.select();
  }
</script>

<div class="input-field">
  {#if label}
    <label for={id} class="input-label">
      {label}
      {#if required}
        <span class="input-required" aria-hidden="true">*</span>
      {/if}
    </label>
  {/if}

  <div class={wrapperClasses}>
    {#if $$slots.leftAdornment}
      <div class="input-adornment input-adornment-left">
        <slot name="leftAdornment" />
      </div>
    {/if}

    <input
      bind:this={inputElement}
      {id}
      type={actualType}
      {value}
      {placeholder}
      {disabled}
      {readonly}
      {required}
      maxlength={maxLength}
      class="input-element"
      aria-invalid={hasError}
      aria-describedby={getAriaDescribedBy(id, hasError, !!helperText)}
      on:input={handleInput}
      on:change={handleChange}
      on:focus={handleFocus}
      on:blur={handleBlur}
      {...$$restProps}
    />

    {#if hasRightAdornment}
      <div class="input-adornment input-adornment-right">
        {#if clearable && value && !disabled && !readonly}
          <button
            type="button"
            class="input-clear-btn"
            on:click={handleClear}
            tabindex="-1"
            aria-label="Clear input"
          >
            <Icon name="x" size={16} />
          </button>
        {/if}

        {#if type === 'password' && showPasswordToggle}
          <button
            type="button"
            class="input-toggle-btn"
            on:click={togglePassword}
            tabindex="-1"
            aria-label={showPassword ? 'Hide password' : 'Show password'}
          >
            <Icon name={showPassword ? 'eye-off' : 'eye'} size={16} />
          </button>
        {/if}

        <slot name="rightAdornment" />
      </div>
    {/if}
  </div>

  {#if helperText || errorMessage || (showCharacterCount && maxLength)}
    <div class="input-footer">
      {#if errorMessage}
        <span id="{id}-error" class="input-helper input-helper-error" role="alert">
          {errorMessage}
        </span>
      {:else if helperText}
        <span id="{id}-helper" class="input-helper">
          {helperText}
        </span>
      {:else}
        <span></span>
      {/if}

      {#if showCharacterCount && maxLength}
        <span class="input-character-count" class:over-limit={characterCount > maxLength}>
          {characterCount}/{maxLength}
        </span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .input-field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1-5);
    width: 100%;
  }

  .input-label {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-fg-default);
  }

  .input-required {
    color: var(--color-error-fg);
  }

  .input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    background-color: var(--color-bg-input);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    transition:
      border-color var(--duration-150) var(--ease-out),
      box-shadow var(--duration-150) var(--ease-out),
      background-color var(--duration-150) var(--ease-out);
  }

  .input-wrapper:hover:not(.input-disabled):not(.input-readonly) {
    border-color: var(--color-border-strong);
  }

  .input-wrapper.input-focused {
    border-color: var(--tachikoma-500);
    box-shadow: var(--focus-ring);
  }

  .input-wrapper.input-error {
    border-color: var(--color-error-fg);
  }

  .input-wrapper.input-error.input-focused {
    box-shadow: var(--focus-ring-error);
  }

  .input-wrapper.input-success {
    border-color: var(--color-success-fg);
  }

  .input-wrapper.input-success.input-focused {
    box-shadow: var(--focus-ring-success);
  }

  .input-wrapper.input-disabled {
    background-color: var(--color-bg-muted);
    opacity: 0.5;
    cursor: not-allowed;
  }

  .input-wrapper.input-readonly {
    background-color: var(--color-bg-muted);
  }

  /* Sizes */
  .input-sm {
    height: var(--spacing-8);
  }

  .input-sm .input-element {
    font-size: var(--text-sm);
    padding: 0 var(--spacing-2-5);
  }

  .input-md {
    height: var(--spacing-10);
  }

  .input-md .input-element {
    font-size: var(--text-sm);
    padding: 0 var(--spacing-3);
  }

  .input-lg {
    height: var(--spacing-12);
  }

  .input-lg .input-element {
    font-size: var(--text-base);
    padding: 0 var(--spacing-4);
  }

  /* Input element */
  .input-element {
    flex: 1;
    min-width: 0;
    height: 100%;
    background: transparent;
    border: none;
    color: var(--color-fg-default);
    font-family: var(--font-sans);
  }

  .input-element::placeholder {
    color: var(--color-fg-subtle);
  }

  .input-element:focus {
    outline: none;
  }

  .input-element:disabled {
    cursor: not-allowed;
  }

  .input-element:read-only {
    cursor: default;
  }

  /* Hide number spinners */
  .input-element[type="number"]::-webkit-outer-spin-button,
  .input-element[type="number"]::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }

  .input-element[type="number"] {
    -moz-appearance: textfield;
  }

  /* Adornments */
  .input-adornment {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    color: var(--color-fg-muted);
    flex-shrink: 0;
  }

  .input-adornment-left {
    padding-left: var(--spacing-3);
  }

  .input-adornment-right {
    padding-right: var(--spacing-2);
  }

  .has-left-adornment .input-element {
    padding-left: var(--spacing-2);
  }

  .has-right-adornment .input-element {
    padding-right: var(--spacing-2);
  }

  /* Clear and toggle buttons */
  .input-clear-btn,
  .input-toggle-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: var(--spacing-6);
    height: var(--spacing-6);
    padding: 0;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-fg-muted);
    cursor: pointer;
    transition: color var(--duration-150) var(--ease-out),
                background-color var(--duration-150) var(--ease-out);
  }

  .input-clear-btn:hover,
  .input-toggle-btn:hover {
    color: var(--color-fg-default);
    background-color: var(--color-bg-hover);
  }

  /* Footer */
  .input-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    min-height: var(--spacing-5);
  }

  .input-helper {
    font-size: var(--text-xs);
    color: var(--color-fg-muted);
  }

  .input-helper-error {
    color: var(--color-error-fg);
  }

  .input-character-count {
    font-size: var(--text-xs);
    color: var(--color-fg-muted);
    font-variant-numeric: tabular-nums;
  }

  .input-character-count.over-limit {
    color: var(--color-error-fg);
  }
</style>
```

### src/lib/components/ui/Input/Textarea.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { HTMLTextareaAttributes } from 'svelte/elements';
  import { generateId, cn } from '@utils/component';
  import { getAriaDescribedBy } from '@utils/a11y';

  type TextareaSize = 'sm' | 'md' | 'lg';

  interface $$Props extends Omit<HTMLTextareaAttributes, 'rows'> {
    size?: TextareaSize;
    value?: string;
    label?: string;
    placeholder?: string;
    helperText?: string;
    error?: string | boolean;
    success?: boolean;
    disabled?: boolean;
    readonly?: boolean;
    required?: boolean;
    rows?: number;
    minRows?: number;
    maxRows?: number;
    autoResize?: boolean;
    maxLength?: number;
    showCharacterCount?: boolean;
    class?: string;
  }

  export let size: TextareaSize = 'md';
  export let value: string = '';
  export let label: string | undefined = undefined;
  export let placeholder: string = '';
  export let helperText: string | undefined = undefined;
  export let error: string | boolean = false;
  export let success: boolean = false;
  export let disabled: boolean = false;
  export let readonly: boolean = false;
  export let required: boolean = false;
  export let rows: number = 3;
  export let minRows: number = 3;
  export let maxRows: number = 10;
  export let autoResize: boolean = false;
  export let maxLength: number | undefined = undefined;
  export let showCharacterCount: boolean = false;
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    input: Event;
    change: Event;
    focus: FocusEvent;
    blur: FocusEvent;
  }>();

  const id = generateId('textarea');
  let textareaElement: HTMLTextAreaElement;
  let isFocused = false;

  $: hasError = !!error;
  $: errorMessage = typeof error === 'string' ? error : undefined;
  $: characterCount = value?.length ?? 0;

  $: wrapperClasses = cn(
    'textarea-wrapper',
    `textarea-${size}`,
    isFocused && 'textarea-focused',
    hasError && 'textarea-error',
    success && !hasError && 'textarea-success',
    disabled && 'textarea-disabled',
    readonly && 'textarea-readonly',
    className
  );

  function handleInput(event: Event) {
    value = (event.target as HTMLTextAreaElement).value;

    if (autoResize) {
      adjustHeight();
    }

    dispatch('input', event);
  }

  function adjustHeight() {
    if (!textareaElement) return;

    textareaElement.style.height = 'auto';
    const scrollHeight = textareaElement.scrollHeight;
    const lineHeight = parseInt(getComputedStyle(textareaElement).lineHeight);
    const minHeight = minRows * lineHeight;
    const maxHeight = maxRows * lineHeight;

    textareaElement.style.height = `${Math.min(Math.max(scrollHeight, minHeight), maxHeight)}px`;
  }

  function handleFocus(event: FocusEvent) {
    isFocused = true;
    dispatch('focus', event);
  }

  function handleBlur(event: FocusEvent) {
    isFocused = false;
    dispatch('blur', event);
  }
</script>

<div class="textarea-field">
  {#if label}
    <label for={id} class="textarea-label">
      {label}
      {#if required}
        <span class="textarea-required" aria-hidden="true">*</span>
      {/if}
    </label>
  {/if}

  <div class={wrapperClasses}>
    <textarea
      bind:this={textareaElement}
      {id}
      {value}
      {placeholder}
      {disabled}
      {readonly}
      {required}
      {rows}
      maxlength={maxLength}
      class="textarea-element"
      aria-invalid={hasError}
      aria-describedby={getAriaDescribedBy(id, hasError, !!helperText)}
      on:input={handleInput}
      on:change={(e) => dispatch('change', e)}
      on:focus={handleFocus}
      on:blur={handleBlur}
      {...$$restProps}
    />
  </div>

  {#if helperText || errorMessage || (showCharacterCount && maxLength)}
    <div class="textarea-footer">
      {#if errorMessage}
        <span id="{id}-error" class="textarea-helper textarea-helper-error" role="alert">
          {errorMessage}
        </span>
      {:else if helperText}
        <span id="{id}-helper" class="textarea-helper">
          {helperText}
        </span>
      {:else}
        <span></span>
      {/if}

      {#if showCharacterCount && maxLength}
        <span class="textarea-character-count" class:over-limit={characterCount > maxLength}>
          {characterCount}/{maxLength}
        </span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .textarea-field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1-5);
    width: 100%;
  }

  .textarea-label {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-fg-default);
  }

  .textarea-required {
    color: var(--color-error-fg);
  }

  .textarea-wrapper {
    position: relative;
    background-color: var(--color-bg-input);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    transition:
      border-color var(--duration-150) var(--ease-out),
      box-shadow var(--duration-150) var(--ease-out);
  }

  .textarea-wrapper:hover:not(.textarea-disabled):not(.textarea-readonly) {
    border-color: var(--color-border-strong);
  }

  .textarea-wrapper.textarea-focused {
    border-color: var(--tachikoma-500);
    box-shadow: var(--focus-ring);
  }

  .textarea-wrapper.textarea-error {
    border-color: var(--color-error-fg);
  }

  .textarea-wrapper.textarea-disabled {
    background-color: var(--color-bg-muted);
    opacity: 0.5;
    cursor: not-allowed;
  }

  .textarea-element {
    width: 100%;
    min-height: 80px;
    padding: var(--spacing-3);
    background: transparent;
    border: none;
    border-radius: var(--radius-md);
    color: var(--color-fg-default);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    line-height: var(--leading-relaxed);
    resize: vertical;
  }

  .textarea-element::placeholder {
    color: var(--color-fg-subtle);
  }

  .textarea-element:focus {
    outline: none;
  }

  .textarea-element:disabled {
    cursor: not-allowed;
    resize: none;
  }

  /* Sizes */
  .textarea-sm .textarea-element {
    padding: var(--spacing-2);
    font-size: var(--text-sm);
  }

  .textarea-lg .textarea-element {
    padding: var(--spacing-4);
    font-size: var(--text-base);
  }

  /* Footer */
  .textarea-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .textarea-helper {
    font-size: var(--text-xs);
    color: var(--color-fg-muted);
  }

  .textarea-helper-error {
    color: var(--color-error-fg);
  }

  .textarea-character-count {
    font-size: var(--text-xs);
    color: var(--color-fg-muted);
    font-variant-numeric: tabular-nums;
  }

  .textarea-character-count.over-limit {
    color: var(--color-error-fg);
  }
</style>
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/Input.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import Input from '@components/ui/Input/Input.svelte';

describe('Input', () => {
  it('should render with label', () => {
    const { getByLabelText } = render(Input, {
      props: { label: 'Email', type: 'email' }
    });

    expect(getByLabelText('Email')).toBeInTheDocument();
  });

  it('should handle value changes', async () => {
    const { getByRole, component } = render(Input);
    const input = getByRole('textbox');

    await fireEvent.input(input, { target: { value: 'test' } });

    expect(component.value).toBe('test');
  });

  it('should show error state', () => {
    const { container, getByRole, getByText } = render(Input, {
      props: { error: 'Invalid email' }
    });

    expect(getByRole('textbox')).toHaveAttribute('aria-invalid', 'true');
    expect(getByText('Invalid email')).toBeInTheDocument();
    expect(container.querySelector('.input-error')).toBeInTheDocument();
  });

  it('should show helper text', () => {
    const { getByText } = render(Input, {
      props: { helperText: 'Enter your email' }
    });

    expect(getByText('Enter your email')).toBeInTheDocument();
  });

  it('should handle clearable input', async () => {
    const { getByLabelText, queryByLabelText } = render(Input, {
      props: { clearable: true, value: 'test' }
    });

    const clearBtn = getByLabelText('Clear input');
    expect(clearBtn).toBeInTheDocument();

    await fireEvent.click(clearBtn);
  });

  it('should toggle password visibility', async () => {
    const { getByRole, getByLabelText } = render(Input, {
      props: { type: 'password', showPasswordToggle: true }
    });

    const input = getByRole('textbox') as HTMLInputElement;
    expect(input.type).toBe('password');

    await fireEvent.click(getByLabelText('Show password'));
    expect(input.type).toBe('text');

    await fireEvent.click(getByLabelText('Hide password'));
    expect(input.type).toBe('password');
  });

  it('should show character count', () => {
    const { getByText } = render(Input, {
      props: {
        value: 'hello',
        maxLength: 10,
        showCharacterCount: true
      }
    });

    expect(getByText('5/10')).toBeInTheDocument();
  });

  it('should be disabled', () => {
    const { getByRole } = render(Input, {
      props: { disabled: true }
    });

    expect(getByRole('textbox')).toBeDisabled();
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [197-button-component.md](./197-button-component.md) - Button component
- [199-select-component.md](./199-select-component.md) - Select component
- [213-form-validation.md](./213-form-validation.md) - Form validation
