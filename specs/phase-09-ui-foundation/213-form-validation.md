# Spec 213: Form Validation System

## Phase
Phase 9: UI Foundation

## Spec ID
213

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 198: Input Component
- Spec 199: Select Component
- Spec 200: Checkbox/Toggle Components

## Estimated Context
~12%

---

## Objective

Implement a comprehensive form validation system for Tachikoma with reactive validation, custom validators, async validation, field-level and form-level validation, error messages, and integration with form components.

---

## Acceptance Criteria

- [ ] Reactive form state management
- [ ] Built-in validators (required, email, min, max, pattern, etc.)
- [ ] Custom validator support
- [ ] Async validation (e.g., username availability)
- [ ] Field-level and form-level validation
- [ ] Validation modes (onChange, onBlur, onSubmit)
- [ ] Error message templates
- [ ] Field dirty/touched tracking
- [ ] Form reset functionality
- [ ] Integration with UI components

---

## Implementation Details

### src/lib/utils/validation/validators.ts

```typescript
export type Validator<T = any> = (value: T, formValues?: Record<string, any>) => string | null | Promise<string | null>;

export interface ValidatorOptions {
  message?: string;
}

// Required validator
export function required(options: ValidatorOptions = {}): Validator {
  return (value) => {
    const isEmpty = value === undefined || value === null || value === '' ||
      (Array.isArray(value) && value.length === 0);

    return isEmpty ? (options.message || 'This field is required') : null;
  };
}

// Email validator
export function email(options: ValidatorOptions = {}): Validator<string> {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return (value) => {
    if (!value) return null;
    return emailRegex.test(value)
      ? null
      : (options.message || 'Please enter a valid email address');
  };
}

// Minimum length validator
export function minLength(min: number, options: ValidatorOptions = {}): Validator<string> {
  return (value) => {
    if (!value) return null;
    return value.length >= min
      ? null
      : (options.message || `Must be at least ${min} characters`);
  };
}

// Maximum length validator
export function maxLength(max: number, options: ValidatorOptions = {}): Validator<string> {
  return (value) => {
    if (!value) return null;
    return value.length <= max
      ? null
      : (options.message || `Must be no more than ${max} characters`);
  };
}

// Minimum value validator
export function min(minValue: number, options: ValidatorOptions = {}): Validator<number> {
  return (value) => {
    if (value === undefined || value === null) return null;
    return value >= minValue
      ? null
      : (options.message || `Must be at least ${minValue}`);
  };
}

// Maximum value validator
export function max(maxValue: number, options: ValidatorOptions = {}): Validator<number> {
  return (value) => {
    if (value === undefined || value === null) return null;
    return value <= maxValue
      ? null
      : (options.message || `Must be no more than ${maxValue}`);
  };
}

// Pattern validator
export function pattern(regex: RegExp, options: ValidatorOptions = {}): Validator<string> {
  return (value) => {
    if (!value) return null;
    return regex.test(value)
      ? null
      : (options.message || 'Invalid format');
  };
}

// Match another field validator
export function matches(fieldName: string, options: ValidatorOptions = {}): Validator<any> {
  return (value, formValues) => {
    if (!formValues) return null;
    return value === formValues[fieldName]
      ? null
      : (options.message || `Must match ${fieldName}`);
  };
}

// URL validator
export function url(options: ValidatorOptions = {}): Validator<string> {
  return (value) => {
    if (!value) return null;
    try {
      new URL(value);
      return null;
    } catch {
      return options.message || 'Please enter a valid URL';
    }
  };
}

// IP address validator
export function ipAddress(options: ValidatorOptions = {}): Validator<string> {
  const ipv4Regex = /^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;
  const ipv6Regex = /^(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$/;

  return (value) => {
    if (!value) return null;
    return (ipv4Regex.test(value) || ipv6Regex.test(value))
      ? null
      : (options.message || 'Please enter a valid IP address');
  };
}

// Port validator
export function port(options: ValidatorOptions = {}): Validator<number | string> {
  return (value) => {
    if (value === undefined || value === null || value === '') return null;
    const portNum = typeof value === 'string' ? parseInt(value, 10) : value;
    return (portNum >= 1 && portNum <= 65535)
      ? null
      : (options.message || 'Please enter a valid port number (1-65535)');
  };
}

// Custom validator factory
export function custom<T>(
  validateFn: (value: T, formValues?: Record<string, any>) => boolean,
  options: ValidatorOptions = {}
): Validator<T> {
  return (value, formValues) => {
    return validateFn(value, formValues)
      ? null
      : (options.message || 'Invalid value');
  };
}

// Async validator factory
export function asyncValidator<T>(
  validateFn: (value: T) => Promise<boolean>,
  options: ValidatorOptions & { debounce?: number } = {}
): Validator<T> {
  let timeoutId: ReturnType<typeof setTimeout>;

  return async (value) => {
    if (options.debounce) {
      return new Promise((resolve) => {
        clearTimeout(timeoutId);
        timeoutId = setTimeout(async () => {
          const isValid = await validateFn(value);
          resolve(isValid ? null : (options.message || 'Invalid value'));
        }, options.debounce);
      });
    }

    const isValid = await validateFn(value);
    return isValid ? null : (options.message || 'Invalid value');
  };
}

// Compose multiple validators
export function compose<T>(...validators: Validator<T>[]): Validator<T> {
  return async (value, formValues) => {
    for (const validator of validators) {
      const error = await validator(value, formValues);
      if (error) return error;
    }
    return null;
  };
}
```

### src/lib/utils/validation/form.ts

```typescript
import { writable, derived, get } from 'svelte/store';
import type { Validator } from './validators';

export type ValidationMode = 'onChange' | 'onBlur' | 'onSubmit';

export interface FieldConfig<T = any> {
  initialValue: T;
  validators?: Validator<T>[];
  validateOn?: ValidationMode[];
}

export interface FieldState<T = any> {
  value: T;
  error: string | null;
  touched: boolean;
  dirty: boolean;
  validating: boolean;
}

export interface FormState {
  isValid: boolean;
  isSubmitting: boolean;
  isDirty: boolean;
  submitCount: number;
  errors: Record<string, string | null>;
}

export interface FormConfig<T extends Record<string, any>> {
  fields: { [K in keyof T]: FieldConfig<T[K]> };
  validateOn?: ValidationMode[];
  onSubmit?: (values: T) => void | Promise<void>;
}

export function createForm<T extends Record<string, any>>(config: FormConfig<T>) {
  const { fields, validateOn = ['onBlur', 'onSubmit'] } = config;

  // Initialize field states
  const initialFieldStates: Record<string, FieldState> = {};
  const initialValues: Record<string, any> = {};

  for (const [name, fieldConfig] of Object.entries(fields)) {
    initialValues[name] = fieldConfig.initialValue;
    initialFieldStates[name] = {
      value: fieldConfig.initialValue,
      error: null,
      touched: false,
      dirty: false,
      validating: false
    };
  }

  const fieldStates = writable<Record<string, FieldState>>(initialFieldStates);
  const formState = writable<FormState>({
    isValid: true,
    isSubmitting: false,
    isDirty: false,
    submitCount: 0,
    errors: {}
  });

  // Derived store for current values
  const values = derived(fieldStates, $fields => {
    const vals: Record<string, any> = {};
    for (const [name, state] of Object.entries($fields)) {
      vals[name] = state.value;
    }
    return vals as T;
  });

  // Validate a single field
  async function validateField(name: string): Promise<string | null> {
    const fieldConfig = fields[name as keyof T];
    if (!fieldConfig?.validators?.length) return null;

    const currentStates = get(fieldStates);
    const currentValues = get(values);
    const value = currentStates[name].value;

    fieldStates.update(states => ({
      ...states,
      [name]: { ...states[name], validating: true }
    }));

    for (const validator of fieldConfig.validators) {
      const error = await validator(value, currentValues);
      if (error) {
        fieldStates.update(states => ({
          ...states,
          [name]: { ...states[name], error, validating: false }
        }));
        updateFormValidity();
        return error;
      }
    }

    fieldStates.update(states => ({
      ...states,
      [name]: { ...states[name], error: null, validating: false }
    }));
    updateFormValidity();
    return null;
  }

  // Validate all fields
  async function validateAll(): Promise<boolean> {
    const errors: Record<string, string | null> = {};

    for (const name of Object.keys(fields)) {
      errors[name] = await validateField(name);
    }

    const isValid = Object.values(errors).every(e => e === null);
    formState.update(state => ({ ...state, isValid, errors }));

    return isValid;
  }

  // Update form validity
  function updateFormValidity() {
    const currentStates = get(fieldStates);
    const hasErrors = Object.values(currentStates).some(s => s.error !== null);
    const isDirty = Object.values(currentStates).some(s => s.dirty);

    formState.update(state => ({
      ...state,
      isValid: !hasErrors,
      isDirty,
      errors: Object.fromEntries(
        Object.entries(currentStates).map(([k, v]) => [k, v.error])
      )
    }));
  }

  // Field handlers
  function handleChange(name: string, value: any) {
    fieldStates.update(states => ({
      ...states,
      [name]: {
        ...states[name],
        value,
        dirty: value !== initialValues[name]
      }
    }));

    const fieldValidateOn = fields[name as keyof T]?.validateOn || validateOn;
    if (fieldValidateOn.includes('onChange')) {
      validateField(name);
    }
  }

  function handleBlur(name: string) {
    fieldStates.update(states => ({
      ...states,
      [name]: { ...states[name], touched: true }
    }));

    const fieldValidateOn = fields[name as keyof T]?.validateOn || validateOn;
    if (fieldValidateOn.includes('onBlur')) {
      validateField(name);
    }
  }

  // Submit handler
  async function handleSubmit(event?: Event) {
    event?.preventDefault();

    formState.update(state => ({
      ...state,
      isSubmitting: true,
      submitCount: state.submitCount + 1
    }));

    // Mark all fields as touched
    fieldStates.update(states => {
      const updated: Record<string, FieldState> = {};
      for (const [name, state] of Object.entries(states)) {
        updated[name] = { ...state, touched: true };
      }
      return updated;
    });

    const isValid = await validateAll();

    if (isValid && config.onSubmit) {
      try {
        await config.onSubmit(get(values));
      } catch (error) {
        console.error('Form submission error:', error);
      }
    }

    formState.update(state => ({ ...state, isSubmitting: false }));
    return isValid;
  }

  // Reset form
  function reset(newValues?: Partial<T>) {
    fieldStates.update(states => {
      const updated: Record<string, FieldState> = {};
      for (const [name, fieldConfig] of Object.entries(fields)) {
        const value = newValues?.[name as keyof T] ?? fieldConfig.initialValue;
        updated[name] = {
          value,
          error: null,
          touched: false,
          dirty: false,
          validating: false
        };
      }
      return updated;
    });

    formState.set({
      isValid: true,
      isSubmitting: false,
      isDirty: false,
      submitCount: 0,
      errors: {}
    });
  }

  // Set field value programmatically
  function setValue(name: string, value: any) {
    handleChange(name, value);
  }

  // Set field error programmatically
  function setError(name: string, error: string | null) {
    fieldStates.update(states => ({
      ...states,
      [name]: { ...states[name], error }
    }));
    updateFormValidity();
  }

  // Get field props for binding
  function getFieldProps(name: string) {
    return {
      name,
      onChange: (e: Event) => handleChange(name, (e.target as HTMLInputElement).value),
      onBlur: () => handleBlur(name)
    };
  }

  return {
    // Stores
    fieldStates,
    formState,
    values,

    // Methods
    validateField,
    validateAll,
    handleChange,
    handleBlur,
    handleSubmit,
    reset,
    setValue,
    setError,
    getFieldProps,

    // Field accessor
    field: (name: keyof T) => derived(fieldStates, $states => $states[name as string])
  };
}
```

### src/lib/components/ui/Form/Form.svelte

```svelte
<script lang="ts">
  import { setContext } from 'svelte';
  import { cn } from '@utils/component';
  import type { createForm } from '@utils/validation/form';

  type FormInstance = ReturnType<typeof createForm<any>>;

  export let form: FormInstance;
  export let autocomplete: 'on' | 'off' = 'off';
  let className: string = '';
  export { className as class };

  setContext('form', form);

  $: classes = cn('form', className);
</script>

<form
  class={classes}
  {autocomplete}
  on:submit={form.handleSubmit}
  {...$$restProps}
>
  <slot
    values={$form.values}
    formState={$form.formState}
    isSubmitting={$form.formState.isSubmitting}
    isValid={$form.formState.isValid}
  />
</form>

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-4);
  }
</style>
```

### src/lib/components/ui/Form/FormField.svelte

```svelte
<script lang="ts">
  import { getContext } from 'svelte';
  import { cn } from '@utils/component';
  import type { createForm } from '@utils/validation/form';

  type FormInstance = ReturnType<typeof createForm<any>>;

  export let name: string;
  export let label: string = '';
  export let hint: string = '';
  export let required: boolean = false;
  let className: string = '';
  export { className as class };

  const form = getContext<FormInstance | undefined>('form');

  $: fieldState = form ? $form.field(name) : null;
  $: error = fieldState?.error;
  $: touched = fieldState?.touched;
  $: showError = touched && error;

  $: classes = cn(
    'form-field',
    showError && 'form-field-error',
    className
  );
</script>

<div class={classes}>
  {#if label}
    <label class="form-field-label" for={name}>
      {label}
      {#if required}
        <span class="form-field-required">*</span>
      {/if}
    </label>
  {/if}

  <div class="form-field-control">
    <slot
      {name}
      value={fieldState?.value}
      error={showError ? error : null}
      onChange={(value) => form?.handleChange(name, value)}
      onBlur={() => form?.handleBlur(name)}
    />
  </div>

  {#if hint && !showError}
    <p class="form-field-hint">{hint}</p>
  {/if}

  {#if showError}
    <p class="form-field-error-message" role="alert">{error}</p>
  {/if}
</div>

<style>
  .form-field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1);
  }

  .form-field-label {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-fg-default);
  }

  .form-field-required {
    color: var(--color-danger-500);
    margin-left: var(--spacing-1);
  }

  .form-field-control {
    display: flex;
    flex-direction: column;
  }

  .form-field-hint {
    font-size: var(--text-xs);
    color: var(--color-fg-muted);
    margin: 0;
  }

  .form-field-error-message {
    font-size: var(--text-xs);
    color: var(--color-danger-500);
    margin: 0;
  }

  .form-field-error :global(input),
  .form-field-error :global(select),
  .form-field-error :global(textarea) {
    border-color: var(--color-danger-500);
  }
</style>
```

### src/lib/components/ui/Form/FormActions.svelte

```svelte
<script lang="ts">
  import { getContext } from 'svelte';
  import { cn } from '@utils/component';
  import Button from '../Button/Button.svelte';
  import type { createForm } from '@utils/validation/form';

  type FormInstance = ReturnType<typeof createForm<any>>;

  export let submitLabel: string = 'Submit';
  export let resetLabel: string = 'Reset';
  export let showReset: boolean = false;
  export let align: 'left' | 'center' | 'right' | 'space-between' = 'right';
  let className: string = '';
  export { className as class };

  const form = getContext<FormInstance | undefined>('form');

  $: formState = form ? $form.formState : null;
  $: isSubmitting = formState?.isSubmitting ?? false;

  $: classes = cn(
    'form-actions',
    `form-actions-${align}`,
    className
  );
</script>

<div class={classes}>
  <slot>
    {#if showReset}
      <Button
        type="button"
        variant="ghost"
        on:click={() => form?.reset()}
        disabled={isSubmitting}
      >
        {resetLabel}
      </Button>
    {/if}

    <Button
      type="submit"
      variant="primary"
      loading={isSubmitting}
      disabled={isSubmitting}
    >
      {submitLabel}
    </Button>
  </slot>
</div>

<style>
  .form-actions {
    display: flex;
    gap: var(--spacing-3);
    padding-top: var(--spacing-4);
  }

  .form-actions-left {
    justify-content: flex-start;
  }

  .form-actions-center {
    justify-content: center;
  }

  .form-actions-right {
    justify-content: flex-end;
  }

  .form-actions-space-between {
    justify-content: space-between;
  }
</style>
```

### src/lib/components/ui/Form/ValidatedInput.svelte

```svelte
<script lang="ts">
  import { getContext, createEventDispatcher } from 'svelte';
  import Input from '../Input/Input.svelte';
  import type { createForm } from '@utils/validation/form';

  type FormInstance = ReturnType<typeof createForm<any>>;

  export let name: string;
  export let type: 'text' | 'email' | 'password' | 'number' | 'url' | 'tel' = 'text';
  export let placeholder: string = '';
  export let disabled: boolean = false;
  export let readonly: boolean = false;

  const dispatch = createEventDispatcher();
  const form = getContext<FormInstance | undefined>('form');

  $: fieldState = form ? $form.field(name) : null;
  $: value = fieldState?.value ?? '';
  $: error = fieldState?.touched ? fieldState?.error : null;
  $: validating = fieldState?.validating ?? false;

  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    const newValue = type === 'number' ? parseFloat(target.value) : target.value;
    form?.handleChange(name, newValue);
    dispatch('input', newValue);
  }

  function handleBlur() {
    form?.handleBlur(name);
    dispatch('blur');
  }
</script>

<Input
  {name}
  {type}
  {placeholder}
  {disabled}
  {readonly}
  {value}
  error={error ?? undefined}
  loading={validating}
  on:input={handleInput}
  on:blur={handleBlur}
  {...$$restProps}
/>
```

### Usage Examples

```svelte
<script>
  import {
    Form,
    FormField,
    FormActions,
    ValidatedInput
  } from '@components/ui';
  import { createForm } from '@utils/validation/form';
  import {
    required,
    email,
    minLength,
    matches,
    compose,
    asyncValidator
  } from '@utils/validation/validators';

  // Create form instance
  const form = createForm({
    fields: {
      username: {
        initialValue: '',
        validators: [
          required({ message: 'Username is required' }),
          minLength(3, { message: 'Username must be at least 3 characters' }),
          asyncValidator(
            async (value) => {
              // Check username availability
              const response = await fetch(`/api/check-username?name=${value}`);
              const { available } = await response.json();
              return available;
            },
            { message: 'Username is already taken', debounce: 500 }
          )
        ]
      },
      email: {
        initialValue: '',
        validators: [
          required(),
          email()
        ]
      },
      password: {
        initialValue: '',
        validators: [
          required(),
          minLength(8, { message: 'Password must be at least 8 characters' })
        ]
      },
      confirmPassword: {
        initialValue: '',
        validators: [
          required(),
          matches('password', { message: 'Passwords do not match' })
        ]
      }
    },
    validateOn: ['onBlur'],
    onSubmit: async (values) => {
      console.log('Submitting:', values);
      await new Promise(r => setTimeout(r, 1000));
      alert('Form submitted!');
    }
  });

  // Access form state reactively
  $: ({ formState, values } = form);
</script>

<Form {form}>
  <FormField name="username" label="Username" required>
    <ValidatedInput name="username" placeholder="Enter username" />
  </FormField>

  <FormField name="email" label="Email" required>
    <ValidatedInput name="email" type="email" placeholder="Enter email" />
  </FormField>

  <FormField name="password" label="Password" required>
    <ValidatedInput name="password" type="password" placeholder="Enter password" />
  </FormField>

  <FormField
    name="confirmPassword"
    label="Confirm Password"
    required
    hint="Re-enter your password"
  >
    <ValidatedInput name="confirmPassword" type="password" placeholder="Confirm password" />
  </FormField>

  <FormActions submitLabel="Create Account" showReset />
</Form>

<!-- Manual form usage without Form component -->
<script>
  const manualForm = createForm({
    fields: {
      search: {
        initialValue: '',
        validators: [minLength(2)]
      }
    },
    validateOn: ['onChange']
  });
</script>

<form on:submit={manualForm.handleSubmit}>
  <input
    type="text"
    value={$manualForm.fieldStates.search.value}
    on:input={(e) => manualForm.handleChange('search', e.target.value)}
    on:blur={() => manualForm.handleBlur('search')}
  />
  {#if $manualForm.fieldStates.search.error}
    <span class="error">{$manualForm.fieldStates.search.error}</span>
  {/if}
  <button type="submit" disabled={!$manualForm.formState.isValid}>
    Search
  </button>
</form>
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/validation/validators.test.ts
import { describe, it, expect, vi } from 'vitest';
import {
  required,
  email,
  minLength,
  maxLength,
  min,
  max,
  pattern,
  matches,
  url,
  ipAddress,
  port,
  compose,
  asyncValidator
} from '@utils/validation/validators';

describe('validators', () => {
  describe('required', () => {
    it('should fail for empty values', () => {
      const validator = required();
      expect(validator('')).not.toBeNull();
      expect(validator(null)).not.toBeNull();
      expect(validator(undefined)).not.toBeNull();
      expect(validator([])).not.toBeNull();
    });

    it('should pass for non-empty values', () => {
      const validator = required();
      expect(validator('test')).toBeNull();
      expect(validator(0)).toBeNull();
      expect(validator([1])).toBeNull();
    });

    it('should use custom message', () => {
      const validator = required({ message: 'Custom required' });
      expect(validator('')).toBe('Custom required');
    });
  });

  describe('email', () => {
    it('should validate email format', () => {
      const validator = email();
      expect(validator('test@example.com')).toBeNull();
      expect(validator('invalid')).not.toBeNull();
      expect(validator('no@domain')).not.toBeNull();
    });

    it('should skip empty values', () => {
      const validator = email();
      expect(validator('')).toBeNull();
    });
  });

  describe('minLength', () => {
    it('should validate minimum length', () => {
      const validator = minLength(3);
      expect(validator('ab')).not.toBeNull();
      expect(validator('abc')).toBeNull();
      expect(validator('abcd')).toBeNull();
    });
  });

  describe('matches', () => {
    it('should validate field matching', () => {
      const validator = matches('password');
      const formValues = { password: 'secret' };

      expect(validator('secret', formValues)).toBeNull();
      expect(validator('different', formValues)).not.toBeNull();
    });
  });

  describe('ipAddress', () => {
    it('should validate IPv4 addresses', () => {
      const validator = ipAddress();
      expect(validator('192.168.1.1')).toBeNull();
      expect(validator('10.0.0.1')).toBeNull();
      expect(validator('256.0.0.1')).not.toBeNull();
      expect(validator('invalid')).not.toBeNull();
    });
  });

  describe('port', () => {
    it('should validate port numbers', () => {
      const validator = port();
      expect(validator(80)).toBeNull();
      expect(validator(443)).toBeNull();
      expect(validator(65535)).toBeNull();
      expect(validator(0)).not.toBeNull();
      expect(validator(65536)).not.toBeNull();
    });
  });

  describe('compose', () => {
    it('should run validators in order', async () => {
      const validator = compose(
        required(),
        minLength(3)
      );

      expect(await validator('')).toBe('This field is required');
      expect(await validator('ab')).toContain('at least 3');
      expect(await validator('abc')).toBeNull();
    });
  });

  describe('asyncValidator', () => {
    it('should handle async validation', async () => {
      const validator = asyncValidator(
        async (value) => value !== 'taken',
        { message: 'Already taken' }
      );

      expect(await validator('available')).toBeNull();
      expect(await validator('taken')).toBe('Already taken');
    });
  });
});

// tests/validation/form.test.ts
import { describe, it, expect, vi } from 'vitest';
import { get } from 'svelte/store';
import { createForm } from '@utils/validation/form';
import { required, email } from '@utils/validation/validators';

describe('createForm', () => {
  it('should initialize with field values', () => {
    const form = createForm({
      fields: {
        name: { initialValue: 'John' },
        email: { initialValue: '' }
      }
    });

    const values = get(form.values);
    expect(values.name).toBe('John');
    expect(values.email).toBe('');
  });

  it('should handle field changes', () => {
    const form = createForm({
      fields: {
        name: { initialValue: '' }
      }
    });

    form.handleChange('name', 'Jane');

    const states = get(form.fieldStates);
    expect(states.name.value).toBe('Jane');
    expect(states.name.dirty).toBe(true);
  });

  it('should validate fields', async () => {
    const form = createForm({
      fields: {
        email: {
          initialValue: '',
          validators: [required(), email()]
        }
      }
    });

    form.handleChange('email', 'invalid');
    const error = await form.validateField('email');

    expect(error).not.toBeNull();
    expect(error).toContain('valid email');
  });

  it('should handle form submission', async () => {
    const onSubmit = vi.fn();
    const form = createForm({
      fields: {
        name: {
          initialValue: 'John',
          validators: [required()]
        }
      },
      onSubmit
    });

    await form.handleSubmit();

    expect(onSubmit).toHaveBeenCalledWith({ name: 'John' });
  });

  it('should prevent submission with invalid fields', async () => {
    const onSubmit = vi.fn();
    const form = createForm({
      fields: {
        name: {
          initialValue: '',
          validators: [required()]
        }
      },
      onSubmit
    });

    await form.handleSubmit();

    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('should reset form', () => {
    const form = createForm({
      fields: {
        name: { initialValue: '' }
      }
    });

    form.handleChange('name', 'Changed');
    form.reset();

    const states = get(form.fieldStates);
    expect(states.name.value).toBe('');
    expect(states.name.dirty).toBe(false);
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [198-input-component.md](./198-input-component.md) - Input component
- [199-select-component.md](./199-select-component.md) - Select component
- [200-checkbox-toggle.md](./200-checkbox-toggle.md) - Checkbox/Toggle components
