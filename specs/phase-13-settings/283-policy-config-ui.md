# Spec 283: Policy Config UI

## Header
- **Spec ID**: 283
- **Phase**: 13 - Settings UI
- **Component**: Policy Config UI
- **Dependencies**: Spec 276 (Settings Layout)
- **Status**: Draft

## Objective
Create a configuration interface for defining and managing content policies, safety guidelines, and behavioral rules that govern AI participant behavior during deliberation sessions.

## Acceptance Criteria
- [x] Configure content safety policies
- [x] Define topic restrictions and allowlists
- [x] Set output format requirements
- [x] Configure citation and sourcing rules
- [x] Define ethical guidelines
- [x] Set up approval workflows for policy changes
- [x] Create policy templates
- [x] Monitor policy violations

## Implementation

### PolicyConfigUI.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { slide, fade } from 'svelte/transition';
  import PolicyCard from './PolicyCard.svelte';
  import PolicyEditor from './PolicyEditor.svelte';
  import PolicyTemplates from './PolicyTemplates.svelte';
  import ViolationLog from './ViolationLog.svelte';
  import { policyConfigStore } from '$lib/stores/policyConfig';
  import type {
    Policy,
    PolicyCategory,
    PolicyRule,
    PolicyViolation
  } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    save: Policy[];
    create: Policy;
    update: Policy;
  }>();

  const categories: PolicyCategory[] = [
    { id: 'safety', name: 'Content Safety', icon: 'shield', description: 'Rules for safe content generation' },
    { id: 'topics', name: 'Topic Restrictions', icon: 'filter', description: 'Allowed and restricted topics' },
    { id: 'format', name: 'Output Format', icon: 'layout', description: 'Response format requirements' },
    { id: 'citations', name: 'Citations', icon: 'link', description: 'Sourcing and citation rules' },
    { id: 'ethics', name: 'Ethical Guidelines', icon: 'heart', description: 'Ethical behavior rules' },
    { id: 'custom', name: 'Custom Rules', icon: 'settings', description: 'User-defined policies' }
  ];

  let selectedCategory = writable<string>('safety');
  let showEditor = writable<boolean>(false);
  let editingPolicyId = writable<string | null>(null);
  let showTemplates = writable<boolean>(false);
  let showViolations = writable<boolean>(false);

  const policies = derived(policyConfigStore, ($store) => $store.policies);

  const policiesByCategory = derived(policies, ($policies) => {
    const grouped = new Map<string, Policy[]>();
    for (const category of categories) {
      grouped.set(category.id, $policies.filter(p => p.category === category.id));
    }
    return grouped;
  });

  const activePolicyCount = derived(policies, ($policies) =>
    $policies.filter(p => p.enabled).length
  );

  const violations = derived(policyConfigStore, ($store) => $store.violations);

  const recentViolations = derived(violations, ($violations) =>
    $violations.slice(0, 10)
  );

  function selectCategory(categoryId: string) {
    selectedCategory.set(categoryId);
  }

  function createPolicy() {
    editingPolicyId.set(null);
    showEditor.set(true);
  }

  function editPolicy(policyId: string) {
    editingPolicyId.set(policyId);
    showEditor.set(true);
  }

  function togglePolicy(policyId: string) {
    policyConfigStore.toggleEnabled(policyId);
  }

  async function savePolicy(policy: Policy) {
    if ($editingPolicyId) {
      await policyConfigStore.update(policy);
      dispatch('update', policy);
    } else {
      await policyConfigStore.add(policy);
      dispatch('create', policy);
    }
    showEditor.set(false);
    editingPolicyId.set(null);
  }

  async function deletePolicy(policyId: string) {
    if (confirm('Delete this policy? This cannot be undone.')) {
      await policyConfigStore.remove(policyId);
    }
  }

  function applyTemplate(template: Policy) {
    const policy: Policy = {
      ...template,
      id: crypto.randomUUID(),
      createdAt: new Date(),
      enabled: true
    };
    policyConfigStore.add(policy);
    showTemplates.set(false);
  }

  async function saveAllPolicies() {
    await policyConfigStore.save();
    dispatch('save', $policies);
  }

  onMount(() => {
    policyConfigStore.load();
  });
</script>

<div class="policy-config-ui" data-testid="policy-config-ui">
  <header class="config-header">
    <div class="header-title">
      <h2>Policy Configuration</h2>
      <p class="description">Define rules and guidelines for AI behavior</p>
    </div>

    <div class="header-stats">
      <div class="stat">
        <span class="stat-value">{$activePolicyCount}</span>
        <span class="stat-label">Active Policies</span>
      </div>
      {#if $violations.length > 0}
        <div class="stat warning">
          <span class="stat-value">{$violations.length}</span>
          <span class="stat-label">Violations</span>
        </div>
      {/if}
    </div>

    <div class="header-actions">
      <button class="btn secondary" on:click={() => showTemplates.set(true)}>
        Templates
      </button>
      <button class="btn secondary" on:click={() => showViolations.set(true)}>
        View Violations
      </button>
      <button class="btn primary" on:click={saveAllPolicies}>
        Save All
      </button>
    </div>
  </header>

  <div class="policy-layout">
    <nav class="category-nav">
      {#each categories as category}
        {@const categoryPolicies = $policiesByCategory.get(category.id) || []}
        <button
          class="category-btn"
          class:active={$selectedCategory === category.id}
          on:click={() => selectCategory(category.id)}
        >
          <span class="category-name">{category.name}</span>
          <span class="category-count">{categoryPolicies.filter(p => p.enabled).length}/{categoryPolicies.length}</span>
        </button>
      {/each}
    </nav>

    <main class="policy-content">
      {@const currentCategory = categories.find(c => c.id === $selectedCategory)}
      {@const currentPolicies = $policiesByCategory.get($selectedCategory) || []}

      <div class="content-header">
        <div class="category-info">
          <h3>{currentCategory?.name}</h3>
          <p>{currentCategory?.description}</p>
        </div>
        <button class="btn primary" on:click={createPolicy}>
          Add Policy
        </button>
      </div>

      {#if currentPolicies.length === 0}
        <div class="empty-state">
          <p>No policies in this category</p>
          <p class="hint">Create a policy or apply a template to get started</p>
        </div>
      {:else}
        <div class="policies-list">
          {#each currentPolicies as policy (policy.id)}
            <PolicyCard
              {policy}
              on:toggle={() => togglePolicy(policy.id)}
              on:edit={() => editPolicy(policy.id)}
              on:delete={() => deletePolicy(policy.id)}
            />
          {/each}
        </div>
      {/if}
    </main>
  </div>

  {#if $recentViolations.length > 0}
    <section class="recent-violations">
      <h3>Recent Policy Violations</h3>
      <div class="violations-preview">
        {#each $recentViolations.slice(0, 3) as violation}
          <div class="violation-item">
            <span class="violation-policy">{violation.policyName}</span>
            <span class="violation-time">
              {new Date(violation.timestamp).toLocaleTimeString()}
            </span>
          </div>
        {/each}
        {#if $recentViolations.length > 3}
          <button
            class="view-all-btn"
            on:click={() => showViolations.set(true)}
          >
            View all {$violations.length} violations
          </button>
        {/if}
      </div>
    </section>
  {/if}

  {#if $showEditor}
    <div class="modal-overlay" transition:fade on:click={() => showEditor.set(false)}>
      <div class="modal-content large" on:click|stopPropagation>
        <PolicyEditor
          policy={$editingPolicyId ? $policies.find(p => p.id === $editingPolicyId) : null}
          category={$selectedCategory}
          on:save={(e) => savePolicy(e.detail)}
          on:close={() => {
            showEditor.set(false);
            editingPolicyId.set(null);
          }}
        />
      </div>
    </div>
  {/if}

  {#if $showTemplates}
    <div class="modal-overlay" transition:fade on:click={() => showTemplates.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <PolicyTemplates
          category={$selectedCategory}
          on:apply={(e) => applyTemplate(e.detail)}
          on:close={() => showTemplates.set(false)}
        />
      </div>
    </div>
  {/if}

  {#if $showViolations}
    <div class="modal-overlay" transition:fade on:click={() => showViolations.set(false)}>
      <div class="modal-content large" on:click|stopPropagation>
        <ViolationLog
          violations={$violations}
          on:close={() => showViolations.set(false)}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .policy-config-ui {
    max-width: 1100px;
  }

  .config-header {
    display: flex;
    align-items: flex-start;
    gap: 2rem;
    margin-bottom: 1.5rem;
  }

  .header-title {
    flex: 1;
  }

  .header-title h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
  }

  .description {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .header-stats {
    display: flex;
    gap: 1rem;
  }

  .stat {
    text-align: center;
    padding: 0.75rem 1rem;
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
  }

  .stat.warning {
    border-color: var(--warning-color);
    background: var(--warning-alpha);
  }

  .stat-value {
    display: block;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .stat.warning .stat-value {
    color: var(--warning-color);
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .header-actions {
    display: flex;
    gap: 0.75rem;
  }

  .policy-layout {
    display: grid;
    grid-template-columns: 220px 1fr;
    gap: 1.5rem;
  }

  .category-nav {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .category-btn {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: transparent;
    border: none;
    border-radius: 6px;
    text-align: left;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .category-btn:hover {
    background: var(--hover-bg);
  }

  .category-btn.active {
    background: var(--primary-alpha);
    color: var(--primary-color);
  }

  .category-name {
    font-size: 0.875rem;
    font-weight: 500;
  }

  .category-count {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .category-btn.active .category-count {
    color: var(--primary-color);
  }

  .policy-content {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.5rem;
  }

  .content-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1.5rem;
  }

  .category-info h3 {
    font-size: 1.125rem;
    font-weight: 600;
    margin-bottom: 0.25rem;
  }

  .category-info p {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .policies-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .empty-state {
    text-align: center;
    padding: 3rem;
    color: var(--text-muted);
  }

  .hint {
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }

  .recent-violations {
    background: var(--card-bg);
    border: 1px solid var(--warning-color);
    border-radius: 8px;
    padding: 1.25rem;
    margin-top: 1.5rem;
  }

  .recent-violations h3 {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--warning-color);
    margin-bottom: 1rem;
  }

  .violations-preview {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .violation-item {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    background: var(--secondary-bg);
    border-radius: 4px;
    font-size: 0.875rem;
  }

  .violation-policy {
    font-weight: 500;
  }

  .violation-time {
    color: var(--text-muted);
  }

  .view-all-btn {
    padding: 0.5rem;
    background: transparent;
    border: none;
    color: var(--primary-color);
    font-size: 0.8125rem;
    cursor: pointer;
    text-align: center;
  }

  .btn {
    padding: 0.625rem 1.25rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
  }

  .btn.primary {
    background: var(--primary-color);
    color: white;
  }

  .btn.secondary {
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
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
    max-width: 800px;
  }

  @media (max-width: 768px) {
    .policy-layout {
      grid-template-columns: 1fr;
    }

    .category-nav {
      flex-direction: row;
      overflow-x: auto;
      padding-bottom: 0.5rem;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test policy CRUD operations
2. **Integration Tests**: Verify policy enforcement
3. **Violation Tests**: Test violation detection and logging
4. **Template Tests**: Test template application
5. **Category Tests**: Test category filtering

## Related Specs
- Spec 276: Settings Layout
- Spec 281: Loop Config UI
- Spec 270: Human Intervention
