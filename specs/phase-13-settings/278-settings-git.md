# 278 - Git Integration Settings

**Phase:** 13 - Settings UI
**Spec ID:** 278
**Status:** Planned
**Dependencies:** 271-settings-layout, 272-settings-store
**Estimated Context:** ~8% of model context window

---

## Objective

Create the Git Integration Settings panel that allows users to configure Git-related preferences including auto-fetch, push behavior, commit signing, user identity, and default branch settings.

---

## Acceptance Criteria

- [ ] `GitSettings.svelte` component with all Git options
- [ ] Enable/disable Git integration toggle
- [ ] Auto-fetch with configurable interval
- [ ] Auto-push configuration
- [ ] Default branch name setting
- [ ] Commit signing with GPG key selection
- [ ] Git user identity configuration
- [ ] Git credential detection display

---

## Implementation Details

### 1. Git Settings Component (src/lib/components/settings/GitSettings.svelte)

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { settingsStore, gitSettings } from '$lib/stores/settings-store';
  import { invoke } from '$lib/ipc';
  import SettingsSection from './SettingsSection.svelte';
  import SettingsRow from './SettingsRow.svelte';
  import Toggle from '$lib/components/ui/Toggle.svelte';
  import Select from '$lib/components/ui/Select.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import Slider from '$lib/components/ui/Slider.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';

  interface GitInfo {
    version: string;
    configuredUser: { name: string; email: string } | null;
    gpgKeys: { id: string; email: string; name: string }[];
    sshKeyExists: boolean;
  }

  let gitInfo: GitInfo | null = null;
  let isLoading = true;
  let isTestingGit = false;
  let gitTestResult: { success: boolean; message: string } | null = null;

  async function loadGitInfo() {
    isLoading = true;
    try {
      gitInfo = await invoke<GitInfo>('git_get_info');
    } catch (error) {
      console.error('Failed to load Git info:', error);
    }
    isLoading = false;
  }

  async function testGitSetup() {
    isTestingGit = true;
    gitTestResult = null;
    try {
      const result = await invoke<{ success: boolean; message: string }>('git_test_setup');
      gitTestResult = result;
    } catch (error) {
      gitTestResult = { success: false, message: (error as Error).message };
    }
    isTestingGit = false;
  }

  function handleToggle<K extends keyof typeof $gitSettings>(key: K) {
    return (event: CustomEvent<boolean>) => {
      settingsStore.updateSetting('git', key, event.detail as any);
    };
  }

  function handleSelect<K extends keyof typeof $gitSettings>(key: K) {
    return (event: CustomEvent<string>) => {
      settingsStore.updateSetting('git', key, event.detail as any);
    };
  }

  function handleInput<K extends keyof typeof $gitSettings>(key: K) {
    return (event: Event) => {
      const target = event.target as HTMLInputElement;
      settingsStore.updateSetting('git', key, target.value as any);
    };
  }

  function formatInterval(ms: number): string {
    const minutes = ms / 60000;
    if (minutes < 60) return `${minutes} min`;
    return `${minutes / 60} hr`;
  }

  onMount(() => {
    loadGitInfo();
  });
</script>

<div class="git-settings">
  <h2 class="settings-title">Git Integration</h2>
  <p class="settings-description">
    Configure Git version control settings and behaviors.
  </p>

  <!-- Git Status -->
  <SettingsSection title="Git Status">
    {#if isLoading}
      <div class="git-status git-status--loading">
        <Icon name="loader" size={24} class="spinning" />
        <span>Detecting Git configuration...</span>
      </div>
    {:else if gitInfo}
      <div class="git-status git-status--found">
        <div class="git-status__icon">
          <Icon name="git-branch" size={32} />
        </div>
        <div class="git-status__info">
          <div class="git-status__row">
            <span class="git-status__label">Git Version:</span>
            <span class="git-status__value">{gitInfo.version}</span>
          </div>
          {#if gitInfo.configuredUser}
            <div class="git-status__row">
              <span class="git-status__label">Configured User:</span>
              <span class="git-status__value">
                {gitInfo.configuredUser.name} &lt;{gitInfo.configuredUser.email}&gt;
              </span>
            </div>
          {/if}
          <div class="git-status__row">
            <span class="git-status__label">SSH Key:</span>
            <span class="git-status__value">
              {gitInfo.sshKeyExists ? 'Found' : 'Not found'}
              {#if gitInfo.sshKeyExists}
                <Icon name="check-circle" size={14} class="success" />
              {:else}
                <Icon name="alert-circle" size={14} class="warning" />
              {/if}
            </span>
          </div>
          <div class="git-status__row">
            <span class="git-status__label">GPG Keys:</span>
            <span class="git-status__value">
              {gitInfo.gpgKeys.length} available
            </span>
          </div>
        </div>
        <div class="git-status__actions">
          <Button
            variant="secondary"
            size="small"
            disabled={isTestingGit}
            on:click={testGitSetup}
          >
            {#if isTestingGit}
              <Icon name="loader" size={14} class="spinning" />
            {:else}
              <Icon name="zap" size={14} />
            {/if}
            Test Setup
          </Button>
        </div>
      </div>

      {#if gitTestResult}
        <div
          class="git-test-result"
          class:git-test-result--success={gitTestResult.success}
          class:git-test-result--error={!gitTestResult.success}
        >
          <Icon
            name={gitTestResult.success ? 'check-circle' : 'x-circle'}
            size={16}
          />
          <span>{gitTestResult.message}</span>
        </div>
      {/if}
    {:else}
      <div class="git-status git-status--not-found">
        <Icon name="alert-triangle" size={32} />
        <div>
          <strong>Git not found</strong>
          <p>Please install Git to use version control features.</p>
        </div>
      </div>
    {/if}
  </SettingsSection>

  <!-- Enable/Disable -->
  <SettingsSection title="Integration">
    <SettingsRow
      label="Enable Git Integration"
      description="Use Git for version control within Tachikoma"
    >
      <Toggle
        checked={$gitSettings.enabled}
        on:change={handleToggle('enabled')}
      />
    </SettingsRow>
  </SettingsSection>

  {#if $gitSettings.enabled}
    <!-- Auto Fetch Section -->
    <SettingsSection title="Auto Fetch">
      <SettingsRow
        label="Auto Fetch"
        description="Automatically fetch changes from remote repositories"
      >
        <Toggle
          checked={$gitSettings.autoFetch}
          on:change={handleToggle('autoFetch')}
        />
      </SettingsRow>

      {#if $gitSettings.autoFetch}
        <SettingsRow
          label="Fetch Interval"
          description="How often to fetch from remotes"
        >
          <div class="slider-with-value">
            <Slider
              min={60000}
              max={3600000}
              step={60000}
              value={$gitSettings.fetchInterval}
              on:change={(e) => settingsStore.updateSetting('git', 'fetchInterval', e.detail)}
            />
            <span class="slider-value">{formatInterval($gitSettings.fetchInterval)}</span>
          </div>
        </SettingsRow>
      {/if}
    </SettingsSection>

    <!-- Push Settings -->
    <SettingsSection title="Push Settings">
      <SettingsRow
        label="Auto Push"
        description="Automatically push commits to remote after committing"
      >
        <Toggle
          checked={$gitSettings.autoPush}
          on:change={handleToggle('autoPush')}
        />
      </SettingsRow>

      <SettingsRow
        label="Default Branch"
        description="Branch name to use for new repositories"
      >
        <Select
          value={$gitSettings.defaultBranch}
          options={[
            { value: 'main', label: 'main' },
            { value: 'master', label: 'master' },
            { value: 'develop', label: 'develop' },
          ]}
          on:change={handleSelect('defaultBranch')}
        />
      </SettingsRow>
    </SettingsSection>

    <!-- Commit Signing -->
    <SettingsSection title="Commit Signing">
      <SettingsRow
        label="Sign Commits"
        description="Cryptographically sign commits with GPG"
      >
        <Toggle
          checked={$gitSettings.signCommits}
          on:change={handleToggle('signCommits')}
        />
      </SettingsRow>

      {#if $gitSettings.signCommits}
        <SettingsRow
          label="GPG Key"
          description="Select the GPG key to use for signing"
        >
          {#if gitInfo && gitInfo.gpgKeys.length > 0}
            <Select
              value={$gitSettings.gpgKey || ''}
              options={[
                { value: '', label: 'Select a key...' },
                ...gitInfo.gpgKeys.map(key => ({
                  value: key.id,
                  label: `${key.name} (${key.id.slice(-8)})`,
                })),
              ]}
              on:change={(e) => settingsStore.updateSetting('git', 'gpgKey', e.detail || undefined)}
            />
          {:else}
            <div class="no-keys-warning">
              <Icon name="alert-circle" size={14} />
              <span>No GPG keys found. Please create or import a GPG key.</span>
            </div>
          {/if}
        </SettingsRow>
      {/if}
    </SettingsSection>

    <!-- User Identity -->
    <SettingsSection title="User Identity">
      <p class="section-note">
        Override the global Git user identity for Tachikoma operations.
        Leave empty to use the global Git configuration.
      </p>

      <SettingsRow
        label="User Name"
        description="Name to use in commit author"
      >
        <Input
          type="text"
          value={$gitSettings.userName || ''}
          placeholder={gitInfo?.configuredUser?.name || 'Git user name'}
          on:input={handleInput('userName')}
        />
      </SettingsRow>

      <SettingsRow
        label="User Email"
        description="Email to use in commit author"
      >
        <Input
          type="email"
          value={$gitSettings.userEmail || ''}
          placeholder={gitInfo?.configuredUser?.email || 'Git user email'}
          on:input={handleInput('userEmail')}
        />
      </SettingsRow>
    </SettingsSection>
  {/if}
</div>

<style>
  .git-settings {
    max-width: 720px;
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

  /* Git Status */
  .git-status {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    background: var(--color-bg-secondary);
    border-radius: 8px;
  }

  .git-status--loading {
    color: var(--color-text-muted);
  }

  .git-status--not-found {
    color: var(--color-warning);
  }

  .git-status--not-found p {
    margin: 4px 0 0;
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .git-status__icon {
    width: 56px;
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-hover);
    border-radius: 12px;
    color: var(--color-primary);
  }

  .git-status__info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .git-status__row {
    display: flex;
    gap: 8px;
    font-size: 13px;
  }

  .git-status__label {
    color: var(--color-text-secondary);
    min-width: 120px;
  }

  .git-status__value {
    color: var(--color-text-primary);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .git-status__value :global(.success) {
    color: var(--color-success);
  }

  .git-status__value :global(.warning) {
    color: var(--color-warning);
  }

  .git-test-result {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    border-radius: 6px;
    font-size: 13px;
    margin-top: 12px;
  }

  .git-test-result--success {
    background: rgba(76, 175, 80, 0.1);
    color: var(--color-success);
  }

  .git-test-result--error {
    background: rgba(244, 67, 54, 0.1);
    color: var(--color-error);
  }

  .slider-with-value {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 200px;
  }

  .slider-value {
    min-width: 60px;
    text-align: right;
    font-size: 13px;
    color: var(--color-text-secondary);
    font-family: monospace;
  }

  .section-note {
    font-size: 13px;
    color: var(--color-text-secondary);
    margin: 0 0 16px 0;
    padding: 12px;
    background: var(--color-bg-secondary);
    border-radius: 6px;
  }

  .no-keys-warning {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--color-warning);
    padding: 8px 12px;
    background: rgba(255, 152, 0, 0.1);
    border-radius: 6px;
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

1. Git status detection works
2. Enable/disable toggle functions
3. Auto-fetch settings work
4. Fetch interval slider updates correctly
5. Auto-push toggle functions
6. Default branch selection works
7. Commit signing toggle works
8. GPG key selection shows available keys
9. User identity fields update store

### Test File (src/lib/components/settings/__tests__/GitSettings.test.ts)

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import GitSettings from '../GitSettings.svelte';
import { settingsStore } from '$lib/stores/settings-store';

vi.mock('$lib/ipc', () => ({
  invoke: vi.fn().mockImplementation((command: string) => {
    if (command === 'git_get_info') {
      return Promise.resolve({
        version: '2.40.0',
        configuredUser: { name: 'Test User', email: 'test@example.com' },
        gpgKeys: [{ id: 'ABCD1234', name: 'Test Key', email: 'test@example.com' }],
        sshKeyExists: true,
      });
    }
    if (command === 'git_test_setup') {
      return Promise.resolve({ success: true, message: 'Git is configured correctly' });
    }
    return Promise.resolve(null);
  }),
}));

describe('GitSettings', () => {
  beforeEach(() => {
    settingsStore.resetAll();
  });

  it('renders Git status after loading', async () => {
    render(GitSettings);

    await waitFor(() => {
      expect(screen.getByText('2.40.0')).toBeInTheDocument();
    });

    expect(screen.getByText(/Test User/)).toBeInTheDocument();
  });

  it('toggles Git integration', async () => {
    render(GitSettings);

    await waitFor(() => {
      expect(screen.getByText('2.40.0')).toBeInTheDocument();
    });

    const toggle = screen.getByRole('switch', { name: /enable git integration/i });
    await fireEvent.click(toggle);

    const state = get(settingsStore);
    expect(state.settings.git.enabled).toBe(false);
  });

  it('toggles auto-fetch', async () => {
    render(GitSettings);

    await waitFor(() => {
      expect(screen.getByRole('switch', { name: /auto fetch/i })).toBeInTheDocument();
    });

    const toggle = screen.getByRole('switch', { name: /auto fetch/i });
    await fireEvent.click(toggle);

    const state = get(settingsStore);
    expect(state.settings.git.autoFetch).toBe(false);
  });

  it('changes default branch', async () => {
    render(GitSettings);

    await waitFor(() => {
      expect(screen.getByText('main')).toBeInTheDocument();
    });

    const select = screen.getAllByRole('combobox').find(s =>
      s.querySelector('option[value="master"]')
    );
    await fireEvent.change(select!, { target: { value: 'master' } });

    const state = get(settingsStore);
    expect(state.settings.git.defaultBranch).toBe('master');
  });

  it('toggles commit signing', async () => {
    render(GitSettings);

    await waitFor(() => {
      expect(screen.getByRole('switch', { name: /sign commits/i })).toBeInTheDocument();
    });

    const toggle = screen.getByRole('switch', { name: /sign commits/i });
    await fireEvent.click(toggle);

    const state = get(settingsStore);
    expect(state.settings.git.signCommits).toBe(true);
  });

  it('tests Git setup', async () => {
    render(GitSettings);

    await waitFor(() => {
      expect(screen.getByText('Test Setup')).toBeInTheDocument();
    });

    const testButton = screen.getByText('Test Setup');
    await fireEvent.click(testButton);

    await waitFor(() => {
      expect(screen.getByText('Git is configured correctly')).toBeInTheDocument();
    });
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
- Previous: [277-settings-editor.md](277-settings-editor.md)
- Next: [279-settings-sync.md](279-settings-sync.md)
