# 283 - Settings Profiles

**Phase:** 13 - Settings UI
**Spec ID:** 283
**Status:** Planned
**Dependencies:** 271-settings-layout, 272-settings-store
**Estimated Context:** ~10% of model context window

---

## Objective

Create the Settings Profiles feature that allows users to save, load, and manage multiple named settings configurations for different projects, workflows, or contexts with quick switching capabilities.

---

## Acceptance Criteria

- [ ] `ProfilesSettings.svelte` component with profile management
- [ ] Create, edit, rename, and delete profiles
- [ ] Quick profile switching from header/menu
- [ ] Profile-specific settings overrides
- [ ] Default profile designation
- [ ] Profile import/export
- [ ] Auto-switch profiles based on project directory
- [ ] Profile comparison view

---

## Implementation Details

### 1. Profile Types (src/lib/types/profiles.ts)

```typescript
/**
 * Settings profile type definitions.
 */

export interface SettingsProfile {
  id: string;
  name: string;
  description?: string;
  icon?: string;
  color?: string;
  createdAt: number;
  updatedAt: number;
  isDefault: boolean;
  settings: Partial<import('./settings').AllSettings>;
  projectPatterns?: string[];
}

export interface ProfilesState {
  profiles: SettingsProfile[];
  activeProfileId: string | null;
  isLoading: boolean;
  error: string | null;
}

export interface ProfileSwitchOptions {
  saveCurrentFirst?: boolean;
  reloadWindow?: boolean;
}

export const PROFILE_ICONS = [
  'user', 'briefcase', 'home', 'code', 'terminal', 'git-branch',
  'folder', 'star', 'zap', 'coffee', 'moon', 'sun'
];

export const PROFILE_COLORS = [
  '#2196f3', '#9c27b0', '#e91e63', '#f44336', '#ff9800',
  '#4caf50', '#009688', '#00bcd4', '#3f51b5', '#607d8b'
];
```

### 2. Profiles Store (src/lib/stores/profiles-store.ts)

```typescript
import { writable, derived, get } from 'svelte/store';
import type { SettingsProfile, ProfilesState, ProfileSwitchOptions } from '$lib/types/profiles';
import { settingsStore } from './settings-store';
import { invoke } from '$lib/ipc';

function createProfilesStore() {
  const state = writable<ProfilesState>({
    profiles: [],
    activeProfileId: null,
    isLoading: true,
    error: null,
  });

  return {
    subscribe: state.subscribe,

    async init(): Promise<void> {
      state.update(s => ({ ...s, isLoading: true }));

      try {
        const profiles = await invoke<SettingsProfile[]>('profiles_load');
        const activeId = await invoke<string | null>('profiles_get_active');

        state.set({
          profiles,
          activeProfileId: activeId,
          isLoading: false,
          error: null,
        });
      } catch (error) {
        state.update(s => ({
          ...s,
          isLoading: false,
          error: (error as Error).message,
        }));
      }
    },

    async createProfile(
      name: string,
      description?: string,
      icon?: string,
      color?: string,
      copyFromCurrent: boolean = true
    ): Promise<SettingsProfile> {
      const currentSettings = copyFromCurrent ? get(settingsStore).settings : {};

      const profile: SettingsProfile = {
        id: crypto.randomUUID(),
        name,
        description,
        icon: icon || 'user',
        color: color || '#2196f3',
        createdAt: Date.now(),
        updatedAt: Date.now(),
        isDefault: false,
        settings: currentSettings,
      };

      await invoke('profiles_save', { profile });

      state.update(s => ({
        ...s,
        profiles: [...s.profiles, profile],
      }));

      return profile;
    },

    async updateProfile(
      profileId: string,
      updates: Partial<Omit<SettingsProfile, 'id' | 'createdAt'>>
    ): Promise<void> {
      state.update(s => ({
        ...s,
        profiles: s.profiles.map(p =>
          p.id === profileId
            ? { ...p, ...updates, updatedAt: Date.now() }
            : p
        ),
      }));

      const profile = get(state).profiles.find(p => p.id === profileId);
      if (profile) {
        await invoke('profiles_save', { profile });
      }
    },

    async deleteProfile(profileId: string): Promise<void> {
      const currentState = get(state);

      // Don't delete the active profile
      if (currentState.activeProfileId === profileId) {
        throw new Error('Cannot delete the active profile');
      }

      // Don't delete the default profile if it's the only one
      const profile = currentState.profiles.find(p => p.id === profileId);
      if (profile?.isDefault && currentState.profiles.length === 1) {
        throw new Error('Cannot delete the only default profile');
      }

      await invoke('profiles_delete', { profileId });

      state.update(s => ({
        ...s,
        profiles: s.profiles.filter(p => p.id !== profileId),
      }));
    },

    async switchToProfile(
      profileId: string,
      options: ProfileSwitchOptions = {}
    ): Promise<void> {
      const { saveCurrentFirst = true, reloadWindow = false } = options;
      const currentState = get(state);

      // Save current settings to active profile first
      if (saveCurrentFirst && currentState.activeProfileId) {
        await this.saveCurrentToProfile(currentState.activeProfileId);
      }

      // Load new profile settings
      const profile = currentState.profiles.find(p => p.id === profileId);
      if (!profile) {
        throw new Error('Profile not found');
      }

      // Apply profile settings
      const currentSettings = get(settingsStore).settings;
      const mergedSettings = {
        ...currentSettings,
        ...profile.settings,
      };

      settingsStore.setSettings(mergedSettings);
      await settingsStore.save();

      // Update active profile
      await invoke('profiles_set_active', { profileId });

      state.update(s => ({
        ...s,
        activeProfileId: profileId,
      }));

      if (reloadWindow) {
        await invoke('app_reload');
      }
    },

    async saveCurrentToProfile(profileId: string): Promise<void> {
      const currentSettings = get(settingsStore).settings;

      await this.updateProfile(profileId, {
        settings: currentSettings,
      });
    },

    async setDefaultProfile(profileId: string): Promise<void> {
      state.update(s => ({
        ...s,
        profiles: s.profiles.map(p => ({
          ...p,
          isDefault: p.id === profileId,
        })),
      }));

      // Save all profiles to persist default change
      for (const profile of get(state).profiles) {
        await invoke('profiles_save', { profile });
      }
    },

    async duplicateProfile(profileId: string, newName: string): Promise<SettingsProfile> {
      const source = get(state).profiles.find(p => p.id === profileId);
      if (!source) {
        throw new Error('Profile not found');
      }

      return this.createProfile(
        newName,
        source.description,
        source.icon,
        source.color,
        false
      ).then(async (newProfile) => {
        await this.updateProfile(newProfile.id, { settings: source.settings });
        return newProfile;
      });
    },

    async addProjectPattern(profileId: string, pattern: string): Promise<void> {
      const profile = get(state).profiles.find(p => p.id === profileId);
      if (!profile) return;

      const patterns = [...(profile.projectPatterns || []), pattern];
      await this.updateProfile(profileId, { projectPatterns: patterns });
    },

    async removeProjectPattern(profileId: string, pattern: string): Promise<void> {
      const profile = get(state).profiles.find(p => p.id === profileId);
      if (!profile) return;

      const patterns = (profile.projectPatterns || []).filter(p => p !== pattern);
      await this.updateProfile(profileId, { projectPatterns: patterns });
    },

    async findMatchingProfile(projectPath: string): Promise<SettingsProfile | null> {
      const profiles = get(state).profiles;

      for (const profile of profiles) {
        if (profile.projectPatterns) {
          for (const pattern of profile.projectPatterns) {
            if (matchesPattern(projectPath, pattern)) {
              return profile;
            }
          }
        }
      }

      return profiles.find(p => p.isDefault) || null;
    },

    exportProfile(profileId: string): string {
      const profile = get(state).profiles.find(p => p.id === profileId);
      if (!profile) {
        throw new Error('Profile not found');
      }

      return JSON.stringify({
        type: 'tachikoma-profile',
        version: 1,
        profile: {
          ...profile,
          id: undefined, // Remove ID for export
          isDefault: false,
        },
      }, null, 2);
    },

    async importProfile(json: string): Promise<SettingsProfile> {
      const data = JSON.parse(json);

      if (data.type !== 'tachikoma-profile') {
        throw new Error('Invalid profile file');
      }

      const profile: SettingsProfile = {
        ...data.profile,
        id: crypto.randomUUID(),
        createdAt: Date.now(),
        updatedAt: Date.now(),
        isDefault: false,
      };

      await invoke('profiles_save', { profile });

      state.update(s => ({
        ...s,
        profiles: [...s.profiles, profile],
      }));

      return profile;
    },
  };
}

function matchesPattern(path: string, pattern: string): boolean {
  // Simple glob matching
  const regex = new RegExp(
    '^' + pattern.replace(/\*/g, '.*').replace(/\?/g, '.') + '$'
  );
  return regex.test(path);
}

export const profilesStore = createProfilesStore();

export const activeProfile = derived(
  profilesStore,
  ($state) => $state.profiles.find(p => p.id === $state.activeProfileId) || null
);

export const defaultProfile = derived(
  profilesStore,
  ($state) => $state.profiles.find(p => p.isDefault) || null
);
```

### 3. Profiles Settings Component (src/lib/components/settings/ProfilesSettings.svelte)

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { profilesStore, activeProfile, defaultProfile } from '$lib/stores/profiles-store';
  import { PROFILE_ICONS, PROFILE_COLORS } from '$lib/types/profiles';
  import type { SettingsProfile } from '$lib/types/profiles';
  import SettingsSection from './SettingsSection.svelte';
  import SettingsRow from './SettingsRow.svelte';
  import Toggle from '$lib/components/ui/Toggle.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';

  let showCreateModal = false;
  let showEditModal = false;
  let editingProfile: SettingsProfile | null = null;

  // Create form state
  let newName = '';
  let newDescription = '';
  let newIcon = 'user';
  let newColor = '#2196f3';
  let copyFromCurrent = true;

  // Edit form state
  let editName = '';
  let editDescription = '';
  let editIcon = '';
  let editColor = '';
  let newPattern = '';

  let isCreating = false;
  let isSwitching = false;

  async function handleCreate() {
    if (!newName.trim()) return;

    isCreating = true;
    try {
      const profile = await profilesStore.createProfile(
        newName,
        newDescription,
        newIcon,
        newColor,
        copyFromCurrent
      );

      showCreateModal = false;
      resetCreateForm();
    } catch (error) {
      console.error('Failed to create profile:', error);
    }
    isCreating = false;
  }

  function resetCreateForm() {
    newName = '';
    newDescription = '';
    newIcon = 'user';
    newColor = '#2196f3';
    copyFromCurrent = true;
  }

  function openEditModal(profile: SettingsProfile) {
    editingProfile = profile;
    editName = profile.name;
    editDescription = profile.description || '';
    editIcon = profile.icon || 'user';
    editColor = profile.color || '#2196f3';
    showEditModal = true;
  }

  async function handleEdit() {
    if (!editingProfile || !editName.trim()) return;

    await profilesStore.updateProfile(editingProfile.id, {
      name: editName,
      description: editDescription,
      icon: editIcon,
      color: editColor,
    });

    showEditModal = false;
    editingProfile = null;
  }

  async function handleDelete(profile: SettingsProfile) {
    if (!confirm(`Are you sure you want to delete "${profile.name}"?`)) return;

    try {
      await profilesStore.deleteProfile(profile.id);
    } catch (error) {
      alert((error as Error).message);
    }
  }

  async function handleSwitch(profile: SettingsProfile) {
    if ($profilesStore.activeProfileId === profile.id) return;

    isSwitching = true;
    try {
      await profilesStore.switchToProfile(profile.id);
    } catch (error) {
      console.error('Failed to switch profile:', error);
    }
    isSwitching = false;
  }

  async function handleSetDefault(profile: SettingsProfile) {
    await profilesStore.setDefaultProfile(profile.id);
  }

  async function handleDuplicate(profile: SettingsProfile) {
    const newName = `${profile.name} (Copy)`;
    await profilesStore.duplicateProfile(profile.id, newName);
  }

  async function handleAddPattern() {
    if (!editingProfile || !newPattern.trim()) return;

    await profilesStore.addProjectPattern(editingProfile.id, newPattern);
    newPattern = '';

    // Refresh editing profile
    editingProfile = $profilesStore.profiles.find(p => p.id === editingProfile?.id) || null;
  }

  async function handleRemovePattern(pattern: string) {
    if (!editingProfile) return;

    await profilesStore.removeProjectPattern(editingProfile.id, pattern);

    // Refresh editing profile
    editingProfile = $profilesStore.profiles.find(p => p.id === editingProfile?.id) || null;
  }

  function handleExport(profile: SettingsProfile) {
    const json = profilesStore.exportProfile(profile.id);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `tachikoma-profile-${profile.name.toLowerCase().replace(/\s+/g, '-')}.json`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  }

  async function handleImport() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;

      try {
        const json = await file.text();
        await profilesStore.importProfile(json);
      } catch (error) {
        alert('Failed to import profile: ' + (error as Error).message);
      }
    };
    input.click();
  }

  onMount(() => {
    profilesStore.init();
  });
</script>

<div class="profiles-settings">
  <h2 class="settings-title">Settings Profiles</h2>
  <p class="settings-description">
    Create and manage multiple settings configurations for different workflows.
  </p>

  <!-- Active Profile -->
  {#if $activeProfile}
    <SettingsSection title="Active Profile">
      <div class="active-profile">
        <div
          class="profile-avatar"
          style="background: {$activeProfile.color}"
        >
          <Icon name={$activeProfile.icon || 'user'} size={24} />
        </div>
        <div class="active-profile__info">
          <span class="active-profile__name">
            {$activeProfile.name}
            {#if $activeProfile.isDefault}
              <span class="badge badge--default">Default</span>
            {/if}
          </span>
          {#if $activeProfile.description}
            <span class="active-profile__desc">{$activeProfile.description}</span>
          {/if}
        </div>
        <Button
          variant="secondary"
          size="small"
          on:click={() => profilesStore.saveCurrentToProfile($activeProfile.id)}
        >
          <Icon name="save" size={14} />
          Save Changes
        </Button>
      </div>
    </SettingsSection>
  {/if}

  <!-- All Profiles -->
  <SettingsSection title="All Profiles">
    <div class="profiles-toolbar">
      <span class="profiles-count">
        {$profilesStore.profiles.length} profile{$profilesStore.profiles.length !== 1 ? 's' : ''}
      </span>
      <div class="profiles-actions">
        <Button variant="ghost" size="small" on:click={handleImport}>
          <Icon name="upload" size={14} />
          Import
        </Button>
        <Button variant="primary" size="small" on:click={() => showCreateModal = true}>
          <Icon name="plus" size={14} />
          New Profile
        </Button>
      </div>
    </div>

    <div class="profiles-list">
      {#each $profilesStore.profiles as profile}
        <div
          class="profile-card"
          class:profile-card--active={profile.id === $profilesStore.activeProfileId}
        >
          <div
            class="profile-card__avatar"
            style="background: {profile.color}"
          >
            <Icon name={profile.icon || 'user'} size={20} />
          </div>

          <div class="profile-card__info">
            <span class="profile-card__name">
              {profile.name}
              {#if profile.isDefault}
                <span class="badge badge--default">Default</span>
              {/if}
              {#if profile.id === $profilesStore.activeProfileId}
                <span class="badge badge--active">Active</span>
              {/if}
            </span>
            {#if profile.description}
              <span class="profile-card__desc">{profile.description}</span>
            {/if}
            {#if profile.projectPatterns && profile.projectPatterns.length > 0}
              <span class="profile-card__patterns">
                <Icon name="folder" size={12} />
                {profile.projectPatterns.length} project pattern{profile.projectPatterns.length !== 1 ? 's' : ''}
              </span>
            {/if}
          </div>

          <div class="profile-card__actions">
            {#if profile.id !== $profilesStore.activeProfileId}
              <Button
                variant="secondary"
                size="small"
                disabled={isSwitching}
                on:click={() => handleSwitch(profile)}
              >
                Switch
              </Button>
            {/if}

            <div class="profile-card__menu">
              <Button variant="ghost" size="small" on:click={() => openEditModal(profile)}>
                <Icon name="edit-2" size={14} />
              </Button>
              <Button variant="ghost" size="small" on:click={() => handleDuplicate(profile)}>
                <Icon name="copy" size={14} />
              </Button>
              <Button variant="ghost" size="small" on:click={() => handleExport(profile)}>
                <Icon name="download" size={14} />
              </Button>
              {#if !profile.isDefault}
                <Button variant="ghost" size="small" on:click={() => handleSetDefault(profile)}>
                  <Icon name="star" size={14} />
                </Button>
              {/if}
              {#if profile.id !== $profilesStore.activeProfileId}
                <Button variant="ghost" size="small" on:click={() => handleDelete(profile)}>
                  <Icon name="trash-2" size={14} />
                </Button>
              {/if}
            </div>
          </div>
        </div>
      {/each}
    </div>
  </SettingsSection>
</div>

<!-- Create Profile Modal -->
{#if showCreateModal}
  <Modal title="Create Profile" on:close={() => showCreateModal = false}>
    <div class="profile-form">
      <div class="form-group">
        <label for="profile-name">Name</label>
        <Input
          id="profile-name"
          bind:value={newName}
          placeholder="My Profile"
        />
      </div>

      <div class="form-group">
        <label for="profile-desc">Description (optional)</label>
        <Input
          id="profile-desc"
          bind:value={newDescription}
          placeholder="Profile for..."
        />
      </div>

      <div class="form-group">
        <label>Icon</label>
        <div class="icon-picker">
          {#each PROFILE_ICONS as icon}
            <button
              class="icon-option"
              class:icon-option--selected={newIcon === icon}
              on:click={() => newIcon = icon}
            >
              <Icon name={icon} size={20} />
            </button>
          {/each}
        </div>
      </div>

      <div class="form-group">
        <label>Color</label>
        <div class="color-picker">
          {#each PROFILE_COLORS as color}
            <button
              class="color-option"
              class:color-option--selected={newColor === color}
              style="background: {color}"
              on:click={() => newColor = color}
            />
          {/each}
        </div>
      </div>

      <div class="form-group form-group--horizontal">
        <Toggle bind:checked={copyFromCurrent} />
        <span>Copy current settings to profile</span>
      </div>

      <div class="form-actions">
        <Button variant="secondary" on:click={() => showCreateModal = false}>
          Cancel
        </Button>
        <Button
          variant="primary"
          disabled={!newName.trim() || isCreating}
          on:click={handleCreate}
        >
          {#if isCreating}
            <Icon name="loader" size={14} class="spinning" />
          {/if}
          Create Profile
        </Button>
      </div>
    </div>
  </Modal>
{/if}

<!-- Edit Profile Modal -->
{#if showEditModal && editingProfile}
  <Modal title="Edit Profile" on:close={() => showEditModal = false}>
    <div class="profile-form">
      <div class="form-group">
        <label for="edit-name">Name</label>
        <Input
          id="edit-name"
          bind:value={editName}
        />
      </div>

      <div class="form-group">
        <label for="edit-desc">Description</label>
        <Input
          id="edit-desc"
          bind:value={editDescription}
        />
      </div>

      <div class="form-group">
        <label>Icon</label>
        <div class="icon-picker">
          {#each PROFILE_ICONS as icon}
            <button
              class="icon-option"
              class:icon-option--selected={editIcon === icon}
              on:click={() => editIcon = icon}
            >
              <Icon name={icon} size={20} />
            </button>
          {/each}
        </div>
      </div>

      <div class="form-group">
        <label>Color</label>
        <div class="color-picker">
          {#each PROFILE_COLORS as color}
            <button
              class="color-option"
              class:color-option--selected={editColor === color}
              style="background: {color}"
              on:click={() => editColor = color}
            />
          {/each}
        </div>
      </div>

      <div class="form-group">
        <label>Project Patterns</label>
        <p class="form-hint">
          Automatically switch to this profile when opening projects matching these patterns.
        </p>
        <div class="patterns-input">
          <Input
            bind:value={newPattern}
            placeholder="*/my-project/*"
          />
          <Button variant="secondary" size="small" on:click={handleAddPattern}>
            Add
          </Button>
        </div>
        {#if editingProfile.projectPatterns && editingProfile.projectPatterns.length > 0}
          <div class="patterns-list">
            {#each editingProfile.projectPatterns as pattern}
              <div class="pattern-tag">
                <code>{pattern}</code>
                <button on:click={() => handleRemovePattern(pattern)}>
                  <Icon name="x" size={12} />
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class="form-actions">
        <Button variant="secondary" on:click={() => showEditModal = false}>
          Cancel
        </Button>
        <Button
          variant="primary"
          disabled={!editName.trim()}
          on:click={handleEdit}
        >
          Save Changes
        </Button>
      </div>
    </div>
  </Modal>
{/if}

<style>
  .profiles-settings {
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

  /* Active Profile */
  .active-profile {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    background: var(--color-bg-secondary);
    border-radius: 8px;
  }

  .profile-avatar {
    width: 56px;
    height: 56px;
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
  }

  .active-profile__info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .active-profile__name {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .active-profile__desc {
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .badge {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: 500;
  }

  .badge--default {
    background: var(--color-warning);
    color: white;
  }

  .badge--active {
    background: var(--color-success);
    color: white;
  }

  /* Profiles Toolbar */
  .profiles-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .profiles-count {
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .profiles-actions {
    display: flex;
    gap: 8px;
  }

  /* Profiles List */
  .profiles-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .profile-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: var(--color-bg-secondary);
    border: 2px solid transparent;
    border-radius: 8px;
    transition: all 0.15s ease;
  }

  .profile-card:hover {
    background: var(--color-bg-hover);
  }

  .profile-card--active {
    border-color: var(--color-primary);
    background: rgba(33, 150, 243, 0.05);
  }

  .profile-card__avatar {
    width: 40px;
    height: 40px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
  }

  .profile-card__info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .profile-card__name {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .profile-card__desc {
    font-size: 12px;
    color: var(--color-text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .profile-card__patterns {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .profile-card__actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .profile-card__menu {
    display: flex;
    gap: 4px;
  }

  /* Form Styles */
  .profile-form {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 8px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .form-group--horizontal {
    flex-direction: row;
    align-items: center;
    gap: 12px;
  }

  .form-group label {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .form-hint {
    font-size: 12px;
    color: var(--color-text-muted);
    margin: 0;
  }

  .icon-picker {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .icon-option {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 2px solid var(--color-border);
    border-radius: 8px;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .icon-option:hover {
    border-color: var(--color-text-secondary);
    color: var(--color-text-primary);
  }

  .icon-option--selected {
    border-color: var(--color-primary);
    color: var(--color-primary);
    background: rgba(33, 150, 243, 0.1);
  }

  .color-picker {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .color-option {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    transition: transform 0.15s ease;
  }

  .color-option:hover {
    transform: scale(1.1);
  }

  .color-option--selected {
    border-color: var(--color-text-primary);
    box-shadow: 0 0 0 2px var(--color-bg-primary);
  }

  .patterns-input {
    display: flex;
    gap: 8px;
  }

  .patterns-input :global(input) {
    flex: 1;
  }

  .patterns-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 8px;
  }

  .pattern-tag {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    background: var(--color-bg-secondary);
    border-radius: 4px;
  }

  .pattern-tag code {
    font-size: 12px;
    color: var(--color-text-primary);
  }

  .pattern-tag button {
    padding: 2px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .pattern-tag button:hover {
    color: var(--color-error);
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 8px;
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

1. Profile list renders correctly
2. Create profile modal works
3. Edit profile updates correctly
4. Delete profile works (not active/default)
5. Switch profile applies settings
6. Set default profile works
7. Duplicate profile creates copy
8. Export/import profiles work
9. Project patterns match correctly

### Test File (src/lib/components/settings/__tests__/ProfilesSettings.test.ts)

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import ProfilesSettings from '../ProfilesSettings.svelte';
import { profilesStore } from '$lib/stores/profiles-store';

vi.mock('$lib/ipc', () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    if (cmd === 'profiles_load') {
      return Promise.resolve([
        { id: '1', name: 'Default', isDefault: true, settings: {} },
        { id: '2', name: 'Work', isDefault: false, settings: {} },
      ]);
    }
    if (cmd === 'profiles_get_active') {
      return Promise.resolve('1');
    }
    return Promise.resolve(null);
  }),
}));

describe('ProfilesSettings', () => {
  beforeEach(() => {
    profilesStore.init();
  });

  it('renders profile list', async () => {
    render(ProfilesSettings);

    await waitFor(() => {
      expect(screen.getByText('Default')).toBeInTheDocument();
      expect(screen.getByText('Work')).toBeInTheDocument();
    });
  });

  it('opens create profile modal', async () => {
    render(ProfilesSettings);

    await waitFor(() => {
      expect(screen.getByText('New Profile')).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByText('New Profile'));

    expect(screen.getByText('Create Profile')).toBeInTheDocument();
  });

  it('creates new profile', async () => {
    render(ProfilesSettings);

    await waitFor(() => {
      expect(screen.getByText('New Profile')).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByText('New Profile'));

    const nameInput = screen.getByPlaceholderText('My Profile');
    await fireEvent.input(nameInput, { target: { value: 'Test Profile' } });

    await fireEvent.click(screen.getByText('Create Profile'));

    // Verify profile was created (mock should have been called)
    expect(screen.queryByText('Create Profile')).not.toBeInTheDocument();
  });

  it('shows active profile badge', async () => {
    render(ProfilesSettings);

    await waitFor(() => {
      expect(screen.getByText('Active')).toBeInTheDocument();
    });
  });

  it('shows default profile badge', async () => {
    render(ProfilesSettings);

    await waitFor(() => {
      const defaultBadges = screen.getAllByText('Default');
      expect(defaultBadges.length).toBeGreaterThan(0);
    });
  });
});
```

---

## Related Specs

- Depends on: [271-settings-layout.md](271-settings-layout.md)
- Depends on: [272-settings-store.md](272-settings-store.md)
- Previous: [282-settings-import.md](282-settings-import.md)
- Next: [284-settings-validation.md](284-settings-validation.md)
