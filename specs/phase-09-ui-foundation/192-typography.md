# Spec 192: Typography System

## Phase
Phase 9: UI Foundation

## Spec ID
192

## Status
Planned

## Dependencies
- Spec 186: SvelteKit Setup
- Spec 191: Design Tokens

## Estimated Context
~8%

---

## Objective

Implement a comprehensive typography system for Tachikoma with support for variable fonts, responsive scaling, and optimized rendering for both UI elements and code/terminal displays.

---

## Acceptance Criteria

- [x] Font loading and optimization
- [x] Typography scale implementation
- [x] Responsive font sizing
- [x] Monospace font for code/terminal
- [x] Text utility classes
- [x] Heading components
- [x] Line height and letter spacing
- [x] Font feature settings

---

## Implementation Details

### src/lib/styles/typography.css

```css
/**
 * Tachikoma Typography System
 *
 * Uses Inter for UI and JetBrains Mono for code
 */

/* Font Face Definitions */
@font-face {
  font-family: 'Inter';
  font-style: normal;
  font-weight: 100 900;
  font-display: swap;
  src: url('/fonts/Inter-Variable.woff2') format('woff2');
  unicode-range: U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA,
                 U+02DC, U+2000-206F, U+2074, U+20AC, U+2122, U+2191, U+2193,
                 U+2212, U+2215, U+FEFF, U+FFFD;
}

@font-face {
  font-family: 'JetBrains Mono';
  font-style: normal;
  font-weight: 100 800;
  font-display: swap;
  src: url('/fonts/JetBrainsMono-Variable.woff2') format('woff2');
  unicode-range: U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA,
                 U+02DC, U+2000-206F, U+2074, U+20AC, U+2122, U+2191, U+2193,
                 U+2212, U+2215, U+FEFF, U+FFFD;
}

/* Font Stacks */
:root {
  --font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto',
               'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans',
               'Helvetica Neue', sans-serif;

  --font-mono: 'JetBrains Mono', 'Fira Code', 'SF Mono', 'Monaco', 'Inconsolata',
               'Roboto Mono', 'Source Code Pro', 'Consolas', monospace;

  /* Font Feature Settings */
  --font-features-default: "liga" 1, "calt" 1;
  --font-features-mono: "liga" 1, "calt" 1, "ss01" 1, "ss02" 1;
  --font-features-numbers: "tnum" 1, "lnum" 1;
}

/* Base Typography */
body {
  font-family: var(--font-sans);
  font-size: var(--text-base);
  font-weight: var(--font-normal);
  line-height: var(--leading-normal);
  color: var(--color-text-primary);
  font-feature-settings: var(--font-features-default);
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* Heading Styles */
h1, h2, h3, h4, h5, h6 {
  margin: 0;
  font-weight: var(--font-semibold);
  line-height: var(--leading-tight);
  color: var(--color-text-primary);
  letter-spacing: -0.02em;
}

h1 {
  font-size: var(--text-4xl);
  letter-spacing: -0.03em;
}

h2 {
  font-size: var(--text-3xl);
}

h3 {
  font-size: var(--text-2xl);
}

h4 {
  font-size: var(--text-xl);
}

h5 {
  font-size: var(--text-lg);
}

h6 {
  font-size: var(--text-base);
}

/* Paragraph Styles */
p {
  margin: 0;
  line-height: var(--leading-relaxed);
}

/* Link Styles */
a {
  color: var(--color-text-link);
  text-decoration: none;
  transition: color var(--duration-150) var(--ease-out);
}

a:hover {
  color: var(--color-text-link-hover);
  text-decoration: underline;
}

/* Code/Monospace Styles */
code, kbd, pre, samp {
  font-family: var(--font-mono);
  font-feature-settings: var(--font-features-mono);
}

code {
  font-size: 0.875em;
  padding: 0.125em 0.375em;
  background-color: var(--color-bg-elevated);
  border-radius: var(--radius-sm);
  color: var(--color-primary);
}

pre {
  margin: 0;
  padding: var(--space-4);
  background-color: var(--color-code-bg);
  border-radius: var(--radius-md);
  overflow-x: auto;
}

pre code {
  padding: 0;
  background: none;
  font-size: var(--text-sm);
  color: var(--color-code-text);
}

kbd {
  font-size: 0.875em;
  padding: 0.125em 0.5em;
  background-color: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  box-shadow: inset 0 -1px 0 var(--color-border);
}

/* Text Utilities */
.text-xs { font-size: var(--text-xs); }
.text-sm { font-size: var(--text-sm); }
.text-base { font-size: var(--text-base); }
.text-lg { font-size: var(--text-lg); }
.text-xl { font-size: var(--text-xl); }
.text-2xl { font-size: var(--text-2xl); }
.text-3xl { font-size: var(--text-3xl); }
.text-4xl { font-size: var(--text-4xl); }
.text-5xl { font-size: var(--text-5xl); }

.font-thin { font-weight: var(--font-thin); }
.font-extralight { font-weight: var(--font-extralight); }
.font-light { font-weight: var(--font-light); }
.font-normal { font-weight: var(--font-normal); }
.font-medium { font-weight: var(--font-medium); }
.font-semibold { font-weight: var(--font-semibold); }
.font-bold { font-weight: var(--font-bold); }
.font-extrabold { font-weight: var(--font-extrabold); }
.font-black { font-weight: var(--font-black); }

.font-sans { font-family: var(--font-sans); }
.font-mono { font-family: var(--font-mono); }

.leading-none { line-height: var(--leading-none); }
.leading-tight { line-height: var(--leading-tight); }
.leading-snug { line-height: var(--leading-snug); }
.leading-normal { line-height: var(--leading-normal); }
.leading-relaxed { line-height: var(--leading-relaxed); }
.leading-loose { line-height: var(--leading-loose); }

.tracking-tighter { letter-spacing: -0.05em; }
.tracking-tight { letter-spacing: -0.025em; }
.tracking-normal { letter-spacing: 0; }
.tracking-wide { letter-spacing: 0.025em; }
.tracking-wider { letter-spacing: 0.05em; }
.tracking-widest { letter-spacing: 0.1em; }

.text-left { text-align: left; }
.text-center { text-align: center; }
.text-right { text-align: right; }
.text-justify { text-align: justify; }

.text-primary { color: var(--color-text-primary); }
.text-secondary { color: var(--color-text-secondary); }
.text-muted { color: var(--color-text-muted); }
.text-disabled { color: var(--color-text-disabled); }
.text-accent { color: var(--color-primary); }
.text-success { color: var(--color-success); }
.text-warning { color: var(--color-warning); }
.text-error { color: var(--color-error); }

.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.line-clamp-1 {
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.line-clamp-3 {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* Tabular Numbers for Data */
.tabular-nums {
  font-feature-settings: var(--font-features-numbers);
}

/* Selection Styling */
::selection {
  background-color: var(--color-primary-muted);
  color: var(--color-text-primary);
}
```

### src/lib/components/ui/Text.svelte

```svelte
<script lang="ts">
  type TextSize = 'xs' | 'sm' | 'base' | 'lg' | 'xl' | '2xl' | '3xl' | '4xl' | '5xl';
  type TextWeight = 'thin' | 'light' | 'normal' | 'medium' | 'semibold' | 'bold';
  type TextColor = 'primary' | 'secondary' | 'muted' | 'accent' | 'success' | 'warning' | 'error';
  type TextAs = 'p' | 'span' | 'div' | 'label';

  export let as: TextAs = 'span';
  export let size: TextSize = 'base';
  export let weight: TextWeight = 'normal';
  export let color: TextColor = 'primary';
  export let mono: boolean = false;
  export let truncate: boolean = false;
  export let lineClamp: number | null = null;

  $: classes = [
    `text-${size}`,
    `font-${weight}`,
    `text-${color}`,
    mono && 'font-mono',
    truncate && 'truncate',
    lineClamp && `line-clamp-${lineClamp}`
  ].filter(Boolean).join(' ');
</script>

<svelte:element this={as} class={classes} {...$$restProps}>
  <slot />
</svelte:element>
```

### src/lib/components/ui/Heading.svelte

```svelte
<script lang="ts">
  type HeadingLevel = 1 | 2 | 3 | 4 | 5 | 6;
  type HeadingSize = 'xs' | 'sm' | 'base' | 'lg' | 'xl' | '2xl' | '3xl' | '4xl' | '5xl';

  export let level: HeadingLevel = 2;
  export let size: HeadingSize | null = null;
  export let id: string | undefined = undefined;

  const defaultSizes: Record<HeadingLevel, HeadingSize> = {
    1: '4xl',
    2: '3xl',
    3: '2xl',
    4: 'xl',
    5: 'lg',
    6: 'base'
  };

  $: tag = `h${level}` as 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6';
  $: actualSize = size || defaultSizes[level];
</script>

<svelte:element
  this={tag}
  {id}
  class="heading text-{actualSize}"
  {...$$restProps}
>
  <slot />
</svelte:element>

<style>
  .heading {
    font-weight: var(--font-semibold);
    line-height: var(--leading-tight);
    color: var(--color-text-primary);
    letter-spacing: -0.02em;
  }

  .text-4xl,
  .text-5xl {
    letter-spacing: -0.03em;
  }
</style>
```

### src/lib/components/ui/Code.svelte

```svelte
<script lang="ts">
  export let inline: boolean = true;
  export let language: string = '';
</script>

{#if inline}
  <code class="inline-code" {...$$restProps}>
    <slot />
  </code>
{:else}
  <pre class="code-block" data-language={language} {...$$restProps}><code><slot /></code></pre>
{/if}

<style>
  .inline-code {
    font-family: var(--font-mono);
    font-size: 0.875em;
    padding: 0.125em 0.375em;
    background-color: var(--color-bg-elevated);
    border-radius: var(--radius-sm);
    color: var(--color-primary);
    font-feature-settings: var(--font-features-mono);
  }

  .code-block {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    line-height: var(--leading-relaxed);
    padding: var(--space-4);
    background-color: var(--color-code-bg);
    border-radius: var(--radius-md);
    overflow-x: auto;
    color: var(--color-code-text);
    font-feature-settings: var(--font-features-mono);
  }

  .code-block code {
    padding: 0;
    background: none;
    font-size: inherit;
    color: inherit;
  }
</style>
```

### src/lib/components/ui/Label.svelte

```svelte
<script lang="ts">
  export let htmlFor: string | undefined = undefined;
  export let required: boolean = false;
  export let disabled: boolean = false;
  export let error: boolean = false;
</script>

<label
  for={htmlFor}
  class="label"
  class:disabled
  class:error
  {...$$restProps}
>
  <slot />
  {#if required}
    <span class="required" aria-hidden="true">*</span>
  {/if}
</label>

<style>
  .label {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-text-primary);
    cursor: pointer;
    user-select: none;
  }

  .label.disabled {
    color: var(--color-text-disabled);
    cursor: not-allowed;
  }

  .label.error {
    color: var(--color-error);
  }

  .required {
    color: var(--color-error);
    font-weight: var(--font-bold);
  }
</style>
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/typography/components.test.ts
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import Text from '@components/ui/Text.svelte';
import Heading from '@components/ui/Heading.svelte';

describe('Text Component', () => {
  it('should render with default props', () => {
    const { container } = render(Text, { props: {}, slots: { default: 'Hello' } });
    expect(container.querySelector('span')).toBeTruthy();
  });

  it('should apply size class', () => {
    const { container } = render(Text, { props: { size: 'lg' } });
    expect(container.querySelector('.text-lg')).toBeTruthy();
  });

  it('should render as different elements', () => {
    const { container } = render(Text, { props: { as: 'p' } });
    expect(container.querySelector('p')).toBeTruthy();
  });
});

describe('Heading Component', () => {
  it('should render correct heading level', () => {
    const { container } = render(Heading, { props: { level: 1 } });
    expect(container.querySelector('h1')).toBeTruthy();
  });

  it('should use default size for level', () => {
    const { container } = render(Heading, { props: { level: 2 } });
    expect(container.querySelector('.text-3xl')).toBeTruthy();
  });

  it('should allow size override', () => {
    const { container } = render(Heading, { props: { level: 2, size: 'xl' } });
    expect(container.querySelector('.text-xl')).toBeTruthy();
  });
});
```

---

## Related Specs

- [191-design-tokens.md](./191-design-tokens.md) - Design tokens
- [193-color-system.md](./193-color-system.md) - Color system
- [196-component-library.md](./196-component-library.md) - Component library
