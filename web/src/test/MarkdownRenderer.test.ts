import { render, screen, fireEvent } from '@testing-library/svelte';
import { vi } from 'vitest';
import MarkdownRenderer from '$lib/components/spec-browser/MarkdownRenderer.svelte';
import type { MarkdownConfig } from '$lib/types/markdown';

describe('MarkdownRenderer', () => {
  const sampleMarkdown = `# Main Heading

This is a paragraph with some **bold text** and *italic text*.

## Sub Heading

Here's a code block:

\`\`\`javascript
function hello() {
  console.log("Hello world");
}
\`\`\`

### Task List

- [x] Completed task
- [ ] Incomplete task

### Table

| Column 1 | Column 2 |
|----------|----------|
| Cell 1   | Cell 2   |

> [!NOTE]
> This is a note callout

> [!WARNING]
> This is a warning callout

### Mermaid Diagram

\`\`\`mermaid
graph TD
  A[Start] --> B[Process]
  B --> C[End]
\`\`\`
`;

  it('renders standard markdown correctly', () => {
    render(MarkdownRenderer, { content: sampleMarkdown });

    expect(screen.getByRole('heading', { name: /main heading/i })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: /sub heading/i })).toBeInTheDocument();
    expect(screen.getByText(/this is a paragraph/i)).toBeInTheDocument();
  });

  it('generates table of contents when enabled', () => {
    render(MarkdownRenderer, { 
      content: sampleMarkdown,
      config: { generateToc: true, syntaxHighlight: true, interactiveCheckboxes: true, mermaidDiagrams: true, customCallouts: true }
    });

    expect(screen.getByRole('navigation', { name: /table of contents/i })).toBeInTheDocument();
    expect(screen.getByText('Contents')).toBeInTheDocument();
  });

  it('renders interactive checkboxes when enabled', () => {
    render(MarkdownRenderer, { 
      content: sampleMarkdown,
      config: { generateToc: false, syntaxHighlight: true, interactiveCheckboxes: true, mermaidDiagrams: true, customCallouts: true }
    });

    const checkboxes = screen.getAllByRole('checkbox');
    expect(checkboxes).toHaveLength(2);
    expect(checkboxes[0]).toBeChecked();
    expect(checkboxes[1]).not.toBeChecked();
  });

  it('emits checkbox change events', async () => {
    const mockDispatch = vi.fn();
    const { component } = render(MarkdownRenderer, { 
      content: sampleMarkdown,
      config: { generateToc: false, syntaxHighlight: true, interactiveCheckboxes: true, mermaidDiagrams: true, customCallouts: true }
    });

    component.$on('checkboxChange', mockDispatch);

    const checkbox = screen.getAllByRole('checkbox')[1];
    await fireEvent.click(checkbox);

    expect(mockDispatch).toHaveBeenCalledWith(expect.objectContaining({
      detail: expect.objectContaining({
        checked: true,
        lineNumber: expect.any(Number)
      })
    }));
  });

  it('renders custom callouts when enabled', () => {
    render(MarkdownRenderer, { 
      content: sampleMarkdown,
      config: { generateToc: false, syntaxHighlight: true, interactiveCheckboxes: true, mermaidDiagrams: true, customCallouts: true }
    });

    expect(screen.getByText(/this is a note callout/i)).toBeInTheDocument();
    expect(screen.getByText(/this is a warning callout/i)).toBeInTheDocument();
  });

  it('handles anchor links correctly', async () => {
    const mockScrollIntoView = vi.fn();
    window.HTMLElement.prototype.scrollIntoView = mockScrollIntoView;

    render(MarkdownRenderer, { content: '# Test Heading' });

    const anchorLink = screen.getByRole('link', { name: /link to test heading/i });
    await fireEvent.click(anchorLink);

    expect(mockScrollIntoView).toHaveBeenCalledWith({
      behavior: 'smooth',
      block: 'start'
    });
  });

  it('disables features when config is false', () => {
    const disabledConfig: MarkdownConfig = {
      generateToc: false,
      syntaxHighlight: false,
      interactiveCheckboxes: false,
      mermaidDiagrams: false,
      customCallouts: false
    };

    render(MarkdownRenderer, { 
      content: sampleMarkdown,
      config: disabledConfig
    });

    expect(screen.queryByRole('navigation', { name: /table of contents/i })).not.toBeInTheDocument();
    
    const checkboxes = screen.getAllByRole('checkbox');
    checkboxes.forEach(checkbox => {
      expect(checkbox).toBeDisabled();
    });
  });

  it('handles empty content gracefully', () => {
    render(MarkdownRenderer, { content: '' });
    
    const renderer = screen.getByRole('article');
    expect(renderer).toBeInTheDocument();
  });

  it('renders code blocks with syntax highlighting', () => {
    const codeContent = '```javascript\nconst x = 1;\n```';
    render(MarkdownRenderer, { content: codeContent });

    const codeBlock = screen.getByText('const x = 1;');
    expect(codeBlock).toBeInTheDocument();
  });
});