<script lang="ts">
  export let name: string;
  export let size: number = 20;
  export let color: string = 'currentColor';
  export let glow: boolean = false;

  // Refined cyberpunk icons - cleaner paths for crisp rendering
  const icons: Record<string, { path: string; viewBox?: string; strokeWidth?: number }> = {
    // Navigation icons
    home: {
      path: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6',
      strokeWidth: 1.5
    },
    play: {
      path: 'M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
      strokeWidth: 1.5
    },
    'file-text': {
      path: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
      strokeWidth: 1.5
    },
    brain: {
      path: 'M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z',
      strokeWidth: 1.5
    },
    settings: {
      path: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z',
      strokeWidth: 1.5
    },
    
    // Status & UI icons
    check: {
      path: 'M5 13l4 4L19 7',
      strokeWidth: 2
    },
    'check-circle': {
      path: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z',
      strokeWidth: 1.5
    },
    x: {
      path: 'M6 18L18 6M6 6l12 12',
      strokeWidth: 2
    },
    'x-circle': {
      path: 'M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z',
      strokeWidth: 1.5
    },
    'alert-triangle': {
      path: 'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z',
      strokeWidth: 1.5
    },
    'alert-circle': {
      path: 'M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
      strokeWidth: 1.5
    },
    info: {
      path: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
      strokeWidth: 1.5
    },
    
    // Navigation arrows
    'chevron-up': {
      path: 'M5 15l7-7 7 7',
      strokeWidth: 2
    },
    'chevron-down': {
      path: 'M19 9l-7 7-7-7',
      strokeWidth: 2
    },
    'chevron-right': {
      path: 'M9 5l7 7-7 7',
      strokeWidth: 2
    },
    'chevron-left': {
      path: 'M15 19l-7-7 7-7',
      strokeWidth: 2
    },
    'arrow-right': {
      path: 'M14 5l7 7m0 0l-7 7m7-7H3',
      strokeWidth: 2
    },
    'arrow-left': {
      path: 'M10 19l-7-7m0 0l7-7m-7 7h18',
      strokeWidth: 2
    },
    
    // Actions
    search: {
      path: 'M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z',
      strokeWidth: 1.5
    },
    'refresh-cw': {
      path: 'M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15',
      strokeWidth: 1.5
    },
    loader: {
      path: 'M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z',
      strokeWidth: 1.5
    },
    
    // User & system
    user: {
      path: 'M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z',
      strokeWidth: 1.5
    },
    users: {
      path: 'M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z',
      strokeWidth: 1.5
    },
    bell: {
      path: 'M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9',
      strokeWidth: 1.5
    },
    'help-circle': {
      path: 'M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
      strokeWidth: 1.5
    },
    
    // Tachikoma-specific
    target: {
      path: 'M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z',
      strokeWidth: 1.5
    },
    zap: {
      path: 'M13 10V3L4 14h7v7l9-11h-7z',
      strokeWidth: 1.5
    },
    activity: {
      path: 'M22 12h-4l-3 9L9 3l-3 9H2',
      strokeWidth: 1.5
    },
    cpu: {
      path: 'M9 3v2m6-2v2M9 19v2m6-2v2M3 9h2m14 0h2M3 14h2m14 0h2M6 6h12v12H6z',
      strokeWidth: 1.5
    },
    terminal: {
      path: 'M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z',
      strokeWidth: 1.5
    },
    code: {
      path: 'M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4',
      strokeWidth: 1.5
    },
    database: {
      path: 'M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4',
      strokeWidth: 1.5
    },
    
    // Trending
    'trending-up': {
      path: 'M13 7h8m0 0v8m0-8l-8 8-4-4-6 6',
      strokeWidth: 1.5
    },
    'trending-down': {
      path: 'M13 17h8m0 0V9m0 8l-8-8-4 4-6-6',
      strokeWidth: 1.5
    },
    minus: {
      path: 'M20 12H4',
      strokeWidth: 2
    },
    plus: {
      path: 'M12 4v16m8-8H4',
      strokeWidth: 2
    },
    
    // Media
    'play-circle': {
      path: 'M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
      strokeWidth: 1.5
    },
    'pause-circle': {
      path: 'M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z',
      strokeWidth: 1.5
    },
    'stop-circle': {
      path: 'M21 12a9 9 0 11-18 0 9 9 0 0118 0z M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z',
      strokeWidth: 1.5
    },
    
    // Files & docs
    folder: {
      path: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z',
      strokeWidth: 1.5
    },
    'folder-open': {
      path: 'M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z',
      strokeWidth: 1.5
    },
    file: {
      path: 'M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z',
      strokeWidth: 1.5
    },
    'file-plus': {
      path: 'M9 13h6m-3-3v6m5 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
      strokeWidth: 1.5
    },
    
    // Misc
    eye: {
      path: 'M15 12a3 3 0 11-6 0 3 3 0 016 0z M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z',
      strokeWidth: 1.5
    },
    'eye-off': {
      path: 'M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21',
      strokeWidth: 1.5
    },
    clock: {
      path: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z',
      strokeWidth: 1.5
    },
    calendar: {
      path: 'M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z',
      strokeWidth: 1.5
    },
    link: {
      path: 'M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1',
      strokeWidth: 1.5
    },
    upload: {
      path: 'M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12',
      strokeWidth: 1.5
    },
    download: {
      path: 'M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4',
      strokeWidth: 1.5
    },
    trash: {
      path: 'M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16',
      strokeWidth: 1.5
    },
    edit: {
      path: 'M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z',
      strokeWidth: 1.5
    },
    copy: {
      path: 'M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z',
      strokeWidth: 1.5
    },
    save: {
      path: 'M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4',
      strokeWidth: 1.5
    },
    menu: {
      path: 'M4 6h16M4 12h16M4 18h16',
      strokeWidth: 2
    },
    'more-horizontal': {
      path: 'M5 12h.01M12 12h.01M19 12h.01M6 12a1 1 0 11-2 0 1 1 0 012 0zm7 0a1 1 0 11-2 0 1 1 0 012 0zm7 0a1 1 0 11-2 0 1 1 0 012 0z',
      strokeWidth: 1.5
    },
    'more-vertical': {
      path: 'M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z',
      strokeWidth: 1.5
    },
    
    // GITS-specific icons
    'neural-link': {
      path: 'M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m3-8.803a4 4 0 11-8 0 4 4 0 018 0z M17 8l4 4m0-4l-4 4',
      strokeWidth: 1.5
    },
    'cyber-eye': {
      path: 'M15 12a3 3 0 11-6 0 3 3 0 016 0z M12 5c-4.478 0-8.268 2.943-9.542 7 1.274 4.057 5.064 7 9.542 7s8.268-2.943 9.542-7c-1.274-4.057-5.064-7-9.542-7z M12 12h.01',
      strokeWidth: 1.5
    },
    robot: {
      path: 'M9 3v2m6-2v2M12 21a9 9 0 110-18 9 9 0 010 18zM9 10h.01M15 10h.01M9 15a3 3 0 006 0',
      strokeWidth: 1.5
    },
    globe: {
      path: 'M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9',
      strokeWidth: 1.5
    },
    shield: {
      path: 'M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z',
      strokeWidth: 1.5
    },
    'shield-check': {
      path: 'M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z',
      strokeWidth: 1.5
    }
  };

  $: iconData = icons[name] || { path: '', strokeWidth: 1.5 };
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke={color}
  stroke-width={iconData.strokeWidth || 1.5}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  class="icon"
  class:glow
>
  {#if iconData.path}
    <path d={iconData.path} />
  {/if}
</svg>

<style>
  .icon {
    flex-shrink: 0;
    transition: all 0.2s ease;
  }
  
  .glow {
    filter: drop-shadow(0 0 4px var(--tachi-cyan-glow, rgba(78, 205, 196, 0.4)));
  }
  
  .icon:hover {
    filter: drop-shadow(0 0 6px var(--tachi-cyan-glow, rgba(78, 205, 196, 0.4)));
  }
</style>
