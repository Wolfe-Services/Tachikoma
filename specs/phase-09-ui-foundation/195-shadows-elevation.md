# Spec 195: Shadows and Elevation

## Phase
Phase 9: UI Foundation

## Spec ID
195

## Status
Planned

## Dependencies
- Spec 186: SvelteKit Setup
- Spec 191: Design Tokens

## Estimated Context
~8%

---

## Objective

Implement a shadow and elevation system for Tachikoma that creates visual hierarchy through depth, including standard shadows, Tachikoma-themed glow effects, and elevation utilities for different UI layers.

---

## Acceptance Criteria

- [x] Shadow scale (sm, md, lg, xl, 2xl)
- [x] Tachikoma blue glow effects
- [x] Elevation levels for UI layers
- [x] Focus ring shadows
- [x] Card and modal shadows
- [x] Dark theme optimized shadows
- [x] Performance-optimized CSS
- [x] Svelte shadow components

---

## Implementation Details

### src/lib/styles/shadows.css

```css
/**
 * Tachikoma Shadow & Elevation System
 *
 * Designed for dark theme with subtle depth cues
 * and signature Tachikoma blue glow effects.
 */

:root {
  /* ============================================
   * BASE SHADOWS
   * Optimized for dark backgrounds
   * ============================================ */

  /* Shadow color base */
  --shadow-color: 0 0 0;
  --shadow-ambient: rgba(var(--shadow-color), 0.2);
  --shadow-direct: rgba(var(--shadow-color), 0.3);

  /* Shadow Scale */
  --shadow-xs: 0 1px 2px rgba(var(--shadow-color), 0.2);

  --shadow-sm:
    0 1px 2px rgba(var(--shadow-color), 0.25),
    0 1px 3px rgba(var(--shadow-color), 0.15);

  --shadow-md:
    0 2px 4px rgba(var(--shadow-color), 0.2),
    0 4px 8px rgba(var(--shadow-color), 0.25),
    0 1px 3px rgba(var(--shadow-color), 0.15);

  --shadow-lg:
    0 4px 8px rgba(var(--shadow-color), 0.2),
    0 8px 16px rgba(var(--shadow-color), 0.25),
    0 16px 32px rgba(var(--shadow-color), 0.15);

  --shadow-xl:
    0 8px 16px rgba(var(--shadow-color), 0.2),
    0 16px 32px rgba(var(--shadow-color), 0.25),
    0 32px 64px rgba(var(--shadow-color), 0.2);

  --shadow-2xl:
    0 16px 32px rgba(var(--shadow-color), 0.25),
    0 32px 64px rgba(var(--shadow-color), 0.3),
    0 64px 128px rgba(var(--shadow-color), 0.2);

  /* Inner shadows */
  --shadow-inner: inset 0 2px 4px rgba(var(--shadow-color), 0.3);
  --shadow-inner-sm: inset 0 1px 2px rgba(var(--shadow-color), 0.2);
  --shadow-inner-lg: inset 0 4px 8px rgba(var(--shadow-color), 0.35);

  /* ============================================
   * TACHIKOMA GLOW EFFECTS
   * Signature blue glow for emphasis
   * ============================================ */

  --glow-color: 0, 212, 255; /* Tachikoma blue RGB */

  /* Glow scale */
  --glow-xs: 0 0 4px rgba(var(--glow-color), 0.3);

  --glow-sm:
    0 0 4px rgba(var(--glow-color), 0.3),
    0 0 8px rgba(var(--glow-color), 0.2);

  --glow-md:
    0 0 8px rgba(var(--glow-color), 0.4),
    0 0 16px rgba(var(--glow-color), 0.3),
    0 0 24px rgba(var(--glow-color), 0.1);

  --glow-lg:
    0 0 16px rgba(var(--glow-color), 0.5),
    0 0 32px rgba(var(--glow-color), 0.35),
    0 0 48px rgba(var(--glow-color), 0.15);

  --glow-xl:
    0 0 24px rgba(var(--glow-color), 0.5),
    0 0 48px rgba(var(--glow-color), 0.4),
    0 0 72px rgba(var(--glow-color), 0.2),
    0 0 96px rgba(var(--glow-color), 0.1);

  /* Glow variations */
  --glow-pulse: 0 0 20px rgba(var(--glow-color), 0.6);
  --glow-ring: 0 0 0 2px rgba(var(--glow-color), 0.4);
  --glow-border: 0 0 0 1px rgba(var(--glow-color), 0.5);

  /* Text glow */
  --text-glow-sm: 0 0 8px rgba(var(--glow-color), 0.5);
  --text-glow-md: 0 0 12px rgba(var(--glow-color), 0.6);
  --text-glow-lg: 0 0 20px rgba(var(--glow-color), 0.7);

  /* ============================================
   * FOCUS SHADOWS
   * Accessibility-friendly focus indicators
   * ============================================ */

  --focus-ring:
    0 0 0 2px var(--color-bg-surface),
    0 0 0 4px var(--tachikoma-500);

  --focus-ring-error:
    0 0 0 2px var(--color-bg-surface),
    0 0 0 4px var(--error-500);

  --focus-ring-success:
    0 0 0 2px var(--color-bg-surface),
    0 0 0 4px var(--success-500);

  --focus-glow:
    0 0 0 2px var(--color-bg-surface),
    0 0 0 4px var(--tachikoma-500),
    0 0 16px rgba(var(--glow-color), 0.3);

  /* ============================================
   * ELEVATION LEVELS
   * Semantic elevation for UI layers
   * ============================================ */

  /* Level 0: Sunken/inset elements */
  --elevation-0: var(--shadow-inner-sm);

  /* Level 1: Base surface (cards, panels) */
  --elevation-1: var(--shadow-sm);

  /* Level 2: Raised elements (buttons, inputs) */
  --elevation-2: var(--shadow-md);

  /* Level 3: Floating elements (dropdowns, popovers) */
  --elevation-3: var(--shadow-lg);

  /* Level 4: Modals, dialogs */
  --elevation-4: var(--shadow-xl);

  /* Level 5: Toasts, notifications */
  --elevation-5: var(--shadow-2xl);

  /* ============================================
   * COMPONENT-SPECIFIC SHADOWS
   * ============================================ */

  /* Cards */
  --card-shadow: var(--shadow-sm);
  --card-shadow-hover: var(--shadow-md);
  --card-shadow-active: var(--shadow-lg);
  --card-shadow-glow:
    var(--shadow-md),
    var(--glow-sm);

  /* Buttons */
  --button-shadow: var(--shadow-xs);
  --button-shadow-hover: var(--shadow-sm);
  --button-shadow-active: var(--shadow-inner-sm);
  --button-shadow-primary:
    var(--shadow-sm),
    var(--glow-xs);
  --button-shadow-primary-hover:
    var(--shadow-md),
    var(--glow-sm);

  /* Inputs */
  --input-shadow: var(--shadow-inner-sm);
  --input-shadow-focus: var(--focus-ring);
  --input-shadow-error: var(--focus-ring-error);

  /* Modals */
  --modal-shadow:
    var(--shadow-2xl),
    0 0 100px rgba(0, 0, 0, 0.5);

  --modal-shadow-glow:
    var(--shadow-2xl),
    0 0 60px rgba(var(--glow-color), 0.15);

  /* Tooltips */
  --tooltip-shadow: var(--shadow-lg);

  /* Dropdowns */
  --dropdown-shadow:
    var(--shadow-lg),
    0 0 1px rgba(var(--shadow-color), 0.3);

  /* Toast notifications */
  --toast-shadow:
    var(--shadow-xl),
    var(--glow-xs);
}

/* ============================================
 * LIGHT THEME ADJUSTMENTS
 * ============================================ */

[data-theme="light"] {
  --shadow-color: 0, 0, 0;

  --shadow-xs: 0 1px 2px rgba(var(--shadow-color), 0.06);

  --shadow-sm:
    0 1px 2px rgba(var(--shadow-color), 0.08),
    0 1px 3px rgba(var(--shadow-color), 0.06);

  --shadow-md:
    0 2px 4px rgba(var(--shadow-color), 0.06),
    0 4px 8px rgba(var(--shadow-color), 0.1);

  --shadow-lg:
    0 4px 8px rgba(var(--shadow-color), 0.06),
    0 8px 16px rgba(var(--shadow-color), 0.1),
    0 16px 32px rgba(var(--shadow-color), 0.06);

  /* Lighter glow for light theme */
  --glow-sm:
    0 0 4px rgba(var(--glow-color), 0.2),
    0 0 8px rgba(var(--glow-color), 0.15);

  --glow-md:
    0 0 8px rgba(var(--glow-color), 0.25),
    0 0 16px rgba(var(--glow-color), 0.2);
}

/* ============================================
 * UTILITY CLASSES
 * ============================================ */

/* Shadow utilities */
.shadow-none { box-shadow: none; }
.shadow-xs { box-shadow: var(--shadow-xs); }
.shadow-sm { box-shadow: var(--shadow-sm); }
.shadow-md { box-shadow: var(--shadow-md); }
.shadow-lg { box-shadow: var(--shadow-lg); }
.shadow-xl { box-shadow: var(--shadow-xl); }
.shadow-2xl { box-shadow: var(--shadow-2xl); }
.shadow-inner { box-shadow: var(--shadow-inner); }

/* Glow utilities */
.glow-none { box-shadow: none; }
.glow-xs { box-shadow: var(--glow-xs); }
.glow-sm { box-shadow: var(--glow-sm); }
.glow-md { box-shadow: var(--glow-md); }
.glow-lg { box-shadow: var(--glow-lg); }
.glow-xl { box-shadow: var(--glow-xl); }

/* Text glow */
.text-glow-sm { text-shadow: var(--text-glow-sm); }
.text-glow-md { text-shadow: var(--text-glow-md); }
.text-glow-lg { text-shadow: var(--text-glow-lg); }

/* Elevation utilities */
.elevation-0 { box-shadow: var(--elevation-0); }
.elevation-1 { box-shadow: var(--elevation-1); }
.elevation-2 { box-shadow: var(--elevation-2); }
.elevation-3 { box-shadow: var(--elevation-3); }
.elevation-4 { box-shadow: var(--elevation-4); }
.elevation-5 { box-shadow: var(--elevation-5); }

/* Animated glow */
@keyframes pulse-glow {
  0%, 100% {
    box-shadow: var(--glow-sm);
  }
  50% {
    box-shadow: var(--glow-lg);
  }
}

.glow-pulse {
  animation: pulse-glow 2s ease-in-out infinite;
}

/* Hover transitions */
.shadow-transition {
  transition: box-shadow var(--duration-200) var(--ease-out);
}

.hover\:shadow-md:hover { box-shadow: var(--shadow-md); }
.hover\:shadow-lg:hover { box-shadow: var(--shadow-lg); }
.hover\:glow-sm:hover { box-shadow: var(--glow-sm); }
.hover\:glow-md:hover { box-shadow: var(--glow-md); }
```

### src/lib/components/ui/Elevation.svelte

```svelte
<script lang="ts">
  type ElevationLevel = 0 | 1 | 2 | 3 | 4 | 5;

  export let level: ElevationLevel = 1;
  export let glow: boolean = false;
  export let hoverLevel: ElevationLevel | null = null;
  export let as: 'div' | 'section' | 'article' = 'div';

  $: shadowVar = `var(--elevation-${level})`;
  $: hoverShadowVar = hoverLevel !== null ? `var(--elevation-${hoverLevel})` : null;
</script>

<svelte:element
  this={as}
  class="elevation"
  class:glow
  class:hoverable={hoverLevel !== null}
  style="
    --current-shadow: {shadowVar};
    {hoverShadowVar ? `--hover-shadow: ${hoverShadowVar};` : ''}
  "
  {...$$restProps}
>
  <slot />
</svelte:element>

<style>
  .elevation {
    box-shadow: var(--current-shadow);
    transition: box-shadow var(--duration-200) var(--ease-out);
  }

  .elevation.glow {
    box-shadow: var(--current-shadow), var(--glow-xs);
  }

  .elevation.hoverable:hover {
    box-shadow: var(--hover-shadow, var(--current-shadow));
  }

  .elevation.hoverable.glow:hover {
    box-shadow: var(--hover-shadow, var(--current-shadow)), var(--glow-sm);
  }
</style>
```

### src/lib/components/ui/GlowText.svelte

```svelte
<script lang="ts">
  type GlowIntensity = 'sm' | 'md' | 'lg';

  export let intensity: GlowIntensity = 'md';
  export let animate: boolean = false;
  export let color: string = 'var(--tachikoma-500)';
</script>

<span
  class="glow-text"
  class:animate
  class:intensity-sm={intensity === 'sm'}
  class:intensity-md={intensity === 'md'}
  class:intensity-lg={intensity === 'lg'}
  style="--glow-color: {color};"
  {...$$restProps}
>
  <slot />
</span>

<style>
  .glow-text {
    color: var(--glow-color);
    transition: text-shadow var(--duration-200) var(--ease-out);
  }

  .glow-text.intensity-sm {
    text-shadow: 0 0 8px currentColor;
  }

  .glow-text.intensity-md {
    text-shadow:
      0 0 8px currentColor,
      0 0 16px currentColor;
  }

  .glow-text.intensity-lg {
    text-shadow:
      0 0 8px currentColor,
      0 0 16px currentColor,
      0 0 32px currentColor;
  }

  .glow-text.animate {
    animation: text-pulse 2s ease-in-out infinite;
  }

  @keyframes text-pulse {
    0%, 100% {
      opacity: 1;
      text-shadow:
        0 0 8px currentColor,
        0 0 16px currentColor;
    }
    50% {
      opacity: 0.9;
      text-shadow:
        0 0 16px currentColor,
        0 0 32px currentColor,
        0 0 48px currentColor;
    }
  }
</style>
```

### src/lib/utils/shadows.ts

```typescript
/**
 * Shadow utility functions
 */

export type ShadowSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';
export type ElevationLevel = 0 | 1 | 2 | 3 | 4 | 5;
export type GlowIntensity = 'xs' | 'sm' | 'md' | 'lg' | 'xl';

/**
 * Get CSS variable for shadow size
 */
export function getShadow(size: ShadowSize): string {
  return `var(--shadow-${size})`;
}

/**
 * Get CSS variable for elevation level
 */
export function getElevation(level: ElevationLevel): string {
  return `var(--elevation-${level})`;
}

/**
 * Get CSS variable for glow intensity
 */
export function getGlow(intensity: GlowIntensity): string {
  return `var(--glow-${intensity})`;
}

/**
 * Combine shadow and glow
 */
export function shadowWithGlow(
  shadow: ShadowSize,
  glow: GlowIntensity
): string {
  return `var(--shadow-${shadow}), var(--glow-${glow})`;
}

/**
 * Create custom glow with color
 */
export function customGlow(
  color: string,
  intensity: 'light' | 'medium' | 'strong' = 'medium'
): string {
  const opacities = {
    light: [0.2, 0.1],
    medium: [0.4, 0.2],
    strong: [0.6, 0.3]
  };
  const [primary, secondary] = opacities[intensity];

  return `
    0 0 8px ${color}${Math.round(primary * 255).toString(16)},
    0 0 16px ${color}${Math.round(secondary * 255).toString(16)}
  `.trim();
}
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/shadows/utilities.test.ts
import { describe, it, expect } from 'vitest';
import {
  getShadow,
  getElevation,
  getGlow,
  shadowWithGlow
} from '@utils/shadows';

describe('Shadow Utilities', () => {
  it('should return correct shadow variable', () => {
    expect(getShadow('md')).toBe('var(--shadow-md)');
    expect(getShadow('lg')).toBe('var(--shadow-lg)');
  });

  it('should return correct elevation variable', () => {
    expect(getElevation(0)).toBe('var(--elevation-0)');
    expect(getElevation(3)).toBe('var(--elevation-3)');
  });

  it('should return correct glow variable', () => {
    expect(getGlow('sm')).toBe('var(--glow-sm)');
    expect(getGlow('lg')).toBe('var(--glow-lg)');
  });

  it('should combine shadow and glow', () => {
    expect(shadowWithGlow('md', 'sm')).toBe('var(--shadow-md), var(--glow-sm)');
  });
});
```

---

## Related Specs

- [191-design-tokens.md](./191-design-tokens.md) - Design tokens
- [193-color-system.md](./193-color-system.md) - Color system
- [196-component-library.md](./196-component-library.md) - Component library
