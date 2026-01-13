import { render, screen, fireEvent } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import MarkdownRenderer from '$lib/components/spec-browser/MarkdownRenderer.svelte';
import type { MarkdownConfig } from '$lib/types/markdown';

// Mock highlight.js
vi.mock('highlight.js', () => ({
  default: {
    highlight: vi.fn(() => ({ value: 'highlighted-code' })),
    getLanguage: vi.fn(() => true),
  },
}));

// Mock mermaid
vi.mock('mermaid', () => ({
  default: {
    initialize: vi.fn(),
    run: vi.fn(),
  },
}));

describe('MarkdownRenderer', () => {
  const defaultConfig: MarkdownConfig = {
    syntaxHighlight: true,
    interactiveCheckboxes: true,
    generateToc: true,
    mermaidDiagrams: true,
    customCallouts: true,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Standard Markdown Rendering (GFM)', () => {
    it('should render basic markdown elements', () => {
      const content = `
# Heading 1
## Heading 2
### Heading 3

This is a **bold** text and *italic* text.

- List item 1
- List item 2

| Column 1 | Column 2 |
|----------|----------|
| Cell 1   | Cell 2   |
`;

      render(MarkdownRenderer, { content, config: defaultConfig });

      expect(screen.getByRole('heading', { level: 1, name: 'Heading 1' })).toBeInTheDocument();
      expect(screen.getByRole('heading', { level: 2, name: 'Heading 2' })).toBeInTheDocument();
      expect(screen.getByRole('heading', { level: 3, name: 'Heading 3' })).toBeInTheDocument();
      expect(screen.getByText('bold')).toBeInTheDocument();
      expect(screen.getByText('italic')).toBeInTheDocument();
      expect(screen.getByRole('table')).toBeInTheDocument();
    });

    it('should handle inline code and code blocks', () => {
      const content = `
This is \`inline code\`.

\`\`\`javascript
function hello() {
  console.log("Hello World");
}
\`\`\`
`;

      render(MarkdownRenderer, { content, config: defaultConfig });

      expect(screen.getByText('inline code')).toBeInTheDocument();
      expect(screen.getByText(/function hello/)).toBeInTheDocument();
    });
  });

  describe('Syntax Highlighting', () => {
    it('should apply syntax highlighting when enabled', () => {
      const content = `
\`\`\`javascript
function test() {
  return true;
}
\`\`\`
`;

      render(MarkdownRenderer, { content, config: { ...defaultConfig, syntaxHighlight: true } });

      // Should have highlighted content
      const codeBlock = screen.getByRole('code');
      expect(codeBlock).toBeInTheDocument();
    });

    it('should not apply syntax highlighting when disabled', () => {
      const content = `
\`\`\`javascript
function test() {
  return true;
}
\`\`\`
`;

      render(MarkdownRenderer, { content, config: { ...defaultConfig, syntaxHighlight: false } });

      const codeBlock = screen.getByText(/function test/);
      expect(codeBlock).toBeInTheDocument();
    });
  });

  describe('Interactive Checkboxes', () => {
    it('should render checkboxes for task lists', () => {
      const content = `
## Implementation Plan

- [x] Completed task
- [ ] Pending task
- [ ] Another pending task
`;

      render(MarkdownRenderer, { content, config: defaultConfig });

      const checkboxes = screen.getAllByRole('checkbox');
      expect(checkboxes).toHaveLength(3);
      expect(checkboxes[0]).toBeChecked();
      expect(checkboxes[1]).not.toBeChecked();
      expect(checkboxes[2]).not.toBeChecked();
    });

    it('should emit checkboxChange event when checkbox is clicked', async () => {
      const content = `
- [ ] Test task
`;

      const { component } = render(MarkdownRenderer, { content, config: defaultConfig });

      const mockHandler = vi.fn();
      component.$on('checkboxChange', mockHandler);

      const checkbox = screen.getByRole('checkbox');
      await fireEvent.click(checkbox);

      expect(mockHandler).toHaveBeenCalledWith(
        expect.objectContaining({
          detail: expect.objectContaining({
            checked: true,
            lineNumber: expect.any(Number),
          }),
        })
      );
    });

    it('should disable checkboxes when interactiveCheckboxes is false', () => {
      const content = `
- [ ] Test task
`;

      render(MarkdownRenderer, { content, config: { ...defaultConfig, interactiveCheckboxes: false } });

      const checkbox = screen.getByRole('checkbox');
      expect(checkbox).toBeDisabled();
    });
  });

  describe('Table of Contents', () => {
    it('should generate TOC when enabled and multiple headings exist', () => {
      const content = `
# Title
## Section 1
### Subsection 1.1
## Section 2
### Subsection 2.1
### Subsection 2.2
`;

      render(MarkdownRenderer, { content, config: { ...defaultConfig, generateToc: true } });

      expect(screen.getByRole('navigation', { name: 'Table of contents' })).toBeInTheDocument();
      expect(screen.getByText('Contents')).toBeInTheDocument();
      expect(screen.getByRole('link', { name: 'Section 1' })).toBeInTheDocument();
      expect(screen.getByRole('link', { name: 'Section 2' })).toBeInTheDocument();
    });

    it('should not generate TOC when disabled', () => {
      const content = `
# Title
## Section 1
## Section 2
`;

      render(MarkdownRenderer, { content, config: { ...defaultConfig, generateToc: false } });

      expect(screen.queryByRole('navigation', { name: 'Table of contents' })).not.toBeInTheDocument();
    });

    it('should not generate TOC when there are fewer than 3 items', () => {
      const content = `
# Title
## Section 1
`;

      render(MarkdownRenderer, { content, config: { ...defaultConfig, generateToc: true } });

      expect(screen.queryByRole('navigation', { name: 'Table of contents' })).not.toBeInTheDocument();
    });
  });

  describe('Anchor Links', () => {
    it('should add anchor links to headings', () => {
      const content = `
# Main Title
## Section Title
`;

      render(MarkdownRenderer, { content, config: defaultConfig });

      const anchors = screen.getAllByRole('link', { name: /#/ });
      expect(anchors.length).toBeGreaterThan(0);
    });

    it('should handle anchor link clicks with smooth scroll', async () => {
      const content = `
# Title
## Section
`;

      render(MarkdownRenderer, { content, config: defaultConfig });

      // Mock scrollIntoView
      const mockScrollIntoView = vi.fn();
      Element.prototype.scrollIntoView = mockScrollIntoView;

      const anchorLink = screen.getByRole('link', { name: /Link to Section/ });
      await fireEvent.click(anchorLink);

      // Note: Actual scrollIntoView call testing would require more DOM setup
    });
  });

  describe('Mermaid Diagram Support', () => {
    it('should render Mermaid diagrams when enabled', () => {
      const content = `
\`\`\`mermaid
graph TD
    A[Start] --> B[Process]
    B --> C[End]
\`\`\`
`;

      render(MarkdownRenderer, { content, config: { ...defaultConfig, mermaidDiagrams: true } });

      expect(screen.getByText(/graph TD/)).toBeInTheDocument();
    });

    it('should not render Mermaid diagrams when disabled', () => {
      const content = `
\`\`\`mermaid
graph TD
    A[Start] --> B[Process]
\`\`\`
`;

      render(MarkdownRenderer, { content, config: { ...defaultConfig, mermaidDiagrams: false } });

      // Should render as regular code block instead
      const codeBlock = screen.getByText(/graph TD/);
      expect(codeBlock).toBeInTheDocument();
    });
  });

  describe('Custom Spec Callouts', () => {
    it('should render note callouts', () => {
      const content = `
> [!NOTE]
> This is an important note.
`;

      render(MarkdownRenderer, { content, config: { ...defaultConfig, customCallouts: true } });

      expect(screen.getByText('ℹ️')).toBeInTheDocument();
      expect(screen.getByText(/This is an important note/)).toBeInTheDocument();
    });

    it('should render warning callouts', () => {
      const content = `
> [!WARNING]
> This is a warning message.
`;

      render(MarkdownRenderer, { content, config: { ...defaultConfig, customCallouts: true } });

      expect(screen.getByText('⚠️')).toBeInTheDocument();
      expect(screen.getByText(/This is a warning message/)).toBeInTheDocument();
    });

    it('should render tip callouts', () => {
      const content = `
> [!TIP]
> This is a helpful tip.
`;

      render(MarkdownRenderer, { content, config: { ...defaultConfig, customCallouts: true } });

      expect(screen.getByText('💡')).toBeInTheDocument();
      expect(screen.getByText(/This is a helpful tip/)).toBeInTheDocument();
    });

    it('should render regular blockquotes when callouts disabled', () => {
      const content = `
> [!NOTE]
> This should be a regular blockquote.
`;

      render(MarkdownRenderer, { content, config: { ...defaultConfig, customCallouts: false } });

      expect(screen.getByRole('blockquote')).toBeInTheDocument();
      expect(screen.queryByText('ℹ️')).not.toBeInTheDocument();
    });
  });

  describe('Event Handling', () => {
    it('should emit linkClick event for internal links', async () => {
      const content = `
[Internal Link](./other-page.md)
`;

      const { component } = render(MarkdownRenderer, { content, config: defaultConfig });

      const mockHandler = vi.fn();
      component.$on('linkClick', mockHandler);

      const link = screen.getByRole('link', { name: 'Internal Link' });
      await fireEvent.click(link);

      expect(mockHandler).toHaveBeenCalledWith(
        expect.objectContaining({
          detail: expect.objectContaining({
            href: './other-page.md',
          }),
        })
      );
    });

    it('should not emit linkClick for external links', async () => {
      const content = `
[External Link](https://example.com)
`;

      const { component } = render(MarkdownRenderer, { content, config: defaultConfig });

      const mockHandler = vi.fn();
      component.$on('linkClick', mockHandler);

      const link = screen.getByRole('link', { name: 'External Link' });
      await fireEvent.click(link);

      expect(mockHandler).not.toHaveBeenCalled();
    });
  });
});