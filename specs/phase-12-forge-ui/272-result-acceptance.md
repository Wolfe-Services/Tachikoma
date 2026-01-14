# Spec 272: Result Acceptance

## Header
- **Spec ID**: 272
- **Phase**: 12 - Forge UI
- **Component**: Result Acceptance
- **Dependencies**: Spec 271 (Result Preview)
- **Status**: Draft

## Objective
Create a formal result acceptance workflow that enables stakeholders to review, approve, or reject finalized deliberation results with appropriate audit trails and acceptance criteria validation.

## Acceptance Criteria
- [x] Display final results with all supporting documentation
- [x] Provide approval workflow with multiple stakeholder support
- [x] Track acceptance status with timestamps
- [x] Enable conditional acceptance with noted concerns
- [x] Support rejection with required justification
- [x] Generate acceptance certificates/records
- [x] Handle partial acceptance scenarios
- [x] Integrate with external approval systems

## Implementation

### ResultAcceptance.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, fly, slide } from 'svelte/transition';
  import AcceptanceSummary from './AcceptanceSummary.svelte';
  import AcceptanceChecklist from './AcceptanceChecklist.svelte';
  import StakeholderApprovals from './StakeholderApprovals.svelte';
  import AcceptanceNotes from './AcceptanceNotes.svelte';
  import CertificateGenerator from './CertificateGenerator.svelte';
  import { resultAcceptanceStore } from '$lib/stores/resultAcceptance';
  import { notificationService } from '$lib/services/notifications';
  import type {
    AcceptanceRecord,
    AcceptanceStatus,
    AcceptanceCriteria,
    StakeholderApproval,
    AcceptanceNote
  } from '$lib/types/forge';

  export let sessionId: string;
  export let resultId: string;

  const dispatch = createEventDispatcher<{
    accepted: AcceptanceRecord;
    rejected: { reason: string; record: AcceptanceRecord };
    conditional: { conditions: string[]; record: AcceptanceRecord };
  }>();

  let acceptanceRecord = writable<AcceptanceRecord | null>(null);
  let showCertificate = writable<boolean>(false);
  let showNotes = writable<boolean>(false);
  let acceptanceNotes = writable<string>('');
  let conditions = writable<string[]>([]);
  let isSubmitting = writable<boolean>(false);

  const criteria = derived(acceptanceRecord, ($record) =>
    $record?.criteria || []
  );

  const criteriaStatus = derived(criteria, ($criteria) => {
    const total = $criteria.length;
    const met = $criteria.filter(c => c.status === 'met').length;
    const notMet = $criteria.filter(c => c.status === 'not_met').length;
    const partial = $criteria.filter(c => c.status === 'partial').length;

    return {
      total,
      met,
      notMet,
      partial,
      allMet: met === total,
      canAccept: notMet === 0
    };
  });

  const stakeholders = derived(acceptanceRecord, ($record) =>
    $record?.stakeholders || []
  );

  const approvalStatus = derived(stakeholders, ($stakeholders) => {
    const total = $stakeholders.length;
    const approved = $stakeholders.filter(s => s.status === 'approved').length;
    const rejected = $stakeholders.filter(s => s.status === 'rejected').length;
    const pending = $stakeholders.filter(s => s.status === 'pending').length;

    return {
      total,
      approved,
      rejected,
      pending,
      allApproved: approved === total && total > 0,
      hasRejection: rejected > 0
    };
  });

  const canAccept = derived(
    [criteriaStatus, approvalStatus],
    ([$criteria, $approvals]) =>
      $criteria.canAccept && !$approvals.hasRejection
  );

  async function loadAcceptanceRecord() {
    const record = await resultAcceptanceStore.getRecord(sessionId, resultId);
    acceptanceRecord.set(record);
  }

  async function updateCriteriaStatus(
    criteriaId: string,
    status: 'met' | 'not_met' | 'partial',
    notes?: string
  ) {
    await resultAcceptanceStore.updateCriteria(resultId, criteriaId, status, notes);
    await loadAcceptanceRecord();
  }

  async function submitApproval(approved: boolean, notes?: string) {
    isSubmitting.set(true);

    try {
      await resultAcceptanceStore.submitApproval(resultId, {
        approved,
        notes,
        timestamp: new Date()
      });
      await loadAcceptanceRecord();
    } finally {
      isSubmitting.set(false);
    }
  }

  async function acceptResult() {
    if (!$canAccept) return;

    isSubmitting.set(true);

    try {
      const record = await resultAcceptanceStore.accept(resultId, {
        notes: $acceptanceNotes,
        timestamp: new Date()
      });

      dispatch('accepted', record);

      notificationService.notify({
        type: 'success',
        message: 'Result accepted successfully'
      });

      showCertificate.set(true);
    } catch (error) {
      console.error('Failed to accept result:', error);
      notificationService.notify({
        type: 'error',
        message: 'Failed to accept result'
      });
    } finally {
      isSubmitting.set(false);
    }
  }

  async function acceptWithConditions() {
    if ($conditions.length === 0) return;

    isSubmitting.set(true);

    try {
      const record = await resultAcceptanceStore.acceptConditionally(resultId, {
        conditions: $conditions,
        notes: $acceptanceNotes,
        timestamp: new Date()
      });

      dispatch('conditional', { conditions: $conditions, record });

      notificationService.notify({
        type: 'info',
        message: 'Result accepted with conditions'
      });
    } catch (error) {
      console.error('Failed to accept result conditionally:', error);
    } finally {
      isSubmitting.set(false);
    }
  }

  async function rejectResult(reason: string) {
    if (!reason.trim()) return;

    isSubmitting.set(true);

    try {
      const record = await resultAcceptanceStore.reject(resultId, {
        reason,
        notes: $acceptanceNotes,
        timestamp: new Date()
      });

      dispatch('rejected', { reason, record });

      notificationService.notify({
        type: 'warning',
        message: 'Result has been rejected'
      });
    } catch (error) {
      console.error('Failed to reject result:', error);
    } finally {
      isSubmitting.set(false);
    }
  }

  function addCondition(condition: string) {
    if (condition.trim()) {
      conditions.update(c => [...c, condition.trim()]);
    }
  }

  function removeCondition(index: number) {
    conditions.update(c => c.filter((_, i) => i !== index));
  }

  onMount(() => {
    loadAcceptanceRecord();
  });
</script>

<div class="result-acceptance" data-testid="result-acceptance">
  <header class="acceptance-header">
    <h3>Result Acceptance</h3>
    <div class="status-badges">
      <span class="badge" class:success={$criteriaStatus.allMet} class:warning={!$criteriaStatus.allMet}>
        {$criteriaStatus.met}/{$criteriaStatus.total} criteria met
      </span>
      <span class="badge" class:success={$approvalStatus.allApproved} class:warning={$approvalStatus.pending > 0}>
        {$approvalStatus.approved}/{$approvalStatus.total} approvals
      </span>
    </div>
  </header>

  {#if $acceptanceRecord}
    <AcceptanceSummary
      record={$acceptanceRecord}
      criteriaStatus={$criteriaStatus}
      approvalStatus={$approvalStatus}
    />

    <div class="acceptance-sections">
      <section class="criteria-section">
        <h4>Acceptance Criteria</h4>
        <AcceptanceChecklist
          criteria={$criteria}
          on:update={(e) => updateCriteriaStatus(e.detail.id, e.detail.status, e.detail.notes)}
        />
      </section>

      <section class="approvals-section">
        <h4>Stakeholder Approvals</h4>
        <StakeholderApprovals
          stakeholders={$stakeholders}
          on:approve={() => submitApproval(true)}
          on:reject={(e) => submitApproval(false, e.detail)}
        />
      </section>
    </div>

    <div class="notes-section">
      <button
        class="notes-toggle"
        on:click={() => showNotes.update(v => !v)}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" stroke-width="2"/>
          <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" stroke-width="2"/>
        </svg>
        {$showNotes ? 'Hide Notes' : 'Add Notes'}
      </button>

      {#if $showNotes}
        <div class="notes-editor" transition:slide>
          <AcceptanceNotes
            bind:value={$acceptanceNotes}
            bind:conditions={$conditions}
            on:addCondition={(e) => addCondition(e.detail)}
            on:removeCondition={(e) => removeCondition(e.detail)}
          />
        </div>
      {/if}
    </div>

    <footer class="acceptance-footer">
      {#if $acceptanceRecord.status === 'pending'}
        <div class="action-buttons">
          <button
            class="btn reject"
            on:click={() => {
              const reason = prompt('Please provide a reason for rejection:');
              if (reason) rejectResult(reason);
            }}
            disabled={$isSubmitting}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <circle cx="12" cy="12" r="10" stroke-width="2"/>
              <path d="M15 9l-6 6M9 9l6 6" stroke-width="2" stroke-linecap="round"/>
            </svg>
            Reject
          </button>

          {#if !$criteriaStatus.allMet || $approvalStatus.hasRejection}
            <button
              class="btn conditional"
              on:click={acceptWithConditions}
              disabled={$isSubmitting || $conditions.length === 0}
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                <path d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" stroke-width="2"/>
              </svg>
              Accept with Conditions ({$conditions.length})
            </button>
          {/if}

          <button
            class="btn accept"
            on:click={acceptResult}
            disabled={!$canAccept || $isSubmitting}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" stroke-width="2"/>
            </svg>
            {$isSubmitting ? 'Processing...' : 'Accept Result'}
          </button>
        </div>

        {#if !$canAccept}
          <p class="warning-message">
            {#if $criteriaStatus.notMet > 0}
              {$criteriaStatus.notMet} criteria not met.
            {/if}
            {#if $approvalStatus.hasRejection}
              Result has been rejected by a stakeholder.
            {/if}
          </p>
        {/if}
      {:else}
        <div class="status-display">
          <span class="final-status {$acceptanceRecord.status}">
            {#if $acceptanceRecord.status === 'accepted'}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
              </svg>
              Accepted
            {:else if $acceptanceRecord.status === 'rejected'}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"/>
              </svg>
              Rejected
            {:else if $acceptanceRecord.status === 'conditional'}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
              </svg>
              Accepted with Conditions
            {/if}
          </span>

          <span class="status-timestamp">
            {new Date($acceptanceRecord.decidedAt).toLocaleString()}
          </span>

          {#if $acceptanceRecord.status === 'accepted'}
            <button
              class="view-cert-btn"
              on:click={() => showCertificate.set(true)}
            >
              View Certificate
            </button>
          {/if}
        </div>
      {/if}
    </footer>
  {:else}
    <div class="loading-state">
      <div class="loading-spinner"></div>
      <p>Loading acceptance record...</p>
    </div>
  {/if}

  {#if $showCertificate}
    <div class="modal-overlay" transition:fade on:click={() => showCertificate.set(false)}>
      <div
        class="modal-content"
        on:click|stopPropagation
        transition:fly={{ y: 50, duration: 200 }}
      >
        <CertificateGenerator
          record={$acceptanceRecord}
          on:close={() => showCertificate.set(false)}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .result-acceptance {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    padding: 1.5rem;
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
  }

  .acceptance-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .acceptance-header h3 {
    font-size: 1.125rem;
    font-weight: 600;
  }

  .status-badges {
    display: flex;
    gap: 0.5rem;
  }

  .badge {
    padding: 0.25rem 0.75rem;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .badge.success {
    background: var(--success-alpha);
    color: var(--success-color);
  }

  .badge.warning {
    background: var(--warning-alpha);
    color: var(--warning-color);
  }

  .acceptance-sections {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.5rem;
  }

  .criteria-section h4,
  .approvals-section h4 {
    font-size: 0.9375rem;
    font-weight: 600;
    margin-bottom: 1rem;
  }

  .notes-section {
    border-top: 1px solid var(--border-color);
    padding-top: 1rem;
  }

  .notes-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    color: var(--text-secondary);
    font-size: 0.875rem;
    cursor: pointer;
  }

  .notes-toggle:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .notes-editor {
    margin-top: 1rem;
  }

  .acceptance-footer {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color);
  }

  .action-buttons {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
  }

  .btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1.25rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn.accept {
    background: var(--success-color);
    color: white;
  }

  .btn.reject {
    background: var(--error-color);
    color: white;
  }

  .btn.conditional {
    background: var(--warning-color);
    color: white;
  }

  .warning-message {
    font-size: 0.875rem;
    color: var(--warning-color);
    text-align: right;
  }

  .status-display {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .final-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    font-weight: 600;
  }

  .final-status.accepted {
    background: var(--success-alpha);
    color: var(--success-color);
  }

  .final-status.rejected {
    background: var(--error-alpha);
    color: var(--error-color);
  }

  .final-status.conditional {
    background: var(--warning-alpha);
    color: var(--warning-color);
  }

  .status-timestamp {
    font-size: 0.8125rem;
    color: var(--text-muted);
  }

  .view-cert-btn {
    margin-left: auto;
    padding: 0.5rem 1rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    color: var(--text-primary);
    font-size: 0.875rem;
    cursor: pointer;
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 3rem;
    color: var(--text-muted);
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

  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background: var(--card-bg);
    border-radius: 8px;
    max-width: 800px;
    width: 90%;
    max-height: 90vh;
    overflow-y: auto;
  }

  @media (max-width: 768px) {
    .acceptance-sections {
      grid-template-columns: 1fr;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test acceptance workflow state transitions
2. **Integration Tests**: Verify multi-stakeholder approval flow
3. **Criteria Tests**: Test criteria status updates
4. **Certificate Tests**: Validate certificate generation
5. **Audit Tests**: Verify complete audit trail recording

## Related Specs
- Spec 271: Result Preview
- Spec 265: Decision Log UI
- Spec 273: History Browser
