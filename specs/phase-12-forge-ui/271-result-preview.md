# Spec 271: Result Preview

## Header
- **Spec ID**: 271
- **Phase**: 12 - Forge UI
- **Component**: Result Preview
- **Dependencies**: Spec 267 (Convergence Indicator)
- **Status**: Draft

## Objective
Create a result preview system that displays emerging consensus, synthesized outputs, and preliminary conclusions before final session completion, allowing stakeholders to preview and validate results.

## Acceptance Criteria
- [x] Display synthesized result as it develops through rounds
- [x] Show confidence levels for different result sections
- [x] Highlight areas still under deliberation
- [x] Provide diff view between preview versions
- [x] Enable inline feedback on preview content
- [x] Support preview export for stakeholder review
- [x] Track preview evolution across rounds
- [x] Display supporting evidence and rationale

## Implementation

### ResultPreview.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, slide } from 'svelte/transition';
  import PreviewSection from './PreviewSection.svelte';
  import ConfidenceIndicator from './ConfidenceIndicator.svelte';
  import PreviewDiff from './PreviewDiff.svelte';
  import PreviewFeedback from './PreviewFeedback.svelte';
  import EvidencePanel from './EvidencePanel.svelte';
  import MarkdownRenderer from '$lib/components/MarkdownRenderer.svelte';
  import { resultPreviewStore } from '$lib/stores/resultPreview';
  import type {
    ResultPreview as Preview,
    PreviewSection as Section,
    PreviewFeedback as Feedback,
    SupportingEvidence
  } from '$lib/types/forge';

  export let sessionId: string;
  export let roundNumber: number;

  const dispatch = createEventDispatcher<{
    feedback: { sectionId: string; feedback: Feedback };
    approve: { previewId: string };
    requestRevision: { sectionId: string; reason: string };
  }>();

  let showDiff = writable<boolean>(false);
  let showEvidence = writable<boolean>(false);
  let selectedSectionId = writable<string | null>(null);
  let previousVersionIndex = writable<number>(0);

  const preview = derived(resultPreviewStore, ($store) =>
    $store.previews.find(p => p.sessionId === sessionId)
  );

  const sections = derived(preview, ($preview) =>
    $preview?.sections || []
  );

  const overallConfidence = derived(sections, ($sections) => {
    if ($sections.length === 0) return 0;
    return $sections.reduce((sum, s) => sum + s.confidence, 0) / $sections.length;
  });

  const versionHistory = derived(preview, ($preview) =>
    $preview?.versionHistory || []
  );

  const selectedSection = derived(
    [sections, selectedSectionId],
    ([$sections, $id]) => $sections.find(s => s.id === $id) || null
  );

  const pendingSections = derived(sections, ($sections) =>
    $sections.filter(s => s.status === 'deliberating' || s.status === 'low_confidence')
  );

  const completedSections = derived(sections, ($sections) =>
    $sections.filter(s => s.status === 'converged' || s.status === 'high_confidence')
  );

  function selectSection(id: string) {
    selectedSectionId.set(id);
    showEvidence.set(true);
  }

  function submitFeedback(sectionId: string, feedback: Feedback) {
    dispatch('feedback', { sectionId, feedback });
    resultPreviewStore.addFeedback(sessionId, sectionId, feedback);
  }

  function approvePreview() {
    if (!$preview) return;
    dispatch('approve', { previewId: $preview.id });
    resultPreviewStore.approve($preview.id);
  }

  function requestRevision(sectionId: string, reason: string) {
    dispatch('requestRevision', { sectionId, reason });
    resultPreviewStore.requestRevision(sessionId, sectionId, reason);
  }

  function getConfidenceColor(confidence: number): string {
    if (confidence >= 0.8) return 'var(--success-color)';
    if (confidence >= 0.6) return 'var(--info-color)';
    if (confidence >= 0.4) return 'var(--warning-color)';
    return 'var(--error-color)';
  }

  function getStatusLabel(status: string): string {
    switch (status) {
      case 'converged': return 'Converged';
      case 'high_confidence': return 'High Confidence';
      case 'deliberating': return 'Still Deliberating';
      case 'low_confidence': return 'Needs Review';
      default: return status;
    }
  }

  async function exportPreview(format: 'markdown' | 'html' | 'pdf') {
    if (!$preview) return;
    await resultPreviewStore.export($preview.id, format);
  }
</script>

<div class="result-preview" data-testid="result-preview">
  <header class="preview-header">
    <div class="header-title">
      <h3>Result Preview</h3>
      <span class="round-badge">Round {roundNumber}</span>
    </div>

    <div class="header-stats">
      <ConfidenceIndicator
        value={$overallConfidence}
        label="Overall Confidence"
        compact
      />
      <div class="section-counts">
        <span class="count completed">{$completedSections.length} ready</span>
        <span class="count pending">{$pendingSections.length} pending</span>
      </div>
    </div>

    <div class="header-actions">
      <button
        class="action-btn"
        class:active={$showDiff}
        on:click={() => showDiff.update(v => !v)}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path d="M16 3h5v5M4 20L21 3M21 16v5h-5M15 15l6 6M4 4l5 5" stroke-width="2" stroke-linecap="round"/>
        </svg>
        Diff
      </button>
      <div class="export-dropdown">
        <button class="action-btn">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3" stroke-width="2" stroke-linecap="round"/>
          </svg>
          Export
        </button>
        <div class="dropdown-menu">
          <button on:click={() => exportPreview('markdown')}>Markdown</button>
          <button on:click={() => exportPreview('html')}>HTML</button>
          <button on:click={() => exportPreview('pdf')}>PDF</button>
        </div>
      </div>
    </div>
  </header>

  {#if $preview}
    {#if $showDiff && $versionHistory.length > 1}
      <div class="diff-controls" transition:slide>
        <label>
          Compare with:
          <select bind:value={$previousVersionIndex}>
            {#each $versionHistory.slice(0, -1) as version, i}
              <option value={i}>Round {version.roundNumber} ({new Date(version.timestamp).toLocaleTimeString()})</option>
            {/each}
          </select>
        </label>
      </div>

      <PreviewDiff
        current={$preview}
        previous={$versionHistory[$previousVersionIndex]}
      />
    {:else}
      <div class="preview-content">
        <div class="content-main">
          {#if $sections.length === 0}
            <div class="empty-state">
              <p>No preview content available yet</p>
              <p class="hint">Results will appear as deliberation progresses</p>
            </div>
          {:else}
            {#each $sections as section (section.id)}
              <PreviewSection
                {section}
                selected={$selectedSectionId === section.id}
                on:click={() => selectSection(section.id)}
                on:feedback={(e) => submitFeedback(section.id, e.detail)}
                on:requestRevision={(e) => requestRevision(section.id, e.detail)}
              />
            {/each}
          {/if}
        </div>

        {#if $showEvidence && $selectedSection}
          <aside class="evidence-sidebar" transition:slide={{ axis: 'x' }}>
            <EvidencePanel
              section={$selectedSection}
              on:close={() => showEvidence.set(false)}
            />
          </aside>
        {/if}
      </div>
    {/if}

    <footer class="preview-footer">
      <div class="footer-info">
        <span class="update-time">
          Last updated: {new Date($preview.updatedAt).toLocaleTimeString()}
        </span>
        {#if $preview.feedbackCount > 0}
          <span class="feedback-count">
            {$preview.feedbackCount} feedback items
          </span>
        {/if}
      </div>

      <div class="footer-actions">
        {#if $pendingSections.length > 0}
          <p class="pending-notice">
            {$pendingSections.length} section(s) still under deliberation
          </p>
        {:else}
          <button
            class="approve-btn"
            on:click={approvePreview}
            disabled={$overallConfidence < 0.6}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" stroke-width="2"/>
            </svg>
            Approve Preview
          </button>
        {/if}
      </div>
    </footer>
  {:else}
    <div class="loading-state">
      <div class="loading-spinner"></div>
      <p>Loading preview...</p>
    </div>
  {/if}
</div>

<style>
  .result-preview {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    overflow: hidden;
  }

  .preview-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border-color);
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .header-title h3 {
    font-size: 1rem;
    font-weight: 600;
  }

  .round-badge {
    padding: 0.25rem 0.5rem;
    background: var(--secondary-bg);
    border-radius: 4px;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .header-stats {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-left: auto;
  }

  .section-counts {
    display: flex;
    gap: 0.5rem;
  }

  .count {
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
  }

  .count.completed {
    background: var(--success-alpha);
    color: var(--success-color);
  }

  .count.pending {
    background: var(--warning-alpha);
    color: var(--warning-color);
  }

  .header-actions {
    display: flex;
    gap: 0.5rem;
  }

  .action-btn {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.5rem 0.75rem;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    color: var(--text-secondary);
    font-size: 0.8125rem;
    cursor: pointer;
  }

  .action-btn:hover,
  .action-btn.active {
    background: var(--secondary-bg);
    color: var(--text-primary);
  }

  .export-dropdown {
    position: relative;
  }

  .dropdown-menu {
    display: none;
    position: absolute;
    right: 0;
    top: 100%;
    background: var(--dropdown-bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    min-width: 120px;
    z-index: 10;
  }

  .export-dropdown:hover .dropdown-menu {
    display: block;
  }

  .dropdown-menu button {
    display: block;
    width: 100%;
    padding: 0.5rem 0.75rem;
    text-align: left;
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 0.8125rem;
    cursor: pointer;
  }

  .dropdown-menu button:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .diff-controls {
    padding: 0.75rem 1.25rem;
    background: var(--secondary-bg);
    border-bottom: 1px solid var(--border-color);
  }

  .diff-controls label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .diff-controls select {
    padding: 0.375rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.8125rem;
  }

  .preview-content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .content-main {
    flex: 1;
    overflow-y: auto;
    padding: 1.25rem;
  }

  .evidence-sidebar {
    width: 350px;
    border-left: 1px solid var(--border-color);
    overflow-y: auto;
  }

  .preview-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--border-color);
    background: var(--secondary-bg);
  }

  .footer-info {
    display: flex;
    gap: 1rem;
    font-size: 0.8125rem;
    color: var(--text-muted);
  }

  .footer-actions {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .pending-notice {
    font-size: 0.8125rem;
    color: var(--warning-color);
  }

  .approve-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1rem;
    background: var(--success-color);
    border: none;
    border-radius: 6px;
    color: white;
    font-weight: 500;
    cursor: pointer;
  }

  .approve-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .empty-state,
  .loading-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    color: var(--text-muted);
  }

  .hint {
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }

  .loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border-color);
    border-top-color: var(--primary-color);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test section confidence calculations
2. **Integration Tests**: Verify preview updates with round progression
3. **Diff Tests**: Validate diff comparison accuracy
4. **Export Tests**: Test export in all formats
5. **Feedback Tests**: Verify feedback submission and display

## Related Specs
- Spec 267: Convergence Indicator
- Spec 272: Result Acceptance
- Spec 265: Decision Log UI
