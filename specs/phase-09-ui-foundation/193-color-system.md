# Spec 193: Color System

## Phase
Phase 9: UI Foundation

## Spec ID
193

## Status
Planned

## Dependencies
- Spec 186: SvelteKit Setup
- Spec 191: Design Tokens

## Estimated Context
~8%

---

## Objective

Implement a comprehensive color system for Tachikoma featuring the signature blue theme, semantic color mappings, accessibility-compliant contrast ratios, and runtime theme customization capabilities.

---

## Acceptance Criteria

- [x] Primary Tachikoma blue palette defined
- [x] Semantic color mappings (success, warning, error, info)
- [x] Background and surface color layers
- [x] Text color hierarchy
- [x] WCAG 2.1 AA contrast compliance
- [x] Theme switching system
- [x] Color utility functions
- [x] Accessibility color blindness considerations

---

## Implementation Details

### src/lib/styles/colors.css

```css
/**
 * Tachikoma Color System
 *
 * A comprehensive color palette with semantic mappings
 * and accessibility-compliant contrast ratios.
 */

:root {
  /* ============================================
   * TACHIKOMA BLUE - PRIMARY BRAND COLOR
   * Base: #00d4ff (HSL: 190, 100%, 50%)
   * ============================================ */

  --tachikoma-50: #e6faff;
  --tachikoma-100: #b3f0ff;
  --tachikoma-200: #80e6ff;
  --tachikoma-300: #4ddbff;
  --tachikoma-400: #1ad1ff;
  --tachikoma-500: #00d4ff;  /* Primary */
  --tachikoma-600: #00a8cc;
  --tachikoma-700: #007d99;
  --tachikoma-800: #005266;
  --tachikoma-900: #002633;
  --tachikoma-950: #001a22;

  /* Glow effects */
  --tachikoma-glow: rgba(0, 212, 255, 0.5);
  --tachikoma-glow-subtle: rgba(0, 212, 255, 0.2);

  /* ============================================
   * SEMANTIC COLOR PALETTES
   * ============================================ */

  /* Success - Green */
  --success-50: #f0fdf4;
  --success-100: #dcfce7;
  --success-200: #bbf7d0;
  --success-300: #86efac;
  --success-400: #4ade80;
  --success-500: #22c55e;
  --success-600: #16a34a;
  --success-700: #15803d;
  --success-800: #166534;
  --success-900: #14532d;

  /* Warning - Amber */
  --warning-50: #fffbeb;
  --warning-100: #fef3c7;
  --warning-200: #fde68a;
  --warning-300: #fcd34d;
  --warning-400: #fbbf24;
  --warning-500: #f59e0b;
  --warning-600: #d97706;
  --warning-700: #b45309;
  --warning-800: #92400e;
  --warning-900: #78350f;

  /* Error - Red */
  --error-50: #fef2f2;
  --error-100: #fee2e2;
  --error-200: #fecaca;
  --error-300: #fca5a5;
  --error-400: #f87171;
  --error-500: #ef4444;
  --error-600: #dc2626;
  --error-700: #b91c1c;
  --error-800: #991b1b;
  --error-900: #7f1d1d;

  /* Info - Blue (distinct from Tachikoma blue) */
  --info-50: #eff6ff;
  --info-100: #dbeafe;
  --info-200: #bfdbfe;
  --info-300: #93c5fd;
  --info-400: #60a5fa;
  --info-500: #3b82f6;
  --info-600: #2563eb;
  --info-700: #1d4ed8;
  --info-800: #1e40af;
  --info-900: #1e3a8a;

  /* Purple - For special highlights */
  --purple-50: #faf5ff;
  --purple-100: #f3e8ff;
  --purple-200: #e9d5ff;
  --purple-300: #d8b4fe;
  --purple-400: #c084fc;
  --purple-500: #a855f7;
  --purple-600: #9333ea;
  --purple-700: #7e22ce;
  --purple-800: #6b21a8;
  --purple-900: #581c87;
}

/* ============================================
 * DARK THEME - Default for Tachikoma
 * ============================================ */

[data-theme="dark"],
:root {
  /* Background Layers (darkest to lightest) */
  --color-bg-canvas: #010409;       /* Deepest background */
  --color-bg-base: #0a0e14;         /* Main app background */
  --color-bg-surface: #0d1117;      /* Cards, panels */
  --color-bg-elevated: #161b22;     /* Elevated surfaces */
  --color-bg-overlay: #1c2128;      /* Overlays, dropdowns */
  --color-bg-muted: #21262d;        /* Muted backgrounds */
  --color-bg-subtle: #30363d;       /* Subtle backgrounds */

  /* Interactive Backgrounds */
  --color-bg-hover: rgba(255, 255, 255, 0.04);
  --color-bg-active: rgba(255, 255, 255, 0.06);
  --color-bg-selected: rgba(0, 212, 255, 0.12);
  --color-bg-selected-hover: rgba(0, 212, 255, 0.18);

  /* Text Colors */
  --color-fg-default: #e6edf3;      /* Primary text */
  --color-fg-muted: #8b949e;        /* Secondary text */
  --color-fg-subtle: #6e7681;       /* Tertiary text */
  --color-fg-disabled: #484f58;     /* Disabled text */
  --color-fg-onEmphasis: #ffffff;   /* Text on emphasis bg */

  /* Border Colors */
  --color-border-default: #30363d;
  --color-border-muted: #21262d;
  --color-border-subtle: #161b22;
  --color-border-emphasis: var(--tachikoma-500);

  /* Accent/Brand */
  --color-accent-fg: var(--tachikoma-500);
  --color-accent-emphasis: var(--tachikoma-600);
  --color-accent-muted: rgba(0, 212, 255, 0.3);
  --color-accent-subtle: rgba(0, 212, 255, 0.1);

  /* Status Colors */
  --color-success-fg: var(--success-500);
  --color-success-emphasis: var(--success-600);
  --color-success-muted: rgba(34, 197, 94, 0.3);
  --color-success-subtle: rgba(34, 197, 94, 0.1);

  --color-warning-fg: var(--warning-500);
  --color-warning-emphasis: var(--warning-600);
  --color-warning-muted: rgba(245, 158, 11, 0.3);
  --color-warning-subtle: rgba(245, 158, 11, 0.1);

  --color-error-fg: var(--error-500);
  --color-error-emphasis: var(--error-600);
  --color-error-muted: rgba(239, 68, 68, 0.3);
  --color-error-subtle: rgba(239, 68, 68, 0.1);

  --color-info-fg: var(--info-500);
  --color-info-emphasis: var(--info-600);
  --color-info-muted: rgba(59, 130, 246, 0.3);
  --color-info-subtle: rgba(59, 130, 246, 0.1);

  /* Syntax Highlighting */
  --color-syntax-keyword: #ff7b72;
  --color-syntax-string: #a5d6ff;
  --color-syntax-number: #79c0ff;
  --color-syntax-function: #d2a8ff;
  --color-syntax-comment: #8b949e;
  --color-syntax-operator: #ff7b72;
  --color-syntax-variable: #ffa657;
  --color-syntax-constant: #79c0ff;
  --color-syntax-class: #7ee787;

  /* Security/Severity Colors */
  --color-severity-critical: #ff4757;
  --color-severity-high: #ff6b6b;
  --color-severity-medium: #ffa502;
  --color-severity-low: #2ed573;
  --color-severity-info: var(--tachikoma-500);
}

/* ============================================
 * LIGHT THEME
 * ============================================ */

[data-theme="light"] {
  /* Background Layers */
  --color-bg-canvas: #f6f8fa;
  --color-bg-base: #ffffff;
  --color-bg-surface: #ffffff;
  --color-bg-elevated: #ffffff;
  --color-bg-overlay: #ffffff;
  --color-bg-muted: #f6f8fa;
  --color-bg-subtle: #eaeef2;

  /* Interactive Backgrounds */
  --color-bg-hover: rgba(0, 0, 0, 0.03);
  --color-bg-active: rgba(0, 0, 0, 0.05);
  --color-bg-selected: rgba(0, 168, 204, 0.08);
  --color-bg-selected-hover: rgba(0, 168, 204, 0.12);

  /* Text Colors */
  --color-fg-default: #1f2328;
  --color-fg-muted: #57606a;
  --color-fg-subtle: #6e7781;
  --color-fg-disabled: #8c959f;
  --color-fg-onEmphasis: #ffffff;

  /* Border Colors */
  --color-border-default: #d0d7de;
  --color-border-muted: #d8dee4;
  --color-border-subtle: #eaeef2;
  --color-border-emphasis: var(--tachikoma-600);

  /* Accent/Brand (darker for light theme) */
  --color-accent-fg: var(--tachikoma-700);
  --color-accent-emphasis: var(--tachikoma-600);
  --color-accent-muted: rgba(0, 168, 204, 0.3);
  --color-accent-subtle: rgba(0, 168, 204, 0.08);

  /* Status Colors (darker variants) */
  --color-success-fg: var(--success-600);
  --color-success-emphasis: var(--success-500);
  --color-success-muted: rgba(22, 163, 74, 0.2);
  --color-success-subtle: rgba(22, 163, 74, 0.08);

  --color-warning-fg: var(--warning-700);
  --color-warning-emphasis: var(--warning-600);
  --color-warning-muted: rgba(217, 119, 6, 0.2);
  --color-warning-subtle: rgba(217, 119, 6, 0.08);

  --color-error-fg: var(--error-600);
  --color-error-emphasis: var(--error-500);
  --color-error-muted: rgba(220, 38, 38, 0.2);
  --color-error-subtle: rgba(220, 38, 38, 0.08);

  /* Syntax Highlighting (light theme) */
  --color-syntax-keyword: #cf222e;
  --color-syntax-string: #0a3069;
  --color-syntax-number: #0550ae;
  --color-syntax-function: #8250df;
  --color-syntax-comment: #6e7781;
  --color-syntax-operator: #cf222e;
  --color-syntax-variable: #953800;
  --color-syntax-constant: #0550ae;
  --color-syntax-class: #116329;
}
```

### src/lib/utils/colors.ts

```typescript
/**
 * Color utility functions for Tachikoma
 */

export interface RGB {
  r: number;
  g: number;
  b: number;
}

export interface HSL {
  h: number;
  s: number;
  l: number;
}

/**
 * Parse hex color to RGB
 */
export function hexToRgb(hex: string): RGB {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (!result) throw new Error(`Invalid hex color: ${hex}`);

  return {
    r: parseInt(result[1], 16),
    g: parseInt(result[2], 16),
    b: parseInt(result[3], 16)
  };
}

/**
 * Convert RGB to hex
 */
export function rgbToHex(rgb: RGB): string {
  const toHex = (n: number) => n.toString(16).padStart(2, '0');
  return `#${toHex(rgb.r)}${toHex(rgb.g)}${toHex(rgb.b)}`;
}

/**
 * Convert RGB to HSL
 */
export function rgbToHsl(rgb: RGB): HSL {
  const r = rgb.r / 255;
  const g = rgb.g / 255;
  const b = rgb.b / 255;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  let h = 0;
  let s = 0;
  const l = (max + min) / 2;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

    switch (max) {
      case r: h = ((g - b) / d + (g < b ? 6 : 0)) / 6; break;
      case g: h = ((b - r) / d + 2) / 6; break;
      case b: h = ((r - g) / d + 4) / 6; break;
    }
  }

  return {
    h: Math.round(h * 360),
    s: Math.round(s * 100),
    l: Math.round(l * 100)
  };
}

/**
 * Calculate relative luminance for WCAG contrast
 */
export function getLuminance(rgb: RGB): number {
  const [r, g, b] = [rgb.r, rgb.g, rgb.b].map(v => {
    v /= 255;
    return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/**
 * Calculate WCAG contrast ratio between two colors
 */
export function getContrastRatio(color1: string, color2: string): number {
  const l1 = getLuminance(hexToRgb(color1));
  const l2 = getLuminance(hexToRgb(color2));
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  return (lighter + 0.05) / (darker + 0.05);
}

/**
 * Check if contrast meets WCAG AA standard
 * Normal text: 4.5:1, Large text: 3:1
 */
export function meetsWcagAA(
  foreground: string,
  background: string,
  isLargeText: boolean = false
): boolean {
  const ratio = getContrastRatio(foreground, background);
  return isLargeText ? ratio >= 3 : ratio >= 4.5;
}

/**
 * Check if contrast meets WCAG AAA standard
 * Normal text: 7:1, Large text: 4.5:1
 */
export function meetsWcagAAA(
  foreground: string,
  background: string,
  isLargeText: boolean = false
): boolean {
  const ratio = getContrastRatio(foreground, background);
  return isLargeText ? ratio >= 4.5 : ratio >= 7;
}

/**
 * Generate color with alpha channel
 */
export function withAlpha(color: string, alpha: number): string {
  const rgb = hexToRgb(color);
  return `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alpha})`;
}

/**
 * Lighten a color
 */
export function lighten(color: string, amount: number): string {
  const rgb = hexToRgb(color);
  const hsl = rgbToHsl(rgb);
  hsl.l = Math.min(100, hsl.l + amount);
  return hslToHex(hsl);
}

/**
 * Darken a color
 */
export function darken(color: string, amount: number): string {
  const rgb = hexToRgb(color);
  const hsl = rgbToHsl(rgb);
  hsl.l = Math.max(0, hsl.l - amount);
  return hslToHex(hsl);
}

/**
 * Convert HSL to hex
 */
export function hslToHex(hsl: HSL): string {
  const h = hsl.h / 360;
  const s = hsl.s / 100;
  const l = hsl.l / 100;

  const hue2rgb = (p: number, q: number, t: number) => {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1/6) return p + (q - p) * 6 * t;
    if (t < 1/2) return q;
    if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
    return p;
  };

  let r, g, b;
  if (s === 0) {
    r = g = b = l;
  } else {
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r = hue2rgb(p, q, h + 1/3);
    g = hue2rgb(p, q, h);
    b = hue2rgb(p, q, h - 1/3);
  }

  return rgbToHex({
    r: Math.round(r * 255),
    g: Math.round(g * 255),
    b: Math.round(b * 255)
  });
}

// Tachikoma blue constants
export const TACHIKOMA_BLUE = '#00d4ff';
export const TACHIKOMA_BLUE_DARK = '#00a8cc';
export const TACHIKOMA_BLUE_LIGHT = '#4ddbff';
```

### src/lib/stores/theme.ts

```typescript
import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';
import { createPersistedStore } from './persistedStore';

export type Theme = 'dark' | 'light' | 'system';
export type ResolvedTheme = 'dark' | 'light';

interface ThemeState {
  theme: Theme;
  resolved: ResolvedTheme;
}

function getSystemTheme(): ResolvedTheme {
  if (!browser) return 'dark';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function resolveTheme(theme: Theme): ResolvedTheme {
  if (theme === 'system') {
    return getSystemTheme();
  }
  return theme;
}

function createThemeStore() {
  const stored = createPersistedStore<Theme>('dark', { key: 'theme' });
  let currentTheme: Theme = 'dark';

  stored.subscribe(t => { currentTheme = t; });

  const state = writable<ThemeState>({
    theme: currentTheme,
    resolved: resolveTheme(currentTheme)
  });

  // Sync persisted store with state
  stored.subscribe(theme => {
    state.update(s => ({
      ...s,
      theme,
      resolved: resolveTheme(theme)
    }));
  });

  // Watch for system theme changes
  if (browser) {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaQuery.addEventListener('change', () => {
      state.update(s => {
        if (s.theme === 'system') {
          return { ...s, resolved: getSystemTheme() };
        }
        return s;
      });
    });
  }

  // Apply theme to document
  if (browser) {
    state.subscribe(({ resolved }) => {
      document.documentElement.setAttribute('data-theme', resolved);
    });
  }

  return {
    subscribe: state.subscribe,

    setTheme: (theme: Theme) => {
      stored.set(theme);
    },

    toggle: () => {
      state.update(s => {
        const newTheme: Theme = s.resolved === 'dark' ? 'light' : 'dark';
        stored.set(newTheme);
        return {
          theme: newTheme,
          resolved: newTheme
        };
      });
    }
  };
}

export const themeStore = createThemeStore();
export const currentTheme = derived(themeStore, $theme => $theme.theme);
export const resolvedTheme = derived(themeStore, $theme => $theme.resolved);
export const isDarkMode = derived(themeStore, $theme => $theme.resolved === 'dark');
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/colors/utils.test.ts
import { describe, it, expect } from 'vitest';
import {
  hexToRgb,
  rgbToHex,
  getContrastRatio,
  meetsWcagAA,
  withAlpha,
  TACHIKOMA_BLUE
} from '@utils/colors';

describe('Color Utilities', () => {
  it('should convert hex to RGB', () => {
    expect(hexToRgb('#00d4ff')).toEqual({ r: 0, g: 212, b: 255 });
  });

  it('should convert RGB to hex', () => {
    expect(rgbToHex({ r: 0, g: 212, b: 255 })).toBe('#00d4ff');
  });

  it('should calculate contrast ratio', () => {
    const ratio = getContrastRatio('#ffffff', '#000000');
    expect(ratio).toBeCloseTo(21, 0);
  });

  it('should check WCAG AA compliance', () => {
    expect(meetsWcagAA('#000000', '#ffffff')).toBe(true);
    expect(meetsWcagAA('#777777', '#ffffff')).toBe(false);
  });

  it('should generate color with alpha', () => {
    expect(withAlpha('#00d4ff', 0.5)).toBe('rgba(0, 212, 255, 0.5)');
  });

  it('should have correct Tachikoma blue', () => {
    expect(TACHIKOMA_BLUE).toBe('#00d4ff');
  });
});
```

---

## Related Specs

- [191-design-tokens.md](./191-design-tokens.md) - Design tokens
- [192-typography.md](./192-typography.md) - Typography system
- [195-shadows-elevation.md](./195-shadows-elevation.md) - Shadows and elevation
