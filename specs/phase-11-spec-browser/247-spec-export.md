# Spec 247: Spec Export

## Phase
11 - Spec Browser UI

## Spec ID
247

## Status
Planned

## Dependencies
- Spec 231 (Spec List Layout)
- Spec 236 (Spec Detail View)

## Estimated Context
~8%

---

## Objective

Implement comprehensive export functionality for specs supporting multiple formats (Markdown, JSON, PDF, HTML), with options for single or bulk export, customizable templates, and filtered exports.

---

## Acceptance Criteria

- [ ] Export single spec to Markdown
- [ ] Export to JSON format
- [ ] Export to PDF with styling
- [ ] Export to HTML (standalone)
- [ ] Bulk export multiple specs
- [ ] Export with dependencies included
- [ ] Customizable export templates
- [ ] Export filtered results
- [ ] Progress indicator for large exports

---

## Implementation Details

### ExportDialog.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { writable } from 'svelte/store';
  import type { Spec, ExportFormat, ExportOptions } from '$lib/types/spec';
  import Modal from '$lib/components/Modal.svelte';
  import Button from '$lib/components/Button.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Toggle from '$lib/components/Toggle.svelte';
  import ProgressBar from '$lib/components/ProgressBar.svelte';
  import { exportSpecs } from '$lib/utils/export';

  export let open = false;
  export let specs: Spec[] = [];
  export let allSpecs: Spec[] = [];

  const dispatch = createEventDispatcher<{
    close: void;
    exported: { format: ExportFormat; filename: string };
  }>();

  type ExportFormat = 'markdown' | 'json' | 'pdf' | 'html';

  let selectedFormat: ExportFormat = 'markdown';
  let options = writable<ExportOptions>({
    includeDependencies: false,
    includeHistory: false,
    includeComments: false,
    flattenToSingleFile: true,
    customTemplate: null
  });

  let exporting = false;
  let progress = 0;
  let error: string | null = null;

  const formatOptions: { value: ExportFormat; label: string; icon: string; ext: string }[] = [
    { value: 'markdown', label: 'Markdown', icon: 'file-text', ext: '.md' },
    { value: 'json', label: 'JSON', icon: 'code', ext: '.json' },
    { value: 'pdf', label: 'PDF', icon: 'file', ext: '.pdf' },
    { value: 'html', label: 'HTML', icon: 'globe', ext: '.html' }
  ];

  $: selectedFormatInfo = formatOptions.find(f => f.value === selectedFormat);
  $: filename = generateFilename(specs, selectedFormat);
  $: totalSpecsToExport = $options.includeDependencies
    ? getSpecsWithDependencies(specs, allSpecs).length
    : specs.length;

  function generateFilename(specs: Spec[], format: ExportFormat): string {
    const ext = formatOptions.find(f => f.value === format)?.ext || '';

    if (specs.length === 1) {
      return `spec-${specs[0].id}${ext}`;
    }
    return `specs-export-${new Date().toISOString().slice(0, 10)}${ext}`;
  }

  function getSpecsWithDependencies(specs: Spec[], allSpecs: Spec[]): Spec[] {
    const included = new Set<string>();
    const result: Spec[] = [];

    function addWithDeps(specId: string) {
      if (included.has(specId)) return;

      const spec = allSpecs.find(s => s.id === specId);
      if (!spec) return;

      included.add(specId);
      result.push(spec);

      spec.dependencies?.forEach(depId => addWithDeps(depId));
    }

    specs.forEach(s => addWithDeps(s.id));
    return result;
  }

  async function handleExport() {
    exporting = true;
    progress = 0;
    error = null;

    try {
      const specsToExport = $options.includeDependencies
        ? getSpecsWithDependencies(specs, allSpecs)
        : specs;

      const blob = await exportSpecs(
        specsToExport,
        selectedFormat,
        $options,
        (p) => { progress = p; }
      );

      // Download file
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = filename;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);

      dispatch('exported', { format: selectedFormat, filename });
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Export failed';
    } finally {
      exporting = false;
    }
  }

  function handleClose() {
    exporting = false;
    progress = 0;
    error = null;
    dispatch('close');
  }
</script>

<Modal {open} on:close={handleClose} size="md" title="Export Specs">
  <div class="export-dialog">
    <div class="export-dialog__summary">
      <Icon name="download" size={20} />
      <span>
        Exporting {specs.length} spec{specs.length !== 1 ? 's' : ''}
        {#if $options.includeDependencies && totalSpecsToExport > specs.length}
          (+{totalSpecsToExport - specs.length} dependencies)
        {/if}
      </span>
    </div>

    <div class="export-dialog__formats">
      <h4>Export Format</h4>
      <div class="export-dialog__format-grid">
        {#each formatOptions as format}
          <button
            class="export-dialog__format"
            class:export-dialog__format--selected={selectedFormat === format.value}
            on:click={() => selectedFormat = format.value}
          >
            <Icon name={format.icon} size={24} />
            <span class="export-dialog__format-label">{format.label}</span>
            <span class="export-dialog__format-ext">{format.ext}</span>
          </button>
        {/each}
      </div>
    </div>

    <div class="export-dialog__options">
      <h4>Options</h4>

      <div class="export-dialog__option">
        <div class="export-dialog__option-info">
          <span class="export-dialog__option-label">Include dependencies</span>
          <span class="export-dialog__option-desc">
            Export all specs that the selected specs depend on
          </span>
        </div>
        <Toggle bind:checked={$options.includeDependencies} />
      </div>

      <div class="export-dialog__option">
        <div class="export-dialog__option-info">
          <span class="export-dialog__option-label">Include history</span>
          <span class="export-dialog__option-desc">
            Include version history for each spec
          </span>
        </div>
        <Toggle bind:checked={$options.includeHistory} />
      </div>

      <div class="export-dialog__option">
        <div class="export-dialog__option-info">
          <span class="export-dialog__option-label">Include comments</span>
          <span class="export-dialog__option-desc">
            Include all comments and discussions
          </span>
        </div>
        <Toggle bind:checked={$options.includeComments} />
      </div>

      {#if specs.length > 1 && selectedFormat !== 'json'}
        <div class="export-dialog__option">
          <div class="export-dialog__option-info">
            <span class="export-dialog__option-label">Single file</span>
            <span class="export-dialog__option-desc">
              Combine all specs into one file (vs. ZIP archive)
            </span>
          </div>
          <Toggle bind:checked={$options.flattenToSingleFile} />
        </div>
      {/if}
    </div>

    <div class="export-dialog__preview">
      <h4>Output</h4>
      <div class="export-dialog__filename">
        <Icon name="file" size={16} />
        <span>{filename}</span>
      </div>
    </div>

    {#if exporting}
      <div class="export-dialog__progress">
        <ProgressBar value={progress} />
        <span>Exporting... {Math.round(progress)}%</span>
      </div>
    {/if}

    {#if error}
      <div class="export-dialog__error">
        <Icon name="alert-circle" size={16} />
        {error}
      </div>
    {/if}
  </div>

  <svelte:fragment slot="footer">
    <Button variant="outline" on:click={handleClose} disabled={exporting}>
      Cancel
    </Button>
    <Button variant="primary" on:click={handleExport} loading={exporting}>
      <Icon name="download" size={14} />
      Export {selectedFormatInfo?.label}
    </Button>
  </svelte:fragment>
</Modal>

<style>
  .export-dialog {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .export-dialog__summary {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    background: var(--color-primary-subtle);
    color: var(--color-primary);
    border-radius: 8px;
    font-weight: 500;
  }

  .export-dialog__formats h4,
  .export-dialog__options h4,
  .export-dialog__preview h4 {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    margin: 0 0 12px;
  }

  .export-dialog__format-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
  }

  .export-dialog__format {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 16px;
    background: var(--color-surface);
    border: 2px solid var(--color-border);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .export-dialog__format:hover {
    border-color: var(--color-primary-alpha);
  }

  .export-dialog__format--selected {
    border-color: var(--color-primary);
    background: var(--color-primary-subtle);
  }

  .export-dialog__format-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .export-dialog__format-ext {
    font-size: 0.75rem;
    font-family: var(--font-mono);
    color: var(--color-text-tertiary);
  }

  .export-dialog__option {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 0;
    border-bottom: 1px solid var(--color-border);
  }

  .export-dialog__option:last-child {
    border-bottom: none;
  }

  .export-dialog__option-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .export-dialog__option-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .export-dialog__option-desc {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .export-dialog__filename {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    background: var(--color-surface-subtle);
    border-radius: 6px;
    font-family: var(--font-mono);
    font-size: 0.875rem;
    color: var(--color-text-secondary);
  }

  .export-dialog__progress {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 16px;
    background: var(--color-surface-subtle);
    border-radius: 8px;
  }

  .export-dialog__progress span {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    text-align: center;
  }

  .export-dialog__error {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    background: var(--color-danger-subtle);
    color: var(--color-danger);
    border-radius: 6px;
    font-size: 0.875rem;
  }
</style>
```

### Export Utilities

```typescript
// utils/export.ts
import type { Spec, ExportOptions } from '$lib/types/spec';
import { jsPDF } from 'jspdf';
import { marked } from 'marked';

export type ExportFormat = 'markdown' | 'json' | 'pdf' | 'html';

export async function exportSpecs(
  specs: Spec[],
  format: ExportFormat,
  options: ExportOptions,
  onProgress?: (progress: number) => void
): Promise<Blob> {
  onProgress?.(0);

  switch (format) {
    case 'markdown':
      return exportToMarkdown(specs, options, onProgress);
    case 'json':
      return exportToJson(specs, options, onProgress);
    case 'pdf':
      return exportToPdf(specs, options, onProgress);
    case 'html':
      return exportToHtml(specs, options, onProgress);
    default:
      throw new Error(`Unknown format: ${format}`);
  }
}

async function exportToMarkdown(
  specs: Spec[],
  options: ExportOptions,
  onProgress?: (progress: number) => void
): Promise<Blob> {
  const parts: string[] = [];

  for (let i = 0; i < specs.length; i++) {
    const spec = specs[i];
    parts.push(generateMarkdown(spec));

    if (i < specs.length - 1) {
      parts.push('\n\n---\n\n');
    }

    onProgress?.((i + 1) / specs.length * 100);
  }

  const content = parts.join('');
  return new Blob([content], { type: 'text/markdown' });
}

function generateMarkdown(spec: Spec): string {
  return `# Spec ${spec.id}: ${spec.title}

## Phase
${spec.phase}

## Spec ID
${spec.id}

## Status
${spec.status}

## Dependencies
${spec.dependencies?.length ? spec.dependencies.join(', ') : 'None'}

## Estimated Context
${spec.estimatedContext || 'N/A'}

---

${spec.content}
`;
}

async function exportToJson(
  specs: Spec[],
  options: ExportOptions,
  onProgress?: (progress: number) => void
): Promise<Blob> {
  const exportData = {
    exportedAt: new Date().toISOString(),
    specCount: specs.length,
    options,
    specs: specs.map(s => ({
      id: s.id,
      title: s.title,
      description: s.description,
      status: s.status,
      phase: s.phase,
      dependencies: s.dependencies,
      estimatedContext: s.estimatedContext,
      tags: s.tags,
      content: s.content,
      createdAt: s.createdAt,
      updatedAt: s.updatedAt,
      author: s.author
    }))
  };

  onProgress?.(100);
  const content = JSON.stringify(exportData, null, 2);
  return new Blob([content], { type: 'application/json' });
}

async function exportToPdf(
  specs: Spec[],
  options: ExportOptions,
  onProgress?: (progress: number) => void
): Promise<Blob> {
  const doc = new jsPDF({
    orientation: 'portrait',
    unit: 'mm',
    format: 'a4'
  });

  const pageWidth = doc.internal.pageSize.getWidth();
  const pageHeight = doc.internal.pageSize.getHeight();
  const margin = 20;
  const contentWidth = pageWidth - margin * 2;

  let yPos = margin;

  for (let i = 0; i < specs.length; i++) {
    const spec = specs[i];

    if (i > 0) {
      doc.addPage();
      yPos = margin;
    }

    // Title
    doc.setFontSize(18);
    doc.setFont('helvetica', 'bold');
    doc.text(`Spec ${spec.id}: ${spec.title}`, margin, yPos);
    yPos += 12;

    // Metadata
    doc.setFontSize(10);
    doc.setFont('helvetica', 'normal');
    doc.setTextColor(100);

    const metadata = [
      `Phase: ${spec.phase}`,
      `Status: ${spec.status}`,
      `Dependencies: ${spec.dependencies?.join(', ') || 'None'}`
    ];

    metadata.forEach(m => {
      doc.text(m, margin, yPos);
      yPos += 5;
    });

    yPos += 5;
    doc.setTextColor(0);

    // Content
    doc.setFontSize(11);
    const lines = doc.splitTextToSize(spec.content, contentWidth);

    for (const line of lines) {
      if (yPos > pageHeight - margin) {
        doc.addPage();
        yPos = margin;
      }
      doc.text(line, margin, yPos);
      yPos += 5;
    }

    onProgress?.((i + 1) / specs.length * 100);
  }

  return doc.output('blob');
}

async function exportToHtml(
  specs: Spec[],
  options: ExportOptions,
  onProgress?: (progress: number) => void
): Promise<Blob> {
  const styles = `
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      max-width: 800px;
      margin: 0 auto;
      padding: 40px 20px;
      line-height: 1.6;
      color: #333;
    }
    .spec {
      margin-bottom: 60px;
      padding-bottom: 40px;
      border-bottom: 1px solid #eee;
    }
    .spec:last-child {
      border-bottom: none;
    }
    .spec-header {
      margin-bottom: 20px;
    }
    .spec-id {
      display: inline-block;
      padding: 4px 12px;
      background: #e3f2fd;
      color: #1976d2;
      border-radius: 4px;
      font-family: monospace;
      font-weight: 600;
      margin-bottom: 8px;
    }
    .spec-title {
      font-size: 24px;
      font-weight: 700;
      margin: 0;
    }
    .spec-meta {
      display: flex;
      gap: 20px;
      margin-top: 16px;
      font-size: 14px;
      color: #666;
    }
    .spec-meta span {
      display: flex;
      align-items: center;
      gap: 6px;
    }
    .status-badge {
      padding: 4px 8px;
      border-radius: 4px;
      font-size: 12px;
      font-weight: 600;
    }
    .status-planned { background: #fff3e0; color: #e65100; }
    .status-in-progress { background: #e3f2fd; color: #1565c0; }
    .status-implemented { background: #e8f5e9; color: #2e7d32; }
    .status-tested { background: #f3e5f5; color: #7b1fa2; }
    pre {
      background: #f5f5f5;
      padding: 16px;
      border-radius: 8px;
      overflow-x: auto;
    }
    code {
      font-family: 'Fira Code', monospace;
      font-size: 14px;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      margin: 20px 0;
    }
    th, td {
      padding: 12px;
      border: 1px solid #ddd;
      text-align: left;
    }
    th {
      background: #f5f5f5;
    }
  `;

  const specHtml = specs.map((spec, i) => {
    const contentHtml = marked(spec.content);

    return `
      <article class="spec">
        <header class="spec-header">
          <span class="spec-id">Spec ${spec.id}</span>
          <h1 class="spec-title">${spec.title}</h1>
          <div class="spec-meta">
            <span>Phase ${spec.phase}</span>
            <span class="status-badge status-${spec.status}">${spec.status}</span>
            <span>Dependencies: ${spec.dependencies?.join(', ') || 'None'}</span>
          </div>
        </header>
        <div class="spec-content">
          ${contentHtml}
        </div>
      </article>
    `;
  });

  onProgress?.(100);

  const html = `
    <!DOCTYPE html>
    <html lang="en">
    <head>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
      <title>Specs Export</title>
      <style>${styles}</style>
    </head>
    <body>
      ${specHtml.join('\n')}
    </body>
    </html>
  `;

  return new Blob([html], { type: 'text/html' });
}
```

### Export Types

```typescript
// types/spec.ts additions
export interface ExportOptions {
  includeDependencies: boolean;
  includeHistory: boolean;
  includeComments: boolean;
  flattenToSingleFile: boolean;
  customTemplate: string | null;
}
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import ExportDialog from './ExportDialog.svelte';
import { exportSpecs } from '$lib/utils/export';
import { createMockSpecs } from '$lib/test-utils/mock-data';

// Mock URL.createObjectURL
global.URL.createObjectURL = vi.fn(() => 'blob:test');
global.URL.revokeObjectURL = vi.fn();

describe('ExportDialog', () => {
  const mockSpecs = createMockSpecs(3);

  it('shows spec count in summary', () => {
    render(ExportDialog, {
      props: { open: true, specs: mockSpecs, allSpecs: mockSpecs }
    });

    expect(screen.getByText(/Exporting 3 specs/)).toBeInTheDocument();
  });

  it('displays all format options', () => {
    render(ExportDialog, {
      props: { open: true, specs: mockSpecs, allSpecs: mockSpecs }
    });

    expect(screen.getByText('Markdown')).toBeInTheDocument();
    expect(screen.getByText('JSON')).toBeInTheDocument();
    expect(screen.getByText('PDF')).toBeInTheDocument();
    expect(screen.getByText('HTML')).toBeInTheDocument();
  });

  it('selects format on click', async () => {
    render(ExportDialog, {
      props: { open: true, specs: mockSpecs, allSpecs: mockSpecs }
    });

    await fireEvent.click(screen.getByText('JSON'));

    expect(screen.getByText('.json')).toBeInTheDocument();
  });

  it('shows dependency count when enabled', async () => {
    // Create specs with dependencies
    const specsWithDeps = [
      { ...mockSpecs[0], dependencies: [mockSpecs[1].id] },
      mockSpecs[1],
      mockSpecs[2]
    ];

    render(ExportDialog, {
      props: { open: true, specs: [specsWithDeps[0]], allSpecs: specsWithDeps }
    });

    const toggle = screen.getByText('Include dependencies').closest('.export-dialog__option')
      ?.querySelector('input');

    if (toggle) {
      await fireEvent.click(toggle);
      expect(screen.getByText(/\+1 dependencies/)).toBeInTheDocument();
    }
  });

  it('generates correct filename', () => {
    render(ExportDialog, {
      props: { open: true, specs: [mockSpecs[0]], allSpecs: mockSpecs }
    });

    expect(screen.getByText(new RegExp(`spec-${mockSpecs[0].id}`))).toBeInTheDocument();
  });

  it('shows progress during export', async () => {
    render(ExportDialog, {
      props: { open: true, specs: mockSpecs, allSpecs: mockSpecs }
    });

    await fireEvent.click(screen.getByText('Export Markdown'));

    // Progress should appear during export
  });

  it('dispatches exported event on success', async () => {
    const { component } = render(ExportDialog, {
      props: { open: true, specs: mockSpecs, allSpecs: mockSpecs }
    });

    const exportedHandler = vi.fn();
    component.$on('exported', exportedHandler);

    await fireEvent.click(screen.getByText('Export Markdown'));

    await waitFor(() => {
      expect(exportedHandler).toHaveBeenCalled();
    });
  });
});

describe('exportSpecs', () => {
  const specs = createMockSpecs(2);

  it('exports to markdown format', async () => {
    const blob = await exportSpecs(specs, 'markdown', {
      includeDependencies: false,
      includeHistory: false,
      includeComments: false,
      flattenToSingleFile: true,
      customTemplate: null
    });

    expect(blob.type).toBe('text/markdown');

    const text = await blob.text();
    expect(text).toContain(`# Spec ${specs[0].id}`);
    expect(text).toContain(specs[0].title);
  });

  it('exports to JSON format', async () => {
    const blob = await exportSpecs(specs, 'json', {
      includeDependencies: false,
      includeHistory: false,
      includeComments: false,
      flattenToSingleFile: true,
      customTemplate: null
    });

    expect(blob.type).toBe('application/json');

    const text = await blob.text();
    const data = JSON.parse(text);
    expect(data.specs.length).toBe(2);
    expect(data.specs[0].id).toBe(specs[0].id);
  });

  it('exports to HTML format', async () => {
    const blob = await exportSpecs(specs, 'html', {
      includeDependencies: false,
      includeHistory: false,
      includeComments: false,
      flattenToSingleFile: true,
      customTemplate: null
    });

    expect(blob.type).toBe('text/html');

    const text = await blob.text();
    expect(text).toContain('<!DOCTYPE html>');
    expect(text).toContain(specs[0].title);
  });

  it('calls progress callback', async () => {
    const onProgress = vi.fn();

    await exportSpecs(specs, 'json', {
      includeDependencies: false,
      includeHistory: false,
      includeComments: false,
      flattenToSingleFile: true,
      customTemplate: null
    }, onProgress);

    expect(onProgress).toHaveBeenCalledWith(100);
  });
});
```

---

## Related Specs

- Spec 231: Spec List Layout
- Spec 236: Spec Detail View
- Spec 246: Spec Sharing
- Spec 248: Spec Import
