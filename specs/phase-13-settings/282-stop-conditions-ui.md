# Spec 282: Stop Conditions UI

## Header
- **Spec ID**: 282
- **Phase**: 13 - Settings UI
- **Component**: Stop Conditions UI
- **Dependencies**: Spec 281 (Loop Config UI)
- **Status**: Draft

## Objective
Create a configuration interface for defining and managing stop conditions that determine when deliberation sessions should terminate, including convergence thresholds, time limits, and custom conditions.

## Acceptance Criteria
- [x] Configure convergence threshold for auto-stop
- [x] Set time-based stop conditions
- [x] Define round-based limits
- [x] Create custom stop condition rules
- [x] Set up cost-based limits
- [x] Configure consensus requirements
- [x] Preview condition triggers
- [x] Enable/disable individual conditions

## Implementation

### StopConditionsUI.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { slide } from 'svelte/transition';
  import ConditionCard from './ConditionCard.svelte';
  import CustomConditionBuilder from './CustomConditionBuilder.svelte';
  import ConditionPreview from './ConditionPreview.svelte';
  import { stopConditionsStore } from '$lib/stores/stopConditions';
  import type {
    StopCondition,
    ConditionType,
    ConditionOperator,
    CustomCondition
  } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    save: StopCondition[];
    test: { conditions: StopCondition[] };
  }>();

  let showCustomBuilder = writable<boolean>(false);
  let editingConditionId = writable<string | null>(null);
  let previewMode = writable<boolean>(false);

  const conditions = derived(stopConditionsStore, ($store) => $store.conditions);

  const activeConditions = derived(conditions, ($conditions) =>
    $conditions.filter(c => c.enabled)
  );

  const conditionGroups = derived(conditions, ($conditions) => {
    const groups = {
      convergence: $conditions.filter(c => c.type === 'convergence'),
      time: $conditions.filter(c => c.type === 'time'),
      rounds: $conditions.filter(c => c.type === 'rounds'),
      cost: $conditions.filter(c => c.type === 'cost'),
      consensus: $conditions.filter(c => c.type === 'consensus'),
      custom: $conditions.filter(c => c.type === 'custom')
    };
    return groups;
  });

  const builtInConditions: Partial<StopCondition>[] = [
    {
      type: 'convergence',
      name: 'Convergence Threshold',
      description: 'Stop when overall convergence reaches target',
      operator: 'gte',
      defaultValue: 0.8,
      unit: 'percent'
    },
    {
      type: 'time',
      name: 'Session Timeout',
      description: 'Stop after maximum session duration',
      operator: 'gte',
      defaultValue: 60,
      unit: 'minutes'
    },
    {
      type: 'rounds',
      name: 'Maximum Rounds',
      description: 'Stop after reaching round limit',
      operator: 'gte',
      defaultValue: 10,
      unit: 'rounds'
    },
    {
      type: 'cost',
      name: 'Cost Limit',
      description: 'Stop when estimated cost exceeds limit',
      operator: 'gte',
      defaultValue: 1.0,
      unit: 'dollars'
    },
    {
      type: 'consensus',
      name: 'Unanimous Agreement',
      description: 'Stop when all participants agree',
      operator: 'eq',
      defaultValue: 100,
      unit: 'percent'
    }
  ];

  function toggleCondition(conditionId: string) {
    stopConditionsStore.toggleEnabled(conditionId);
  }

  function updateConditionValue(conditionId: string, value: number) {
    stopConditionsStore.updateValue(conditionId, value);
  }

  function updateConditionOperator(conditionId: string, operator: ConditionOperator) {
    stopConditionsStore.updateOperator(conditionId, operator);
  }

  function addBuiltInCondition(template: Partial<StopCondition>) {
    const condition: StopCondition = {
      id: crypto.randomUUID(),
      type: template.type!,
      name: template.name!,
      description: template.description!,
      operator: template.operator!,
      value: template.defaultValue!,
      unit: template.unit!,
      enabled: true,
      priority: $conditions.length
    };

    stopConditionsStore.add(condition);
  }

  function removeCondition(conditionId: string) {
    if (confirm('Remove this stop condition?')) {
      stopConditionsStore.remove(conditionId);
    }
  }

  function saveCustomCondition(condition: CustomCondition) {
    const stopCondition: StopCondition = {
      id: crypto.randomUUID(),
      type: 'custom',
      name: condition.name,
      description: condition.description,
      operator: 'custom',
      value: 0,
      unit: 'custom',
      enabled: true,
      priority: $conditions.length,
      customLogic: condition.logic
    };

    stopConditionsStore.add(stopCondition);
    showCustomBuilder.set(false);
  }

  async function saveConditions() {
    await stopConditionsStore.save();
    dispatch('save', $conditions);
  }

  function testConditions() {
    dispatch('test', { conditions: $activeConditions });
    previewMode.set(true);
  }

  function formatValue(value: number, unit: string): string {
    switch (unit) {
      case 'percent':
        return `${(value * 100).toFixed(0)}%`;
      case 'minutes':
        return `${value} min`;
      case 'dollars':
        return `$${value.toFixed(2)}`;
      case 'rounds':
        return `${value} rounds`;
      default:
        return String(value);
    }
  }

  function getOperatorLabel(operator: ConditionOperator): string {
    switch (operator) {
      case 'gte': return 'reaches or exceeds';
      case 'lte': return 'falls to or below';
      case 'eq': return 'equals';
      case 'gt': return 'exceeds';
      case 'lt': return 'falls below';
      default: return operator;
    }
  }

  onMount(() => {
    stopConditionsStore.load();
  });
</script>

<div class="stop-conditions-ui" data-testid="stop-conditions-ui">
  <header class="config-header">
    <div class="header-title">
      <h2>Stop Conditions</h2>
      <p class="description">Configure when deliberation sessions should terminate</p>
    </div>

    <div class="header-actions">
      <button class="btn secondary" on:click={testConditions}>
        Preview
      </button>
      <button class="btn primary" on:click={saveConditions}>
        Save Conditions
      </button>
    </div>
  </header>

  <div class="conditions-summary">
    <div class="summary-stat">
      <span class="stat-value">{$activeConditions.length}</span>
      <span class="stat-label">Active Conditions</span>
    </div>
    <div class="summary-stat">
      <span class="stat-value">{$conditions.length}</span>
      <span class="stat-label">Total Configured</span>
    </div>
  </div>

  <section class="add-condition-section">
    <h3>Add Condition</h3>
    <div class="condition-templates">
      {#each builtInConditions.filter(t =>
        !$conditions.some(c => c.type === t.type && c.name === t.name)
      ) as template}
        <button
          class="template-btn"
          on:click={() => addBuiltInCondition(template)}
        >
          <span class="template-name">{template.name}</span>
          <span class="template-desc">{template.description}</span>
        </button>
      {/each}

      <button
        class="template-btn custom"
        on:click={() => showCustomBuilder.set(true)}
      >
        <span class="template-name">Custom Condition</span>
        <span class="template-desc">Create a custom stop rule</span>
      </button>
    </div>
  </section>

  <section class="active-conditions">
    <h3>Active Conditions</h3>

    {#if $conditions.length === 0}
      <div class="empty-state">
        <p>No stop conditions configured</p>
        <p class="hint">Add conditions to control when sessions stop</p>
      </div>
    {:else}
      <div class="conditions-list">
        {#each $conditions as condition (condition.id)}
          <ConditionCard
            {condition}
            on:toggle={() => toggleCondition(condition.id)}
            on:updateValue={(e) => updateConditionValue(condition.id, e.detail)}
            on:updateOperator={(e) => updateConditionOperator(condition.id, e.detail)}
            on:remove={() => removeCondition(condition.id)}
          />
        {/each}
      </div>
    {/if}
  </section>

  <section class="condition-logic">
    <h3>Evaluation Logic</h3>

    <div class="logic-config">
      <label class="logic-option">
        <input
          type="radio"
          name="logic"
          value="any"
          checked={$stopConditionsStore.evaluationLogic === 'any'}
          on:change={() => stopConditionsStore.setEvaluationLogic('any')}
        />
        <span class="logic-label">Any condition</span>
        <span class="logic-desc">Stop when ANY condition is met</span>
      </label>

      <label class="logic-option">
        <input
          type="radio"
          name="logic"
          value="all"
          checked={$stopConditionsStore.evaluationLogic === 'all'}
          on:change={() => stopConditionsStore.setEvaluationLogic('all')}
        />
        <span class="logic-label">All conditions</span>
        <span class="logic-desc">Stop only when ALL conditions are met</span>
      </label>

      <label class="logic-option">
        <input
          type="radio"
          name="logic"
          value="priority"
          checked={$stopConditionsStore.evaluationLogic === 'priority'}
          on:change={() => stopConditionsStore.setEvaluationLogic('priority')}
        />
        <span class="logic-label">Priority-based</span>
        <span class="logic-desc">Evaluate by condition priority order</span>
      </label>
    </div>

    <div class="logic-preview">
      <h4>Current Rule Summary</h4>
      <div class="rule-text">
        {#if $activeConditions.length === 0}
          <p class="warning">No active conditions - sessions will run until manually stopped</p>
        {:else}
          <p>Stop when
            <strong>{$stopConditionsStore.evaluationLogic === 'any' ? 'any' : 'all'}</strong>
            of the following:
          </p>
          <ul>
            {#each $activeConditions as condition}
              <li>
                {condition.name} {getOperatorLabel(condition.operator)}
                {formatValue(condition.value, condition.unit)}
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </div>
  </section>

  {#if $showCustomBuilder}
    <div class="modal-overlay" on:click={() => showCustomBuilder.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <CustomConditionBuilder
          on:save={(e) => saveCustomCondition(e.detail)}
          on:close={() => showCustomBuilder.set(false)}
        />
      </div>
    </div>
  {/if}

  {#if $previewMode}
    <div class="modal-overlay" on:click={() => previewMode.set(false)}>
      <div class="modal-content large" on:click|stopPropagation>
        <ConditionPreview
          conditions={$activeConditions}
          evaluationLogic={$stopConditionsStore.evaluationLogic}
          on:close={() => previewMode.set(false)}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .stop-conditions-ui {
    max-width: 900px;
  }

  .config-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1.5rem;
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

  .header-actions {
    display: flex;
    gap: 0.75rem;
  }

  .conditions-summary {
    display: flex;
    gap: 1.5rem;
    margin-bottom: 1.5rem;
  }

  .summary-stat {
    padding: 1rem 1.5rem;
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
  }

  .stat-value {
    display: block;
    font-size: 2rem;
    font-weight: 700;
    color: var(--primary-color);
  }

  .stat-label {
    font-size: 0.8125rem;
    color: var(--text-muted);
  }

  .add-condition-section,
  .active-conditions,
  .condition-logic {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
  }

  section h3 {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 1rem;
  }

  .condition-templates {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.75rem;
  }

  .template-btn {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 1rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .template-btn:hover {
    border-color: var(--primary-color);
  }

  .template-btn.custom {
    border-style: dashed;
  }

  .template-name {
    font-weight: 500;
    font-size: 0.875rem;
    margin-bottom: 0.25rem;
  }

  .template-desc {
    font-size: 0.75rem;
    color: var(--text-muted);
    text-align: left;
  }

  .conditions-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .empty-state {
    padding: 2rem;
    text-align: center;
    color: var(--text-muted);
  }

  .hint {
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }

  .logic-config {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 1.5rem;
  }

  .logic-option {
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-rows: auto auto;
    gap: 0.25rem 0.75rem;
    align-items: center;
    padding: 0.75rem;
    background: var(--secondary-bg);
    border-radius: 6px;
    cursor: pointer;
  }

  .logic-option input {
    grid-row: span 2;
  }

  .logic-label {
    font-weight: 500;
    font-size: 0.875rem;
  }

  .logic-desc {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .logic-preview {
    padding: 1rem;
    background: var(--secondary-bg);
    border-radius: 6px;
  }

  .logic-preview h4 {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-secondary);
    margin-bottom: 0.75rem;
  }

  .rule-text {
    font-size: 0.875rem;
    line-height: 1.6;
  }

  .rule-text ul {
    margin: 0.5rem 0 0 1.25rem;
  }

  .rule-text .warning {
    color: var(--warning-color);
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
</style>
```

## Testing Requirements
1. **Unit Tests**: Test condition evaluation logic
2. **Integration Tests**: Verify condition persistence
3. **Preview Tests**: Test condition preview accuracy
4. **Custom Tests**: Test custom condition builder
5. **Logic Tests**: Test evaluation logic modes

## Related Specs
- Spec 281: Loop Config UI
- Spec 267: Convergence Indicator
- Spec 268: Session Controls
