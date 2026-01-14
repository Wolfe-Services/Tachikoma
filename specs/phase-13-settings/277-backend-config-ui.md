# Spec 277: Backend Config UI

## Header
- **Spec ID**: 277
- **Phase**: 13 - Settings UI
- **Component**: Backend Config UI
- **Dependencies**: Spec 276 (Settings Layout)
- **Status**: Draft

## Objective
Create a configuration interface for managing backend connections, API endpoints, authentication settings, and service configurations for the Tachikoma application.

## Acceptance Criteria
- [x] Configure API endpoint URLs with validation
- [x] Manage authentication tokens and credentials
- [x] Test connection functionality with status indicators
- [x] Configure timeout and retry settings
- [x] Support multiple backend environments (dev, staging, prod)
- [x] Display connection health monitoring
- [x] Configure proxy and network settings
- [x] Secure credential storage with encryption indicators

## Implementation

### BackendConfigUI.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { fade, slide } from 'svelte/transition';
  import EndpointConfig from './EndpointConfig.svelte';
  import AuthConfig from './AuthConfig.svelte';
  import ConnectionTest from './ConnectionTest.svelte';
  import EnvironmentSelector from './EnvironmentSelector.svelte';
  import HealthMonitor from './HealthMonitor.svelte';
  import { backendConfigStore } from '$lib/stores/backendConfig';
  import { validateUrl, validateApiKey } from '$lib/utils/validation';
  import type {
    BackendConfig,
    Environment,
    ConnectionStatus,
    HealthCheck
  } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    save: BackendConfig;
    test: { endpoint: string };
  }>();

  let activeEnvironment = writable<Environment>('production');
  let showAdvanced = writable<boolean>(false);
  let testingConnection = writable<boolean>(false);
  let connectionStatus = writable<ConnectionStatus>('unknown');
  let healthChecks = writable<HealthCheck[]>([]);

  const config = derived(
    [backendConfigStore, activeEnvironment],
    ([$store, $env]) => $store.configs[$env] || getDefaultConfig()
  );

  const validationErrors = writable<Map<string, string>>(new Map());

  const isValid = derived(validationErrors, ($errors) => $errors.size === 0);

  function getDefaultConfig(): BackendConfig {
    return {
      apiBaseUrl: '',
      wsUrl: '',
      authType: 'api_key',
      credentials: {},
      timeout: 30000,
      retryAttempts: 3,
      retryDelay: 1000,
      proxy: null,
      healthCheckInterval: 60000
    };
  }

  function updateConfig(field: keyof BackendConfig, value: unknown) {
    backendConfigStore.update($activeEnvironment, field, value);
    validateField(field, value);
  }

  function validateField(field: string, value: unknown) {
    const errors = new Map($validationErrors);

    switch (field) {
      case 'apiBaseUrl':
        if (!validateUrl(value as string)) {
          errors.set(field, 'Invalid URL format');
        } else {
          errors.delete(field);
        }
        break;
      case 'wsUrl':
        if (value && !validateUrl(value as string, ['ws', 'wss'])) {
          errors.set(field, 'Invalid WebSocket URL format');
        } else {
          errors.delete(field);
        }
        break;
      case 'timeout':
        if ((value as number) < 1000 || (value as number) > 300000) {
          errors.set(field, 'Timeout must be between 1-300 seconds');
        } else {
          errors.delete(field);
        }
        break;
    }

    validationErrors.set(errors);
  }

  async function testConnection() {
    testingConnection.set(true);
    connectionStatus.set('testing');

    try {
      const result = await backendConfigStore.testConnection($activeEnvironment);

      connectionStatus.set(result.success ? 'connected' : 'failed');

      if (result.healthChecks) {
        healthChecks.set(result.healthChecks);
      }
    } catch (error) {
      connectionStatus.set('error');
    } finally {
      testingConnection.set(false);
    }
  }

  async function saveConfig() {
    if (!$isValid) return;

    try {
      await backendConfigStore.save($activeEnvironment);
      dispatch('save', $config);
    } catch (error) {
      console.error('Failed to save config:', error);
    }
  }

  function importConfig() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (file) {
        const text = await file.text();
        const imported = JSON.parse(text);
        backendConfigStore.import($activeEnvironment, imported);
      }
    };
    input.click();
  }

  function exportConfig() {
    const data = JSON.stringify($config, null, 2);
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `backend-config-${$activeEnvironment}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  onMount(() => {
    backendConfigStore.load();
  });
</script>

<div class="backend-config" data-testid="backend-config-ui">
  <header class="config-header">
    <div class="header-title">
      <h2>Backend Configuration</h2>
      <p class="description">Configure API connections and authentication settings</p>
    </div>

    <div class="header-actions">
      <EnvironmentSelector
        value={$activeEnvironment}
        on:change={(e) => activeEnvironment.set(e.detail)}
      />

      <div class="action-buttons">
        <button class="btn secondary" on:click={importConfig}>
          Import
        </button>
        <button class="btn secondary" on:click={exportConfig}>
          Export
        </button>
      </div>
    </div>
  </header>

  <div class="config-content">
    <section class="config-section">
      <h3>Endpoints</h3>

      <div class="form-group">
        <label for="api-base-url">API Base URL</label>
        <input
          id="api-base-url"
          type="url"
          value={$config.apiBaseUrl}
          on:input={(e) => updateConfig('apiBaseUrl', (e.target as HTMLInputElement).value)}
          placeholder="https://api.example.com/v1"
          class:error={$validationErrors.has('apiBaseUrl')}
        />
        {#if $validationErrors.has('apiBaseUrl')}
          <span class="error-message">{$validationErrors.get('apiBaseUrl')}</span>
        {/if}
      </div>

      <div class="form-group">
        <label for="ws-url">WebSocket URL (optional)</label>
        <input
          id="ws-url"
          type="url"
          value={$config.wsUrl}
          on:input={(e) => updateConfig('wsUrl', (e.target as HTMLInputElement).value)}
          placeholder="wss://api.example.com/ws"
          class:error={$validationErrors.has('wsUrl')}
        />
        {#if $validationErrors.has('wsUrl')}
          <span class="error-message">{$validationErrors.get('wsUrl')}</span>
        {/if}
      </div>
    </section>

    <section class="config-section">
      <h3>Authentication</h3>

      <AuthConfig
        authType={$config.authType}
        credentials={$config.credentials}
        on:typeChange={(e) => updateConfig('authType', e.detail)}
        on:credentialsChange={(e) => updateConfig('credentials', e.detail)}
      />
    </section>

    <section class="config-section">
      <h3>Connection Settings</h3>

      <div class="form-row">
        <div class="form-group">
          <label for="timeout">Request Timeout (ms)</label>
          <input
            id="timeout"
            type="number"
            value={$config.timeout}
            on:input={(e) => updateConfig('timeout', parseInt((e.target as HTMLInputElement).value))}
            min="1000"
            max="300000"
            step="1000"
          />
        </div>

        <div class="form-group">
          <label for="retry-attempts">Retry Attempts</label>
          <input
            id="retry-attempts"
            type="number"
            value={$config.retryAttempts}
            on:input={(e) => updateConfig('retryAttempts', parseInt((e.target as HTMLInputElement).value))}
            min="0"
            max="10"
          />
        </div>

        <div class="form-group">
          <label for="retry-delay">Retry Delay (ms)</label>
          <input
            id="retry-delay"
            type="number"
            value={$config.retryDelay}
            on:input={(e) => updateConfig('retryDelay', parseInt((e.target as HTMLInputElement).value))}
            min="100"
            max="30000"
            step="100"
          />
        </div>
      </div>
    </section>

    <section class="config-section">
      <div class="section-header">
        <h3>Connection Test</h3>
        <button
          class="test-btn"
          on:click={testConnection}
          disabled={!$isValid || $testingConnection}
        >
          {$testingConnection ? 'Testing...' : 'Test Connection'}
        </button>
      </div>

      <ConnectionTest
        status={$connectionStatus}
        testing={$testingConnection}
      />

      {#if $healthChecks.length > 0}
        <HealthMonitor checks={$healthChecks} />
      {/if}
    </section>

    <button
      class="toggle-advanced"
      on:click={() => showAdvanced.update(v => !v)}
    >
      {$showAdvanced ? 'Hide' : 'Show'} Advanced Settings
    </button>

    {#if $showAdvanced}
      <section class="config-section advanced" transition:slide>
        <h3>Advanced Settings</h3>

        <div class="form-group">
          <label for="health-check-interval">Health Check Interval (ms)</label>
          <input
            id="health-check-interval"
            type="number"
            value={$config.healthCheckInterval}
            on:input={(e) => updateConfig('healthCheckInterval', parseInt((e.target as HTMLInputElement).value))}
            min="10000"
            max="300000"
            step="1000"
          />
        </div>

        <div class="form-group">
          <label for="proxy-url">Proxy URL (optional)</label>
          <input
            id="proxy-url"
            type="url"
            value={$config.proxy || ''}
            on:input={(e) => updateConfig('proxy', (e.target as HTMLInputElement).value || null)}
            placeholder="http://proxy.example.com:8080"
          />
        </div>

        <div class="form-group checkbox">
          <label>
            <input
              type="checkbox"
              checked={$config.enableCompression}
              on:change={(e) => updateConfig('enableCompression', (e.target as HTMLInputElement).checked)}
            />
            Enable response compression
          </label>
        </div>

        <div class="form-group checkbox">
          <label>
            <input
              type="checkbox"
              checked={$config.enableCaching}
              on:change={(e) => updateConfig('enableCaching', (e.target as HTMLInputElement).checked)}
            />
            Enable response caching
          </label>
        </div>
      </section>
    {/if}
  </div>

  <footer class="config-footer">
    <button
      class="btn primary"
      on:click={saveConfig}
      disabled={!$isValid}
    >
      Save Configuration
    </button>
  </footer>
</div>

<style>
  .backend-config {
    max-width: 800px;
  }

  .config-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 2rem;
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
    gap: 1rem;
    align-items: center;
  }

  .action-buttons {
    display: flex;
    gap: 0.5rem;
  }

  .config-section {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
  }

  .config-section h3 {
    font-size: 1rem;
    font-weight: 600;
    margin-bottom: 1rem;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-group label {
    display: block;
    font-size: 0.875rem;
    font-weight: 500;
    margin-bottom: 0.5rem;
    color: var(--text-primary);
  }

  .form-group input[type="text"],
  .form-group input[type="url"],
  .form-group input[type="number"],
  .form-group input[type="password"] {
    width: 100%;
    padding: 0.625rem 0.875rem;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  .form-group input:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 3px var(--primary-alpha);
  }

  .form-group input.error {
    border-color: var(--error-color);
  }

  .form-group.checkbox label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
  }

  .error-message {
    display: block;
    font-size: 0.75rem;
    color: var(--error-color);
    margin-top: 0.25rem;
  }

  .form-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .test-btn {
    padding: 0.5rem 1rem;
    background: var(--secondary-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .test-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toggle-advanced {
    display: block;
    width: 100%;
    padding: 0.75rem;
    background: transparent;
    border: 1px dashed var(--border-color);
    border-radius: 6px;
    color: var(--text-secondary);
    font-size: 0.875rem;
    cursor: pointer;
    margin-bottom: 1.5rem;
  }

  .toggle-advanced:hover {
    border-color: var(--primary-color);
    color: var(--primary-color);
  }

  .config-footer {
    display: flex;
    justify-content: flex-end;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color);
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

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  @media (max-width: 768px) {
    .form-row {
      grid-template-columns: 1fr;
    }

    .config-header {
      flex-direction: column;
      gap: 1rem;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test validation logic and config updates
2. **Integration Tests**: Verify connection testing works
3. **Security Tests**: Validate credential encryption
4. **Environment Tests**: Test environment switching
5. **Import/Export Tests**: Verify config portability

## Related Specs
- Spec 276: Settings Layout
- Spec 280: API Key Management
- Spec 295: Settings Tests
