# Spec 270: Human Intervention

## Header
- **Spec ID**: 270
- **Phase**: 12 - Forge UI
- **Component**: Human Intervention
- **Dependencies**: Spec 268 (Session Controls), Spec 269 (Pause/Resume)
- **Status**: Draft

## Objective
Create a comprehensive human intervention system that allows operators to inject guidance, override decisions, provide additional context, and steer deliberation when AI participants need human input.

## Acceptance Criteria
1. Provide intervention request triggers and notifications
2. Support multiple intervention types (guidance, override, context, clarification)
3. Display intervention queue with priority ordering
4. Track intervention history and impact
5. Allow inline responses during active deliberation
6. Support intervention approval workflows
7. Enable intervention templating for common scenarios
8. Record intervention audit trail for compliance

## Implementation

### HumanInterventionPanel.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, fly, slide } from 'svelte/transition';
  import InterventionRequest from './InterventionRequest.svelte';
  import InterventionForm from './InterventionForm.svelte';
  import InterventionHistory from './InterventionHistory.svelte';
  import InterventionTemplates from './InterventionTemplates.svelte';
  import { interventionStore } from '$lib/stores/intervention';
  import { notificationService } from '$lib/services/notifications';
  import type {
    Intervention,
    InterventionType,
    InterventionRequest as Request,
    InterventionTemplate
  } from '$lib/types/forge';

  export let sessionId: string;
  export let allowedTypes: InterventionType[] = ['guidance', 'override', 'context', 'clarification'];

  const dispatch = createEventDispatcher<{
    submit: Intervention;
    dismiss: { requestId: string };
    escalate: { requestId: string };
  }>();

  let activeRequest = writable<Request | null>(null);
  let showForm = writable<boolean>(false);
  let showHistory = writable<boolean>(false);
  let showTemplates = writable<boolean>(false);
  let selectedType = writable<InterventionType>('guidance');
  let unsubscribe: (() => void) | null = null;

  const pendingRequests = derived(interventionStore, ($store) =>
    $store.requests.filter(r => r.sessionId === sessionId && r.status === 'pending')
      .sort((a, b) => {
        // Sort by priority then by time
        const priorityOrder = { critical: 0, high: 1, medium: 2, low: 3 };
        const priorityDiff = priorityOrder[a.priority] - priorityOrder[b.priority];
        if (priorityDiff !== 0) return priorityDiff;
        return new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
      })
  );

  const interventionHistory = derived(interventionStore, ($store) =>
    $store.interventions.filter(i => i.sessionId === sessionId)
  );

  const interventionStats = derived(interventionHistory, ($history) => ({
    total: $history.length,
    byType: {
      guidance: $history.filter(i => i.type === 'guidance').length,
      override: $history.filter(i => i.type === 'override').length,
      context: $history.filter(i => i.type === 'context').length,
      clarification: $history.filter(i => i.type === 'clarification').length
    },
    impactful: $history.filter(i => i.impactLevel === 'high').length
  }));

  async function submitIntervention(intervention: Partial<Intervention>) {
    try {
      const fullIntervention: Intervention = {
        id: crypto.randomUUID(),
        sessionId,
        type: intervention.type || $selectedType,
        content: intervention.content || '',
        context: intervention.context,
        targetRound: intervention.targetRound,
        targetParticipants: intervention.targetParticipants,
        priority: intervention.priority || 'medium',
        requestId: $activeRequest?.id,
        createdAt: new Date(),
        createdBy: 'operator',
        status: 'pending',
        impactLevel: intervention.impactLevel || 'medium'
      };

      await interventionStore.submit(fullIntervention);

      // Mark request as handled if responding to one
      if ($activeRequest) {
        await interventionStore.resolveRequest($activeRequest.id, fullIntervention.id);
        activeRequest.set(null);
      }

      dispatch('submit', fullIntervention);
      showForm.set(false);

      notificationService.notify({
        type: 'success',
        message: 'Intervention submitted successfully'
      });
    } catch (error) {
      console.error('Failed to submit intervention:', error);
      notificationService.notify({
        type: 'error',
        message: 'Failed to submit intervention'
      });
    }
  }

  function selectRequest(request: Request) {
    activeRequest.set(request);
    selectedType.set(request.suggestedType || 'guidance');
    showForm.set(true);
  }

  function dismissRequest(requestId: string) {
    interventionStore.dismissRequest(requestId);
    dispatch('dismiss', { requestId });

    if ($activeRequest?.id === requestId) {
      activeRequest.set(null);
      showForm.set(false);
    }
  }

  function escalateRequest(requestId: string) {
    interventionStore.escalateRequest(requestId);
    dispatch('escalate', { requestId });
  }

  function applyTemplate(template: InterventionTemplate) {
    showTemplates.set(false);
    submitIntervention({
      type: template.type,
      content: template.content,
      priority: template.defaultPriority,
      impactLevel: template.expectedImpact
    });
  }

  function startQuickIntervention(type: InterventionType) {
    selectedType.set(type);
    activeRequest.set(null);
    showForm.set(true);
  }

  function getTypeIcon(type: InterventionType): string {
    switch (type) {
      case 'guidance': return 'compass';
      case 'override': return 'shield';
      case 'context': return 'info';
      case 'clarification': return 'help-circle';
      default: return 'message';
    }
  }

  function getTypeDescription(type: InterventionType): string {
    switch (type) {
      case 'guidance':
        return 'Provide direction or suggestions to guide deliberation';
      case 'override':
        return 'Override a decision or direction taken by participants';
      case 'context':
        return 'Add additional context or information participants should consider';
      case 'clarification':
        return 'Clarify goals, requirements, or previous interventions';
      default:
        return 'Provide input to the session';
    }
  }

  function subscribeToRequests() {
    unsubscribe = interventionStore.subscribeToRequests(sessionId, (request) => {
      // Notify of new high-priority requests
      if (request.priority === 'critical' || request.priority === 'high') {
        notificationService.notify({
          type: 'warning',
          message: `${request.priority.toUpperCase()}: Intervention requested`,
          action: {
            label: 'Respond',
            callback: () => selectRequest(request)
          }
        });
      }
    });
  }

  onMount(() => {
    interventionStore.loadForSession(sessionId);
    subscribeToRequests();
  });

  onDestroy(() => {
    if (unsubscribe) unsubscribe();
  });
</script>

<div class="human-intervention-panel" data-testid="human-intervention-panel">
  <header class="panel-header">
    <div class="header-title">
      <h3>Human Intervention</h3>
      {#if $pendingRequests.length > 0}
        <span class="request-badge">{$pendingRequests.length}</span>
      {/if}
    </div>
    <div class="header-actions">
      <button
        class="action-btn"
        on:click={() => showHistory.set(true)}
      >
        History ({$interventionStats.total})
      </button>
      <button
        class="action-btn"
        on:click={() => showTemplates.set(true)}
      >
        Templates
      </button>
    </div>
  </header>

  {#if $pendingRequests.length > 0}
    <div class="request-queue" transition:slide>
      <h4>Pending Requests</h4>
      <div class="request-list">
        {#each $pendingRequests as request (request.id)}
          <InterventionRequest
            {request}
            on:select={() => selectRequest(request)}
            on:dismiss={() => dismissRequest(request.id)}
            on:escalate={() => escalateRequest(request.id)}
          />
        {/each}
      </div>
    </div>
  {/if}

  <div class="quick-actions">
    <h4>Quick Intervention</h4>
    <div class="action-grid">
      {#each allowedTypes as type}
        <button
          class="quick-action-btn"
          on:click={() => startQuickIntervention(type)}
        >
          <span class="action-icon {type}">
            {#if type === 'guidance'}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                <circle cx="12" cy="12" r="10" stroke-width="2"/>
                <path d="M16.24 7.76l-2.12 6.36-6.36 2.12 2.12-6.36 6.36-2.12z" stroke-width="2"/>
              </svg>
            {:else if type === 'override'}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" stroke-width="2"/>
              </svg>
            {:else if type === 'context'}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                <circle cx="12" cy="12" r="10" stroke-width="2"/>
                <path d="M12 16v-4M12 8h.01" stroke-width="2" stroke-linecap="round"/>
              </svg>
            {:else if type === 'clarification'}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                <circle cx="12" cy="12" r="10" stroke-width="2"/>
                <path d="M9.09 9a3 3 0 015.83 1c0 2-3 3-3 3M12 17h.01" stroke-width="2" stroke-linecap="round"/>
              </svg>
            {/if}
          </span>
          <span class="action-label">{type}</span>
          <span class="action-description">{getTypeDescription(type)}</span>
        </button>
      {/each}
    </div>
  </div>

  {#if $interventionStats.total > 0}
    <div class="intervention-summary">
      <h4>Session Interventions</h4>
      <div class="summary-stats">
        <div class="stat">
          <span class="stat-value">{$interventionStats.total}</span>
          <span class="stat-label">Total</span>
        </div>
        <div class="stat">
          <span class="stat-value">{$interventionStats.byType.guidance}</span>
          <span class="stat-label">Guidance</span>
        </div>
        <div class="stat">
          <span class="stat-value">{$interventionStats.byType.override}</span>
          <span class="stat-label">Overrides</span>
        </div>
        <div class="stat highlight">
          <span class="stat-value">{$interventionStats.impactful}</span>
          <span class="stat-label">High Impact</span>
        </div>
      </div>
    </div>
  {/if}
</div>

{#if $showForm}
  <div class="modal-overlay" transition:fade on:click={() => showForm.set(false)}>
    <div
      class="modal-content"
      on:click|stopPropagation
      transition:fly={{ y: 50, duration: 200 }}
    >
      <InterventionForm
        type={$selectedType}
        request={$activeRequest}
        on:submit={(e) => submitIntervention(e.detail)}
        on:cancel={() => {
          showForm.set(false);
          activeRequest.set(null);
        }}
        on:typeChange={(e) => selectedType.set(e.detail)}
      />
    </div>
  </div>
{/if}

{#if $showHistory}
  <div class="modal-overlay" transition:fade on:click={() => showHistory.set(false)}>
    <div
      class="modal-content large"
      on:click|stopPropagation
      transition:fly={{ y: 50, duration: 200 }}
    >
      <InterventionHistory
        interventions={$interventionHistory}
        on:close={() => showHistory.set(false)}
      />
    </div>
  </div>
{/if}

{#if $showTemplates}
  <div class="modal-overlay" transition:fade on:click={() => showTemplates.set(false)}>
    <div
      class="modal-content"
      on:click|stopPropagation
      transition:fly={{ y: 50, duration: 200 }}
    >
      <InterventionTemplates
        on:select={(e) => applyTemplate(e.detail)}
        on:close={() => showTemplates.set(false)}
      />
    </div>
  </div>
{/if}

<style>
  .human-intervention-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1.25rem;
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
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

  .request-badge {
    padding: 0.125rem 0.5rem;
    background: var(--warning-color);
    color: white;
    border-radius: 10px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .header-actions {
    display: flex;
    gap: 0.5rem;
  }

  .action-btn {
    padding: 0.375rem 0.75rem;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    color: var(--text-secondary);
    font-size: 0.75rem;
    cursor: pointer;
  }

  .action-btn:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .request-queue {
    padding: 1rem;
    background: var(--warning-alpha);
    border: 1px solid var(--warning-color);
    border-radius: 6px;
  }

  .request-queue h4 {
    font-size: 0.875rem;
    font-weight: 600;
    margin-bottom: 0.75rem;
    color: var(--warning-color);
  }

  .request-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .quick-actions h4 {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
    margin-bottom: 0.75rem;
  }

  .action-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.75rem;
  }

  .quick-action-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 1rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    cursor: pointer;
    text-align: center;
    transition: all 0.15s ease;
  }

  .quick-action-btn:hover {
    border-color: var(--primary-color);
    background: var(--hover-bg);
  }

  .action-icon {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 8px;
    color: white;
  }

  .action-icon.guidance {
    background: var(--primary-color);
  }

  .action-icon.override {
    background: var(--error-color);
  }

  .action-icon.context {
    background: var(--info-color);
  }

  .action-icon.clarification {
    background: var(--warning-color);
  }

  .action-label {
    font-weight: 500;
    font-size: 0.875rem;
    text-transform: capitalize;
  }

  .action-description {
    font-size: 0.75rem;
    color: var(--text-muted);
    line-height: 1.4;
  }

  .intervention-summary h4 {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
    margin-bottom: 0.75rem;
  }

  .summary-stats {
    display: flex;
    gap: 1rem;
  }

  .stat {
    text-align: center;
    padding: 0.5rem 1rem;
    background: var(--secondary-bg);
    border-radius: 4px;
  }

  .stat-value {
    display: block;
    font-size: 1.125rem;
    font-weight: 600;
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .stat.highlight .stat-value {
    color: var(--primary-color);
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
    max-width: 600px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
  }

  .modal-content.large {
    max-width: 900px;
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test intervention submission and request handling
2. **Integration Tests**: Verify intervention impacts session state
3. **Priority Tests**: Test request queue ordering
4. **Template Tests**: Validate template application
5. **Audit Tests**: Verify intervention history recording

## Related Specs
- Spec 268: Session Controls
- Spec 269: Pause/Resume
- Spec 265: Decision Log UI
