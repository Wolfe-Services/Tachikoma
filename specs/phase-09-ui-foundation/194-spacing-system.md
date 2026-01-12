# Spec 194: Spacing System

## Phase
Phase 9: UI Foundation

## Spec ID
194

## Status
Planned

## Dependencies
- Spec 186: SvelteKit Setup
- Spec 191: Design Tokens

## Estimated Context
~8%

---

## Objective

Implement a consistent spacing system for Tachikoma using a 4px base unit, providing utility classes, component spacing patterns, and responsive spacing adjustments for cohesive visual rhythm across the application.

---

## Acceptance Criteria

- [x] 4px base unit spacing scale
- [x] Padding and margin utility classes
- [x] Gap utilities for flexbox/grid
- [x] Responsive spacing modifiers
- [x] Component-specific spacing tokens
- [x] Negative margin support
- [x] Spacing composition patterns
- [x] TypeScript spacing utilities

---

## Implementation Details

### src/lib/styles/spacing.css

```css
/**
 * Tachikoma Spacing System
 *
 * Based on a 4px (0.25rem) base unit
 * Scale: 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16, 20, 24, 28, 32, 36, 40
 */

:root {
  /* Base unit */
  --spacing-unit: 0.25rem; /* 4px */

  /* Spacing scale */
  --spacing-0: 0;
  --spacing-px: 1px;
  --spacing-0-5: calc(var(--spacing-unit) * 0.5);  /* 2px */
  --spacing-1: var(--spacing-unit);                 /* 4px */
  --spacing-1-5: calc(var(--spacing-unit) * 1.5);  /* 6px */
  --spacing-2: calc(var(--spacing-unit) * 2);      /* 8px */
  --spacing-2-5: calc(var(--spacing-unit) * 2.5);  /* 10px */
  --spacing-3: calc(var(--spacing-unit) * 3);      /* 12px */
  --spacing-3-5: calc(var(--spacing-unit) * 3.5);  /* 14px */
  --spacing-4: calc(var(--spacing-unit) * 4);      /* 16px */
  --spacing-5: calc(var(--spacing-unit) * 5);      /* 20px */
  --spacing-6: calc(var(--spacing-unit) * 6);      /* 24px */
  --spacing-7: calc(var(--spacing-unit) * 7);      /* 28px */
  --spacing-8: calc(var(--spacing-unit) * 8);      /* 32px */
  --spacing-9: calc(var(--spacing-unit) * 9);      /* 36px */
  --spacing-10: calc(var(--spacing-unit) * 10);    /* 40px */
  --spacing-11: calc(var(--spacing-unit) * 11);    /* 44px */
  --spacing-12: calc(var(--spacing-unit) * 12);    /* 48px */
  --spacing-14: calc(var(--spacing-unit) * 14);    /* 56px */
  --spacing-16: calc(var(--spacing-unit) * 16);    /* 64px */
  --spacing-20: calc(var(--spacing-unit) * 20);    /* 80px */
  --spacing-24: calc(var(--spacing-unit) * 24);    /* 96px */
  --spacing-28: calc(var(--spacing-unit) * 28);    /* 112px */
  --spacing-32: calc(var(--spacing-unit) * 32);    /* 128px */
  --spacing-36: calc(var(--spacing-unit) * 36);    /* 144px */
  --spacing-40: calc(var(--spacing-unit) * 40);    /* 160px */

  /* Semantic spacing */
  --spacing-xs: var(--spacing-1);     /* 4px - Tight spacing */
  --spacing-sm: var(--spacing-2);     /* 8px - Small elements */
  --spacing-md: var(--spacing-4);     /* 16px - Default */
  --spacing-lg: var(--spacing-6);     /* 24px - Larger elements */
  --spacing-xl: var(--spacing-8);     /* 32px - Sections */
  --spacing-2xl: var(--spacing-12);   /* 48px - Major sections */
  --spacing-3xl: var(--spacing-16);   /* 64px - Page sections */

  /* Component-specific spacing */
  --spacing-button-x: var(--spacing-4);
  --spacing-button-y: var(--spacing-2);
  --spacing-input-x: var(--spacing-3);
  --spacing-input-y: var(--spacing-2);
  --spacing-card: var(--spacing-4);
  --spacing-modal: var(--spacing-6);
  --spacing-section: var(--spacing-8);
  --spacing-page: var(--spacing-6);

  /* Stack spacing (vertical rhythm) */
  --stack-xs: var(--spacing-2);
  --stack-sm: var(--spacing-4);
  --stack-md: var(--spacing-6);
  --stack-lg: var(--spacing-8);
  --stack-xl: var(--spacing-12);

  /* Inline spacing (horizontal) */
  --inline-xs: var(--spacing-1);
  --inline-sm: var(--spacing-2);
  --inline-md: var(--spacing-3);
  --inline-lg: var(--spacing-4);
  --inline-xl: var(--spacing-6);
}

/* ============================================
 * MARGIN UTILITIES
 * ============================================ */

/* All sides */
.m-0 { margin: var(--spacing-0); }
.m-px { margin: var(--spacing-px); }
.m-0-5 { margin: var(--spacing-0-5); }
.m-1 { margin: var(--spacing-1); }
.m-1-5 { margin: var(--spacing-1-5); }
.m-2 { margin: var(--spacing-2); }
.m-2-5 { margin: var(--spacing-2-5); }
.m-3 { margin: var(--spacing-3); }
.m-4 { margin: var(--spacing-4); }
.m-5 { margin: var(--spacing-5); }
.m-6 { margin: var(--spacing-6); }
.m-8 { margin: var(--spacing-8); }
.m-10 { margin: var(--spacing-10); }
.m-12 { margin: var(--spacing-12); }
.m-16 { margin: var(--spacing-16); }
.m-auto { margin: auto; }

/* Horizontal (x-axis) */
.mx-0 { margin-left: var(--spacing-0); margin-right: var(--spacing-0); }
.mx-1 { margin-left: var(--spacing-1); margin-right: var(--spacing-1); }
.mx-2 { margin-left: var(--spacing-2); margin-right: var(--spacing-2); }
.mx-3 { margin-left: var(--spacing-3); margin-right: var(--spacing-3); }
.mx-4 { margin-left: var(--spacing-4); margin-right: var(--spacing-4); }
.mx-6 { margin-left: var(--spacing-6); margin-right: var(--spacing-6); }
.mx-8 { margin-left: var(--spacing-8); margin-right: var(--spacing-8); }
.mx-auto { margin-left: auto; margin-right: auto; }

/* Vertical (y-axis) */
.my-0 { margin-top: var(--spacing-0); margin-bottom: var(--spacing-0); }
.my-1 { margin-top: var(--spacing-1); margin-bottom: var(--spacing-1); }
.my-2 { margin-top: var(--spacing-2); margin-bottom: var(--spacing-2); }
.my-3 { margin-top: var(--spacing-3); margin-bottom: var(--spacing-3); }
.my-4 { margin-top: var(--spacing-4); margin-bottom: var(--spacing-4); }
.my-6 { margin-top: var(--spacing-6); margin-bottom: var(--spacing-6); }
.my-8 { margin-top: var(--spacing-8); margin-bottom: var(--spacing-8); }
.my-auto { margin-top: auto; margin-bottom: auto; }

/* Individual sides */
.mt-0 { margin-top: var(--spacing-0); }
.mt-1 { margin-top: var(--spacing-1); }
.mt-2 { margin-top: var(--spacing-2); }
.mt-3 { margin-top: var(--spacing-3); }
.mt-4 { margin-top: var(--spacing-4); }
.mt-6 { margin-top: var(--spacing-6); }
.mt-8 { margin-top: var(--spacing-8); }
.mt-auto { margin-top: auto; }

.mr-0 { margin-right: var(--spacing-0); }
.mr-1 { margin-right: var(--spacing-1); }
.mr-2 { margin-right: var(--spacing-2); }
.mr-3 { margin-right: var(--spacing-3); }
.mr-4 { margin-right: var(--spacing-4); }
.mr-6 { margin-right: var(--spacing-6); }
.mr-8 { margin-right: var(--spacing-8); }
.mr-auto { margin-right: auto; }

.mb-0 { margin-bottom: var(--spacing-0); }
.mb-1 { margin-bottom: var(--spacing-1); }
.mb-2 { margin-bottom: var(--spacing-2); }
.mb-3 { margin-bottom: var(--spacing-3); }
.mb-4 { margin-bottom: var(--spacing-4); }
.mb-6 { margin-bottom: var(--spacing-6); }
.mb-8 { margin-bottom: var(--spacing-8); }
.mb-auto { margin-bottom: auto; }

.ml-0 { margin-left: var(--spacing-0); }
.ml-1 { margin-left: var(--spacing-1); }
.ml-2 { margin-left: var(--spacing-2); }
.ml-3 { margin-left: var(--spacing-3); }
.ml-4 { margin-left: var(--spacing-4); }
.ml-6 { margin-left: var(--spacing-6); }
.ml-8 { margin-left: var(--spacing-8); }
.ml-auto { margin-left: auto; }

/* Negative margins */
.-m-1 { margin: calc(var(--spacing-1) * -1); }
.-m-2 { margin: calc(var(--spacing-2) * -1); }
.-mt-1 { margin-top: calc(var(--spacing-1) * -1); }
.-mt-2 { margin-top: calc(var(--spacing-2) * -1); }
.-mr-1 { margin-right: calc(var(--spacing-1) * -1); }
.-mr-2 { margin-right: calc(var(--spacing-2) * -1); }
.-mb-1 { margin-bottom: calc(var(--spacing-1) * -1); }
.-mb-2 { margin-bottom: calc(var(--spacing-2) * -1); }
.-ml-1 { margin-left: calc(var(--spacing-1) * -1); }
.-ml-2 { margin-left: calc(var(--spacing-2) * -1); }

/* ============================================
 * PADDING UTILITIES
 * ============================================ */

/* All sides */
.p-0 { padding: var(--spacing-0); }
.p-px { padding: var(--spacing-px); }
.p-0-5 { padding: var(--spacing-0-5); }
.p-1 { padding: var(--spacing-1); }
.p-1-5 { padding: var(--spacing-1-5); }
.p-2 { padding: var(--spacing-2); }
.p-2-5 { padding: var(--spacing-2-5); }
.p-3 { padding: var(--spacing-3); }
.p-4 { padding: var(--spacing-4); }
.p-5 { padding: var(--spacing-5); }
.p-6 { padding: var(--spacing-6); }
.p-8 { padding: var(--spacing-8); }
.p-10 { padding: var(--spacing-10); }
.p-12 { padding: var(--spacing-12); }
.p-16 { padding: var(--spacing-16); }

/* Horizontal (x-axis) */
.px-0 { padding-left: var(--spacing-0); padding-right: var(--spacing-0); }
.px-1 { padding-left: var(--spacing-1); padding-right: var(--spacing-1); }
.px-2 { padding-left: var(--spacing-2); padding-right: var(--spacing-2); }
.px-3 { padding-left: var(--spacing-3); padding-right: var(--spacing-3); }
.px-4 { padding-left: var(--spacing-4); padding-right: var(--spacing-4); }
.px-6 { padding-left: var(--spacing-6); padding-right: var(--spacing-6); }
.px-8 { padding-left: var(--spacing-8); padding-right: var(--spacing-8); }

/* Vertical (y-axis) */
.py-0 { padding-top: var(--spacing-0); padding-bottom: var(--spacing-0); }
.py-1 { padding-top: var(--spacing-1); padding-bottom: var(--spacing-1); }
.py-2 { padding-top: var(--spacing-2); padding-bottom: var(--spacing-2); }
.py-3 { padding-top: var(--spacing-3); padding-bottom: var(--spacing-3); }
.py-4 { padding-top: var(--spacing-4); padding-bottom: var(--spacing-4); }
.py-6 { padding-top: var(--spacing-6); padding-bottom: var(--spacing-6); }
.py-8 { padding-top: var(--spacing-8); padding-bottom: var(--spacing-8); }

/* Individual sides */
.pt-0 { padding-top: var(--spacing-0); }
.pt-1 { padding-top: var(--spacing-1); }
.pt-2 { padding-top: var(--spacing-2); }
.pt-3 { padding-top: var(--spacing-3); }
.pt-4 { padding-top: var(--spacing-4); }
.pt-6 { padding-top: var(--spacing-6); }
.pt-8 { padding-top: var(--spacing-8); }

.pr-0 { padding-right: var(--spacing-0); }
.pr-1 { padding-right: var(--spacing-1); }
.pr-2 { padding-right: var(--spacing-2); }
.pr-3 { padding-right: var(--spacing-3); }
.pr-4 { padding-right: var(--spacing-4); }
.pr-6 { padding-right: var(--spacing-6); }
.pr-8 { padding-right: var(--spacing-8); }

.pb-0 { padding-bottom: var(--spacing-0); }
.pb-1 { padding-bottom: var(--spacing-1); }
.pb-2 { padding-bottom: var(--spacing-2); }
.pb-3 { padding-bottom: var(--spacing-3); }
.pb-4 { padding-bottom: var(--spacing-4); }
.pb-6 { padding-bottom: var(--spacing-6); }
.pb-8 { padding-bottom: var(--spacing-8); }

.pl-0 { padding-left: var(--spacing-0); }
.pl-1 { padding-left: var(--spacing-1); }
.pl-2 { padding-left: var(--spacing-2); }
.pl-3 { padding-left: var(--spacing-3); }
.pl-4 { padding-left: var(--spacing-4); }
.pl-6 { padding-left: var(--spacing-6); }
.pl-8 { padding-left: var(--spacing-8); }

/* ============================================
 * GAP UTILITIES (Flexbox/Grid)
 * ============================================ */

.gap-0 { gap: var(--spacing-0); }
.gap-px { gap: var(--spacing-px); }
.gap-0-5 { gap: var(--spacing-0-5); }
.gap-1 { gap: var(--spacing-1); }
.gap-1-5 { gap: var(--spacing-1-5); }
.gap-2 { gap: var(--spacing-2); }
.gap-2-5 { gap: var(--spacing-2-5); }
.gap-3 { gap: var(--spacing-3); }
.gap-4 { gap: var(--spacing-4); }
.gap-5 { gap: var(--spacing-5); }
.gap-6 { gap: var(--spacing-6); }
.gap-8 { gap: var(--spacing-8); }
.gap-10 { gap: var(--spacing-10); }
.gap-12 { gap: var(--spacing-12); }

/* Column gap */
.gap-x-0 { column-gap: var(--spacing-0); }
.gap-x-1 { column-gap: var(--spacing-1); }
.gap-x-2 { column-gap: var(--spacing-2); }
.gap-x-3 { column-gap: var(--spacing-3); }
.gap-x-4 { column-gap: var(--spacing-4); }
.gap-x-6 { column-gap: var(--spacing-6); }
.gap-x-8 { column-gap: var(--spacing-8); }

/* Row gap */
.gap-y-0 { row-gap: var(--spacing-0); }
.gap-y-1 { row-gap: var(--spacing-1); }
.gap-y-2 { row-gap: var(--spacing-2); }
.gap-y-3 { row-gap: var(--spacing-3); }
.gap-y-4 { row-gap: var(--spacing-4); }
.gap-y-6 { row-gap: var(--spacing-6); }
.gap-y-8 { row-gap: var(--spacing-8); }

/* ============================================
 * LAYOUT PATTERNS
 * ============================================ */

/* Stack - Vertical spacing between children */
.stack {
  display: flex;
  flex-direction: column;
}

.stack-xs > * + * { margin-top: var(--stack-xs); }
.stack-sm > * + * { margin-top: var(--stack-sm); }
.stack-md > * + * { margin-top: var(--stack-md); }
.stack-lg > * + * { margin-top: var(--stack-lg); }
.stack-xl > * + * { margin-top: var(--stack-xl); }

/* Inline - Horizontal spacing between children */
.inline {
  display: flex;
  flex-direction: row;
  align-items: center;
}

.inline-xs { gap: var(--inline-xs); }
.inline-sm { gap: var(--inline-sm); }
.inline-md { gap: var(--inline-md); }
.inline-lg { gap: var(--inline-lg); }
.inline-xl { gap: var(--inline-xl); }

/* Cluster - Wrapping inline items */
.cluster {
  display: flex;
  flex-wrap: wrap;
}

.cluster-xs { gap: var(--spacing-1); }
.cluster-sm { gap: var(--spacing-2); }
.cluster-md { gap: var(--spacing-3); }
.cluster-lg { gap: var(--spacing-4); }
```

### src/lib/components/ui/Stack.svelte

```svelte
<script lang="ts">
  type StackGap = 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' | '3xl';
  type StackAlign = 'start' | 'center' | 'end' | 'stretch';
  type StackJustify = 'start' | 'center' | 'end' | 'between' | 'around';

  export let gap: StackGap = 'md';
  export let align: StackAlign = 'stretch';
  export let justify: StackJustify = 'start';
  export let dividers: boolean = false;

  const gapMap: Record<StackGap, string> = {
    'xs': 'var(--spacing-1)',
    'sm': 'var(--spacing-2)',
    'md': 'var(--spacing-4)',
    'lg': 'var(--spacing-6)',
    'xl': 'var(--spacing-8)',
    '2xl': 'var(--spacing-12)',
    '3xl': 'var(--spacing-16)'
  };

  const alignMap: Record<StackAlign, string> = {
    'start': 'flex-start',
    'center': 'center',
    'end': 'flex-end',
    'stretch': 'stretch'
  };

  const justifyMap: Record<StackJustify, string> = {
    'start': 'flex-start',
    'center': 'center',
    'end': 'flex-end',
    'between': 'space-between',
    'around': 'space-around'
  };
</script>

<div
  class="stack"
  class:dividers
  style="
    --stack-gap: {gapMap[gap]};
    --stack-align: {alignMap[align]};
    --stack-justify: {justifyMap[justify]};
  "
  {...$$restProps}
>
  <slot />
</div>

<style>
  .stack {
    display: flex;
    flex-direction: column;
    gap: var(--stack-gap);
    align-items: var(--stack-align);
    justify-content: var(--stack-justify);
  }

  .stack.dividers > :global(* + *) {
    padding-top: var(--stack-gap);
    border-top: 1px solid var(--color-border-default);
  }

  .stack.dividers {
    gap: 0;
  }
</style>
```

### src/lib/components/ui/Inline.svelte

```svelte
<script lang="ts">
  type InlineGap = 'xs' | 'sm' | 'md' | 'lg' | 'xl';
  type InlineAlign = 'start' | 'center' | 'end' | 'baseline' | 'stretch';
  type InlineJustify = 'start' | 'center' | 'end' | 'between' | 'around';

  export let gap: InlineGap = 'md';
  export let align: InlineAlign = 'center';
  export let justify: InlineJustify = 'start';
  export let wrap: boolean = false;

  const gapMap: Record<InlineGap, string> = {
    'xs': 'var(--spacing-1)',
    'sm': 'var(--spacing-2)',
    'md': 'var(--spacing-3)',
    'lg': 'var(--spacing-4)',
    'xl': 'var(--spacing-6)'
  };

  const alignMap: Record<InlineAlign, string> = {
    'start': 'flex-start',
    'center': 'center',
    'end': 'flex-end',
    'baseline': 'baseline',
    'stretch': 'stretch'
  };

  const justifyMap: Record<InlineJustify, string> = {
    'start': 'flex-start',
    'center': 'center',
    'end': 'flex-end',
    'between': 'space-between',
    'around': 'space-around'
  };
</script>

<div
  class="inline"
  class:wrap
  style="
    --inline-gap: {gapMap[gap]};
    --inline-align: {alignMap[align]};
    --inline-justify: {justifyMap[justify]};
  "
  {...$$restProps}
>
  <slot />
</div>

<style>
  .inline {
    display: flex;
    flex-direction: row;
    gap: var(--inline-gap);
    align-items: var(--inline-align);
    justify-content: var(--inline-justify);
  }

  .inline.wrap {
    flex-wrap: wrap;
  }
</style>
```

### src/lib/utils/spacing.ts

```typescript
/**
 * Spacing utility functions
 */

export type SpacingScale =
  | 0 | 0.5 | 1 | 1.5 | 2 | 2.5 | 3 | 3.5 | 4 | 5 | 6 | 7 | 8
  | 9 | 10 | 11 | 12 | 14 | 16 | 20 | 24 | 28 | 32 | 36 | 40;

const spacingValues: Record<SpacingScale, string> = {
  0: '0',
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
  36: '9rem',
  40: '10rem'
};

/**
 * Get spacing value in rem
 */
export function spacing(scale: SpacingScale): string {
  return spacingValues[scale];
}

/**
 * Convert spacing scale to pixels (assuming 16px base)
 */
export function spacingPx(scale: SpacingScale): number {
  return scale * 4;
}

/**
 * Create margin CSS
 */
export function margin(
  top?: SpacingScale,
  right?: SpacingScale,
  bottom?: SpacingScale,
  left?: SpacingScale
): string {
  const values = [
    top !== undefined ? spacing(top) : '0',
    right !== undefined ? spacing(right) : top !== undefined ? spacing(top) : '0',
    bottom !== undefined ? spacing(bottom) : top !== undefined ? spacing(top) : '0',
    left !== undefined ? spacing(left) : right !== undefined ? spacing(right) : top !== undefined ? spacing(top) : '0'
  ];
  return values.join(' ');
}

/**
 * Create padding CSS
 */
export function padding(
  top?: SpacingScale,
  right?: SpacingScale,
  bottom?: SpacingScale,
  left?: SpacingScale
): string {
  return margin(top, right, bottom, left);
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/spacing/utilities.test.ts
import { describe, it, expect } from 'vitest';
import { spacing, spacingPx, margin, padding } from '@utils/spacing';

describe('Spacing Utilities', () => {
  it('should return correct spacing values', () => {
    expect(spacing(0)).toBe('0');
    expect(spacing(4)).toBe('1rem');
    expect(spacing(8)).toBe('2rem');
  });

  it('should convert to pixels', () => {
    expect(spacingPx(4)).toBe(16);
    expect(spacingPx(8)).toBe(32);
  });

  it('should create margin string', () => {
    expect(margin(4)).toBe('1rem 1rem 1rem 1rem');
    expect(margin(4, 2)).toBe('1rem 0.5rem 1rem 0.5rem');
    expect(margin(4, 2, 4, 2)).toBe('1rem 0.5rem 1rem 0.5rem');
  });
});
```

---

## Related Specs

- [191-design-tokens.md](./191-design-tokens.md) - Design tokens
- [193-color-system.md](./193-color-system.md) - Color system
- [196-component-library.md](./196-component-library.md) - Component library
