# Spec 191: Design Tokens

## Phase
Phase 9: UI Foundation

## Spec ID
191

## Status
Planned

## Dependencies
- Spec 186: SvelteKit Setup

## Estimated Context
~8%

---

## Objective

Establish a comprehensive design token system using CSS custom properties for Tachikoma's UI, providing consistent theming, easy customization, and support for dark/light modes with the signature Tachikoma blue color scheme.

---

## Acceptance Criteria

- [ ] Complete CSS custom property system
- [ ] Semantic token naming convention
- [ ] Color, spacing, typography, and shadow tokens
- [ ] Theme switching support
- [ ] TypeScript token definitions
- [ ] Design token documentation
- [ ] Token export for design tools
- [ ] Runtime theme customization

---

## Implementation Details

### src/lib/styles/tokens.css

```css
/**
 * Tachikoma Design Tokens
 *
 * A comprehensive design system using CSS custom properties
 * organized into semantic layers for maintainability.
 */

:root {
  /* ============================================
   * PRIMITIVE TOKENS
   * Raw values that form the foundation
   * ============================================ */

  /* Tachikoma Blue Palette */
  --blue-50: #e6faff;
  --blue-100: #b3f0ff;
  --blue-200: #80e6ff;
  --blue-300: #4ddbff;
  --blue-400: #1ad1ff;
  --blue-500: #00d4ff; /* Primary Tachikoma Blue */
  --blue-600: #00a8cc;
  --blue-700: #007d99;
  --blue-800: #005266;
  --blue-900: #002633;

  /* Neutral Palette */
  --gray-50: #f8fafc;
  --gray-100: #f1f5f9;
  --gray-200: #e2e8f0;
  --gray-300: #cbd5e1;
  --gray-400: #94a3b8;
  --gray-500: #64748b;
  --gray-600: #475569;
  --gray-700: #334155;
  --gray-800: #1e293b;
  --gray-900: #0f172a;
  --gray-950: #020617;

  /* Status Colors */
  --green-500: #22c55e;
  --green-600: #16a34a;
  --yellow-500: #eab308;
  --yellow-600: #ca8a04;
  --red-500: #ef4444;
  --red-600: #dc2626;
  --orange-500: #f97316;
  --orange-600: #ea580c;
  --purple-500: #a855f7;
  --purple-600: #9333ea;

  /* Spacing Scale (4px base) */
  --space-0: 0;
  --space-px: 1px;
  --space-0-5: 0.125rem;  /* 2px */
  --space-1: 0.25rem;     /* 4px */
  --space-1-5: 0.375rem;  /* 6px */
  --space-2: 0.5rem;      /* 8px */
  --space-2-5: 0.625rem;  /* 10px */
  --space-3: 0.75rem;     /* 12px */
  --space-3-5: 0.875rem;  /* 14px */
  --space-4: 1rem;        /* 16px */
  --space-5: 1.25rem;     /* 20px */
  --space-6: 1.5rem;      /* 24px */
  --space-7: 1.75rem;     /* 28px */
  --space-8: 2rem;        /* 32px */
  --space-9: 2.25rem;     /* 36px */
  --space-10: 2.5rem;     /* 40px */
  --space-11: 2.75rem;    /* 44px */
  --space-12: 3rem;       /* 48px */
  --space-14: 3.5rem;     /* 56px */
  --space-16: 4rem;       /* 64px */
  --space-20: 5rem;       /* 80px */
  --space-24: 6rem;       /* 96px */
  --space-28: 7rem;       /* 112px */
  --space-32: 8rem;       /* 128px */

  /* Font Sizes */
  --text-xs: 0.75rem;     /* 12px */
  --text-sm: 0.875rem;    /* 14px */
  --text-base: 1rem;      /* 16px */
  --text-lg: 1.125rem;    /* 18px */
  --text-xl: 1.25rem;     /* 20px */
  --text-2xl: 1.5rem;     /* 24px */
  --text-3xl: 1.875rem;   /* 30px */
  --text-4xl: 2.25rem;    /* 36px */
  --text-5xl: 3rem;       /* 48px */

  /* Font Weights */
  --font-thin: 100;
  --font-extralight: 200;
  --font-light: 300;
  --font-normal: 400;
  --font-medium: 500;
  --font-semibold: 600;
  --font-bold: 700;
  --font-extrabold: 800;
  --font-black: 900;

  /* Line Heights */
  --leading-none: 1;
  --leading-tight: 1.25;
  --leading-snug: 1.375;
  --leading-normal: 1.5;
  --leading-relaxed: 1.625;
  --leading-loose: 2;

  /* Border Radius */
  --radius-none: 0;
  --radius-sm: 0.25rem;   /* 4px */
  --radius-md: 0.375rem;  /* 6px */
  --radius-lg: 0.5rem;    /* 8px */
  --radius-xl: 0.75rem;   /* 12px */
  --radius-2xl: 1rem;     /* 16px */
  --radius-3xl: 1.5rem;   /* 24px */
  --radius-full: 9999px;

  /* Z-Index Scale */
  --z-0: 0;
  --z-10: 10;
  --z-20: 20;
  --z-30: 30;
  --z-40: 40;
  --z-50: 50;
  --z-dropdown: 1000;
  --z-sticky: 1020;
  --z-fixed: 1030;
  --z-modal-backdrop: 1040;
  --z-modal: 1050;
  --z-popover: 1060;
  --z-tooltip: 1070;
  --z-toast: 1080;

  /* Transitions */
  --duration-75: 75ms;
  --duration-100: 100ms;
  --duration-150: 150ms;
  --duration-200: 200ms;
  --duration-300: 300ms;
  --duration-500: 500ms;
  --duration-700: 700ms;
  --duration-1000: 1000ms;

  --ease-linear: linear;
  --ease-in: cubic-bezier(0.4, 0, 1, 1);
  --ease-out: cubic-bezier(0, 0, 0.2, 1);
  --ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);
  --ease-bounce: cubic-bezier(0.68, -0.55, 0.265, 1.55);
}

/* ============================================
 * SEMANTIC TOKENS - DARK THEME (Default)
 * ============================================ */

[data-theme="dark"],
:root {
  /* Brand */
  --color-primary: var(--blue-500);
  --color-primary-hover: var(--blue-400);
  --color-primary-active: var(--blue-600);
  --color-primary-subtle: rgba(0, 212, 255, 0.1);
  --color-primary-muted: rgba(0, 212, 255, 0.2);

  /* Backgrounds */
  --color-bg-base: #0a0e14;
  --color-bg-surface: #0d1117;
  --color-bg-elevated: #161b22;
  --color-bg-overlay: #1c2128;
  --color-bg-input: #21262d;
  --color-bg-hover: rgba(255, 255, 255, 0.05);
  --color-bg-active: rgba(255, 255, 255, 0.08);
  --color-bg-selected: rgba(0, 212, 255, 0.15);

  /* Text */
  --color-text-primary: #e6edf3;
  --color-text-secondary: #8b949e;
  --color-text-muted: #6e7681;
  --color-text-disabled: #484f58;
  --color-text-inverse: #0a0e14;
  --color-text-link: var(--blue-400);
  --color-text-link-hover: var(--blue-300);

  /* Borders */
  --color-border: #30363d;
  --color-border-subtle: #21262d;
  --color-border-strong: #484f58;
  --color-border-focus: var(--blue-500);

  /* Status */
  --color-success: var(--green-500);
  --color-success-subtle: rgba(34, 197, 94, 0.15);
  --color-warning: var(--yellow-500);
  --color-warning-subtle: rgba(234, 179, 8, 0.15);
  --color-error: var(--red-500);
  --color-error-subtle: rgba(239, 68, 68, 0.15);
  --color-info: var(--blue-500);
  --color-info-subtle: rgba(0, 212, 255, 0.15);

  /* Shadows */
  --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.3);
  --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.4), 0 2px 4px -2px rgba(0, 0, 0, 0.3);
  --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.4), 0 4px 6px -4px rgba(0, 0, 0, 0.3);
  --shadow-xl: 0 20px 25px -5px rgba(0, 0, 0, 0.4), 0 8px 10px -6px rgba(0, 0, 0, 0.3);
  --shadow-2xl: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  --shadow-glow: 0 0 20px rgba(0, 212, 255, 0.3);
  --shadow-glow-lg: 0 0 40px rgba(0, 212, 255, 0.4);

  /* Focus Ring */
  --focus-ring: 0 0 0 2px var(--color-bg-surface), 0 0 0 4px var(--color-primary);

  /* Code/Terminal */
  --color-code-bg: #161b22;
  --color-code-text: #e6edf3;
  --color-code-keyword: #ff7b72;
  --color-code-string: #a5d6ff;
  --color-code-number: #79c0ff;
  --color-code-function: #d2a8ff;
  --color-code-comment: #8b949e;
  --color-code-operator: #ff7b72;
}

/* ============================================
 * SEMANTIC TOKENS - LIGHT THEME
 * ============================================ */

[data-theme="light"] {
  /* Brand */
  --color-primary: var(--blue-600);
  --color-primary-hover: var(--blue-500);
  --color-primary-active: var(--blue-700);
  --color-primary-subtle: rgba(0, 168, 204, 0.1);
  --color-primary-muted: rgba(0, 168, 204, 0.2);

  /* Backgrounds */
  --color-bg-base: #ffffff;
  --color-bg-surface: #f6f8fa;
  --color-bg-elevated: #ffffff;
  --color-bg-overlay: #ffffff;
  --color-bg-input: #ffffff;
  --color-bg-hover: rgba(0, 0, 0, 0.04);
  --color-bg-active: rgba(0, 0, 0, 0.06);
  --color-bg-selected: rgba(0, 168, 204, 0.12);

  /* Text */
  --color-text-primary: #1f2328;
  --color-text-secondary: #57606a;
  --color-text-muted: #6e7781;
  --color-text-disabled: #8c959f;
  --color-text-inverse: #ffffff;
  --color-text-link: var(--blue-600);
  --color-text-link-hover: var(--blue-700);

  /* Borders */
  --color-border: #d0d7de;
  --color-border-subtle: #e2e8f0;
  --color-border-strong: #afb8c1;
  --color-border-focus: var(--blue-600);

  /* Status */
  --color-success: var(--green-600);
  --color-success-subtle: rgba(22, 163, 74, 0.12);
  --color-warning: var(--yellow-600);
  --color-warning-subtle: rgba(202, 138, 4, 0.12);
  --color-error: var(--red-600);
  --color-error-subtle: rgba(220, 38, 38, 0.12);
  --color-info: var(--blue-600);
  --color-info-subtle: rgba(0, 168, 204, 0.12);

  /* Shadows */
  --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1);
  --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1);
  --shadow-xl: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1);
  --shadow-2xl: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  --shadow-glow: 0 0 20px rgba(0, 168, 204, 0.2);
  --shadow-glow-lg: 0 0 40px rgba(0, 168, 204, 0.3);

  /* Code/Terminal */
  --color-code-bg: #f6f8fa;
  --color-code-text: #1f2328;
  --color-code-keyword: #cf222e;
  --color-code-string: #0a3069;
  --color-code-number: #0550ae;
  --color-code-function: #8250df;
  --color-code-comment: #6e7781;
  --color-code-operator: #cf222e;
}

/* ============================================
 * COMPONENT TOKENS
 * ============================================ */

:root,
[data-theme="dark"],
[data-theme="light"] {
  /* Button */
  --btn-height-sm: var(--space-8);
  --btn-height-md: var(--space-10);
  --btn-height-lg: var(--space-12);
  --btn-padding-x-sm: var(--space-3);
  --btn-padding-x-md: var(--space-4);
  --btn-padding-x-lg: var(--space-6);
  --btn-font-size-sm: var(--text-sm);
  --btn-font-size-md: var(--text-sm);
  --btn-font-size-lg: var(--text-base);
  --btn-radius: var(--radius-md);

  /* Input */
  --input-height-sm: var(--space-8);
  --input-height-md: var(--space-10);
  --input-height-lg: var(--space-12);
  --input-padding-x: var(--space-3);
  --input-font-size: var(--text-sm);
  --input-radius: var(--radius-md);

  /* Card */
  --card-padding: var(--space-4);
  --card-radius: var(--radius-lg);

  /* Modal */
  --modal-padding: var(--space-6);
  --modal-radius: var(--radius-xl);
  --modal-width-sm: 400px;
  --modal-width-md: 560px;
  --modal-width-lg: 720px;

  /* Sidebar */
  --sidebar-width: 240px;
  --sidebar-width-collapsed: 56px;

  /* Header */
  --header-height: 56px;
  --titlebar-height: 40px;
}
```

### src/lib/styles/tokens.ts

```typescript
/**
 * TypeScript design token definitions
 * Mirror of CSS custom properties for programmatic access
 */

export const colors = {
  blue: {
    50: '#e6faff',
    100: '#b3f0ff',
    200: '#80e6ff',
    300: '#4ddbff',
    400: '#1ad1ff',
    500: '#00d4ff',
    600: '#00a8cc',
    700: '#007d99',
    800: '#005266',
    900: '#002633',
  },
  gray: {
    50: '#f8fafc',
    100: '#f1f5f9',
    200: '#e2e8f0',
    300: '#cbd5e1',
    400: '#94a3b8',
    500: '#64748b',
    600: '#475569',
    700: '#334155',
    800: '#1e293b',
    900: '#0f172a',
    950: '#020617',
  },
  status: {
    success: '#22c55e',
    warning: '#eab308',
    error: '#ef4444',
    info: '#00d4ff',
  },
} as const;

export const spacing = {
  0: '0',
  px: '1px',
  0.5: '0.125rem',
  1: '0.25rem',
  1.5: '0.375rem',
  2: '0.5rem',
  2.5: '0.625rem',
  3: '0.75rem',
  3.5: '0.875rem',
  4: '1rem',
  5: '1.25rem',
  6: '1.5rem',
  7: '1.75rem',
  8: '2rem',
  9: '2.25rem',
  10: '2.5rem',
  11: '2.75rem',
  12: '3rem',
  14: '3.5rem',
  16: '4rem',
  20: '5rem',
  24: '6rem',
  28: '7rem',
  32: '8rem',
} as const;

export const fontSize = {
  xs: '0.75rem',
  sm: '0.875rem',
  base: '1rem',
  lg: '1.125rem',
  xl: '1.25rem',
  '2xl': '1.5rem',
  '3xl': '1.875rem',
  '4xl': '2.25rem',
  '5xl': '3rem',
} as const;

export const fontWeight = {
  thin: 100,
  extralight: 200,
  light: 300,
  normal: 400,
  medium: 500,
  semibold: 600,
  bold: 700,
  extrabold: 800,
  black: 900,
} as const;

export const borderRadius = {
  none: '0',
  sm: '0.25rem',
  md: '0.375rem',
  lg: '0.5rem',
  xl: '0.75rem',
  '2xl': '1rem',
  '3xl': '1.5rem',
  full: '9999px',
} as const;

export const zIndex = {
  0: 0,
  10: 10,
  20: 20,
  30: 30,
  40: 40,
  50: 50,
  dropdown: 1000,
  sticky: 1020,
  fixed: 1030,
  modalBackdrop: 1040,
  modal: 1050,
  popover: 1060,
  tooltip: 1070,
  toast: 1080,
} as const;

export const transition = {
  duration: {
    75: '75ms',
    100: '100ms',
    150: '150ms',
    200: '200ms',
    300: '300ms',
    500: '500ms',
    700: '700ms',
    1000: '1000ms',
  },
  timing: {
    linear: 'linear',
    easeIn: 'cubic-bezier(0.4, 0, 1, 1)',
    easeOut: 'cubic-bezier(0, 0, 0.2, 1)',
    easeInOut: 'cubic-bezier(0.4, 0, 0.2, 1)',
    bounce: 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
  },
} as const;

// Type exports
export type ColorScale = keyof typeof colors.blue;
export type SpacingScale = keyof typeof spacing;
export type FontSizeScale = keyof typeof fontSize;
export type FontWeightScale = keyof typeof fontWeight;
export type BorderRadiusScale = keyof typeof borderRadius;
export type ZIndexScale = keyof typeof zIndex;
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/tokens/tokens.test.ts
import { describe, it, expect } from 'vitest';
import { colors, spacing, fontSize } from '@styles/tokens';

describe('Design Tokens', () => {
  it('should have complete blue color scale', () => {
    expect(colors.blue[500]).toBe('#00d4ff');
    expect(Object.keys(colors.blue)).toHaveLength(10);
  });

  it('should have consistent spacing scale', () => {
    expect(spacing[4]).toBe('1rem');
    expect(spacing[8]).toBe('2rem');
  });

  it('should have typography scale', () => {
    expect(fontSize.base).toBe('1rem');
    expect(fontSize.sm).toBe('0.875rem');
  });
});
```

---

## Related Specs

- [186-sveltekit-setup.md](./186-sveltekit-setup.md) - SvelteKit setup
- [192-typography.md](./192-typography.md) - Typography system
- [193-color-system.md](./193-color-system.md) - Color system
- [194-spacing-system.md](./194-spacing-system.md) - Spacing system
- [195-shadows-elevation.md](./195-shadows-elevation.md) - Shadows and elevation
