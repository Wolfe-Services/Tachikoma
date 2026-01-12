# Spec 208: Code Block Component

## Phase
Phase 9: UI Foundation

## Spec ID
208

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 191-195: Design System

## Estimated Context
~10%

---

## Objective

Implement a Code Block component for Tachikoma with syntax highlighting, line numbers, copy functionality, and support for multiple programming languages commonly used in penetration testing.

---

## Acceptance Criteria

- [ ] Syntax highlighting for multiple languages
- [ ] Line numbers (optional)
- [ ] Line highlighting
- [ ] Copy to clipboard button
- [ ] Language label
- [ ] Word wrap option
- [ ] Dark theme optimized colors
- [ ] Collapsible long code blocks
- [ ] Support for bash, python, javascript, json, yaml, rust, and more
- [ ] Custom Tachikoma-themed syntax colors

---

## Implementation Details

### src/lib/components/ui/CodeBlock/CodeBlock.svelte

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { cn } from '@utils/component';
  import { toast } from '@stores/toast';
  import Icon from '../Icon/Icon.svelte';
  import Button from '../Button/Button.svelte';

  // Language type includes common security/pentest languages
  type Language =
    | 'bash' | 'shell' | 'sh'
    | 'python' | 'py'
    | 'javascript' | 'js'
    | 'typescript' | 'ts'
    | 'json'
    | 'yaml' | 'yml'
    | 'rust' | 'rs'
    | 'go'
    | 'ruby' | 'rb'
    | 'perl'
    | 'php'
    | 'sql'
    | 'xml'
    | 'html'
    | 'css'
    | 'markdown' | 'md'
    | 'powershell' | 'ps1'
    | 'text' | 'plaintext';

  export let code: string = '';
  export let language: Language = 'text';
  export let filename: string | undefined = undefined;
  export let showLineNumbers: boolean = true;
  export let highlightLines: number[] = [];
  export let showCopy: boolean = true;
  export let showLanguage: boolean = true;
  export let wordWrap: boolean = false;
  export let maxHeight: number | undefined = undefined;
  export let collapsible: boolean = false;
  export let collapsed: boolean = false;
  let className: string = '';
  export { className as class };

  let copied = false;

  $: lines = code.split('\n');
  $: highlightedCode = highlightSyntax(code, language);

  async function copyToClipboard() {
    try {
      await navigator.clipboard.writeText(code);
      copied = true;
      toast.success('Copied to clipboard');
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch {
      toast.error('Failed to copy');
    }
  }

  function toggleCollapse() {
    collapsed = !collapsed;
  }

  // Simple syntax highlighting (can be enhanced with a library)
  function highlightSyntax(code: string, lang: Language): string {
    if (lang === 'text' || lang === 'plaintext') {
      return escapeHtml(code);
    }

    let result = escapeHtml(code);

    // Apply language-specific highlighting
    const rules = getHighlightRules(lang);
    rules.forEach(({ pattern, className }) => {
      result = result.replace(pattern, (match, ...groups) => {
        // If there are capture groups, use the first one
        const text = groups.find(g => typeof g === 'string' && g !== undefined) || match;
        return `<span class="token ${className}">${text}</span>`;
      });
    });

    return result;
  }

  function escapeHtml(str: string): string {
    return str
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }

  function getHighlightRules(lang: Language): { pattern: RegExp; className: string }[] {
    const commonRules = [
      // Strings
      { pattern: /(&quot;[^&]*&quot;|&#039;[^&]*&#039;)/g, className: 'string' },
      // Comments
      { pattern: /(\/\/.*$|#.*$)/gm, className: 'comment' },
      // Numbers
      { pattern: /\b(\d+\.?\d*)\b/g, className: 'number' },
    ];

    const langRules: Record<string, { pattern: RegExp; className: string }[]> = {
      bash: [
        ...commonRules,
        { pattern: /\b(if|then|else|elif|fi|for|while|do|done|case|esac|function|return|exit|echo|export|source|alias|cd|ls|grep|awk|sed|cat|curl|wget|sudo|chmod|chown|rm|cp|mv|mkdir)\b/g, className: 'keyword' },
        { pattern: /(\$\w+|\$\{[^}]+\})/g, className: 'variable' },
      ],
      python: [
        ...commonRules,
        { pattern: /\b(def|class|if|elif|else|for|while|try|except|finally|with|as|import|from|return|yield|lambda|and|or|not|in|is|True|False|None|pass|break|continue|raise|assert)\b/g, className: 'keyword' },
        { pattern: /\b(print|len|range|str|int|float|list|dict|set|tuple|open|input|type|isinstance|hasattr|getattr|setattr)\b/g, className: 'function' },
        { pattern: /(@\w+)/g, className: 'decorator' },
      ],
      javascript: [
        ...commonRules,
        { pattern: /\b(const|let|var|function|return|if|else|for|while|do|switch|case|break|continue|try|catch|finally|throw|new|class|extends|import|export|from|default|async|await|typeof|instanceof)\b/g, className: 'keyword' },
        { pattern: /\b(console|document|window|Array|Object|String|Number|Boolean|Promise|Map|Set|JSON)\b/g, className: 'builtin' },
      ],
      typescript: [
        ...commonRules,
        { pattern: /\b(const|let|var|function|return|if|else|for|while|do|switch|case|break|continue|try|catch|finally|throw|new|class|extends|import|export|from|default|async|await|typeof|instanceof|interface|type|enum|implements|private|public|protected|readonly|abstract|static|as|is|keyof|infer|extends)\b/g, className: 'keyword' },
        { pattern: /:\s*(string|number|boolean|void|any|unknown|never|object|null|undefined)/g, className: 'type' },
      ],
      json: [
        { pattern: /(&quot;[^&]*&quot;)\s*:/g, className: 'property' },
        { pattern: /:\s*(&quot;[^&]*&quot;)/g, className: 'string' },
        { pattern: /:\s*(\d+\.?\d*)/g, className: 'number' },
        { pattern: /:\s*(true|false|null)/g, className: 'keyword' },
      ],
      yaml: [
        { pattern: /^(\s*[\w-]+):/gm, className: 'property' },
        { pattern: /:\s*(&quot;[^&]*&quot;|&#039;[^&]*&#039;)/g, className: 'string' },
        { pattern: /:\s*(true|false|null|yes|no)/gi, className: 'keyword' },
        { pattern: /(#.*$)/gm, className: 'comment' },
      ],
      rust: [
        ...commonRules,
        { pattern: /\b(fn|let|mut|const|struct|enum|impl|trait|pub|mod|use|self|Self|super|crate|where|async|await|move|ref|match|if|else|for|while|loop|break|continue|return|type|as|in|unsafe|static|extern)\b/g, className: 'keyword' },
        { pattern: /\b(i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize|f32|f64|bool|char|str|String|Vec|Option|Result|Box|Rc|Arc)\b/g, className: 'type' },
        { pattern: /(&amp;mut|&amp;)/g, className: 'operator' },
      ],
      sql: [
        ...commonRules,
        { pattern: /\b(SELECT|FROM|WHERE|JOIN|LEFT|RIGHT|INNER|OUTER|ON|AND|OR|NOT|IN|LIKE|ORDER|BY|GROUP|HAVING|INSERT|INTO|VALUES|UPDATE|SET|DELETE|CREATE|TABLE|INDEX|DROP|ALTER|ADD|COLUMN|PRIMARY|KEY|FOREIGN|REFERENCES|NULL|DEFAULT|UNIQUE|CHECK|CONSTRAINT)\b/gi, className: 'keyword' },
        { pattern: /\b(COUNT|SUM|AVG|MIN|MAX|CONCAT|SUBSTRING|UPPER|LOWER|TRIM|COALESCE|NULLIF|CAST|CONVERT)\b/gi, className: 'function' },
      ],
    };

    // Handle language aliases
    const aliasMap: Record<string, string> = {
      sh: 'bash', shell: 'bash',
      py: 'python',
      js: 'javascript',
      ts: 'typescript',
      rs: 'rust',
      yml: 'yaml',
      rb: 'ruby',
      ps1: 'powershell',
      md: 'markdown'
    };

    const normalizedLang = aliasMap[lang] || lang;
    return langRules[normalizedLang] || commonRules;
  }

  $: classes = cn(
    'code-block',
    wordWrap && 'code-block-wrap',
    className
  );
</script>

<div class={classes}>
  {#if filename || showLanguage || showCopy || collapsible}
    <div class="code-block-header">
      <div class="code-block-info">
        {#if filename}
          <span class="code-block-filename">
            <Icon name="file" size={14} />
            {filename}
          </span>
        {/if}
        {#if showLanguage && language !== 'text'}
          <span class="code-block-language">{language}</span>
        {/if}
      </div>

      <div class="code-block-actions">
        {#if collapsible}
          <Button
            variant="ghost"
            size="sm"
            iconOnly
            on:click={toggleCollapse}
            aria-label={collapsed ? 'Expand code' : 'Collapse code'}
          >
            <Icon name={collapsed ? 'chevron-down' : 'chevron-up'} size={16} />
          </Button>
        {/if}
        {#if showCopy}
          <Button
            variant="ghost"
            size="sm"
            iconOnly
            on:click={copyToClipboard}
            aria-label="Copy code"
          >
            <Icon name={copied ? 'check' : 'copy'} size={16} />
          </Button>
        {/if}
      </div>
    </div>
  {/if}

  {#if !collapsed}
    <div
      class="code-block-content"
      style={maxHeight ? `max-height: ${maxHeight}px` : undefined}
    >
      {#if showLineNumbers}
        <div class="code-block-lines" aria-hidden="true">
          {#each lines as _, i}
            <span
              class="code-block-line-number"
              class:highlighted={highlightLines.includes(i + 1)}
            >{i + 1}</span>
          {/each}
        </div>
      {/if}

      <pre class="code-block-pre"><code class="code-block-code">{@html highlightedCode}</code></pre>
    </div>
  {/if}
</div>

<style>
  .code-block {
    display: flex;
    flex-direction: column;
    background-color: var(--color-code-bg);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    overflow: hidden;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
  }

  .code-block-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-2) var(--spacing-3);
    background-color: var(--color-bg-elevated);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .code-block-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-3);
  }

  .code-block-filename {
    display: flex;
    align-items: center;
    gap: var(--spacing-1-5);
    font-size: var(--text-xs);
    color: var(--color-fg-default);
  }

  .code-block-language {
    font-size: var(--text-xs);
    color: var(--color-fg-muted);
    text-transform: uppercase;
  }

  .code-block-actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
  }

  .code-block-content {
    display: flex;
    overflow: auto;
  }

  .code-block-lines {
    display: flex;
    flex-direction: column;
    padding: var(--spacing-3) 0;
    background-color: var(--color-bg-surface);
    border-right: 1px solid var(--color-border-subtle);
    user-select: none;
  }

  .code-block-line-number {
    padding: 0 var(--spacing-3);
    text-align: right;
    color: var(--color-fg-muted);
    font-size: var(--text-xs);
    line-height: 1.7;
    min-width: 40px;
  }

  .code-block-line-number.highlighted {
    background-color: var(--color-warning-subtle);
    color: var(--color-warning-fg);
  }

  .code-block-pre {
    flex: 1;
    margin: 0;
    padding: var(--spacing-3);
    overflow-x: auto;
  }

  .code-block-wrap .code-block-pre {
    white-space: pre-wrap;
    word-break: break-word;
  }

  .code-block-code {
    color: var(--color-code-text);
    line-height: 1.7;
  }

  /* Syntax highlighting tokens */
  .code-block :global(.token.keyword) {
    color: var(--color-syntax-keyword);
  }

  .code-block :global(.token.string) {
    color: var(--color-syntax-string);
  }

  .code-block :global(.token.number) {
    color: var(--color-syntax-number);
  }

  .code-block :global(.token.function),
  .code-block :global(.token.builtin) {
    color: var(--color-syntax-function);
  }

  .code-block :global(.token.comment) {
    color: var(--color-syntax-comment);
    font-style: italic;
  }

  .code-block :global(.token.operator) {
    color: var(--color-syntax-operator);
  }

  .code-block :global(.token.variable) {
    color: var(--color-syntax-variable);
  }

  .code-block :global(.token.type) {
    color: var(--color-syntax-class);
  }

  .code-block :global(.token.property) {
    color: var(--tachikoma-400);
  }

  .code-block :global(.token.decorator) {
    color: var(--color-syntax-function);
  }
</style>
```

### Usage Examples

```svelte
<script>
  import { CodeBlock } from '@components/ui';

  const bashCode = `#!/bin/bash
# Nmap scan example
nmap -sV -sC -oA scan_results 192.168.1.0/24

# Parse results
grep "open" scan_results.nmap`;

  const pythonCode = `import nmap

scanner = nmap.PortScanner()
scanner.scan('192.168.1.0/24', '22-443')

for host in scanner.all_hosts():
    print(f'Host: {host}')
    for proto in scanner[host].all_protocols():
        ports = scanner[host][proto].keys()
        for port in ports:
            print(f'  Port {port}: {scanner[host][proto][port]["state"]}')`;
</script>

<!-- Basic usage -->
<CodeBlock code={bashCode} language="bash" />

<!-- With filename -->
<CodeBlock
  code={pythonCode}
  language="python"
  filename="scanner.py"
/>

<!-- Highlight specific lines -->
<CodeBlock
  code={bashCode}
  language="bash"
  highlightLines={[3, 6]}
/>

<!-- Without line numbers -->
<CodeBlock
  code={bashCode}
  language="bash"
  showLineNumbers={false}
/>

<!-- Collapsible -->
<CodeBlock
  code={pythonCode}
  language="python"
  collapsible
/>
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/CodeBlock.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import CodeBlock from '@components/ui/CodeBlock/CodeBlock.svelte';

describe('CodeBlock', () => {
  it('should render code', () => {
    const { container } = render(CodeBlock, {
      props: { code: 'const x = 1;', language: 'javascript' }
    });

    expect(container.querySelector('code')).toHaveTextContent('const x = 1;');
  });

  it('should show line numbers by default', () => {
    const { container } = render(CodeBlock, {
      props: { code: 'line1\nline2', language: 'text' }
    });

    const lineNumbers = container.querySelectorAll('.code-block-line-number');
    expect(lineNumbers).toHaveLength(2);
  });

  it('should highlight specified lines', () => {
    const { container } = render(CodeBlock, {
      props: {
        code: 'line1\nline2\nline3',
        language: 'text',
        highlightLines: [2]
      }
    });

    const highlighted = container.querySelector('.code-block-line-number.highlighted');
    expect(highlighted).toHaveTextContent('2');
  });

  it('should copy code to clipboard', async () => {
    const mockClipboard = { writeText: vi.fn().mockResolvedValue(undefined) };
    Object.assign(navigator, { clipboard: mockClipboard });

    const { getByLabelText } = render(CodeBlock, {
      props: { code: 'test code', language: 'text' }
    });

    await fireEvent.click(getByLabelText('Copy code'));
    expect(mockClipboard.writeText).toHaveBeenCalledWith('test code');
  });

  it('should show language label', () => {
    const { getByText } = render(CodeBlock, {
      props: { code: 'code', language: 'python' }
    });

    expect(getByText('python')).toBeInTheDocument();
  });

  it('should show filename when provided', () => {
    const { getByText } = render(CodeBlock, {
      props: {
        code: 'code',
        language: 'python',
        filename: 'script.py'
      }
    });

    expect(getByText('script.py')).toBeInTheDocument();
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [209-diff-viewer.md](./209-diff-viewer.md) - Diff viewer component
- [210-terminal-component.md](./210-terminal-component.md) - Terminal component
