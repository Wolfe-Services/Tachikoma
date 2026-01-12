# Spec 190b: Glassmorphic Design System

## Phase
Phase 9: UI Foundation

## Spec ID
190b

## Status
Planned

## Dependencies
- Spec 186: SvelteKit Setup

## Estimated Context
~8%

---

## Objective

Establish a glassmorphic (frosted glass) design language for Tachikoma's UI, featuring translucent panels with backdrop blur, subtle borders, layered depth, and vibrant accent colors showing through the glass. This creates a modern, premium feel while maintaining the signature Tachikoma blue identity.

---

## Design Principles

### 1. Frosted Glass Aesthetic
- Semi-transparent backgrounds with backdrop-filter blur
- Content behind panels is visible but softened
- Creates depth and layering without harsh boundaries

### 2. Luminous Borders
- Subtle 1px borders with gradient or semi-transparent white
- Borders catch light and define edges without being heavy
- Inner glow effects for elevated elements

### 3. Vibrant Underlays
- Gradient meshes or color washes visible through glass
- Tachikoma blue and purple accents as background elements
- Dynamic, living feel to the interface

### 4. Depth Through Blur
- Multiple blur levels create visual hierarchy
- More blur = further back in the visual stack
- Sharp elements draw focus

### 5. Soft Shadows
- Diffused, colored shadows (not pure black)
- Shadow colors tinted with nearby accent colors
- Creates floating, ethereal feel

---

## Acceptance Criteria

- [ ] Glass effect CSS utilities with multiple intensity levels
- [ ] Gradient mesh background system
- [ ] Luminous border styles
- [ ] Updated shadow system with colored/soft shadows
- [ ] Glass card component styles
- [ ] Glass modal/overlay styles
- [ ] Sidebar glass panel styles
- [ ] Performance considerations for backdrop-filter

---

## Implementation Details

### src/lib/styles/glass.css

```css
/**
 * Tachikoma Glassmorphic Design System
 *
 * Frosted glass effects with luminous borders and vibrant underlays.
 * Inspired by modern UI trends - translucent, layered, and alive.
 */

:root {
  /* ============================================
   * GLASS EFFECT TOKENS
   * ============================================ */

  /* Blur intensities */
  --glass-blur-xs: 4px;
  --glass-blur-sm: 8px;
  --glass-blur-md: 16px;
  --glass-blur-lg: 24px;
  --glass-blur-xl: 40px;

  /* Glass background colors (dark theme) */
  --glass-bg-dark: rgba(13, 17, 23, 0.7);
  --glass-bg-dark-solid: rgba(13, 17, 23, 0.85);
  --glass-bg-dark-subtle: rgba(13, 17, 23, 0.5);
  --glass-bg-dark-ultra: rgba(13, 17, 23, 0.95);

  /* Glass background colors (can be overridden for light theme) */
  --glass-bg: var(--glass-bg-dark);
  --glass-bg-solid: var(--glass-bg-dark-solid);
  --glass-bg-subtle: var(--glass-bg-dark-subtle);
  --glass-bg-ultra: var(--glass-bg-dark-ultra);

  /* Luminous border colors */
  --glass-border: rgba(255, 255, 255, 0.08);
  --glass-border-light: rgba(255, 255, 255, 0.12);
  --glass-border-accent: rgba(0, 212, 255, 0.3);
  --glass-border-glow: rgba(0, 212, 255, 0.5);

  /* Inner glow / highlight */
  --glass-highlight: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.1) 0%,
    rgba(255, 255, 255, 0.05) 50%,
    transparent 100%
  );

  /* Soft shadows with color tinting */
  --glass-shadow-sm:
    0 2px 8px rgba(0, 0, 0, 0.3),
    0 1px 2px rgba(0, 0, 0, 0.2);
  --glass-shadow-md:
    0 4px 16px rgba(0, 0, 0, 0.4),
    0 2px 4px rgba(0, 0, 0, 0.2);
  --glass-shadow-lg:
    0 8px 32px rgba(0, 0, 0, 0.5),
    0 4px 8px rgba(0, 0, 0, 0.3);
  --glass-shadow-xl:
    0 16px 48px rgba(0, 0, 0, 0.6),
    0 8px 16px rgba(0, 0, 0, 0.3);

  /* Accent-tinted shadows (for elevated interactive elements) */
  --glass-shadow-accent:
    0 4px 20px rgba(0, 212, 255, 0.15),
    0 2px 8px rgba(0, 0, 0, 0.3);
  --glass-shadow-accent-hover:
    0 8px 32px rgba(0, 212, 255, 0.25),
    0 4px 12px rgba(0, 0, 0, 0.3);

  /* Glass border radius (larger for glass aesthetic) */
  --glass-radius-sm: 8px;
  --glass-radius-md: 12px;
  --glass-radius-lg: 16px;
  --glass-radius-xl: 24px;
}

/* ============================================
 * GRADIENT MESH BACKGROUNDS
 * These create the vibrant underlays visible through glass
 * ============================================ */

.gradient-mesh-base {
  background:
    radial-gradient(
      ellipse 80% 50% at 20% 40%,
      rgba(0, 212, 255, 0.15) 0%,
      transparent 50%
    ),
    radial-gradient(
      ellipse 60% 40% at 80% 20%,
      rgba(168, 85, 247, 0.1) 0%,
      transparent 50%
    ),
    radial-gradient(
      ellipse 50% 60% at 60% 80%,
      rgba(0, 168, 204, 0.12) 0%,
      transparent 50%
    ),
    linear-gradient(
      180deg,
      #0a0e14 0%,
      #0d1117 100%
    );
}

.gradient-mesh-vibrant {
  background:
    radial-gradient(
      ellipse 100% 80% at 10% 20%,
      rgba(0, 212, 255, 0.25) 0%,
      transparent 50%
    ),
    radial-gradient(
      ellipse 80% 60% at 90% 30%,
      rgba(168, 85, 247, 0.2) 0%,
      transparent 50%
    ),
    radial-gradient(
      ellipse 70% 70% at 50% 90%,
      rgba(34, 197, 94, 0.1) 0%,
      transparent 50%
    ),
    linear-gradient(
      180deg,
      #0a0e14 0%,
      #0d1117 100%
    );
}

.gradient-mesh-subtle {
  background:
    radial-gradient(
      ellipse 60% 40% at 30% 30%,
      rgba(0, 212, 255, 0.08) 0%,
      transparent 50%
    ),
    radial-gradient(
      ellipse 40% 50% at 70% 70%,
      rgba(168, 85, 247, 0.05) 0%,
      transparent 50%
    ),
    #0a0e14;
}

/* Animated gradient mesh for special areas */
.gradient-mesh-animated {
  background:
    radial-gradient(
      ellipse 80% 50% at var(--mouse-x, 50%) var(--mouse-y, 50%),
      rgba(0, 212, 255, 0.2) 0%,
      transparent 50%
    ),
    radial-gradient(
      ellipse 60% 40% at 80% 20%,
      rgba(168, 85, 247, 0.1) 0%,
      transparent 50%
    ),
    #0a0e14;
  transition: background 0.3s ease;
}

/* ============================================
 * GLASS PANEL UTILITIES
 * ============================================ */

/* Base glass effect */
.glass {
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur-md));
  -webkit-backdrop-filter: blur(var(--glass-blur-md));
  border: 1px solid var(--glass-border);
  border-radius: var(--glass-radius-md);
}

/* Glass with inner highlight */
.glass-highlight {
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur-md));
  -webkit-backdrop-filter: blur(var(--glass-blur-md));
  border: 1px solid var(--glass-border);
  border-radius: var(--glass-radius-md);
  position: relative;
}

.glass-highlight::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background: var(--glass-highlight);
  pointer-events: none;
}

/* Glass intensity variants */
.glass-subtle {
  background: var(--glass-bg-subtle);
  backdrop-filter: blur(var(--glass-blur-sm));
  -webkit-backdrop-filter: blur(var(--glass-blur-sm));
  border: 1px solid var(--glass-border);
}

.glass-solid {
  background: var(--glass-bg-solid);
  backdrop-filter: blur(var(--glass-blur-lg));
  -webkit-backdrop-filter: blur(var(--glass-blur-lg));
  border: 1px solid var(--glass-border-light);
}

.glass-ultra {
  background: var(--glass-bg-ultra);
  backdrop-filter: blur(var(--glass-blur-xl));
  -webkit-backdrop-filter: blur(var(--glass-blur-xl));
  border: 1px solid var(--glass-border-light);
}

/* ============================================
 * LUMINOUS BORDER EFFECTS
 * ============================================ */

/* Gradient border using pseudo-element */
.border-luminous {
  position: relative;
  border: none;
  background: var(--glass-bg);
  border-radius: var(--glass-radius-md);
}

.border-luminous::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  padding: 1px;
  background: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.15) 0%,
    rgba(255, 255, 255, 0.05) 50%,
    rgba(0, 212, 255, 0.1) 100%
  );
  -webkit-mask:
    linear-gradient(#fff 0 0) content-box,
    linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  pointer-events: none;
}

/* Accent glow border */
.border-glow {
  border: 1px solid var(--glass-border-accent);
  box-shadow:
    0 0 0 1px var(--glass-border-accent),
    0 0 20px rgba(0, 212, 255, 0.1);
}

.border-glow:hover {
  border-color: var(--glass-border-glow);
  box-shadow:
    0 0 0 1px var(--glass-border-glow),
    0 0 30px rgba(0, 212, 255, 0.2);
}

/* ============================================
 * GLASS COMPONENT STYLES
 * ============================================ */

/* Glass Card */
.glass-card {
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur-md));
  -webkit-backdrop-filter: blur(var(--glass-blur-md));
  border: 1px solid var(--glass-border);
  border-radius: var(--glass-radius-lg);
  box-shadow: var(--glass-shadow-md);
  padding: 1.5rem;
  transition: all 0.2s ease;
}

.glass-card:hover {
  border-color: var(--glass-border-light);
  box-shadow: var(--glass-shadow-lg);
  transform: translateY(-2px);
}

.glass-card-interactive:hover {
  border-color: var(--glass-border-accent);
  box-shadow: var(--glass-shadow-accent-hover);
}

/* Glass Modal/Dialog */
.glass-modal {
  background: var(--glass-bg-solid);
  backdrop-filter: blur(var(--glass-blur-xl));
  -webkit-backdrop-filter: blur(var(--glass-blur-xl));
  border: 1px solid var(--glass-border-light);
  border-radius: var(--glass-radius-xl);
  box-shadow: var(--glass-shadow-xl);
}

/* Glass modal backdrop */
.glass-modal-backdrop {
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(var(--glass-blur-sm));
  -webkit-backdrop-filter: blur(var(--glass-blur-sm));
}

/* Glass Sidebar */
.glass-sidebar {
  background: var(--glass-bg-solid);
  backdrop-filter: blur(var(--glass-blur-lg));
  -webkit-backdrop-filter: blur(var(--glass-blur-lg));
  border-right: 1px solid var(--glass-border);
}

/* Glass Header/Navbar */
.glass-header {
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur-md));
  -webkit-backdrop-filter: blur(var(--glass-blur-md));
  border-bottom: 1px solid var(--glass-border);
}

/* Glass Input */
.glass-input {
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(var(--glass-blur-sm));
  -webkit-backdrop-filter: blur(var(--glass-blur-sm));
  border: 1px solid var(--glass-border);
  border-radius: var(--glass-radius-sm);
  transition: all 0.2s ease;
}

.glass-input:focus {
  background: rgba(255, 255, 255, 0.05);
  border-color: var(--glass-border-accent);
  box-shadow: 0 0 0 3px rgba(0, 212, 255, 0.1);
  outline: none;
}

/* Glass Button */
.glass-button {
  background: var(--glass-bg-subtle);
  backdrop-filter: blur(var(--glass-blur-sm));
  -webkit-backdrop-filter: blur(var(--glass-blur-sm));
  border: 1px solid var(--glass-border);
  border-radius: var(--glass-radius-sm);
  padding: 0.5rem 1rem;
  color: var(--color-fg-default);
  transition: all 0.15s ease;
}

.glass-button:hover {
  background: var(--glass-bg);
  border-color: var(--glass-border-light);
  transform: translateY(-1px);
}

.glass-button-primary {
  background: rgba(0, 212, 255, 0.15);
  border-color: var(--glass-border-accent);
}

.glass-button-primary:hover {
  background: rgba(0, 212, 255, 0.25);
  border-color: var(--glass-border-glow);
  box-shadow: var(--glass-shadow-accent);
}

/* ============================================
 * LIGHT THEME OVERRIDES
 * ============================================ */

[data-theme="light"] {
  --glass-bg: rgba(255, 255, 255, 0.7);
  --glass-bg-solid: rgba(255, 255, 255, 0.85);
  --glass-bg-subtle: rgba(255, 255, 255, 0.5);
  --glass-bg-ultra: rgba(255, 255, 255, 0.95);

  --glass-border: rgba(0, 0, 0, 0.08);
  --glass-border-light: rgba(0, 0, 0, 0.12);
  --glass-border-accent: rgba(0, 168, 204, 0.4);
  --glass-border-glow: rgba(0, 168, 204, 0.6);

  --glass-highlight: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.5) 0%,
    rgba(255, 255, 255, 0.2) 50%,
    transparent 100%
  );

  --glass-shadow-sm:
    0 2px 8px rgba(0, 0, 0, 0.08),
    0 1px 2px rgba(0, 0, 0, 0.05);
  --glass-shadow-md:
    0 4px 16px rgba(0, 0, 0, 0.1),
    0 2px 4px rgba(0, 0, 0, 0.05);
  --glass-shadow-lg:
    0 8px 32px rgba(0, 0, 0, 0.12),
    0 4px 8px rgba(0, 0, 0, 0.08);
}

[data-theme="light"] .gradient-mesh-base,
[data-theme="light"] .gradient-mesh-vibrant,
[data-theme="light"] .gradient-mesh-subtle {
  background:
    radial-gradient(
      ellipse 80% 50% at 20% 40%,
      rgba(0, 212, 255, 0.08) 0%,
      transparent 50%
    ),
    radial-gradient(
      ellipse 60% 40% at 80% 20%,
      rgba(168, 85, 247, 0.05) 0%,
      transparent 50%
    ),
    linear-gradient(
      180deg,
      #f6f8fa 0%,
      #ffffff 100%
    );
}

/* ============================================
 * PERFORMANCE CONSIDERATIONS
 * ============================================ */

/* Reduce motion for users who prefer it */
@media (prefers-reduced-motion: reduce) {
  .glass-card,
  .glass-button,
  .glass-input {
    transition: none;
  }

  .glass-card:hover {
    transform: none;
  }

  .glass-button:hover {
    transform: none;
  }
}

/* Fallback for browsers without backdrop-filter */
@supports not (backdrop-filter: blur(1px)) {
  .glass,
  .glass-subtle,
  .glass-solid,
  .glass-ultra,
  .glass-card,
  .glass-modal,
  .glass-sidebar,
  .glass-header {
    background: var(--glass-bg-ultra);
  }
}

/* GPU acceleration hint for smooth animations */
.glass,
.glass-card,
.glass-modal {
  will-change: transform;
  transform: translateZ(0);
}
```

### src/lib/components/GlassPanel.svelte

```svelte
<script lang="ts">
  /**
   * GlassPanel - A reusable glassmorphic container component
   */

  export let variant: 'default' | 'subtle' | 'solid' | 'ultra' = 'default';
  export let interactive: boolean = false;
  export let glow: boolean = false;
  export let padding: 'none' | 'sm' | 'md' | 'lg' = 'md';
  export let radius: 'sm' | 'md' | 'lg' | 'xl' = 'lg';

  const paddingClasses = {
    none: '',
    sm: 'p-3',
    md: 'p-4',
    lg: 'p-6',
  };

  const radiusClasses = {
    sm: 'rounded-lg',
    md: 'rounded-xl',
    lg: 'rounded-2xl',
    xl: 'rounded-3xl',
  };

  const variantClasses = {
    default: 'glass',
    subtle: 'glass-subtle',
    solid: 'glass-solid',
    ultra: 'glass-ultra',
  };
</script>

<div
  class="
    {variantClasses[variant]}
    {paddingClasses[padding]}
    {radiusClasses[radius]}
    {interactive ? 'glass-card-interactive cursor-pointer' : ''}
    {glow ? 'border-glow' : ''}
  "
  class:shadow-lg={variant !== 'subtle'}
  on:click
  on:keypress
  role={interactive ? 'button' : undefined}
  tabindex={interactive ? 0 : undefined}
>
  <slot />
</div>

<style>
  /* Component-specific styles if needed */
</style>
```

---

## Visual Reference

The glassmorphic design should achieve:

1. **Panels that feel like frosted glass** - Content behind is visible but blurred
2. **Subtle light catching on edges** - Borders that glow softly
3. **Vibrant color showing through** - Tachikoma blue and purple gradients visible beneath
4. **Floating, layered depth** - Elements feel lifted off the background
5. **Premium, modern aesthetic** - Clean, minimal, sophisticated

---

## Testing Requirements

### Visual Tests

```typescript
// tests/glass/visual.test.ts
import { describe, it, expect } from 'vitest';

describe('Glassmorphic Design', () => {
  it('should apply backdrop-filter blur', () => {
    // Test that glass class applies correct blur
  });

  it('should have correct border opacity', () => {
    // Test border colors are semi-transparent
  });

  it('should have fallback for unsupported browsers', () => {
    // Test fallback solid backgrounds work
  });

  it('should respect prefers-reduced-motion', () => {
    // Test animations are disabled when requested
  });
});
```

---

## Related Specs

- [191-design-tokens.md](./191-design-tokens.md) - Design tokens (must integrate glass tokens)
- [193-color-system.md](./193-color-system.md) - Color system
- [195-shadows-elevation.md](./195-shadows-elevation.md) - Shadows and elevation
- [201-card-component.md](./201-card-component.md) - Card component (use glass-card)
- [202-modal-component.md](./202-modal-component.md) - Modal component (use glass-modal)
