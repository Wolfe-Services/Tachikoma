# 310 - Export Reports

**Phase:** 14 - Dashboard
**Spec ID:** 310
**Status:** Planned
**Dependencies:** 296-dashboard-layout, 300-cost-summary
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Create export and reporting functionality for dashboard data, including PDF report generation, CSV exports, and scheduled report delivery.

---

## Acceptance Criteria

- [x] `ExportModal.svelte` component created
- [x] PDF report generation
- [x] CSV data export
- [x] JSON data export
- [x] Date range selection
- [x] Custom field selection
- [x] Scheduled report configuration
- [x] Export history tracking

---

## Implementation Details

### 1. Export Modal Component (web/src/lib/components/export/ExportModal.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fly, fade } from 'svelte/transition';
  import type { ExportConfig, ExportFormat, DateRange } from '$lib/types/export';
  import Icon from '$lib/components/common/Icon.svelte';
  import DateRangePicker from '$lib/components/common/DateRangePicker.svelte';
  import MultiSelect from '$lib/components/common/MultiSelect.svelte';

  export let open: boolean = false;
  export let dataType: 'missions' | 'costs' | 'metrics' | 'errors' = 'missions';

  const dispatch = createEventDispatcher<{
    close: void;
    export: ExportConfig;
  }>();

  let format: ExportFormat = 'csv';
  let dateRange: DateRange = {
    start: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000),
    end: new Date()
  };
  let selectedFields: string[] = [];
  let includeCharts: boolean = true;
  let exporting: boolean = false;

  const formatOptions: Array<{ value: ExportFormat; label: string; icon: string; desc: string }> = [
    { value: 'csv', label: 'CSV', icon: 'file-text', desc: 'Comma-separated values for spreadsheets' },
    { value: 'json', label: 'JSON', icon: 'code', desc: 'Machine-readable data format' },
    { value: 'pdf', label: 'PDF Report', icon: 'file', desc: 'Formatted report with charts' }
  ];

  const fieldOptions: Record<string, Array<{ value: string; label: string }>> = {
    missions: [
      { value: 'id', label: 'Mission ID' },
      { value: 'title', label: 'Title' },
      { value: 'spec_id', label: 'Spec ID' },
      { value: 'state', label: 'State' },
      { value: 'created_at', label: 'Created At' },
      { value: 'completed_at', label: 'Completed At' },
      { value: 'duration', label: 'Duration' },
      { value: 'token_usage', label: 'Token Usage' },
      { value: 'cost', label: 'Cost' },
      { value: 'error', label: 'Error Details' }
    ],
    costs: [
      { value: 'date', label: 'Date' },
      { value: 'mission_id', label: 'Mission ID' },
      { value: 'model', label: 'Model' },
      { value: 'input_tokens', label: 'Input Tokens' },
      { value: 'output_tokens', label: 'Output Tokens' },
      { value: 'cost', label: 'Cost' }
    ],
    metrics: [
      { value: 'timestamp', label: 'Timestamp' },
      { value: 'metric_type', label: 'Metric Type' },
      { value: 'value', label: 'Value' },
      { value: 'unit', label: 'Unit' }
    ],
    errors: [
      { value: 'timestamp', label: 'Timestamp' },
      { value: 'type', label: 'Error Type' },
      { value: 'message', label: 'Message' },
      { value: 'severity', label: 'Severity' },
      { value: 'mission_id', label: 'Mission ID' },
      { value: 'stack_trace', label: 'Stack Trace' }
    ]
  };

  $: availableFields = fieldOptions[dataType] || [];
  $: if (selectedFields.length === 0) {
    selectedFields = availableFields.map(f => f.value);
  }

  async function handleExport() {
    exporting = true;
    try {
      const config: ExportConfig = {
        format,
        dataType,
        dateRange,
        fields: selectedFields,
        includeCharts: format === 'pdf' && includeCharts
      };
      dispatch('export', config);
    } finally {
      exporting = false;
    }
  }

  function close() {
    dispatch('close');
  }

  function selectAllFields() {
    selectedFields = availableFields.map(f => f.value);
  }

  function deselectAllFields() {
    selectedFields = [];
  }
</script>

{#if open}
  <div class="modal-overlay" on:click|self={close} transition:fade={{ duration: 150 }}>
    <div class="modal" role="dialog" aria-modal="true" transition:fly={{ y: 20, duration: 200 }}>
      <div class="modal-header">
        <h2>Export Data</h2>
        <button class="close-btn" on:click={close} aria-label="Close">
          <Icon name="x" size={20} />
        </button>
      </div>

      <div class="modal-body">
        <section class="export-section">
          <h3>Export Format</h3>
          <div class="format-options">
            {#each formatOptions as option}
              <label class="format-option" class:selected={format === option.value}>
                <input
                  type="radio"
                  name="format"
                  value={option.value}
                  bind:group={format}
                />
                <div class="format-icon">
                  <Icon name={option.icon} size={24} />
                </div>
                <div class="format-info">
                  <span class="format-label">{option.label}</span>
                  <span class="format-desc">{option.desc}</span>
                </div>
                {#if format === option.value}
                  <Icon name="check" size={16} class="check-icon" />
                {/if}
              </label>
            {/each}
          </div>
        </section>

        <section class="export-section">
          <h3>Date Range</h3>
          <DateRangePicker bind:value={dateRange} />
        </section>

        <section class="export-section">
          <div class="section-header">
            <h3>Fields to Include</h3>
            <div class="field-actions">
              <button class="text-btn" on:click={selectAllFields}>Select All</button>
              <button class="text-btn" on:click={deselectAllFields}>Deselect All</button>
            </div>
          </div>
          <div class="field-grid">
            {#each availableFields as field}
              <label class="field-option">
                <input
                  type="checkbox"
                  value={field.value}
                  bind:group={selectedFields}
                />
                <span>{field.label}</span>
              </label>
            {/each}
          </div>
        </section>

        {#if format === 'pdf'}
          <section class="export-section">
            <h3>PDF Options</h3>
            <label class="checkbox-option">
              <input type="checkbox" bind:checked={includeCharts} />
              <span>Include charts and visualizations</span>
            </label>
          </section>
        {/if}
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary" on:click={close}>
          Cancel
        </button>
        <button
          class="btn btn-primary"
          on:click={handleExport}
          disabled={exporting || selectedFields.length === 0}
        >
          {#if exporting}
            <Icon name="loader" size={16} class="spinning" />
            Exporting...
          {:else}
            <Icon name="download" size={16} />
            Export {format.toUpperCase()}
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    width: 100%;
    max-width: 560px;
    max-height: 90vh;
    background: var(--bg-card);
    border-radius: 0.75rem;
    box-shadow: var(--shadow-xl);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--border-color);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .close-btn {
    padding: 0.375rem;
    border: none;
    background: transparent;
    border-radius: 0.375rem;
    cursor: pointer;
    color: var(--text-secondary);
  }

  .close-btn:hover {
    background: var(--bg-hover);
  }

  .modal-body {
    flex: 1;
    padding: 1.5rem;
    overflow-y: auto;
  }

  .export-section {
    margin-bottom: 1.5rem;
  }

  .export-section:last-child {
    margin-bottom: 0;
  }

  .export-section h3 {
    margin: 0 0 0.75rem;
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .section-header h3 {
    margin: 0;
  }

  .field-actions {
    display: flex;
    gap: 0.5rem;
  }

  .text-btn {
    padding: 0;
    border: none;
    background: transparent;
    font-size: 0.75rem;
    color: var(--accent-color);
    cursor: pointer;
  }

  .text-btn:hover {
    text-decoration: underline;
  }

  .format-options {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .format-option {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 0.5rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .format-option:hover {
    border-color: var(--accent-color);
  }

  .format-option.selected {
    border-color: var(--accent-color);
    background: var(--accent-color-light, rgba(59, 130, 246, 0.05));
  }

  .format-option input {
    display: none;
  }

  .format-icon {
    width: 2.5rem;
    height: 2.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-secondary);
    border-radius: 0.5rem;
    color: var(--text-secondary);
  }

  .format-option.selected .format-icon {
    background: var(--accent-color);
    color: white;
  }

  .format-info {
    flex: 1;
  }

  .format-label {
    display: block;
    font-weight: 500;
    color: var(--text-primary);
  }

  .format-desc {
    display: block;
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  :global(.check-icon) {
    color: var(--accent-color);
  }

  .field-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.5rem;
  }

  .field-option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
    border-radius: 0.375rem;
    font-size: 0.8125rem;
    color: var(--text-primary);
    cursor: pointer;
  }

  .field-option:hover {
    background: var(--bg-hover);
  }

  .checkbox-option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: var(--text-primary);
    cursor: pointer;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding: 1rem 1.5rem;
    border-top: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  .btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-secondary {
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
  }

  .btn-secondary:hover {
    background: var(--bg-hover);
  }

  .btn-primary {
    background: var(--accent-color);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  :global(.spinning) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
```

### 2. Export Service (web/src/lib/services/export.ts)

```typescript
import type { ExportConfig, ExportResult } from '$lib/types/export';

export async function exportData(config: ExportConfig): Promise<ExportResult> {
  const { format, dataType, dateRange, fields, includeCharts } = config;

  // Fetch data based on config
  const data = await fetchExportData(dataType, dateRange, fields);

  switch (format) {
    case 'csv':
      return generateCSV(data, fields);
    case 'json':
      return generateJSON(data);
    case 'pdf':
      return generatePDF(data, fields, includeCharts);
    default:
      throw new Error(`Unsupported format: ${format}`);
  }
}

async function fetchExportData(
  dataType: string,
  dateRange: { start: Date; end: Date },
  fields: string[]
): Promise<any[]> {
  const response = await fetch(`/api/export/${dataType}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ dateRange, fields })
  });

  if (!response.ok) {
    throw new Error('Failed to fetch export data');
  }

  return response.json();
}

function generateCSV(data: any[], fields: string[]): ExportResult {
  const headers = fields.join(',');
  const rows = data.map(item =>
    fields.map(field => {
      const value = item[field];
      if (typeof value === 'string' && value.includes(',')) {
        return `"${value}"`;
      }
      return value ?? '';
    }).join(',')
  );

  const content = [headers, ...rows].join('\n');
  const blob = new Blob([content], { type: 'text/csv' });

  return {
    blob,
    filename: `export-${Date.now()}.csv`,
    mimeType: 'text/csv'
  };
}

function generateJSON(data: any[]): ExportResult {
  const content = JSON.stringify(data, null, 2);
  const blob = new Blob([content], { type: 'application/json' });

  return {
    blob,
    filename: `export-${Date.now()}.json`,
    mimeType: 'application/json'
  };
}

async function generatePDF(
  data: any[],
  fields: string[],
  includeCharts: boolean
): Promise<ExportResult> {
  // Call server-side PDF generation
  const response = await fetch('/api/export/pdf', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ data, fields, includeCharts })
  });

  if (!response.ok) {
    throw new Error('Failed to generate PDF');
  }

  const blob = await response.blob();

  return {
    blob,
    filename: `report-${Date.now()}.pdf`,
    mimeType: 'application/pdf'
  };
}

export function downloadExport(result: ExportResult): void {
  const url = URL.createObjectURL(result.blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = result.filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
```

### 3. Export Types (web/src/lib/types/export.ts)

```typescript
export type ExportFormat = 'csv' | 'json' | 'pdf';

export interface DateRange {
  start: Date;
  end: Date;
}

export interface ExportConfig {
  format: ExportFormat;
  dataType: 'missions' | 'costs' | 'metrics' | 'errors';
  dateRange: DateRange;
  fields: string[];
  includeCharts?: boolean;
}

export interface ExportResult {
  blob: Blob;
  filename: string;
  mimeType: string;
}

export interface ScheduledExport {
  id: string;
  config: ExportConfig;
  schedule: 'daily' | 'weekly' | 'monthly';
  recipients: string[];
  lastRun: string | null;
  nextRun: string;
  enabled: boolean;
}
```

---

## Testing Requirements

1. Format selection updates correctly
2. Date range picker works
3. Field selection toggles properly
4. CSV export generates valid data
5. JSON export formats correctly
6. PDF generation calls API
7. Download triggers file save

---

## Related Specs

- Depends on: [296-dashboard-layout.md](296-dashboard-layout.md)
- Next: [311-dashboard-filters.md](311-dashboard-filters.md)
- Used by: All dashboard views with data
