# Spec 280: API Key Management

## Header
- **Spec ID**: 280
- **Phase**: 13 - Settings UI
- **Component**: API Key Management
- **Dependencies**: Spec 277 (Backend Config UI)
- **Status**: Draft

## Objective
Create a secure interface for managing API keys for various AI providers, with encryption, usage tracking, quota monitoring, and secure storage capabilities.

## Acceptance Criteria
- [x] Add, edit, and delete API keys for supported providers
- [x] Secure key display with masked view toggle
- [x] Test key validity with provider APIs
- [x] Track key usage and remaining quota
- [x] Set key priorities and fallback order
- [x] Configure rate limiting per key
- [x] Audit log for key access and usage
- [x] Secure encryption at rest indicators

## Implementation

### ApiKeyManagement.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, slide } from 'svelte/transition';
  import ApiKeyCard from './ApiKeyCard.svelte';
  import ApiKeyEditor from './ApiKeyEditor.svelte';
  import KeyUsageChart from './KeyUsageChart.svelte';
  import KeyAuditLog from './KeyAuditLog.svelte';
  import ProviderStatus from './ProviderStatus.svelte';
  import { apiKeyStore } from '$lib/stores/apiKeys';
  import { validateApiKey } from '$lib/services/keyValidation';
  import type {
    ApiKey,
    ApiKeyProvider,
    KeyUsage,
    KeyAuditEntry
  } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    add: ApiKey;
    update: ApiKey;
    delete: { keyId: string };
    test: { keyId: string; result: boolean };
  }>();

  const providers: ApiKeyProvider[] = [
    { id: 'openai', name: 'OpenAI', icon: 'openai', docs: 'https://platform.openai.com/api-keys' },
    { id: 'anthropic', name: 'Anthropic', icon: 'anthropic', docs: 'https://console.anthropic.com/settings/keys' },
    { id: 'google', name: 'Google AI', icon: 'google', docs: 'https://makersuite.google.com/app/apikey' },
    { id: 'cohere', name: 'Cohere', icon: 'cohere', docs: 'https://dashboard.cohere.ai/api-keys' },
    { id: 'mistral', name: 'Mistral AI', icon: 'mistral', docs: 'https://console.mistral.ai/api-keys/' }
  ];

  let showEditor = writable<boolean>(false);
  let editingKeyId = writable<string | null>(null);
  let showAuditLog = writable<boolean>(false);
  let selectedProvider = writable<string | null>(null);
  let testingKeyId = writable<string | null>(null);

  const apiKeys = derived(apiKeyStore, ($store) => $store.keys);

  const keysByProvider = derived(apiKeys, ($keys) => {
    const grouped = new Map<string, ApiKey[]>();

    for (const key of $keys) {
      if (!grouped.has(key.provider)) {
        grouped.set(key.provider, []);
      }
      grouped.get(key.provider)!.push(key);
    }

    return grouped;
  });

  const keyUsage = derived(apiKeyStore, ($store) => $store.usage);

  const auditLog = derived(apiKeyStore, ($store) => $store.auditLog);

  const activeKeyCount = derived(apiKeys, ($keys) =>
    $keys.filter(k => k.status === 'active').length
  );

  function openAddKey(providerId?: string) {
    editingKeyId.set(null);
    selectedProvider.set(providerId || null);
    showEditor.set(true);
  }

  function editKey(keyId: string) {
    editingKeyId.set(keyId);
    showEditor.set(true);
  }

  async function saveKey(key: ApiKey) {
    if ($editingKeyId) {
      await apiKeyStore.update(key);
      dispatch('update', key);
    } else {
      await apiKeyStore.add(key);
      dispatch('add', key);
    }
    showEditor.set(false);
    editingKeyId.set(null);
  }

  async function deleteKey(keyId: string) {
    if (confirm('Are you sure you want to delete this API key? This action cannot be undone.')) {
      await apiKeyStore.remove(keyId);
      dispatch('delete', { keyId });
    }
  }

  async function testKey(keyId: string) {
    testingKeyId.set(keyId);
    const key = $apiKeys.find(k => k.id === keyId);

    if (!key) return;

    try {
      const result = await validateApiKey(key.provider, key.key);
      apiKeyStore.updateStatus(keyId, result.valid ? 'active' : 'invalid');
      dispatch('test', { keyId, result: result.valid });
    } catch (error) {
      apiKeyStore.updateStatus(keyId, 'error');
    } finally {
      testingKeyId.set(null);
    }
  }

  function setKeyPriority(keyId: string, priority: number) {
    apiKeyStore.setPriority(keyId, priority);
  }

  function toggleKeyActive(keyId: string) {
    const key = $apiKeys.find(k => k.id === keyId);
    if (key) {
      apiKeyStore.updateStatus(keyId, key.status === 'active' ? 'disabled' : 'active');
    }
  }

  function maskKey(key: string): string {
    if (key.length <= 8) return '********';
    return key.slice(0, 4) + '*'.repeat(key.length - 8) + key.slice(-4);
  }

  onMount(() => {
    apiKeyStore.load();
  });
</script>

<div class="api-key-management" data-testid="api-key-management">
  <header class="management-header">
    <div class="header-title">
      <h2>API Key Management</h2>
      <p class="description">Manage API keys for AI providers</p>
    </div>

    <div class="header-stats">
      <div class="stat">
        <span class="stat-value">{$activeKeyCount}</span>
        <span class="stat-label">Active Keys</span>
      </div>
      <div class="stat">
        <span class="stat-value">{$apiKeys.length}</span>
        <span class="stat-label">Total Keys</span>
      </div>
    </div>

    <div class="header-actions">
      <button class="btn secondary" on:click={() => showAuditLog.set(true)}>
        Audit Log
      </button>
      <button class="btn primary" on:click={() => openAddKey()}>
        Add API Key
      </button>
    </div>
  </header>

  <div class="security-notice">
    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
      <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
    </svg>
    <p>
      API keys are encrypted at rest and never transmitted in plain text.
      <a href="/docs/security" target="_blank">Learn more about our security practices</a>
    </p>
  </div>

  <section class="providers-section">
    <h3>Providers</h3>
    <div class="providers-grid">
      {#each providers as provider}
        {@const providerKeys = $keysByProvider.get(provider.id) || []}
        <ProviderStatus
          {provider}
          keys={providerKeys}
          on:addKey={() => openAddKey(provider.id)}
          on:viewDocs={() => window.open(provider.docs, '_blank')}
        />
      {/each}
    </div>
  </section>

  <section class="keys-section">
    <h3>All API Keys</h3>

    {#if $apiKeys.length === 0}
      <div class="empty-state">
        <p>No API keys configured</p>
        <p class="hint">Add an API key to start using AI providers</p>
        <button class="btn primary" on:click={() => openAddKey()}>
          Add Your First Key
        </button>
      </div>
    {:else}
      <div class="keys-list">
        {#each $apiKeys as key (key.id)}
          <ApiKeyCard
            {key}
            maskedKey={maskKey(key.key)}
            usage={$keyUsage.get(key.id)}
            testing={$testingKeyId === key.id}
            on:edit={() => editKey(key.id)}
            on:delete={() => deleteKey(key.id)}
            on:test={() => testKey(key.id)}
            on:toggle={() => toggleKeyActive(key.id)}
            on:priorityChange={(e) => setKeyPriority(key.id, e.detail)}
          />
        {/each}
      </div>
    {/if}
  </section>

  {#if $apiKeys.length > 0}
    <section class="usage-section">
      <h3>Usage Overview</h3>
      <KeyUsageChart keys={$apiKeys} usage={$keyUsage} />
    </section>
  {/if}

  {#if $showEditor}
    <div class="modal-overlay" transition:fade on:click={() => showEditor.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <ApiKeyEditor
          key={$editingKeyId ? $apiKeys.find(k => k.id === $editingKeyId) : null}
          {providers}
          preselectedProvider={$selectedProvider}
          on:save={(e) => saveKey(e.detail)}
          on:close={() => {
            showEditor.set(false);
            editingKeyId.set(null);
            selectedProvider.set(null);
          }}
        />
      </div>
    </div>
  {/if}

  {#if $showAuditLog}
    <div class="modal-overlay" transition:fade on:click={() => showAuditLog.set(false)}>
      <div class="modal-content large" on:click|stopPropagation>
        <KeyAuditLog
          entries={$auditLog}
          on:close={() => showAuditLog.set(false)}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .api-key-management {
    max-width: 1000px;
  }

  .management-header {
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
    gap: 1.5rem;
  }

  .stat {
    text-align: center;
    padding: 0.75rem 1.25rem;
    background: var(--secondary-bg);
    border-radius: 6px;
  }

  .stat-value {
    display: block;
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--primary-color);
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .header-actions {
    display: flex;
    gap: 0.75rem;
  }

  .security-notice {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    background: var(--success-alpha);
    border: 1px solid var(--success-color);
    border-radius: 6px;
    margin-bottom: 1.5rem;
    color: var(--success-color);
  }

  .security-notice p {
    font-size: 0.875rem;
    margin: 0;
  }

  .security-notice a {
    color: inherit;
    text-decoration: underline;
  }

  .providers-section,
  .keys-section,
  .usage-section {
    margin-bottom: 2rem;
  }

  .providers-section h3,
  .keys-section h3,
  .usage-section h3 {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 1rem;
  }

  .providers-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 1rem;
  }

  .keys-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .empty-state {
    background: var(--card-bg);
    border: 1px dashed var(--border-color);
    border-radius: 8px;
    padding: 3rem;
    text-align: center;
    color: var(--text-muted);
  }

  .hint {
    font-size: 0.875rem;
    margin-top: 0.5rem;
    margin-bottom: 1.5rem;
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
    max-width: 500px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
  }

  .modal-content.large {
    max-width: 800px;
  }

  @media (max-width: 768px) {
    .management-header {
      flex-direction: column;
      gap: 1rem;
    }

    .header-stats {
      order: 2;
    }

    .header-actions {
      order: 3;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test CRUD operations and masking
2. **Security Tests**: Verify encryption and access controls
3. **Integration Tests**: Test key validation with providers
4. **Audit Tests**: Verify audit log accuracy
5. **Usage Tests**: Test quota tracking

## Related Specs
- Spec 277: Backend Config UI
- Spec 278: Brain Selection
- Spec 295: Settings Tests
