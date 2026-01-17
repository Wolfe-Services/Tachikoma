<script lang="ts">
  import type { ForgeSession } from '$lib/types/forge';
  import { forgeService, type ForgeOutputFormat } from '$lib/services/forgeService';
  import Icon from '$lib/components/common/Icon.svelte';
  import Spinner from '$lib/components/ui/Spinner/Spinner.svelte';

  export let session: ForgeSession | null = null;
  export let visible: boolean = false;

  let isExporting = false;
  let exportError: string | null = null;
  let previewContent: string | null = null;
  let previewFormat: ForgeOutputFormat | null = null;

  $: hasResults = session?.hasResults ?? false;
  $: completedRounds = session?.rounds.filter(r => r.status === 'completed') ?? [];
  $: totalContributions = completedRounds.reduce((sum, round) => sum + round.contributions.length, 0);
  $: totalCritiques = completedRounds.reduce((sum, round) => sum + round.critiques.length, 0);

  const outputFormats: { value: ForgeOutputFormat; label: string; icon: string }[] = [
    { value: 'markdown', label: 'Markdown', icon: 'file-text' },
    { value: 'json', label: 'JSON', icon: 'code' },
    { value: 'yaml', label: 'YAML', icon: 'file' },
    { value: 'html', label: 'HTML', icon: 'globe' },
    { value: 'beads', label: 'Beads Issue', icon: 'circle' }
  ];

  async function handleExport(format: ForgeOutputFormat) {
    if (!session) return;
    
    isExporting = true;
    exportError = null;
    
    try {
      const result = await forgeService.generateOutput({
        sessionId: session.id,
        format,
        includeMetadata: true,
        includeHistory: true,
        includeMetrics: true
      });
      
      // Download the file
      downloadFile(result.content, result.filename, getMimeType(format));
    } catch (error) {
      exportError = error instanceof Error ? error.message : 'Export failed';
    } finally {
      isExporting = false;
    }
  }

  async function handlePreview(format: ForgeOutputFormat) {
    if (!session) return;
    
    isExporting = true;
    exportError = null;
    
    try {
      const result = await forgeService.generateOutput({
        sessionId: session.id,
        format,
        includeMetadata: true
      });
      
      previewContent = result.content;
      previewFormat = format;
    } catch (error) {
      exportError = error instanceof Error ? error.message : 'Preview failed';
    } finally {
      isExporting = false;
    }
  }

  function closePreview() {
    previewContent = null;
    previewFormat = null;
  }

  function downloadFile(content: string, filename: string, mimeType: string) {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function getMimeType(format: ForgeOutputFormat): string {
    const mimeTypes: Record<ForgeOutputFormat, string> = {
      markdown: 'text/markdown',
      json: 'application/json',
      yaml: 'text/yaml',
      html: 'text/html',
      plain: 'text/plain',
      beads: 'text/yaml'
    };
    return mimeTypes[format] || 'text/plain';
  }

  function formatTimestamp(date: Date): string {
    return new Intl.DateTimeFormat('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    }).format(date);
  }

  function getContributionTypeColor(type: string): string {
    switch (type) {
      case 'proposal': return '#4a9eff';
      case 'refinement': return '#f39c12';
      case 'alternative': return '#e74c3c';
      default: return '#95a5a6';
    }
  }

  function getCritiqueColor(severity: string): string {
    switch (severity) {
      case 'critical': return '#e74c3c';
      case 'concern': return '#f39c12';
      case 'suggestion': return '#3498db';
      case 'info': return '#95a5a6';
      default: return '#95a5a6';
    }
  }
</script>

<div class="result-panel" class:visible>
  <header class="panel-header">
    <h2>Session Results</h2>
    {#if session}
      <div class="session-status">
        <span class="phase-badge phase-{session.phase}">{session.phase}</span>
      </div>
    {/if}
  </header>

  <div class="panel-content">
    {#if !session}
      <div class="empty-state">
        <p>No active session</p>
      </div>
    {:else if !hasResults}
      <div class="empty-state">
        <p>No results yet</p>
        <small>Results will appear as the session progresses</small>
      </div>
    {:else}
      <div class="results-summary">
        <div class="stat-card">
          <div class="stat-value">{completedRounds.length}</div>
          <div class="stat-label">Rounds Completed</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">{totalContributions}</div>
          <div class="stat-label">Contributions</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">{totalCritiques}</div>
          <div class="stat-label">Critiques</div>
        </div>
      </div>

      <div class="results-sections">
        <section class="rounds-section">
          <h3>Round History</h3>
          <div class="rounds-list">
            {#each completedRounds as round (round.id)}
              <div class="round-item">
                <div class="round-header">
                  <span class="round-number">Round {round.number}</span>
                  <span class="round-duration">
                    {#if round.endTime}
                      {Math.round((round.endTime.getTime() - round.startTime.getTime()) / 60000)}m
                    {/if}
                  </span>
                </div>
                
                <div class="round-contributions">
                  {#each round.contributions as contribution (contribution.id)}
                    <div class="contribution-item">
                      <div class="contribution-header">
                        <div 
                          class="contribution-type"
                          style="background-color: {getContributionTypeColor(contribution.type)}"
                        >
                          {contribution.type}
                        </div>
                        <span class="timestamp">
                          {formatTimestamp(contribution.timestamp)}
                        </span>
                      </div>
                      <div class="contribution-content">
                        {contribution.content.substring(0, 100)}...
                      </div>
                    </div>
                  {/each}
                </div>

                <div class="round-critiques">
                  {#each round.critiques as critique (critique.id)}
                    <div class="critique-item">
                      <div class="critique-header">
                        <div 
                          class="critique-severity"
                          style="background-color: {getCritiqueColor(critique.severity)}"
                        >
                          {critique.severity}
                        </div>
                        <span class="timestamp">
                          {formatTimestamp(critique.timestamp)}
                        </span>
                      </div>
                      <div class="critique-content">
                        {critique.content.substring(0, 80)}...
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        </section>

        <section class="insights-section">
          <h3>Key Insights</h3>
          <div class="insights-placeholder">
            <p>Insights will be generated automatically as the session progresses.</p>
          </div>
        </section>

        <!-- Export Section -->
        <section class="export-section">
          <h3>Export Results</h3>
          {#if exportError}
            <div class="export-error">
              <Icon name="alert-triangle" size={14} />
              <span>{exportError}</span>
            </div>
          {/if}
          
          <div class="export-buttons">
            {#each outputFormats as format}
              <div class="export-button-group">
                <button
                  class="export-btn"
                  on:click={() => handleExport(format.value)}
                  disabled={isExporting}
                  title="Download as {format.label}"
                >
                  {#if isExporting}
                    <Spinner size={14} />
                  {:else}
                    <Icon name={format.icon} size={16} />
                  {/if}
                  <span>{format.label}</span>
                </button>
                <button
                  class="preview-btn"
                  on:click={() => handlePreview(format.value)}
                  disabled={isExporting}
                  title="Preview {format.label}"
                >
                  <Icon name="eye" size={14} />
                </button>
              </div>
            {/each}
          </div>
        </section>
      </div>
    {/if}
  </div>

  <!-- Preview Modal -->
  {#if previewContent && previewFormat}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <div class="preview-overlay" on:click={closePreview}>
      <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
      <div class="preview-modal" on:click|stopPropagation={() => {}} role="dialog" aria-modal="true">
        <div class="preview-header">
          <h3>Preview: {previewFormat.toUpperCase()}</h3>
          <button class="close-btn" on:click={closePreview} aria-label="Close preview">
            <Icon name="x" size={18} />
          </button>
        </div>
        <div class="preview-content">
          <pre><code>{previewContent}</code></pre>
        </div>
        <div class="preview-actions">
          <button class="btn-secondary" on:click={closePreview}>Close</button>
          <button class="btn-primary" on:click={() => previewFormat && handleExport(previewFormat)}>
            <Icon name="download" size={16} />
            Download
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .result-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem;
    border-bottom: 1px solid var(--border-color, #2a2a4a);
    background: var(--panel-header-bg, #1a1a2e);
  }

  .panel-header h2 {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--forge-text, #eaeaea);
    margin: 0;
  }

  .session-status {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .phase-badge {
    padding: 0.25rem 0.5rem;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: capitalize;
  }

  .phase-idle { background: #374151; color: #9CA3AF; }
  .phase-configuring { background: #1E40AF; color: #DBEAFE; }
  .phase-drafting { background: #059669; color: #D1FAE5; }
  .phase-critiquing { background: #DC2626; color: #FEE2E2; }
  .phase-deliberating { background: #7C2D12; color: #FED7AA; }
  .phase-converging { background: #7C3AED; color: #EDE9FE; }
  .phase-completed { background: #166534; color: #DCFCE7; }
  .phase-paused { background: #CA8A04; color: #FEF3C7; }
  .phase-error { background: #DC2626; color: #FEE2E2; }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 2rem;
    color: var(--text-muted, #6b7280);
  }

  .empty-state small {
    margin-top: 0.5rem;
    font-size: 0.875rem;
    opacity: 0.7;
  }

  .results-summary {
    display: flex;
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .stat-card {
    flex: 1;
    background: var(--card-bg, #1e293b);
    border: 1px solid var(--border-color, #2a2a4a);
    border-radius: 8px;
    padding: 1rem;
    text-align: center;
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--accent-color, #4a9eff);
    margin-bottom: 0.25rem;
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-muted, #6b7280);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .results-sections {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  .rounds-section h3,
  .insights-section h3 {
    font-size: 1rem;
    font-weight: 600;
    color: var(--forge-text, #eaeaea);
    margin: 0 0 1rem 0;
  }

  .rounds-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .round-item {
    background: var(--card-bg, #1e293b);
    border: 1px solid var(--border-color, #2a2a4a);
    border-radius: 8px;
    padding: 1rem;
  }

  .round-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .round-number {
    font-weight: 600;
    color: var(--forge-text, #eaeaea);
  }

  .round-duration {
    font-size: 0.875rem;
    color: var(--text-muted, #6b7280);
  }

  .round-contributions,
  .round-critiques {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .contribution-item,
  .critique-item {
    background: var(--sub-card-bg, #0f172a);
    border-radius: 6px;
    padding: 0.75rem;
  }

  .contribution-header,
  .critique-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .contribution-type,
  .critique-severity {
    padding: 0.125rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: capitalize;
    color: white;
  }

  .timestamp {
    font-size: 0.75rem;
    color: var(--text-muted, #6b7280);
  }

  .contribution-content,
  .critique-content {
    font-size: 0.875rem;
    color: var(--forge-text, #eaeaea);
    line-height: 1.4;
  }

  .insights-placeholder {
    background: var(--card-bg, #1e293b);
    border: 1px dashed var(--border-color, #2a2a4a);
    border-radius: 8px;
    padding: 2rem;
    text-align: center;
    color: var(--text-muted, #6b7280);
  }

  /* Export Section */
  .export-section h3 {
    font-size: 1rem;
    font-weight: 600;
    color: var(--forge-text, #eaeaea);
    margin: 0 0 1rem 0;
  }

  .export-error {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    background: rgba(220, 38, 38, 0.15);
    border: 1px solid rgba(220, 38, 38, 0.3);
    border-radius: 8px;
    color: #fca5a5;
    font-size: 0.875rem;
    margin-bottom: 1rem;
  }

  .export-buttons {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .export-button-group {
    display: flex;
    gap: 0.25rem;
  }

  .export-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background: rgba(78, 205, 196, 0.1);
    border: 1px solid rgba(78, 205, 196, 0.25);
    border-radius: 8px 0 0 8px;
    color: var(--tachi-cyan, #4ecdc4);
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .export-btn:hover:not(:disabled) {
    background: rgba(78, 205, 196, 0.2);
    border-color: rgba(78, 205, 196, 0.4);
  }

  .export-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .preview-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.75rem;
    background: rgba(78, 205, 196, 0.1);
    border: 1px solid rgba(78, 205, 196, 0.25);
    border-left: none;
    border-radius: 0 8px 8px 0;
    color: var(--tachi-cyan, #4ecdc4);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .preview-btn:hover:not(:disabled) {
    background: rgba(78, 205, 196, 0.2);
  }

  .preview-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Preview Modal */
  .preview-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.75);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 2rem;
  }

  .preview-modal {
    width: 100%;
    max-width: 800px;
    max-height: 80vh;
    background: var(--bg-secondary, #161b22);
    border: 1px solid rgba(78, 205, 196, 0.25);
    border-radius: 16px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  }

  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    background: rgba(78, 205, 196, 0.06);
    border-bottom: 1px solid rgba(78, 205, 196, 0.15);
  }

  .preview-header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--tachi-cyan, #4ecdc4);
    letter-spacing: 0.5px;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: rgba(230, 237, 243, 0.6);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .close-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: rgba(230, 237, 243, 0.9);
  }

  .preview-content {
    flex: 1;
    overflow: auto;
    padding: 1rem;
  }

  .preview-content pre {
    margin: 0;
    padding: 1rem;
    background: rgba(13, 17, 23, 0.6);
    border-radius: 8px;
    overflow-x: auto;
  }

  .preview-content code {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 0.85rem;
    line-height: 1.5;
    color: rgba(230, 237, 243, 0.9);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .preview-actions {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
    padding: 1rem 1.25rem;
    border-top: 1px solid rgba(78, 205, 196, 0.15);
  }

  .btn-secondary {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 1rem;
    background: rgba(13, 17, 23, 0.35);
    border: 1px solid rgba(78, 205, 196, 0.18);
    border-radius: 8px;
    color: rgba(230, 237, 243, 0.85);
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-secondary:hover {
    background: rgba(78, 205, 196, 0.1);
    border-color: rgba(78, 205, 196, 0.35);
  }

  .btn-primary {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 1rem;
    background: linear-gradient(135deg, var(--tachi-cyan-dark, #2d7a7a), var(--tachi-cyan, #4ecdc4));
    border: 1px solid rgba(78, 205, 196, 0.5);
    border-radius: 8px;
    color: var(--bg-primary, #0d1117);
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-primary:hover {
    box-shadow: 0 0 20px rgba(78, 205, 196, 0.3);
  }

  @media (max-width: 768px) {
    .results-summary {
      flex-direction: column;
      gap: 0.5rem;
    }

    .stat-card {
      padding: 0.75rem;
    }

    .stat-value {
      font-size: 1.25rem;
    }

    .preview-modal {
      max-height: 90vh;
    }
  }
</style>