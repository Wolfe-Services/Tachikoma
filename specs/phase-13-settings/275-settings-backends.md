# 275 - LLM Backend Configuration Settings

**Phase:** 13 - Settings UI
**Spec ID:** 275
**Status:** Planned
**Dependencies:** 271-settings-layout, 272-settings-store, 051-backend-trait
**Estimated Context:** ~12% of model context window

---

## Objective

Create the LLM Backend Configuration panel that allows users to add, configure, and manage multiple LLM backends including Anthropic Claude, OpenAI, Ollama, Azure OpenAI, and custom endpoints with support for API key management, model selection, and parameter tuning.

---

## Acceptance Criteria

- [ ] `BackendSettings.svelte` component with backend list
- [ ] Add/edit/remove backend configurations
- [ ] Secure API key input with masking
- [ ] Model selection dropdowns with available models
- [ ] Parameter configuration (temperature, max tokens)
- [ ] Backend connection testing
- [ ] Default backend selection
- [ ] Drag-and-drop backend reordering
- [ ] Backend enable/disable toggles

---

## Implementation Details

### 1. Backend Settings Types (src/lib/types/backend-settings.ts)

```typescript
/**
 * Backend configuration UI types.
 */

export interface BackendFormState {
  id: string;
  name: string;
  type: BackendType;
  apiKey: string;
  baseUrl: string;
  model: string;
  maxTokens: number;
  temperature: number;
  enabled: boolean;
  isNew: boolean;
  isDirty: boolean;
  isValid: boolean;
  errors: Record<string, string>;
}

export type BackendType = 'anthropic' | 'openai' | 'ollama' | 'azure' | 'custom';

export interface BackendTypeOption {
  type: BackendType;
  name: string;
  icon: string;
  description: string;
  requiresApiKey: boolean;
  defaultBaseUrl: string;
}

export const BACKEND_TYPES: BackendTypeOption[] = [
  {
    type: 'anthropic',
    name: 'Anthropic Claude',
    icon: 'anthropic',
    description: 'Claude models from Anthropic',
    requiresApiKey: true,
    defaultBaseUrl: 'https://api.anthropic.com',
  },
  {
    type: 'openai',
    name: 'OpenAI',
    icon: 'openai',
    description: 'GPT models from OpenAI',
    requiresApiKey: true,
    defaultBaseUrl: 'https://api.openai.com/v1',
  },
  {
    type: 'ollama',
    name: 'Ollama',
    icon: 'server',
    description: 'Local models via Ollama',
    requiresApiKey: false,
    defaultBaseUrl: 'http://localhost:11434',
  },
  {
    type: 'azure',
    name: 'Azure OpenAI',
    icon: 'cloud',
    description: 'Azure-hosted OpenAI models',
    requiresApiKey: true,
    defaultBaseUrl: '',
  },
  {
    type: 'custom',
    name: 'Custom Endpoint',
    icon: 'settings',
    description: 'OpenAI-compatible API endpoint',
    requiresApiKey: false,
    defaultBaseUrl: '',
  },
];

export interface ModelOption {
  id: string;
  name: string;
  contextWindow: number;
  description?: string;
}

export const MODELS_BY_TYPE: Record<BackendType, ModelOption[]> = {
  anthropic: [
    { id: 'claude-opus-4-20250514', name: 'Claude Opus 4', contextWindow: 200000, description: 'Most powerful model' },
    { id: 'claude-sonnet-4-20250514', name: 'Claude Sonnet 4', contextWindow: 200000, description: 'Balanced performance' },
    { id: 'claude-3-5-sonnet-20241022', name: 'Claude 3.5 Sonnet', contextWindow: 200000 },
    { id: 'claude-3-haiku-20240307', name: 'Claude 3 Haiku', contextWindow: 200000, description: 'Fast and efficient' },
  ],
  openai: [
    { id: 'gpt-4o', name: 'GPT-4o', contextWindow: 128000 },
    { id: 'gpt-4-turbo', name: 'GPT-4 Turbo', contextWindow: 128000 },
    { id: 'gpt-4', name: 'GPT-4', contextWindow: 8192 },
    { id: 'gpt-3.5-turbo', name: 'GPT-3.5 Turbo', contextWindow: 16385 },
  ],
  ollama: [
    { id: 'llama3.2', name: 'Llama 3.2', contextWindow: 128000 },
    { id: 'llama3.1', name: 'Llama 3.1', contextWindow: 128000 },
    { id: 'mistral', name: 'Mistral', contextWindow: 32768 },
    { id: 'codellama', name: 'Code Llama', contextWindow: 16384 },
    { id: 'deepseek-coder', name: 'DeepSeek Coder', contextWindow: 16384 },
  ],
  azure: [],
  custom: [],
};

export interface ConnectionTestResult {
  success: boolean;
  message: string;
  latency?: number;
  model?: string;
}
```

### 2. Backend Settings Component (src/lib/components/settings/BackendSettings.svelte)

```svelte
<script lang="ts">
  import { flip } from 'svelte/animate';
  import { dndzone } from 'svelte-dnd-action';
  import { settingsStore, backendSettings } from '$lib/stores/settings-store';
  import { BACKEND_TYPES, MODELS_BY_TYPE } from '$lib/types/backend-settings';
  import type { BackendConfig } from '$lib/types/settings';
  import type { BackendType, ConnectionTestResult } from '$lib/types/backend-settings';
  import { invoke } from '$lib/ipc';
  import { generateId } from '$lib/utils/id';
  import SettingsSection from './SettingsSection.svelte';
  import SettingsRow from './SettingsRow.svelte';
  import Toggle from '$lib/components/ui/Toggle.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import Slider from '$lib/components/ui/Slider.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';

  let showAddModal = false;
  let editingBackend: BackendConfig | null = null;
  let testingBackendId: string | null = null;
  let testResults: Map<string, ConnectionTestResult> = new Map();
  let dragDisabled = true;

  // Form state for add/edit modal
  let formState: Partial<BackendConfig> = {};
  let formErrors: Record<string, string> = {};
  let showApiKey = false;

  function openAddModal() {
    formState = {
      id: generateId(),
      name: '',
      type: 'anthropic',
      apiKey: '',
      baseUrl: BACKEND_TYPES[0].defaultBaseUrl,
      model: MODELS_BY_TYPE.anthropic[0]?.id || '',
      maxTokens: 4096,
      temperature: 0.7,
      enabled: true,
    };
    formErrors = {};
    showApiKey = false;
    showAddModal = true;
  }

  function openEditModal(backend: BackendConfig) {
    editingBackend = backend;
    formState = { ...backend };
    formErrors = {};
    showApiKey = false;
    showAddModal = true;
  }

  function closeModal() {
    showAddModal = false;
    editingBackend = null;
    formState = {};
    formErrors = {};
  }

  function handleTypeChange(type: BackendType) {
    const typeOption = BACKEND_TYPES.find(t => t.type === type);
    formState = {
      ...formState,
      type,
      baseUrl: typeOption?.defaultBaseUrl || '',
      model: MODELS_BY_TYPE[type][0]?.id || '',
    };
  }

  function validateForm(): boolean {
    formErrors = {};

    if (!formState.name?.trim()) {
      formErrors.name = 'Name is required';
    }

    const typeOption = BACKEND_TYPES.find(t => t.type === formState.type);
    if (typeOption?.requiresApiKey && !formState.apiKey?.trim()) {
      formErrors.apiKey = 'API key is required';
    }

    if (!formState.model?.trim()) {
      formErrors.model = 'Model is required';
    }

    if (formState.type === 'custom' && !formState.baseUrl?.trim()) {
      formErrors.baseUrl = 'Base URL is required for custom endpoints';
    }

    return Object.keys(formErrors).length === 0;
  }

  function handleSave() {
    if (!validateForm()) return;

    const backend: BackendConfig = {
      id: formState.id!,
      name: formState.name!,
      type: formState.type!,
      apiKey: formState.apiKey,
      baseUrl: formState.baseUrl,
      model: formState.model!,
      maxTokens: formState.maxTokens!,
      temperature: formState.temperature!,
      enabled: formState.enabled!,
    };

    if (editingBackend) {
      // Update existing
      const backends = $backendSettings.backends.map(b =>
        b.id === backend.id ? backend : b
      );
      settingsStore.updateCategory('backends', { backends });
    } else {
      // Add new
      const backends = [...$backendSettings.backends, backend];
      settingsStore.updateCategory('backends', { backends });
    }

    closeModal();
  }

  function handleDelete(id: string) {
    const backends = $backendSettings.backends.filter(b => b.id !== id);
    settingsStore.updateCategory('backends', { backends });

    // If deleted backend was default, set new default
    if ($backendSettings.defaultBackend === id && backends.length > 0) {
      settingsStore.updateCategory('backends', { defaultBackend: backends[0].id });
    }
  }

  function handleToggleEnabled(id: string, enabled: boolean) {
    const backends = $backendSettings.backends.map(b =>
      b.id === id ? { ...b, enabled } : b
    );
    settingsStore.updateCategory('backends', { backends });
  }

  function handleSetDefault(id: string) {
    settingsStore.updateCategory('backends', { defaultBackend: id });
  }

  async function handleTestConnection(backend: BackendConfig) {
    testingBackendId = backend.id;
    testResults.delete(backend.id);
    testResults = testResults;

    try {
      const result = await invoke<ConnectionTestResult>('test_backend_connection', {
        config: backend,
      });
      testResults.set(backend.id, result);
    } catch (error) {
      testResults.set(backend.id, {
        success: false,
        message: (error as Error).message,
      });
    }

    testResults = testResults;
    testingBackendId = null;
  }

  function handleDndConsider(event: CustomEvent<{ items: BackendConfig[] }>) {
    const backends = event.detail.items;
    settingsStore.updateCategory('backends', { backends });
  }

  function handleDndFinalize(event: CustomEvent<{ items: BackendConfig[] }>) {
    const backends = event.detail.items;
    settingsStore.updateCategory('backends', { backends });
    dragDisabled = true;
  }

  function startDrag() {
    dragDisabled = false;
  }

  $: availableModels = formState.type ? MODELS_BY_TYPE[formState.type] : [];
  $: currentTypeOption = BACKEND_TYPES.find(t => t.type === formState.type);
</script>

<div class="backend-settings">
  <h2 class="settings-title">LLM Backends</h2>
  <p class="settings-description">
    Configure and manage your AI model backends.
  </p>

  <!-- Global Settings -->
  <SettingsSection title="Global Settings">
    <SettingsRow
      label="Default Backend"
      description="The backend used for new missions"
    >
      <Select
        value={$backendSettings.defaultBackend}
        options={$backendSettings.backends
          .filter(b => b.enabled)
          .map(b => ({ value: b.id, label: b.name }))}
        on:change={(e) => handleSetDefault(e.detail)}
      />
    </SettingsRow>

    <SettingsRow
      label="Request Timeout"
      description="Maximum time to wait for a response (seconds)"
    >
      <div class="slider-with-value">
        <Slider
          min={10}
          max={300}
          step={10}
          value={$backendSettings.timeout / 1000}
          on:change={(e) => settingsStore.updateSetting('backends', 'timeout', e.detail * 1000)}
        />
        <span class="slider-value">{$backendSettings.timeout / 1000}s</span>
      </div>
    </SettingsRow>

    <SettingsRow
      label="Max Retries"
      description="Number of retry attempts on failure"
    >
      <Select
        value={$backendSettings.maxRetries.toString()}
        options={[
          { value: '0', label: 'No retries' },
          { value: '1', label: '1 retry' },
          { value: '2', label: '2 retries' },
          { value: '3', label: '3 retries' },
          { value: '5', label: '5 retries' },
        ]}
        on:change={(e) => settingsStore.updateSetting('backends', 'maxRetries', parseInt(e.detail))}
      />
    </SettingsRow>

    <SettingsRow
      label="Stream Responses"
      description="Enable streaming for real-time output"
    >
      <Toggle
        checked={$backendSettings.streamResponses}
        on:change={(e) => settingsStore.updateSetting('backends', 'streamResponses', e.detail)}
      />
    </SettingsRow>
  </SettingsSection>

  <!-- Configured Backends -->
  <SettingsSection title="Configured Backends">
    <div class="backends-header">
      <span class="backends-count">
        {$backendSettings.backends.length} backend{$backendSettings.backends.length !== 1 ? 's' : ''} configured
      </span>
      <Button variant="primary" size="small" on:click={openAddModal}>
        <Icon name="plus" size={16} />
        Add Backend
      </Button>
    </div>

    {#if $backendSettings.backends.length === 0}
      <div class="backends-empty">
        <Icon name="server" size={48} />
        <h3>No backends configured</h3>
        <p>Add a backend to start using AI models</p>
        <Button variant="primary" on:click={openAddModal}>
          Add Your First Backend
        </Button>
      </div>
    {:else}
      <div
        class="backends-list"
        use:dndzone={{
          items: $backendSettings.backends,
          flipDurationMs: 200,
          dragDisabled,
        }}
        on:consider={handleDndConsider}
        on:finalize={handleDndFinalize}
      >
        {#each $backendSettings.backends as backend (backend.id)}
          <div
            class="backend-card"
            class:backend-card--disabled={!backend.enabled}
            class:backend-card--default={backend.id === $backendSettings.defaultBackend}
            animate:flip={{ duration: 200 }}
          >
            <div class="backend-card__drag" on:mousedown={startDrag}>
              <Icon name="grip-vertical" size={16} />
            </div>

            <div class="backend-card__icon">
              <Icon name={BACKEND_TYPES.find(t => t.type === backend.type)?.icon || 'server'} size={24} />
            </div>

            <div class="backend-card__info">
              <div class="backend-card__header">
                <h4 class="backend-card__name">
                  {backend.name}
                  {#if backend.id === $backendSettings.defaultBackend}
                    <span class="backend-card__badge">Default</span>
                  {/if}
                </h4>
                <span class="backend-card__type">
                  {BACKEND_TYPES.find(t => t.type === backend.type)?.name}
                </span>
              </div>
              <p class="backend-card__model">{backend.model}</p>
            </div>

            <div class="backend-card__status">
              {#if testResults.has(backend.id)}
                {@const result = testResults.get(backend.id)}
                <span
                  class="backend-card__test-result"
                  class:backend-card__test-result--success={result?.success}
                  class:backend-card__test-result--error={!result?.success}
                >
                  <Icon name={result?.success ? 'check-circle' : 'x-circle'} size={16} />
                  {result?.success ? `${result.latency}ms` : 'Failed'}
                </span>
              {/if}
            </div>

            <div class="backend-card__actions">
              <Toggle
                checked={backend.enabled}
                on:change={(e) => handleToggleEnabled(backend.id, e.detail)}
                aria-label="Enable backend"
              />

              <Button
                variant="ghost"
                size="small"
                disabled={testingBackendId === backend.id}
                on:click={() => handleTestConnection(backend)}
                title="Test connection"
              >
                {#if testingBackendId === backend.id}
                  <Icon name="loader" size={16} class="spinning" />
                {:else}
                  <Icon name="zap" size={16} />
                {/if}
              </Button>

              <Button
                variant="ghost"
                size="small"
                on:click={() => openEditModal(backend)}
                title="Edit backend"
              >
                <Icon name="edit-2" size={16} />
              </Button>

              <Button
                variant="ghost"
                size="small"
                on:click={() => handleDelete(backend.id)}
                title="Delete backend"
              >
                <Icon name="trash-2" size={16} />
              </Button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </SettingsSection>
</div>

<!-- Add/Edit Modal -->
{#if showAddModal}
  <Modal
    title={editingBackend ? 'Edit Backend' : 'Add Backend'}
    on:close={closeModal}
  >
    <form class="backend-form" on:submit|preventDefault={handleSave}>
      <div class="form-group">
        <label for="backend-type">Backend Type</label>
        <div class="backend-type-grid">
          {#each BACKEND_TYPES as typeOption}
            <button
              type="button"
              class="backend-type-option"
              class:backend-type-option--selected={formState.type === typeOption.type}
              on:click={() => handleTypeChange(typeOption.type)}
            >
              <Icon name={typeOption.icon} size={24} />
              <span class="backend-type-option__name">{typeOption.name}</span>
              <span class="backend-type-option__desc">{typeOption.description}</span>
            </button>
          {/each}
        </div>
      </div>

      <div class="form-group">
        <label for="backend-name">Display Name</label>
        <Input
          id="backend-name"
          bind:value={formState.name}
          placeholder="My Backend"
          error={formErrors.name}
        />
      </div>

      {#if currentTypeOption?.requiresApiKey}
        <div class="form-group">
          <label for="backend-apikey">API Key</label>
          <div class="api-key-input">
            <Input
              id="backend-apikey"
              type={showApiKey ? 'text' : 'password'}
              bind:value={formState.apiKey}
              placeholder="sk-..."
              error={formErrors.apiKey}
            />
            <Button
              type="button"
              variant="ghost"
              size="small"
              on:click={() => showApiKey = !showApiKey}
            >
              <Icon name={showApiKey ? 'eye-off' : 'eye'} size={16} />
            </Button>
          </div>
        </div>
      {/if}

      {#if formState.type === 'custom' || formState.type === 'azure'}
        <div class="form-group">
          <label for="backend-baseurl">Base URL</label>
          <Input
            id="backend-baseurl"
            bind:value={formState.baseUrl}
            placeholder="https://api.example.com"
            error={formErrors.baseUrl}
          />
        </div>
      {/if}

      <div class="form-group">
        <label for="backend-model">Model</label>
        {#if availableModels.length > 0}
          <Select
            value={formState.model}
            options={availableModels.map(m => ({
              value: m.id,
              label: `${m.name} (${Math.round(m.contextWindow / 1000)}K context)`,
            }))}
            on:change={(e) => formState.model = e.detail}
          />
        {:else}
          <Input
            id="backend-model"
            bind:value={formState.model}
            placeholder="model-name"
            error={formErrors.model}
          />
        {/if}
      </div>

      <div class="form-row">
        <div class="form-group">
          <label for="backend-maxtokens">Max Tokens</label>
          <Input
            id="backend-maxtokens"
            type="number"
            bind:value={formState.maxTokens}
            min={1}
            max={100000}
          />
        </div>

        <div class="form-group">
          <label for="backend-temp">Temperature: {formState.temperature?.toFixed(2)}</label>
          <Slider
            min={0}
            max={2}
            step={0.1}
            value={formState.temperature}
            on:change={(e) => formState.temperature = e.detail}
          />
        </div>
      </div>

      <div class="form-actions">
        <Button type="button" variant="secondary" on:click={closeModal}>
          Cancel
        </Button>
        <Button type="submit" variant="primary">
          {editingBackend ? 'Save Changes' : 'Add Backend'}
        </Button>
      </div>
    </form>
  </Modal>
{/if}

<style>
  .backend-settings {
    max-width: 800px;
  }

  .settings-title {
    font-size: 24px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 8px 0;
  }

  .settings-description {
    color: var(--color-text-secondary);
    font-size: 14px;
    margin: 0 0 24px 0;
  }

  .slider-with-value {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 200px;
  }

  .slider-value {
    min-width: 50px;
    text-align: right;
    font-size: 13px;
    color: var(--color-text-secondary);
    font-family: monospace;
  }

  /* Backends Header */
  .backends-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .backends-count {
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  /* Empty State */
  .backends-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 48px 24px;
    text-align: center;
    background: var(--color-bg-secondary);
    border-radius: 12px;
    border: 2px dashed var(--color-border);
  }

  .backends-empty h3 {
    margin: 16px 0 8px;
    color: var(--color-text-primary);
  }

  .backends-empty p {
    margin: 0 0 24px;
    color: var(--color-text-secondary);
  }

  /* Backend List */
  .backends-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  /* Backend Card */
  .backend-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    transition: all 0.15s ease;
  }

  .backend-card:hover {
    border-color: var(--color-text-muted);
  }

  .backend-card--disabled {
    opacity: 0.6;
  }

  .backend-card--default {
    border-color: var(--color-primary);
  }

  .backend-card__drag {
    cursor: grab;
    color: var(--color-text-muted);
    padding: 4px;
  }

  .backend-card__drag:active {
    cursor: grabbing;
  }

  .backend-card__icon {
    width: 48px;
    height: 48px;
    border-radius: 8px;
    background: var(--color-bg-hover);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-primary);
  }

  .backend-card__info {
    flex: 1;
    min-width: 0;
  }

  .backend-card__header {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .backend-card__name {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .backend-card__badge {
    font-size: 11px;
    font-weight: 500;
    padding: 2px 6px;
    background: var(--color-primary);
    color: white;
    border-radius: 4px;
  }

  .backend-card__type {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .backend-card__model {
    font-size: 13px;
    color: var(--color-text-secondary);
    margin: 4px 0 0;
    font-family: monospace;
  }

  .backend-card__status {
    min-width: 80px;
    text-align: center;
  }

  .backend-card__test-result {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .backend-card__test-result--success {
    color: var(--color-success);
    background: rgba(76, 175, 80, 0.1);
  }

  .backend-card__test-result--error {
    color: var(--color-error);
    background: rgba(244, 67, 54, 0.1);
  }

  .backend-card__actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  /* Form Styles */
  .backend-form {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .form-group label {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .backend-type-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 12px;
  }

  .backend-type-option {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 16px 12px;
    border: 2px solid var(--color-border);
    border-radius: 8px;
    background: transparent;
    cursor: pointer;
    text-align: center;
    transition: all 0.15s ease;
  }

  .backend-type-option:hover {
    border-color: var(--color-text-muted);
  }

  .backend-type-option--selected {
    border-color: var(--color-primary);
    background: var(--color-bg-active);
  }

  .backend-type-option__name {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .backend-type-option__desc {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .api-key-input {
    display: flex;
    gap: 8px;
  }

  .api-key-input :global(input) {
    flex: 1;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--color-border);
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

---

## Testing Requirements

1. Backend list renders correctly
2. Add backend modal opens and creates backends
3. Edit backend updates configuration
4. Delete backend removes from list
5. Connection test shows results
6. API key is masked by default
7. Default backend selection works
8. Enable/disable toggle functions
9. Drag and drop reordering works

### Test File (src/lib/components/settings/__tests__/BackendSettings.test.ts)

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import BackendSettings from '../BackendSettings.svelte';
import { settingsStore } from '$lib/stores/settings-store';

vi.mock('$lib/ipc', () => ({
  invoke: vi.fn().mockImplementation((command: string) => {
    if (command === 'test_backend_connection') {
      return Promise.resolve({ success: true, latency: 150, message: 'Connected' });
    }
    return Promise.resolve(null);
  }),
}));

describe('BackendSettings', () => {
  beforeEach(() => {
    settingsStore.resetAll();
  });

  it('renders backend list', () => {
    render(BackendSettings);

    expect(screen.getByText('LLM Backends')).toBeInTheDocument();
    expect(screen.getByText('Anthropic Claude')).toBeInTheDocument();
  });

  it('opens add backend modal', async () => {
    render(BackendSettings);

    const addButton = screen.getByRole('button', { name: /add backend/i });
    await fireEvent.click(addButton);

    expect(screen.getByText('Add Backend')).toBeInTheDocument();
  });

  it('creates new backend', async () => {
    render(BackendSettings);

    const addButton = screen.getByRole('button', { name: /add backend/i });
    await fireEvent.click(addButton);

    const nameInput = screen.getByLabelText(/display name/i);
    await fireEvent.input(nameInput, { target: { value: 'Test Backend' } });

    const apiKeyInput = screen.getByLabelText(/api key/i);
    await fireEvent.input(apiKeyInput, { target: { value: 'sk-test-key' } });

    const submitButton = screen.getByRole('button', { name: /add backend/i });
    await fireEvent.click(submitButton);

    const state = get(settingsStore);
    expect(state.settings.backends.backends).toHaveLength(2);
  });

  it('tests backend connection', async () => {
    render(BackendSettings);

    const testButton = screen.getAllByTitle('Test connection')[0];
    await fireEvent.click(testButton);

    await waitFor(() => {
      expect(screen.getByText('150ms')).toBeInTheDocument();
    });
  });

  it('toggles backend enabled state', async () => {
    render(BackendSettings);

    const toggle = screen.getAllByRole('switch')[0];
    await fireEvent.click(toggle);

    const state = get(settingsStore);
    expect(state.settings.backends.backends[0].enabled).toBe(false);
  });

  it('deletes backend', async () => {
    render(BackendSettings);

    const deleteButton = screen.getAllByTitle('Delete backend')[0];
    await fireEvent.click(deleteButton);

    const state = get(settingsStore);
    expect(state.settings.backends.backends).toHaveLength(0);
  });
});

function get<T>(store: { subscribe: (fn: (value: T) => void) => void }): T {
  let value: T;
  store.subscribe(v => value = v)();
  return value!;
}
```

---

## Related Specs

- Depends on: [271-settings-layout.md](271-settings-layout.md)
- Depends on: [272-settings-store.md](272-settings-store.md)
- Depends on: [051-backend-trait.md](../phase-03-backends/051-backend-trait.md)
- Previous: [274-settings-appearance.md](274-settings-appearance.md)
- Next: [276-settings-keybindings.md](276-settings-keybindings.md)
