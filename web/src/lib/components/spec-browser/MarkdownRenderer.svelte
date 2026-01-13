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
  });

  // Custom renderer
  const renderer = {
    heading(text: string, level: number): string {
      const cleanText = typeof text === 'string' ? text : String(text);
      const id = cleanText.toLowerCase().replace(/[^\w\s-]/g, '').replace(/\s+/g, '-');
      if (config.generateToc) {
        toc.push({ id, text: cleanText, level });
      }
      return `<h${level} id="${id}">
        <a class="anchor" href="#${id}" aria-label="Link to ${cleanText}">#</a>
        ${cleanText}
      </h${level}>`;
    },

    listitem(text: string, task?: boolean, checked?: boolean): string {
      if (task) {
        const checkboxId = `checkbox-${Math.random().toString(36).substr(2, 9)}`;
        const checkedAttr = checked ? 'checked' : '';
        return `<li class="task-item">
          <input type="checkbox" id="${checkboxId}" ${checkedAttr} ${config.interactiveCheckboxes ? '' : 'disabled'}>
          <label for="${checkboxId}">${text}</label>
        </li>`;
      }
      return `<li>${text}</li>`;
    },

    blockquote(quote: string): string {
      const noteMatch = quote.match(/^<p>\[!NOTE\](.*)<\/p>$/s);
      const warnMatch = quote.match(/^<p>\[!WARNING\](.*)<\/p>$/s);
      const tipMatch = quote.match(/^<p>\[!TIP\](.*)<\/p>$/s);

      if (config.customCallouts) {
        if (noteMatch) {
          return `<div class="callout callout--note">
            <span class="callout-icon" aria-label="Note">ℹ️</span>
            <div class="callout-content">${noteMatch[1]}</div>
          </div>`;
        }
        if (warnMatch) {
          return `<div class="callout callout--warning">
            <span class="callout-icon" aria-label="Warning">⚠️</span>
            <div class="callout-content">${warnMatch[1]}</div>
          </div>`;
        }
        if (tipMatch) {
          return `<div class="callout callout--tip">
            <span class="callout-icon" aria-label="Tip">💡</span>
            <div class="callout-content">${tipMatch[1]}</div>
          </div>`;
        }
      }
      return `<blockquote>${quote}</blockquote>`;
    },

    code(code: string, infostring?: string): string {
      const cleanCode = typeof code === 'string' ? code : String(code);
      const lang = infostring || '';
      
      if (lang === 'mermaid' && config.mermaidDiagrams) {
        const id = `mermaid-${Math.random().toString(36).substr(2, 9)}`;
        return `<div class="mermaid-container">
          <pre class="mermaid" id="${id}">${cleanCode}</pre>
        </div>`;
      }
      
      if (config.syntaxHighlight && lang && hljs.getLanguage(lang)) {
        const highlighted = hljs.highlight(cleanCode, { language: lang }).value;
        return `<pre><code class="hljs ${lang}">${highlighted}</code></pre>`;
      }
      
      const escapedCode = cleanCode.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
      return `<pre><code${lang ? ` class="${lang}"` : ''}>${escapedCode}</code></pre>`;
    }
  };

  function render() {
    toc = [];
    marked.use({ renderer });
    renderedHtml = marked.parse(content);
    
    // Initialize Mermaid diagrams after rendering
    if (config.mermaidDiagrams) {
      setTimeout(async () => {
        const { default: mermaid } = await import('mermaid');
        mermaid.initialize({ 
          theme: 'default',
          startOnLoad: true,
          flowchart: { useMaxWidth: true }
        });
        mermaid.run();
      }, 100);
    }
  }

  function handleClick(event: MouseEvent) {
    const target = event.target as HTMLElement;

    // Handle checkbox clicks
    if (target.matches('input[type="checkbox"]') && config.interactiveCheckboxes) {
      const checkbox = target as HTMLInputElement;
      const listItem = checkbox.closest('li');
      
      // Calculate line number from content
      const lines = content.split('\n');
      let lineNumber = 0;
      const labelText = listItem?.textContent?.trim() || '';
      
      for (let i = 0; i < lines.length; i++) {
        if (lines[i].includes(labelText.substring(0, 20))) {
          lineNumber = i + 1;
          break;
        }
      }
      
      dispatch('checkboxChange', { lineNumber, checked: checkbox.checked });
    }

    // Handle link clicks
    if (target.matches('a')) {
      const link = target as HTMLAnchorElement;
      if (link.href.startsWith('#')) {
        event.preventDefault();
        const id = link.href.split('#')[1];
        const element = document.getElementById(id);
        if (element) {
          element.scrollIntoView({ behavior: 'smooth', block: 'start' });
        }
      } else if (!link.href.startsWith('http')) {
        event.preventDefault();
        dispatch('linkClick', { href: link.href });
      }
    }
  }

  $: if (content) render();

  onMount(() => {
    // Load highlight.js theme
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = 'https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css';
    document.head.appendChild(link);

    return () => {
      document.head.removeChild(link);
    };
  });
</script>

<div
  bind:this={containerRef}
  class="markdown-renderer"
  on:click={handleClick}
  role="article"
  aria-label="Markdown content"
>
  {#if config.generateToc && toc.length > 2}
    <nav class="markdown-toc" role="navigation" aria-label="Table of contents">
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
    color: var(--color-fg-default);
    background: var(--color-bg-base);
  }

  .markdown-toc {
    float: right;
    width: 220px;
    margin: 0 0 16px 24px;
    padding: 12px;
    background: var(--color-bg-surface);
    border-radius: 8px;
    font-size: 13px;
    border: 1px solid var(--color-border-subtle);
  }

  .markdown-toc h4 {
    margin: 0 0 8px 0;
    font-size: 12px;
    text-transform: uppercase;
    color: var(--color-fg-muted);
    font-weight: 600;
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
    color: var(--color-fg-muted);
    text-decoration: none;
    display: block;
    padding: 2px 6px;
    border-radius: 4px;
    transition: all 0.15s ease;
  }

  .markdown-toc a:hover {
    color: var(--color-accent-fg);
    background: var(--color-bg-hover);
  }

  .markdown-toc a:focus-visible {
    outline: 2px solid var(--color-accent-fg);
    outline-offset: 2px;
  }

  .markdown-content :global(h1),
  .markdown-content :global(h2),
  .markdown-content :global(h3),
  .markdown-content :global(h4),
  .markdown-content :global(h5),
  .markdown-content :global(h6) {
    margin-top: 24px;
    margin-bottom: 16px;
    font-weight: 600;
    position: relative;
    line-height: 1.3;
  }

  .markdown-content :global(h1) { 
    font-size: 28px; 
    border-bottom: 1px solid var(--color-border-subtle);
    padding-bottom: 8px;
  }
  .markdown-content :global(h2) { 
    font-size: 22px; 
    border-bottom: 1px solid var(--color-border-subtle);
    padding-bottom: 6px;
  }
  .markdown-content :global(h3) { font-size: 18px; }
  .markdown-content :global(h4) { font-size: 16px; }
  .markdown-content :global(h5) { font-size: 14px; }
  .markdown-content :global(h6) { font-size: 13px; }

  .markdown-content :global(.anchor) {
    position: absolute;
    left: -20px;
    color: var(--color-fg-muted);
    opacity: 0;
    text-decoration: none;
    font-weight: normal;
    transition: opacity 0.15s ease;
  }

  .markdown-content :global(h1:hover .anchor),
  .markdown-content :global(h2:hover .anchor),
  .markdown-content :global(h3:hover .anchor),
  .markdown-content :global(h4:hover .anchor),
  .markdown-content :global(h5:hover .anchor),
  .markdown-content :global(h6:hover .anchor) {
    opacity: 1;
  }

  .markdown-content :global(p) {
    margin: 16px 0;
  }

  .markdown-content :global(pre) {
    background: var(--color-bg-surface);
    padding: 16px;
    border-radius: 8px;
    overflow-x: auto;
    font-size: 13px;
    border: 1px solid var(--color-border-subtle);
    margin: 16px 0;
  }

  .markdown-content :global(code) {
    font-family: 'SF Mono', 'Monaco', 'Cascadia Code', 'Roboto Mono', monospace;
    font-size: 0.9em;
  }

  .markdown-content :global(:not(pre) > code) {
    background: var(--color-bg-hover);
    padding: 2px 6px;
    border-radius: 4px;
    color: var(--color-danger-fg);
  }

  .markdown-content :global(.task-item) {
    list-style: none;
    margin-left: -20px;
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin-bottom: 4px;
  }

  .markdown-content :global(.task-item input[type="checkbox"]) {
    margin-top: 4px;
    cursor: pointer;
    flex-shrink: 0;
  }

  .markdown-content :global(.task-item label) {
    cursor: pointer;
    flex: 1;
  }

  .markdown-content :global(.task-item input[type="checkbox"]:disabled) {
    cursor: not-allowed;
    opacity: 0.6;
  }

  .markdown-content :global(.callout) {
    padding: 12px 16px;
    border-radius: 8px;
    margin: 16px 0;
    display: flex;
    gap: 12px;
    border-left: 4px solid;
  }

  .markdown-content :global(.callout-content) {
    flex: 1;
  }

  .markdown-content :global(.callout-icon) {
    flex-shrink: 0;
    font-size: 16px;
  }

  .markdown-content :global(.callout--note) {
    background: rgba(33, 150, 243, 0.1);
    border-left-color: var(--color-accent-fg);
    color: var(--color-fg-default);
  }

  .markdown-content :global(.callout--warning) {
    background: rgba(255, 152, 0, 0.1);
    border-left-color: var(--color-attention-fg);
    color: var(--color-fg-default);
  }

  .markdown-content :global(.callout--tip) {
    background: rgba(76, 175, 80, 0.1);
    border-left-color: var(--color-success-fg);
    color: var(--color-fg-default);
  }

  .markdown-content :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 16px 0;
    background: var(--color-bg-surface);
    border-radius: 8px;
    overflow: hidden;
  }

  .markdown-content :global(th),
  .markdown-content :global(td) {
    padding: 10px 12px;
    border: 1px solid var(--color-border-subtle);
    text-align: left;
  }

  .markdown-content :global(th) {
    background: var(--color-bg-elevated);
    font-weight: 600;
    border-bottom: 2px solid var(--color-border-default);
  }

  .markdown-content :global(tr:nth-child(even)) {
    background: var(--color-bg-hover);
  }

  .markdown-content :global(blockquote) {
    border-left: 4px solid var(--color-border-emphasis);
    margin: 16px 0;
    padding: 0 16px;
    color: var(--color-fg-muted);
    font-style: italic;
  }

  .markdown-content :global(ul),
  .markdown-content :global(ol) {
    padding-left: 24px;
    margin: 16px 0;
  }

  .markdown-content :global(li) {
    margin: 4px 0;
  }

  .markdown-content :global(a) {
    color: var(--color-accent-fg);
    text-decoration: none;
  }

  .markdown-content :global(a:hover) {
    text-decoration: underline;
  }

  .markdown-content :global(a:focus-visible) {
    outline: 2px solid var(--color-accent-fg);
    outline-offset: 2px;
    border-radius: 2px;
  }

  .markdown-content :global(.mermaid-container) {
    margin: 24px 0;
    text-align: center;
    background: var(--color-bg-surface);
    border-radius: 8px;
    padding: 16px;
    border: 1px solid var(--color-border-subtle);
  }

  .markdown-content :global(.mermaid) {
    background: transparent;
    border: none;
    padding: 0;
    margin: 0;
    font-size: 16px;
  }

  /* Mobile responsive */
  @media (max-width: 768px) {
    .markdown-toc {
      float: none;
      width: 100%;
      margin: 0 0 16px 0;
    }

    .markdown-content :global(.anchor) {
      display: none;
    }

    .markdown-content :global(pre) {
      font-size: 12px;
      padding: 12px;
    }
  }

  /* Reduce motion for accessibility */
  @media (prefers-reduced-motion: reduce) {
    .markdown-toc a,
    .markdown-content :global(.anchor) {
      transition: none;
    }
  }

  /* High contrast mode */
  @media (prefers-contrast: high) {
    .markdown-content :global(.callout) {
      border-width: 2px;
    }

    .markdown-content :global(th) {
      border-bottom-width: 3px;
    }
  }
</style>