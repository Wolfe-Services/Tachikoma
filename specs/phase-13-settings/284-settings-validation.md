# 284 - Settings Validation

**Phase:** 13 - Settings UI
**Spec ID:** 284
**Status:** Planned
**Dependencies:** 272-settings-store
**Estimated Context:** ~8% of model context window

---

## Objective

Create a comprehensive settings validation system with type-safe validators, real-time validation feedback, cross-field validation support, and detailed error messages for all settings categories.

---

## Acceptance Criteria

- [ ] Type-safe validation functions for all settings
- [ ] Real-time validation on input change
- [ ] Cross-field validation (dependencies between settings)
- [ ] Detailed error messages with suggestions
- [ ] Warning vs error distinction
- [ ] Validation result caching for performance
- [ ] Custom validator registration
- [ ] Async validation support (e.g., API key verification)

---

## Implementation Details

### 1. Validation Types (src/lib/types/validation.ts)

```typescript
/**
 * Settings validation type definitions.
 */

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

export interface ValidationError {
  path: string;
  field: string;
  message: string;
  code: ValidationErrorCode;
  suggestion?: string;
  value?: unknown;
}

export interface ValidationWarning {
  path: string;
  field: string;
  message: string;
  code: string;
}

export type ValidationErrorCode =
  | 'required'
  | 'type'
  | 'min'
  | 'max'
  | 'pattern'
  | 'enum'
  | 'custom'
  | 'dependency'
  | 'format'
  | 'range';

export interface FieldValidator<T = unknown> {
  validate: (value: T, context: ValidationContext) => ValidationResult;
  async?: boolean;
}

export interface ValidationContext {
  settings: import('./settings').AllSettings;
  path: string;
  field: string;
}

export interface ValidatorOptions {
  required?: boolean;
  min?: number;
  max?: number;
  pattern?: RegExp;
  enum?: readonly unknown[];
  custom?: (value: unknown, context: ValidationContext) => ValidationResult;
}

export type ValidatorMap = {
  [K in keyof import('./settings').AllSettings]?: {
    [F in keyof import('./settings').AllSettings[K]]?: FieldValidator<import('./settings').AllSettings[K][F]>;
  };
};
```

### 2. Validation Utilities (src/lib/utils/validators.ts)

```typescript
import type {
  ValidationResult,
  ValidationError,
  ValidationWarning,
  ValidationContext,
  ValidatorOptions,
  FieldValidator,
} from '$lib/types/validation';

export function createResult(
  errors: ValidationError[] = [],
  warnings: ValidationWarning[] = []
): ValidationResult {
  return {
    valid: errors.length === 0,
    errors,
    warnings,
  };
}

export function mergeResults(...results: ValidationResult[]): ValidationResult {
  return {
    valid: results.every(r => r.valid),
    errors: results.flatMap(r => r.errors),
    warnings: results.flatMap(r => r.warnings),
  };
}

export function createError(
  context: ValidationContext,
  code: ValidationError['code'],
  message: string,
  suggestion?: string,
  value?: unknown
): ValidationError {
  return {
    path: `${context.path}.${context.field}`,
    field: context.field,
    code,
    message,
    suggestion,
    value,
  };
}

export function createWarning(
  context: ValidationContext,
  code: string,
  message: string
): ValidationWarning {
  return {
    path: `${context.path}.${context.field}`,
    field: context.field,
    code,
    message,
  };
}

// Common validators
export function required<T>(context: ValidationContext, value: T): ValidationResult {
  if (value === null || value === undefined || value === '') {
    return createResult([
      createError(context, 'required', 'This field is required'),
    ]);
  }
  return createResult();
}

export function minValue(min: number): FieldValidator<number> {
  return {
    validate: (value, context) => {
      if (value < min) {
        return createResult([
          createError(
            context,
            'min',
            `Value must be at least ${min}`,
            `Try a value of ${min} or higher`,
            value
          ),
        ]);
      }
      return createResult();
    },
  };
}

export function maxValue(max: number): FieldValidator<number> {
  return {
    validate: (value, context) => {
      if (value > max) {
        return createResult([
          createError(
            context,
            'max',
            `Value must be at most ${max}`,
            `Try a value of ${max} or lower`,
            value
          ),
        ]);
      }
      return createResult();
    },
  };
}

export function range(min: number, max: number): FieldValidator<number> {
  return {
    validate: (value, context) => {
      if (value < min || value > max) {
        return createResult([
          createError(
            context,
            'range',
            `Value must be between ${min} and ${max}`,
            `Try a value between ${min} and ${max}`,
            value
          ),
        ]);
      }
      return createResult();
    },
  };
}

export function pattern(regex: RegExp, message: string): FieldValidator<string> {
  return {
    validate: (value, context) => {
      if (!regex.test(value)) {
        return createResult([
          createError(context, 'pattern', message, undefined, value),
        ]);
      }
      return createResult();
    },
  };
}

export function enumValue<T>(allowed: readonly T[]): FieldValidator<T> {
  return {
    validate: (value, context) => {
      if (!allowed.includes(value)) {
        return createResult([
          createError(
            context,
            'enum',
            `Value must be one of: ${allowed.join(', ')}`,
            `Choose from: ${allowed.join(', ')}`,
            value
          ),
        ]);
      }
      return createResult();
    },
  };
}

export function url(): FieldValidator<string> {
  return {
    validate: (value, context) => {
      if (!value) return createResult();

      try {
        new URL(value);
        return createResult();
      } catch {
        return createResult([
          createError(
            context,
            'format',
            'Invalid URL format',
            'URL should start with http:// or https://',
            value
          ),
        ]);
      }
    },
  };
}

export function email(): FieldValidator<string> {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

  return {
    validate: (value, context) => {
      if (!value) return createResult();

      if (!emailRegex.test(value)) {
        return createResult([
          createError(
            context,
            'format',
            'Invalid email format',
            'Enter a valid email address like user@example.com',
            value
          ),
        ]);
      }
      return createResult();
    },
  };
}

export function hexColor(): FieldValidator<string> {
  const hexRegex = /^#[0-9A-Fa-f]{6}$/;

  return {
    validate: (value, context) => {
      if (!hexRegex.test(value)) {
        return createResult([
          createError(
            context,
            'format',
            'Invalid color format',
            'Use hex format like #FF5500',
            value
          ),
        ]);
      }
      return createResult();
    },
  };
}

export function compose<T>(...validators: FieldValidator<T>[]): FieldValidator<T> {
  return {
    validate: (value, context) => {
      const results = validators.map(v => v.validate(value, context));
      return mergeResults(...results);
    },
  };
}

export function conditional<T>(
  predicate: (settings: any) => boolean,
  validator: FieldValidator<T>
): FieldValidator<T> {
  return {
    validate: (value, context) => {
      if (predicate(context.settings)) {
        return validator.validate(value, context);
      }
      return createResult();
    },
  };
}
```

### 3. Settings Validators (src/lib/utils/settings-validation.ts)

```typescript
import type { AllSettings, SettingsValidationError } from '$lib/types/settings';
import type { ValidationResult, ValidatorMap, ValidationContext } from '$lib/types/validation';
import {
  createResult,
  mergeResults,
  createError,
  createWarning,
  range,
  hexColor,
  url,
  email,
  enumValue,
  compose,
  conditional,
} from './validators';

const validators: ValidatorMap = {
  general: {
    language: {
      validate: (value, context) => {
        if (!value || value.length < 2) {
          return createResult([
            createError(context, 'min', 'Language code must be at least 2 characters'),
          ]);
        }
        return createResult();
      },
    },
  },

  appearance: {
    theme: enumValue(['light', 'dark', 'system']),
    accentColor: hexColor(),
    fontSize: range(8, 32),
    lineHeight: range(1, 3),
    sidebarPosition: enumValue(['left', 'right']),
  },

  editor: {
    tabSize: range(1, 8),
    wordWrap: enumValue(['off', 'on', 'wordWrapColumn', 'bounded']),
    wordWrapColumn: conditional(
      (s) => s.editor.wordWrap === 'wordWrapColumn' || s.editor.wordWrap === 'bounded',
      range(40, 200)
    ),
    lineNumbers: enumValue(['on', 'off', 'relative']),
    minimapScale: range(0.5, 2),
    autoSave: enumValue(['off', 'afterDelay', 'onFocusChange', 'onWindowChange']),
    autoSaveDelay: conditional(
      (s) => s.editor.autoSave === 'afterDelay',
      range(100, 60000)
    ),
    renderWhitespace: enumValue(['none', 'boundary', 'selection', 'trailing', 'all']),
    autoClosingBrackets: enumValue(['always', 'languageDefined', 'never']),
    autoClosingQuotes: enumValue(['always', 'languageDefined', 'never']),
  },

  backends: {
    timeout: range(5000, 600000),
    maxRetries: range(0, 10),
  },

  git: {
    fetchInterval: {
      validate: (value, context) => {
        const errors = [];
        const warnings = [];

        if (value < 60000) {
          errors.push(createError(
            context,
            'min',
            'Fetch interval must be at least 1 minute',
            'Set to at least 60000 (1 minute)'
          ));
        }

        if (value < 300000) {
          warnings.push(createWarning(
            context,
            'performance',
            'Frequent fetching may impact performance'
          ));
        }

        return createResult(errors, warnings);
      },
    },
    gpgKey: conditional(
      (s) => s.git.enabled && s.git.signCommits,
      {
        validate: (value, context) => {
          if (!value) {
            return createResult([], [
              createWarning(
                context,
                'dependency',
                'GPG key is recommended when commit signing is enabled'
              ),
            ]);
          }
          return createResult();
        },
      }
    ),
    userEmail: conditional(
      (s) => !!s.git.userEmail,
      email()
    ),
  },

  sync: {
    gistId: conditional(
      (s) => s.sync.enabled && s.sync.provider === 'github-gist',
      {
        validate: (value, context) => {
          if (!value) {
            return createResult([], [
              createWarning(
                context,
                'dependency',
                'Gist ID is recommended for GitHub Gist sync'
              ),
            ]);
          }
          return createResult();
        },
      }
    ),
    customEndpoint: conditional(
      (s) => s.sync.enabled && s.sync.provider === 'custom',
      compose(
        {
          validate: (value, context) => {
            if (!value) {
              return createResult([
                createError(context, 'required', 'Custom endpoint is required'),
              ]);
            }
            return createResult();
          },
        },
        url()
      )
    ),
    syncInterval: range(60000, 86400000),
  },
};

// Backend array validation
function validateBackends(settings: AllSettings): ValidationResult {
  const errors: SettingsValidationError[] = [];
  const warnings: SettingsValidationError[] = [];
  const { backends } = settings;

  if (backends.backends.length === 0) {
    warnings.push({
      path: 'backends.backends',
      message: 'At least one backend should be configured',
      severity: 'warning',
    });
  }

  const enabledBackends = backends.backends.filter(b => b.enabled);
  if (enabledBackends.length === 0 && backends.backends.length > 0) {
    warnings.push({
      path: 'backends.backends',
      message: 'At least one backend should be enabled',
      severity: 'warning',
    });
  }

  const defaultExists = backends.backends.some(b => b.id === backends.defaultBackend);
  if (backends.defaultBackend && !defaultExists) {
    errors.push({
      path: 'backends.defaultBackend',
      message: 'Default backend does not exist',
      severity: 'error',
    });
  }

  backends.backends.forEach((backend, index) => {
    const basePath = `backends.backends[${index}]`;

    if (!backend.name?.trim()) {
      errors.push({
        path: `${basePath}.name`,
        message: 'Backend name is required',
        severity: 'error',
      });
    }

    if (!backend.model?.trim()) {
      errors.push({
        path: `${basePath}.model`,
        message: 'Backend model is required',
        severity: 'error',
      });
    }

    if (backend.maxTokens < 1 || backend.maxTokens > 200000) {
      errors.push({
        path: `${basePath}.maxTokens`,
        message: 'Max tokens must be between 1 and 200000',
        severity: 'error',
      });
    }

    if (backend.temperature < 0 || backend.temperature > 2) {
      errors.push({
        path: `${basePath}.temperature`,
        message: 'Temperature must be between 0 and 2',
        severity: 'error',
      });
    }

    // Check for API key requirement
    const requiresApiKey = ['anthropic', 'openai', 'azure'].includes(backend.type);
    if (requiresApiKey && !backend.apiKey && backend.enabled) {
      errors.push({
        path: `${basePath}.apiKey`,
        message: 'API key is required for this backend type',
        severity: 'error',
      });
    }
  });

  return {
    valid: errors.filter(e => e.severity === 'error').length === 0,
    errors: errors.filter(e => e.severity === 'error').map(e => ({
      path: e.path,
      field: e.path.split('.').pop() || '',
      code: 'custom' as const,
      message: e.message,
    })),
    warnings: warnings.concat(errors.filter(e => e.severity === 'warning')).map(w => ({
      path: w.path,
      field: w.path.split('.').pop() || '',
      code: 'custom',
      message: w.message,
    })),
  };
}

export function validateSettings(settings: AllSettings): SettingsValidationError[] {
  const allErrors: SettingsValidationError[] = [];

  // Validate each category
  for (const [category, categoryValidators] of Object.entries(validators)) {
    if (!categoryValidators) continue;

    const categorySettings = settings[category as keyof AllSettings];
    if (!categorySettings) continue;

    for (const [field, validator] of Object.entries(categoryValidators)) {
      if (!validator) continue;

      const value = (categorySettings as any)[field];
      const context: ValidationContext = {
        settings,
        path: category,
        field,
      };

      const result = validator.validate(value, context);

      for (const error of result.errors) {
        allErrors.push({
          path: error.path,
          message: error.message,
          severity: 'error',
        });
      }

      for (const warning of result.warnings) {
        allErrors.push({
          path: warning.path,
          message: warning.message,
          severity: 'warning',
        });
      }
    }
  }

  // Validate backends array
  const backendsResult = validateBackends(settings);
  for (const error of backendsResult.errors) {
    allErrors.push({
      path: error.path,
      message: error.message,
      severity: 'error',
    });
  }
  for (const warning of backendsResult.warnings) {
    allErrors.push({
      path: warning.path,
      message: warning.message,
      severity: 'warning',
    });
  }

  return allErrors;
}

export function validateField<K extends keyof AllSettings>(
  settings: AllSettings,
  category: K,
  field: keyof AllSettings[K]
): SettingsValidationError[] {
  const categoryValidators = validators[category];
  if (!categoryValidators) return [];

  const validator = (categoryValidators as any)[field];
  if (!validator) return [];

  const value = settings[category][field];
  const context: ValidationContext = {
    settings,
    path: category,
    field: field as string,
  };

  const result = validator.validate(value, context);
  const errors: SettingsValidationError[] = [];

  for (const error of result.errors) {
    errors.push({
      path: error.path,
      message: error.message,
      severity: 'error',
    });
  }

  for (const warning of result.warnings) {
    errors.push({
      path: warning.path,
      message: warning.message,
      severity: 'warning',
    });
  }

  return errors;
}
```

### 4. Validation Hook (src/lib/hooks/useValidation.ts)

```typescript
import { derived, writable, get } from 'svelte/store';
import type { AllSettings, SettingsValidationError } from '$lib/types/settings';
import { validateSettings, validateField } from '$lib/utils/settings-validation';

export function useValidation(settingsStore: { subscribe: (fn: (value: { settings: AllSettings }) => void) => void }) {
  const validationCache = writable<Map<string, SettingsValidationError[]>>(new Map());

  const errors = derived(settingsStore, ($state) => {
    return validateSettings($state.settings);
  });

  const errorsByPath = derived(errors, ($errors) => {
    const map = new Map<string, SettingsValidationError[]>();

    for (const error of $errors) {
      const existing = map.get(error.path) || [];
      existing.push(error);
      map.set(error.path, existing);
    }

    return map;
  });

  function getFieldErrors(path: string): SettingsValidationError[] {
    return get(errorsByPath).get(path) || [];
  }

  function hasErrors(): boolean {
    return get(errors).some(e => e.severity === 'error');
  }

  function hasWarnings(): boolean {
    return get(errors).some(e => e.severity === 'warning');
  }

  function getErrorCount(): number {
    return get(errors).filter(e => e.severity === 'error').length;
  }

  function getWarningCount(): number {
    return get(errors).filter(e => e.severity === 'warning').length;
  }

  return {
    errors,
    errorsByPath,
    getFieldErrors,
    hasErrors,
    hasWarnings,
    getErrorCount,
    getWarningCount,
  };
}
```

---

## Testing Requirements

1. All validators work correctly
2. Range validation catches out-of-bounds
3. Pattern validation works
4. Enum validation restricts values
5. Conditional validation respects predicates
6. Cross-field validation works
7. Backend array validation catches errors
8. Validation results merge correctly

### Test File (src/lib/utils/__tests__/settings-validation.test.ts)

```typescript
import { describe, it, expect } from 'vitest';
import { validateSettings, validateField } from '../settings-validation';
import { DEFAULT_SETTINGS } from '$lib/stores/settings-defaults';
import type { AllSettings } from '$lib/types/settings';

describe('validateSettings', () => {
  it('returns no errors for valid default settings', () => {
    const errors = validateSettings(DEFAULT_SETTINGS);
    const actualErrors = errors.filter(e => e.severity === 'error');
    expect(actualErrors).toHaveLength(0);
  });

  it('catches invalid font size', () => {
    const settings: AllSettings = {
      ...DEFAULT_SETTINGS,
      appearance: {
        ...DEFAULT_SETTINGS.appearance,
        fontSize: 50,
      },
    };

    const errors = validateSettings(settings);
    expect(errors.some(e => e.path === 'appearance.fontSize')).toBe(true);
  });

  it('catches invalid color format', () => {
    const settings: AllSettings = {
      ...DEFAULT_SETTINGS,
      appearance: {
        ...DEFAULT_SETTINGS.appearance,
        accentColor: 'not-a-color',
      },
    };

    const errors = validateSettings(settings);
    expect(errors.some(e => e.path === 'appearance.accentColor')).toBe(true);
  });

  it('catches invalid tab size', () => {
    const settings: AllSettings = {
      ...DEFAULT_SETTINGS,
      editor: {
        ...DEFAULT_SETTINGS.editor,
        tabSize: 20,
      },
    };

    const errors = validateSettings(settings);
    expect(errors.some(e => e.path === 'editor.tabSize')).toBe(true);
  });

  it('validates conditional fields', () => {
    const settings: AllSettings = {
      ...DEFAULT_SETTINGS,
      editor: {
        ...DEFAULT_SETTINGS.editor,
        wordWrap: 'wordWrapColumn',
        wordWrapColumn: 30, // Below minimum
      },
    };

    const errors = validateSettings(settings);
    expect(errors.some(e => e.path === 'editor.wordWrapColumn')).toBe(true);
  });

  it('validates backend array', () => {
    const settings: AllSettings = {
      ...DEFAULT_SETTINGS,
      backends: {
        ...DEFAULT_SETTINGS.backends,
        backends: [{
          id: 'test',
          name: '',
          type: 'anthropic',
          model: '',
          maxTokens: 0,
          temperature: 3,
          enabled: true,
        }],
      },
    };

    const errors = validateSettings(settings);
    expect(errors.filter(e => e.path.includes('backends.backends')).length).toBeGreaterThan(0);
  });

  it('warns about missing GPG key when signing enabled', () => {
    const settings: AllSettings = {
      ...DEFAULT_SETTINGS,
      git: {
        ...DEFAULT_SETTINGS.git,
        enabled: true,
        signCommits: true,
        gpgKey: undefined,
      },
    };

    const errors = validateSettings(settings);
    const gpgWarnings = errors.filter(e => e.path === 'git.gpgKey' && e.severity === 'warning');
    expect(gpgWarnings.length).toBeGreaterThan(0);
  });
});

describe('validateField', () => {
  it('validates single field', () => {
    const settings: AllSettings = {
      ...DEFAULT_SETTINGS,
      appearance: {
        ...DEFAULT_SETTINGS.appearance,
        fontSize: 100,
      },
    };

    const errors = validateField(settings, 'appearance', 'fontSize');
    expect(errors.length).toBeGreaterThan(0);
  });

  it('returns empty for valid field', () => {
    const errors = validateField(DEFAULT_SETTINGS, 'appearance', 'fontSize');
    expect(errors.filter(e => e.severity === 'error')).toHaveLength(0);
  });
});
```

---

## Related Specs

- Depends on: [272-settings-store.md](272-settings-store.md)
- Used by: [273-settings-general.md](273-settings-general.md) through [283-settings-profiles.md](283-settings-profiles.md)
- Previous: [283-settings-profiles.md](283-settings-profiles.md)
- Next: [285-settings-tests.md](285-settings-tests.md)
