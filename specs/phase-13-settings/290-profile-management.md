# Spec 290: Profile Management

## Header
- **Spec ID**: 290
- **Phase**: 13 - Settings UI
- **Component**: Profile Management
- **Dependencies**: Spec 289 (Export/Import)
- **Status**: Draft

## Objective
Create a profile management interface that allows users to create, switch between, and manage multiple configuration profiles for different use cases, workspaces, or team environments.

## Acceptance Criteria
- [x] Create and manage multiple profiles
- [x] Switch between profiles quickly
- [x] Configure profile-specific settings
- [x] Share profiles with team members
- [x] Import/export individual profiles
- [x] Set default profile
- [x] Clone existing profiles
- [x] View profile change history

## Implementation

### ProfileManagement.svelte
```svelte
<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { writable, derived } from 'svelte/store';
  import { slide, fade, fly } from 'svelte/transition';
  import ProfileEditor from './ProfileEditor.svelte';
  import ProfileComparison from './ProfileComparison.svelte';
  import ShareProfile from './ShareProfile.svelte';
  import { profileStore } from '$lib/stores/profiles';
  import type {
    Profile,
    ProfileSettings,
    ProfileHistory
  } from '$lib/types/settings';

  const dispatch = createEventDispatcher<{
    switch: Profile;
    create: Profile;
    update: Profile;
    delete: string;
  }>();

  let showCreateModal = writable<boolean>(false);
  let showEditModal = writable<boolean>(false);
  let showShareModal = writable<boolean>(false);
  let showCompareModal = writable<boolean>(false);
  let editingProfile = writable<Profile | null>(null);
  let compareProfiles = writable<[Profile, Profile] | null>(null);
  let searchQuery = writable<string>('');

  const profiles = derived(profileStore, ($store) => $store.profiles);
  const activeProfile = derived(profileStore, ($store) => $store.activeProfile);
  const defaultProfileId = derived(profileStore, ($store) => $store.defaultProfileId);

  const filteredProfiles = derived([profiles, searchQuery], ([$profiles, $query]) => {
    if (!$query) return $profiles;
    const q = $query.toLowerCase();
    return $profiles.filter(p =>
      p.name.toLowerCase().includes(q) ||
      p.description?.toLowerCase().includes(q) ||
      p.tags?.some(t => t.toLowerCase().includes(q))
    );
  });

  const sortedProfiles = derived(filteredProfiles, ($filtered) =>
    [...$filtered].sort((a, b) => {
      if (a.id === $defaultProfileId) return -1;
      if (b.id === $defaultProfileId) return 1;
      return new Date(b.lastUsed || 0).getTime() - new Date(a.lastUsed || 0).getTime();
    })
  );

  function formatDate(date: Date | null): string {
    if (!date) return 'Never';
    return new Date(date).toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    });
  }

  async function switchProfile(profile: Profile) {
    if (profile.id === $activeProfile?.id) return;

    const confirmed = await confirmSwitch(profile);
    if (confirmed) {
      await profileStore.switchTo(profile.id);
      dispatch('switch', profile);
    }
  }

  async function confirmSwitch(profile: Profile): Promise<boolean> {
    if (profileStore.hasUnsavedChanges()) {
      return confirm(`You have unsaved changes. Switch to "${profile.name}" anyway?`);
    }
    return true;
  }

  function openCreateModal() {
    editingProfile.set(null);
    showCreateModal.set(true);
  }

  function openEditModal(profile: Profile) {
    editingProfile.set(profile);
    showEditModal.set(true);
  }

  async function createProfile(profile: Omit<Profile, 'id' | 'createdAt'>) {
    const newProfile = await profileStore.create(profile);
    dispatch('create', newProfile);
    showCreateModal.set(false);
  }

  async function updateProfile(profile: Profile) {
    await profileStore.update(profile);
    dispatch('update', profile);
    showEditModal.set(false);
    editingProfile.set(null);
  }

  async function deleteProfile(profileId: string) {
    const profile = $profiles.find(p => p.id === profileId);
    if (!profile) return;

    if (profile.id === $activeProfile?.id) {
      alert('Cannot delete the active profile. Switch to another profile first.');
      return;
    }

    if (confirm(`Delete profile "${profile.name}"? This cannot be undone.`)) {
      await profileStore.delete(profileId);
      dispatch('delete', profileId);
    }
  }

  async function cloneProfile(profile: Profile) {
    const cloned = await profileStore.clone(profile.id, `${profile.name} (Copy)`);
    dispatch('create', cloned);
  }

  async function setAsDefault(profileId: string) {
    await profileStore.setDefault(profileId);
  }

  function exportProfile(profile: Profile) {
    const data = JSON.stringify(profile, null, 2);
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `profile-${profile.name.toLowerCase().replace(/\s+/g, '-')}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function importProfile(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = async (e) => {
      try {
        const imported = JSON.parse(e.target?.result as string);
        const profile = await profileStore.import(imported);
        dispatch('create', profile);
      } catch (err) {
        alert('Failed to import profile: ' + (err as Error).message);
      }
    };
    reader.readAsText(file);
  }

  function openShareModal(profile: Profile) {
    editingProfile.set(profile);
    showShareModal.set(true);
  }

  function openCompareModal(profile1: Profile, profile2: Profile) {
    compareProfiles.set([profile1, profile2]);
    showCompareModal.set(true);
  }

  onMount(() => {
    profileStore.load();
  });
</script>

<div class="profile-management" data-testid="profile-management">
  <header class="config-header">
    <div class="header-title">
      <h2>Profile Management</h2>
      <p class="description">Manage configuration profiles for different use cases</p>
    </div>

    <div class="header-actions">
      <label class="btn secondary import-btn">
        Import
        <input type="file" accept=".json" on:change={importProfile} hidden />
      </label>
      <button class="btn primary" on:click={openCreateModal}>
        Create Profile
      </button>
    </div>
  </header>

  {#if $activeProfile}
    <section class="active-profile-banner">
      <div class="active-profile-info">
        <span class="active-label">Active Profile</span>
        <span class="active-name">{$activeProfile.name}</span>
        {#if $activeProfile.description}
          <span class="active-desc">{$activeProfile.description}</span>
        {/if}
      </div>
      <div class="active-profile-actions">
        <button class="btn secondary small" on:click={() => openEditModal($activeProfile)}>
          Edit
        </button>
      </div>
    </section>
  {/if}

  <div class="search-bar">
    <input
      type="text"
      placeholder="Search profiles..."
      bind:value={$searchQuery}
    />
  </div>

  <section class="profiles-grid">
    {#each $sortedProfiles as profile (profile.id)}
      <div
        class="profile-card"
        class:active={profile.id === $activeProfile?.id}
        class:default={profile.id === $defaultProfileId}
        transition:fly={{ y: 20, duration: 200 }}
      >
        <div class="profile-header">
          <div class="profile-title">
            <h4>{profile.name}</h4>
            {#if profile.id === $defaultProfileId}
              <span class="badge default">Default</span>
            {/if}
            {#if profile.id === $activeProfile?.id}
              <span class="badge active">Active</span>
            {/if}
          </div>
          {#if profile.icon}
            <span class="profile-icon">{profile.icon}</span>
          {/if}
        </div>

        {#if profile.description}
          <p class="profile-desc">{profile.description}</p>
        {/if}

        {#if profile.tags && profile.tags.length > 0}
          <div class="profile-tags">
            {#each profile.tags as tag}
              <span class="tag">{tag}</span>
            {/each}
          </div>
        {/if}

        <div class="profile-meta">
          <span>Created: {formatDate(profile.createdAt)}</span>
          <span>Last used: {formatDate(profile.lastUsed)}</span>
        </div>

        <div class="profile-stats">
          <div class="stat">
            <span class="stat-value">{profile.sessionCount || 0}</span>
            <span class="stat-label">Sessions</span>
          </div>
          <div class="stat">
            <span class="stat-value">{profile.templateCount || 0}</span>
            <span class="stat-label">Templates</span>
          </div>
        </div>

        <div class="profile-actions">
          {#if profile.id !== $activeProfile?.id}
            <button
              class="btn primary small"
              on:click={() => switchProfile(profile)}
            >
              Switch
            </button>
          {/if}
          <button
            class="btn secondary small"
            on:click={() => openEditModal(profile)}
          >
            Edit
          </button>
          <div class="dropdown">
            <button class="btn icon-btn">...</button>
            <div class="dropdown-menu">
              <button on:click={() => cloneProfile(profile)}>Clone</button>
              <button on:click={() => exportProfile(profile)}>Export</button>
              <button on:click={() => openShareModal(profile)}>Share</button>
              {#if profile.id !== $defaultProfileId}
                <button on:click={() => setAsDefault(profile.id)}>Set as Default</button>
              {/if}
              {#if $profiles.length > 1 && profile.id !== $activeProfile?.id}
                <button class="danger" on:click={() => deleteProfile(profile.id)}>Delete</button>
              {/if}
            </div>
          </div>
        </div>
      </div>
    {/each}
  </section>

  {#if $filteredProfiles.length === 0}
    <div class="empty-state">
      {#if $searchQuery}
        <p>No profiles match your search</p>
        <button class="link-btn" on:click={() => searchQuery.set('')}>Clear search</button>
      {:else}
        <p>No profiles yet</p>
        <p class="hint">Create a profile to save your configuration</p>
      {/if}
    </div>
  {/if}

  {#if $showCreateModal}
    <div class="modal-overlay" transition:fade on:click={() => showCreateModal.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <ProfileEditor
          mode="create"
          on:save={(e) => createProfile(e.detail)}
          on:close={() => showCreateModal.set(false)}
        />
      </div>
    </div>
  {/if}

  {#if $showEditModal && $editingProfile}
    <div class="modal-overlay" transition:fade on:click={() => showEditModal.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <ProfileEditor
          mode="edit"
          profile={$editingProfile}
          on:save={(e) => updateProfile(e.detail)}
          on:close={() => {
            showEditModal.set(false);
            editingProfile.set(null);
          }}
        />
      </div>
    </div>
  {/if}

  {#if $showShareModal && $editingProfile}
    <div class="modal-overlay" transition:fade on:click={() => showShareModal.set(false)}>
      <div class="modal-content" on:click|stopPropagation>
        <ShareProfile
          profile={$editingProfile}
          on:close={() => {
            showShareModal.set(false);
            editingProfile.set(null);
          }}
        />
      </div>
    </div>
  {/if}

  {#if $showCompareModal && $compareProfiles}
    <div class="modal-overlay" transition:fade on:click={() => showCompareModal.set(false)}>
      <div class="modal-content large" on:click|stopPropagation>
        <ProfileComparison
          profiles={$compareProfiles}
          on:close={() => {
            showCompareModal.set(false);
            compareProfiles.set(null);
          }}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .profile-management {
    max-width: 1200px;
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

  .active-profile-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.25rem;
    background: linear-gradient(135deg, var(--primary-alpha) 0%, var(--card-bg) 100%);
    border: 1px solid var(--primary-color);
    border-radius: 8px;
    margin-bottom: 1.5rem;
  }

  .active-label {
    font-size: 0.75rem;
    color: var(--primary-color);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .active-name {
    display: block;
    font-size: 1.125rem;
    font-weight: 600;
    margin-top: 0.25rem;
  }

  .active-desc {
    display: block;
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-top: 0.25rem;
  }

  .search-bar {
    margin-bottom: 1.5rem;
  }

  .search-bar input {
    width: 100%;
    padding: 0.75rem 1rem;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    background: var(--input-bg);
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  .profiles-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 1.25rem;
  }

  .profile-card {
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.25rem;
    transition: all 0.2s ease;
  }

  .profile-card:hover {
    border-color: var(--primary-color);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .profile-card.active {
    border-color: var(--primary-color);
    background: var(--primary-alpha);
  }

  .profile-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 0.75rem;
  }

  .profile-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .profile-title h4 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0;
  }

  .profile-icon {
    font-size: 1.5rem;
  }

  .badge {
    padding: 0.125rem 0.5rem;
    border-radius: 4px;
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .badge.default {
    background: var(--secondary-bg);
    color: var(--text-secondary);
  }

  .badge.active {
    background: var(--primary-color);
    color: white;
  }

  .profile-desc {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: 0.75rem;
    line-height: 1.4;
  }

  .profile-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.375rem;
    margin-bottom: 0.75rem;
  }

  .tag {
    padding: 0.125rem 0.5rem;
    background: var(--secondary-bg);
    border-radius: 4px;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .profile-meta {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-bottom: 0.75rem;
  }

  .profile-stats {
    display: flex;
    gap: 1.5rem;
    padding: 0.75rem 0;
    border-top: 1px solid var(--border-color);
    border-bottom: 1px solid var(--border-color);
    margin-bottom: 0.75rem;
  }

  .stat {
    text-align: center;
  }

  .stat-value {
    display: block;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--primary-color);
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .profile-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .dropdown {
    position: relative;
    margin-left: auto;
  }

  .dropdown-menu {
    display: none;
    position: absolute;
    right: 0;
    top: 100%;
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    min-width: 150px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 10;
  }

  .dropdown:hover .dropdown-menu {
    display: block;
  }

  .dropdown-menu button {
    display: block;
    width: 100%;
    padding: 0.625rem 1rem;
    background: none;
    border: none;
    text-align: left;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .dropdown-menu button:hover {
    background: var(--hover-bg);
  }

  .dropdown-menu button.danger {
    color: var(--error-color);
  }

  .icon-btn {
    padding: 0.375rem 0.625rem;
    font-weight: bold;
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

  .link-btn {
    background: none;
    border: none;
    color: var(--primary-color);
    cursor: pointer;
    text-decoration: underline;
  }

  .btn {
    padding: 0.625rem 1.25rem;
    border: none;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
  }

  .btn.small {
    padding: 0.375rem 0.75rem;
    font-size: 0.8125rem;
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

  .import-btn {
    cursor: pointer;
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

  @media (max-width: 768px) {
    .profiles-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
```

## Testing Requirements
1. **Unit Tests**: Test profile CRUD operations
2. **Switch Tests**: Test profile switching
3. **Export/Import Tests**: Test profile serialization
4. **Share Tests**: Test profile sharing
5. **Clone Tests**: Test profile cloning

## Related Specs
- Spec 289: Export/Import
- Spec 291: Workspace Settings
- Spec 295: Settings Tests
