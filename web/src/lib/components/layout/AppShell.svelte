<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import Sidebar from './Sidebar.svelte';
  import Header from './Header.svelte';
  import { missionStore } from '$lib/stores/mission';

  export let collapseSidebar = false;

  // Initialize mission store on app load
  onMount(() => {
    // Any initialization logic can go here
  });

  $: currentRoute = $page.route.id || '';
</script>

<div class="app-shell" class:sidebar-collapsed={collapseSidebar}>
  <Header />
  
  <div class="app-body">
    <Sidebar {currentRoute} bind:collapsed={collapseSidebar} />
    
    <main class="main-content">
      <slot />
    </main>
  </div>
</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    background: var(--bg);
    color: var(--text);
  }

  .app-body {
    flex: 1;
    display: flex;
    min-height: 0;
  }

  .main-content {
    flex: 1;
    padding: 1.5rem;
    overflow-y: auto;
    background: var(--bg);
  }

  .sidebar-collapsed .main-content {
    margin-left: 0;
  }

  @media (max-width: 768px) {
    .main-content {
      padding: 1rem;
    }
  }
</style>