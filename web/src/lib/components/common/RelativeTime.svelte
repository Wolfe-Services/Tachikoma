<script lang="ts">
  import { onMount } from 'svelte';

  export let date: Date | string;
  export let updateInterval: number = 60000; // Update every minute

  let relativeTime = '';

  function formatRelativeTime(date: Date): string {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (seconds < 60) return 'just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;
    
    return date.toLocaleDateString();
  }

  function updateTime() {
    const dateObj = date instanceof Date ? date : new Date(date);
    relativeTime = formatRelativeTime(dateObj);
  }

  onMount(() => {
    updateTime();
    const interval = setInterval(updateTime, updateInterval);
    return () => clearInterval(interval);
  });

  $: if (date) updateTime();
</script>

<time datetime={date instanceof Date ? date.toISOString() : date} {...$$restProps}>
  {relativeTime}
</time>