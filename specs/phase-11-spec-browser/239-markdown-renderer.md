# 239 - Markdown Renderer

**Phase:** 11 - Spec Browser UI
**Spec ID:** 239
**Status:** Planned
**Dependencies:** 236-spec-browser-layout
**Estimated Context:** ~11% of Sonnet window

---

## Objective

Create a markdown renderer component that displays spec files with proper formatting, syntax highlighting for code blocks, interactive checkboxes, and custom spec-specific extensions.

---

## Acceptance Criteria

- [x] Standard markdown rendering (GFM)
- [x] Syntax highlighting for code blocks
- [x] Interactive checkboxes for implementation plans
- [x] Table of contents generation
- [x] Anchor links for headings
- [x] Mermaid diagram support
- [x] Custom spec callouts (Notes, Warnings)

---

## Implementation Details

### 1. Types (src/lib/types/markdown.ts)

```typescript
export interface MarkdownConfig {
  syntaxHighlight: boolean;
  interactiveCheckboxes: boolean;
  generateToc: boolean;
  mermaidDiagrams: boolean;
  customCallouts: boolean;
}

export interface TocItem {
  id: string;
  text: string;
  level: number;
}

export interface CheckboxState {
  lineNumber: number;
  checked: boolean;
}

export const DEFAULT_MARKDOWN_CONFIG: MarkdownConfig = {
  syntaxHighlight: true,
  interactiveCheckboxes: true,
  generateToc: true,
  mermaidDiagrams: true,
  customCallouts: true,
};
```

### 2. Markdown Renderer Component (src/lib/components/spec-browser/MarkdownRenderer.svelte)

```svelte
<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { marked } from 'marked';
  import hljs from 'highlight.js';
  import type { MarkdownConfig, TocItem } from '$lib/types/markdown';
  import { DEFAULT_MARKDOWN_CONFIG } from '$lib/types/markdown';

  export let content: string;
  export let config: MarkdownConfig = DEFAULT_MARKDOWN_CONFIG;

  const dispatch = createEventDispatcher<{
    checkboxChange: { lineNumber: number; checked: boolean };
    linkClick: { href: string };
  }>();

  let renderedHtml = '';
  let toc: TocItem[] = [];
  let containerRef: HTMLElement;

  // Configure marked
  marked.setOptions({
    gfm: true,
    breaks: true,
    highlight: (code, lang) => {
      if (config.syntaxHighlight && lang && hljs.getLanguage(lang)) {
        return hljs.highlight(code, { language: lang }).value;
      }
      return code;
    },
  });

  // Custom renderer
  const renderer = new marked.Renderer();

  // Heading with anchors
  renderer.heading = (text, level) => {
    const id = text.toLowerCase().replace(/[^\w]+/g, '-');
    toc.push({ id, text, level });
    return `<h${level} id="${id}">
      <a class="anchor" href="#${id}">#</a>
      ${text}
    </h${level}>`;
  };

  // Interactive checkboxes
  renderer.listitem = (text) => {
    if (text.startsWith('<input')) {
      return `<li class="task-item">${text}</li>`;
    }
    return `<li>${text}</li>`;
  };

  // Custom callouts
  renderer.blockquote = (quote) => {
    const noteMatch = quote.match(/^<p>\[!NOTE\](.*)/);
    const warnMatch = quote.match(/^<p>\[!WARNING\](.*)/);
    const tipMatch = quote.match(/^<p>\[!TIP\](.*)/);

    if (noteMatch) {
      return `<div class="callout callout--note"><span class="callout-icon">ℹ️</span>${noteMatch[1]}</div>`;
    }
    if (warnMatch) {
      return `<div class="callout callout--warning"><span class="callout-icon">⚠️</span>${warnMatch[1]}</div>`;
    }
    if (tipMatch) {
      return `<div class="callout callout--tip"><span class="callout-icon">💡</span>${tipMatch[1]}</div>`;
    }
    return `<blockquote>${quote}</blockquote>`;
  };

  function render() {
    toc = [];
    marked.use({ renderer });
    renderedHtml = marked.parse(content);
  }

  function handleClick(event: MouseEvent) {
    const target = event.target as HTMLElement;

    // Handle checkbox clicks
    if (target.matches('input[type="checkbox"]')) {
      const checkbox = target as HTMLInputElement;
      const listItem = checkbox.closest('li');
      const lineNumber = parseInt(listItem?.dataset.line || '0');
      dispatch('checkboxChange', { lineNumber, checked: checkbox.checked });
    }

    // Handle link clicks
    if (target.matches('a')) {
      const link = target as HTMLAnchorElement;
      if (link.href.startsWith('#')) {
        event.preventDefault();
        const id = link.href.split('#')[1];
        document.getElementById(id)?.scrollIntoView({ behavior: 'smooth' });
      } else if (!link.href.startsWith('http')) {
        event.preventDefault();
        dispatch('linkClick', { href: link.href });
      }
    }
  }

  $: if (content) render();
</script>

<div
  bind:this={containerRef}
  class="markdown-renderer"
  on:click={handleClick}
>
  {#if config.generateToc && toc.length > 2}
    <nav class="markdown-toc">
      <h4>Contents</h4>
      <ul>
        {#each toc as item}
          <li style="margin-left: {(item.level - 1) * 12}px">
            <a href="#{item.id}">{item.text}</a>
          </li>
        {/each}
      </ul>
    </nav>
  {/if}

  <div class="markdown-content">
    {@html renderedHtml}
  </div>
</div>

<style>
  .markdown-renderer {
    font-size: 15px;
    line-height: 1.7;
    color: var(--color-text-primary);
  }

  .markdown-toc {
    float: right;
    width: 220px;
    margin: 0 0 16px 24px;
    padding: 12px;
    background: var(--color-bg-secondary);
    border-radius: 8px;
    font-size: 13px;
  }

  .markdown-toc h4 {
    margin: 0 0 8px 0;
    font-size: 12px;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .markdown-toc ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .markdown-toc li {
    padding: 4px 0;
  }

  .markdown-toc a {
    color: var(--color-text-secondary);
    text-decoration: none;
  }

  .markdown-toc a:hover {
    color: var(--color-primary);
  }

  .markdown-content :global(h1),
  .markdown-content :global(h2),
  .markdown-content :global(h3) {
    margin-top: 24px;
    margin-bottom: 16px;
    font-weight: 600;
    position: relative;
  }

  .markdown-content :global(h1) { font-size: 28px; }
  .markdown-content :global(h2) { font-size: 22px; }
  .markdown-content :global(h3) { font-size: 18px; }

  .markdown-content :global(.anchor) {
    position: absolute;
    left: -20px;
    color: var(--color-text-muted);
    opacity: 0;
    text-decoration: none;
  }

  .markdown-content :global(h1:hover .anchor),
  .markdown-content :global(h2:hover .anchor),
  .markdown-content :global(h3:hover .anchor) {
    opacity: 1;
  }

  .markdown-content :global(pre) {
    background: var(--color-bg-secondary);
    padding: 16px;
    border-radius: 8px;
    overflow-x: auto;
    font-size: 13px;
  }

  .markdown-content :global(code) {
    font-family: 'SF Mono', monospace;
    font-size: 0.9em;
  }

  .markdown-content :global(:not(pre) > code) {
    background: var(--color-bg-hover);
    padding: 2px 6px;
    border-radius: 4px;
  }

  .markdown-content :global(.task-item) {
    list-style: none;
    margin-left: -20px;
  }

  .markdown-content :global(.task-item input) {
    margin-right: 8px;
    cursor: pointer;
  }

  .markdown-content :global(.callout) {
    padding: 12px 16px;
    border-radius: 8px;
    margin: 16px 0;
    display: flex;
    gap: 12px;
  }

  .markdown-content :global(.callout--note) {
    background: rgba(33, 150, 243, 0.1);
    border-left: 4px solid var(--color-primary);
  }

  .markdown-content :global(.callout--warning) {
    background: rgba(255, 152, 0, 0.1);
    border-left: 4px solid var(--color-warning);
  }

  .markdown-content :global(.callout--tip) {
    background: rgba(76, 175, 80, 0.1);
    border-left: 4px solid var(--color-success);
  }

  .markdown-content :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 16px 0;
  }

  .markdown-content :global(th),
  .markdown-content :global(td) {
    padding: 10px 12px;
    border: 1px solid var(--color-border);
    text-align: left;
  }

  .markdown-content :global(th) {
    background: var(--color-bg-secondary);
    font-weight: 600;
  }
</style>
```

---

## Testing Requirements

1. Standard markdown renders correctly
2. Code syntax highlighting works
3. Interactive checkboxes emit events
4. TOC generates properly
5. Callouts render styled
6. Anchor links scroll

---

## Related Specs

- Depends on: [236-spec-browser-layout.md](236-spec-browser-layout.md)
- Next: [240-spec-editor.md](240-spec-editor.md)
