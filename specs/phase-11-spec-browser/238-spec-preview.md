# Spec 238: Markdown Preview Component

## Phase
11 - Spec Browser UI

## Spec ID
238

## Status
Planned

## Dependencies
- Phase 10 (Core UI Components)
- Spec 236 (Spec Detail View)
- Spec 237 (Spec Editor)

## Estimated Context
~9%

---

## Objective

Create a high-quality markdown preview component with syntax highlighting for code blocks, support for spec-specific syntax extensions, clickable spec references, and proper rendering of tables, checklists, and diagrams.

---

## Acceptance Criteria

- [ ] Render standard markdown syntax correctly
- [ ] Syntax highlighting for code blocks (20+ languages)
- [ ] Render tables with styling
- [ ] Render checkbox lists interactively
- [ ] Auto-link spec references (e.g., "Spec 231")
- [ ] Support mermaid diagrams
- [ ] Copy button for code blocks
- [ ] Responsive images with lightbox
- [ ] Accessible heading anchors
- [ ] Print-friendly styles

---

## Implementation Details

### MarkdownPreview.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { marked } from 'marked';
  import hljs from 'highlight.js';
  import mermaid from 'mermaid';
  import DOMPurify from 'dompurify';
  import type { Spec } from '$lib/types/spec';
  import Icon from '$lib/components/Icon.svelte';
  import Lightbox from '$lib/components/Lightbox.svelte';

  export let content: string = '';
  export let specReferences = true;
  export let interactiveCheckboxes = false;
  export let showCopyButtons = true;
  export let theme: 'light' | 'dark' = 'dark';

  const dispatch = createEventDispatcher<{
    specClick: { specId: string };
    checkboxToggle: { index: number; checked: boolean };
    linkClick: { href: string };
  }>();

  let containerRef: HTMLElement;
  let renderedHtml = '';
  let lightboxImage: string | null = null;
  let checkboxIndex = 0;

  // Configure marked
  const renderer = new marked.Renderer();

  // Custom heading renderer with anchors
  renderer.heading = (text: string, level: number) => {
    const slug = text.toLowerCase().replace(/[^\w]+/g, '-');
    return `
      <h${level} id="${slug}" class="md-heading">
        <a href="#${slug}" class="md-heading__anchor" aria-hidden="true">
          <svg width="16" height="16" viewBox="0 0 16 16">
            <path fill="currentColor" d="M4 9h1v1H4c-1.5 0-3-1.69-3-3.5S2.55 3 4 3h4c1.45 0 3 1.69 3 3.5 0 1.41-.91 2.72-2 3.25V8.59c.58-.45 1-1.27 1-2.09C10 5.22 8.98 4 8 4H4c-.98 0-2 1.22-2 2.5S3 9 4 9zm9-3h-1v1h1c1 0 2 1.22 2 2.5S13.98 12 13 12H9c-.98 0-2-1.22-2-2.5 0-.83.42-1.64 1-2.09V6.25c-1.09.53-2 1.84-2 3.25C6 11.31 7.55 13 9 13h4c1.45 0 3-1.69 3-3.5S14.5 6 13 6z"></path>
          </svg>
        </a>
        ${text}
      </h${level}>
    `;
  };

  // Custom code block renderer with syntax highlighting
  renderer.code = (code: string, language: string | undefined) => {
    const lang = language || 'plaintext';
    const validLang = hljs.getLanguage(lang) ? lang : 'plaintext';

    // Handle mermaid diagrams
    if (lang === 'mermaid') {
      const id = `mermaid-${Math.random().toString(36).substr(2, 9)}`;
      return `<div class="md-mermaid" id="${id}">${code}</div>`;
    }

    const highlighted = hljs.highlight(code, { language: validLang }).value;
    const copyId = `code-${Math.random().toString(36).substr(2, 9)}`;

    return `
      <div class="md-code-block">
        <div class="md-code-block__header">
          <span class="md-code-block__lang">${validLang}</span>
          ${showCopyButtons ? `
            <button
              class="md-code-block__copy"
              data-copy-id="${copyId}"
              aria-label="Copy code"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
              </svg>
              <span>Copy</span>
            </button>
          ` : ''}
        </div>
        <pre><code id="${copyId}" class="hljs language-${validLang}">${highlighted}</code></pre>
      </div>
    `;
  };

  // Custom checkbox renderer
  renderer.listitem = (text: string) => {
    const checkboxMatch = text.match(/^\s*<input\s+type="checkbox"([^>]*)>\s*/);
    if (checkboxMatch) {
      const isChecked = checkboxMatch[1].includes('checked');
      const content = text.replace(checkboxMatch[0], '');
      const idx = checkboxIndex++;

      if (interactiveCheckboxes) {
        return `
          <li class="md-checkbox-item">
            <button
              class="md-checkbox"
              data-checkbox-index="${idx}"
              aria-checked="${isChecked}"
              role="checkbox"
            >
              ${isChecked
                ? '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 11l3 3L22 4"></path><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"></path></svg>'
                : '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect></svg>'
              }
            </button>
            <span class="${isChecked ? 'md-checkbox-item--checked' : ''}">${content}</span>
          </li>
        `;
      }

      return `
        <li class="md-checkbox-item md-checkbox-item--readonly">
          <span class="md-checkbox md-checkbox--${isChecked ? 'checked' : 'unchecked'}">
            ${isChecked ? '☑' : '☐'}
          </span>
          <span class="${isChecked ? 'md-checkbox-item--checked' : ''}">${content}</span>
        </li>
      `;
    }

    return `<li>${text}</li>`;
  };

  // Custom table renderer
  renderer.table = (header: string, body: string) => {
    return `
      <div class="md-table-wrapper">
        <table class="md-table">
          <thead>${header}</thead>
          <tbody>${body}</tbody>
        </table>
      </div>
    `;
  };

  // Custom image renderer with lightbox support
  renderer.image = (href: string, title: string | null, text: string) => {
    const titleAttr = title ? `title="${title}"` : '';
    return `
      <figure class="md-figure">
        <img
          src="${href}"
          alt="${text}"
          ${titleAttr}
          class="md-image"
          loading="lazy"
          data-lightbox
        />
        ${title ? `<figcaption>${title}</figcaption>` : ''}
      </figure>
    `;
  };

  // Custom link renderer for spec references
  renderer.link = (href: string, title: string | null, text: string) => {
    const titleAttr = title ? `title="${title}"` : '';
    const isExternal = href.startsWith('http://') || href.startsWith('https://');

    return `
      <a
        href="${href}"
        ${titleAttr}
        class="md-link ${isExternal ? 'md-link--external' : ''}"
        ${isExternal ? 'target="_blank" rel="noopener noreferrer"' : ''}
      >
        ${text}
        ${isExternal ? '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>' : ''}
      </a>
    `;
  };

  // Configure marked options
  marked.setOptions({
    renderer,
    gfm: true,
    breaks: true,
    smartLists: true,
    smartypants: true
  });

  // Process spec references
  function processSpecReferences(html: string): string {
    if (!specReferences) return html;

    // Match "Spec NNN" or "spec NNN" patterns
    return html.replace(
      /\b(Spec|spec)\s+(\d{1,4})\b/g,
      '<a href="#" class="md-spec-ref" data-spec-id="$2">$1 $2</a>'
    );
  }

  // Render markdown
  $: {
    checkboxIndex = 0;
    const rawHtml = marked(content) as string;
    const withSpecRefs = processSpecReferences(rawHtml);
    renderedHtml = DOMPurify.sanitize(withSpecRefs, {
      ADD_ATTR: ['data-spec-id', 'data-checkbox-index', 'data-copy-id', 'data-lightbox'],
      ADD_TAGS: ['button']
    });
  }

  // Handle click events
  function handleClick(event: MouseEvent) {
    const target = event.target as HTMLElement;

    // Handle spec reference clicks
    const specRef = target.closest('.md-spec-ref');
    if (specRef) {
      event.preventDefault();
      const specId = specRef.getAttribute('data-spec-id');
      if (specId) {
        dispatch('specClick', { specId });
      }
      return;
    }

    // Handle checkbox clicks
    const checkbox = target.closest('.md-checkbox');
    if (checkbox && interactiveCheckboxes) {
      const index = parseInt(checkbox.getAttribute('data-checkbox-index') ?? '0', 10);
      const isChecked = checkbox.getAttribute('aria-checked') === 'true';
      dispatch('checkboxToggle', { index, checked: !isChecked });
      return;
    }

    // Handle copy button clicks
    const copyBtn = target.closest('.md-code-block__copy');
    if (copyBtn) {
      const copyId = copyBtn.getAttribute('data-copy-id');
      const codeEl = document.getElementById(copyId ?? '');
      if (codeEl) {
        navigator.clipboard.writeText(codeEl.textContent ?? '');
        const span = copyBtn.querySelector('span');
        if (span) {
          span.textContent = 'Copied!';
          setTimeout(() => span.textContent = 'Copy', 2000);
        }
      }
      return;
    }

    // Handle image lightbox
    const img = target.closest('.md-image');
    if (img) {
      lightboxImage = (img as HTMLImageElement).src;
      return;
    }

    // Handle external links
    const link = target.closest('.md-link');
    if (link) {
      const href = link.getAttribute('href');
      if (href) {
        dispatch('linkClick', { href });
      }
    }
  }

  // Initialize mermaid diagrams
  onMount(() => {
    mermaid.initialize({
      startOnLoad: false,
      theme: theme === 'dark' ? 'dark' : 'default',
      securityLevel: 'strict'
    });

    // Render mermaid diagrams after content update
    const observer = new MutationObserver(() => {
      const mermaidDivs = containerRef.querySelectorAll('.md-mermaid');
      mermaidDivs.forEach(async (div) => {
        const code = div.textContent ?? '';
        try {
          const { svg } = await mermaid.render(div.id + '-svg', code);
          div.innerHTML = svg;
        } catch (e) {
          div.innerHTML = `<div class="md-mermaid-error">Diagram error: ${e}</div>`;
        }
      });
    });

    observer.observe(containerRef, { childList: true, subtree: true });

    return () => observer.disconnect();
  });
</script>

<div
  bind:this={containerRef}
  class="markdown-preview markdown-preview--{theme}"
  on:click={handleClick}
>
  {@html renderedHtml}
</div>

{#if lightboxImage}
  <Lightbox
    src={lightboxImage}
    on:close={() => lightboxImage = null}
  />
{/if}

<style>
  .markdown-preview {
    font-size: 1rem;
    line-height: 1.7;
    color: var(--color-text-primary);
  }

  /* Headings */
  .markdown-preview :global(.md-heading) {
    position: relative;
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    font-weight: 600;
    line-height: 1.3;
  }

  .markdown-preview :global(.md-heading__anchor) {
    position: absolute;
    left: -1.5em;
    padding-right: 0.5em;
    opacity: 0;
    color: var(--color-text-tertiary);
    text-decoration: none;
    transition: opacity 0.2s;
  }

  .markdown-preview :global(.md-heading:hover .md-heading__anchor) {
    opacity: 1;
  }

  .markdown-preview :global(h1) { font-size: 1.75em; }
  .markdown-preview :global(h2) { font-size: 1.5em; }
  .markdown-preview :global(h3) { font-size: 1.25em; }
  .markdown-preview :global(h4) { font-size: 1.1em; }

  /* Paragraphs */
  .markdown-preview :global(p) {
    margin: 0 0 1em;
  }

  /* Code blocks */
  .markdown-preview :global(.md-code-block) {
    margin: 1em 0;
    border-radius: 8px;
    overflow: hidden;
    background: var(--color-code-bg);
    border: 1px solid var(--color-border);
  }

  .markdown-preview :global(.md-code-block__header) {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: var(--color-surface-subtle);
    border-bottom: 1px solid var(--color-border);
  }

  .markdown-preview :global(.md-code-block__lang) {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
  }

  .markdown-preview :global(.md-code-block__copy) {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    font-size: 0.75rem;
    background: none;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    color: var(--color-text-tertiary);
    transition: all 0.15s;
  }

  .markdown-preview :global(.md-code-block__copy:hover) {
    background: var(--color-hover);
    color: var(--color-text-primary);
  }

  .markdown-preview :global(.md-code-block pre) {
    margin: 0;
    padding: 16px;
    overflow-x: auto;
  }

  .markdown-preview :global(.md-code-block code) {
    font-family: var(--font-mono);
    font-size: 0.875rem;
    line-height: 1.6;
  }

  /* Inline code */
  .markdown-preview :global(code:not(.hljs)) {
    padding: 2px 6px;
    font-family: var(--font-mono);
    font-size: 0.875em;
    background: var(--color-code-inline-bg);
    border-radius: 4px;
  }

  /* Tables */
  .markdown-preview :global(.md-table-wrapper) {
    margin: 1em 0;
    overflow-x: auto;
  }

  .markdown-preview :global(.md-table) {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
  }

  .markdown-preview :global(.md-table th),
  .markdown-preview :global(.md-table td) {
    padding: 10px 14px;
    border: 1px solid var(--color-border);
    text-align: left;
  }

  .markdown-preview :global(.md-table th) {
    background: var(--color-surface-subtle);
    font-weight: 600;
  }

  .markdown-preview :global(.md-table tr:nth-child(even)) {
    background: var(--color-surface-subtle);
  }

  /* Checkboxes */
  .markdown-preview :global(.md-checkbox-item) {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    list-style: none;
    margin: 0.5em 0;
  }

  .markdown-preview :global(.md-checkbox) {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-tertiary);
    flex-shrink: 0;
  }

  .markdown-preview :global(.md-checkbox:hover) {
    color: var(--color-primary);
  }

  .markdown-preview :global(.md-checkbox[aria-checked="true"]) {
    color: var(--color-success);
  }

  .markdown-preview :global(.md-checkbox-item--checked) {
    color: var(--color-text-tertiary);
    text-decoration: line-through;
  }

  /* Links */
  .markdown-preview :global(.md-link) {
    color: var(--color-primary);
    text-decoration: none;
    transition: color 0.15s;
  }

  .markdown-preview :global(.md-link:hover) {
    text-decoration: underline;
  }

  .markdown-preview :global(.md-link--external svg) {
    display: inline-block;
    margin-left: 2px;
    vertical-align: middle;
  }

  /* Spec references */
  .markdown-preview :global(.md-spec-ref) {
    padding: 2px 6px;
    font-family: var(--font-mono);
    font-size: 0.875em;
    font-weight: 500;
    background: var(--color-primary-subtle);
    color: var(--color-primary);
    border-radius: 4px;
    text-decoration: none;
    cursor: pointer;
  }

  .markdown-preview :global(.md-spec-ref:hover) {
    background: var(--color-primary);
    color: white;
  }

  /* Images */
  .markdown-preview :global(.md-figure) {
    margin: 1.5em 0;
    text-align: center;
  }

  .markdown-preview :global(.md-image) {
    max-width: 100%;
    height: auto;
    border-radius: 8px;
    cursor: zoom-in;
  }

  .markdown-preview :global(.md-figure figcaption) {
    margin-top: 8px;
    font-size: 0.875rem;
    color: var(--color-text-tertiary);
    font-style: italic;
  }

  /* Blockquotes */
  .markdown-preview :global(blockquote) {
    margin: 1em 0;
    padding: 0.5em 1em;
    border-left: 4px solid var(--color-primary);
    background: var(--color-surface-subtle);
    color: var(--color-text-secondary);
  }

  /* Lists */
  .markdown-preview :global(ul),
  .markdown-preview :global(ol) {
    margin: 1em 0;
    padding-left: 1.5em;
  }

  .markdown-preview :global(li) {
    margin: 0.25em 0;
  }

  /* Horizontal rules */
  .markdown-preview :global(hr) {
    margin: 2em 0;
    border: none;
    border-top: 1px solid var(--color-border);
  }

  /* Mermaid diagrams */
  .markdown-preview :global(.md-mermaid) {
    margin: 1.5em 0;
    text-align: center;
  }

  .markdown-preview :global(.md-mermaid-error) {
    padding: 1em;
    background: var(--color-danger-subtle);
    color: var(--color-danger);
    border-radius: 4px;
    font-size: 0.875rem;
  }

  /* Print styles */
  @media print {
    .markdown-preview :global(.md-code-block__copy) {
      display: none;
    }

    .markdown-preview :global(.md-heading__anchor) {
      display: none;
    }
  }
</style>
```

### Lightbox.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, scale } from 'svelte/transition';

  export let src: string;
  export let alt: string = '';

  const dispatch = createEventDispatcher<{ close: void }>();

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      dispatch('close');
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      dispatch('close');
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div
  class="lightbox"
  on:click={handleBackdropClick}
  transition:fade={{ duration: 150 }}
  role="dialog"
  aria-modal="true"
>
  <button
    class="lightbox__close"
    on:click={() => dispatch('close')}
    aria-label="Close"
  >
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <line x1="18" y1="6" x2="6" y2="18"></line>
      <line x1="6" y1="6" x2="18" y2="18"></line>
    </svg>
  </button>

  <img
    {src}
    {alt}
    class="lightbox__image"
    transition:scale={{ duration: 200, start: 0.9 }}
  />
</div>

<style>
  .lightbox {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.9);
    z-index: 1000;
    padding: 40px;
  }

  .lightbox__close {
    position: absolute;
    top: 20px;
    right: 20px;
    padding: 8px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 50%;
    cursor: pointer;
    color: white;
    transition: background 0.15s;
  }

  .lightbox__close:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .lightbox__image {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: 4px;
  }
</style>
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import MarkdownPreview from './MarkdownPreview.svelte';

describe('MarkdownPreview', () => {
  it('renders basic markdown', () => {
    render(MarkdownPreview, { props: { content: '# Hello World' } });

    expect(screen.getByText('Hello World')).toBeInTheDocument();
    expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument();
  });

  it('renders code blocks with syntax highlighting', () => {
    const content = '```typescript\nconst x = 1;\n```';
    render(MarkdownPreview, { props: { content } });

    expect(screen.getByText('typescript')).toBeInTheDocument();
    expect(screen.getByText('const')).toBeInTheDocument();
  });

  it('renders tables', () => {
    const content = '| A | B |\n|---|---|\n| 1 | 2 |';
    render(MarkdownPreview, { props: { content } });

    expect(screen.getByText('A')).toBeInTheDocument();
    expect(screen.getByRole('table')).toBeInTheDocument();
  });

  it('renders checkboxes', () => {
    const content = '- [ ] Todo\n- [x] Done';
    render(MarkdownPreview, { props: { content } });

    expect(screen.getByText('Todo')).toBeInTheDocument();
    expect(screen.getByText('Done')).toBeInTheDocument();
  });

  it('handles interactive checkbox toggle', async () => {
    const content = '- [ ] Todo';
    const { component } = render(MarkdownPreview, {
      props: { content, interactiveCheckboxes: true }
    });

    const toggleHandler = vi.fn();
    component.$on('checkboxToggle', toggleHandler);

    const checkbox = screen.getByRole('checkbox');
    await fireEvent.click(checkbox);

    expect(toggleHandler).toHaveBeenCalledWith(
      expect.objectContaining({
        detail: { index: 0, checked: true }
      })
    );
  });

  it('auto-links spec references', () => {
    const content = 'See Spec 231 for details';
    render(MarkdownPreview, { props: { content, specReferences: true } });

    const link = screen.getByText('Spec 231');
    expect(link).toHaveClass('md-spec-ref');
    expect(link).toHaveAttribute('data-spec-id', '231');
  });

  it('dispatches specClick on reference click', async () => {
    const content = 'See Spec 231 for details';
    const { component } = render(MarkdownPreview, {
      props: { content, specReferences: true }
    });

    const clickHandler = vi.fn();
    component.$on('specClick', clickHandler);

    await fireEvent.click(screen.getByText('Spec 231'));

    expect(clickHandler).toHaveBeenCalledWith(
      expect.objectContaining({
        detail: { specId: '231' }
      })
    );
  });

  it('shows copy button for code blocks', () => {
    const content = '```js\nconsole.log("test")\n```';
    render(MarkdownPreview, { props: { content, showCopyButtons: true } });

    expect(screen.getByText('Copy')).toBeInTheDocument();
  });

  it('handles copy button click', async () => {
    const mockClipboard = { writeText: vi.fn() };
    Object.assign(navigator, { clipboard: mockClipboard });

    const content = '```js\ntest code\n```';
    render(MarkdownPreview, { props: { content } });

    await fireEvent.click(screen.getByText('Copy'));

    expect(mockClipboard.writeText).toHaveBeenCalledWith('test code');
  });

  it('sanitizes HTML content', () => {
    const content = '<script>alert("xss")</script>Hello';
    render(MarkdownPreview, { props: { content } });

    expect(document.querySelector('script')).toBeNull();
    expect(screen.getByText('Hello')).toBeInTheDocument();
  });

  it('renders external links with icon', () => {
    const content = '[External](https://example.com)';
    render(MarkdownPreview, { props: { content } });

    const link = screen.getByText('External');
    expect(link).toHaveClass('md-link--external');
    expect(link).toHaveAttribute('target', '_blank');
  });
});
```

---

## Related Specs

- Spec 236: Spec Detail View
- Spec 237: Spec Editor
- Spec 243: Dependency Visualization
