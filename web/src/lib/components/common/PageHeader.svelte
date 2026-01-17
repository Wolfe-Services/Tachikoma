<script lang="ts">
  import Icon from './Icon.svelte';
  
  export let title: string;
  export let subtitle: string = '';
  export let tag: string = '';
  export let icon: string = '';
  export let iconSrc: string = '';
  export let iconSize: number = 48;
</script>

<header class="page-header">
  <div class="header-background">
    <div class="circuit-lines"></div>
  </div>
  
  <div class="header-content">
    <div class="header-left">
      {#if iconSrc}
        <div class="header-icon header-icon-img" style="--icon-size: {iconSize}px">
          <img src={iconSrc} alt="" class="tachi-icon" />
        </div>
      {:else if icon}
        <div class="header-icon">
          <Icon name={icon} size={32} glow />
        </div>
      {/if}
      
      <div class="header-text">
        {#if tag}
          <div class="header-tag">{tag}</div>
        {/if}
        <h1 class="header-title">{title}</h1>
        {#if subtitle}
          <p class="header-subtitle">{subtitle}</p>
        {/if}
      </div>
    </div>
    
    <div class="header-actions">
      <slot name="actions" />
    </div>
  </div>
  
  <div class="header-border"></div>
</header>

<style>
  .page-header {
    position: relative;
    padding: 1.5rem 0;
    margin-bottom: 1.5rem;
    overflow: hidden;
  }
  
  .header-background {
    position: absolute;
    inset: 0;
    opacity: 0.3;
    pointer-events: none;
  }
  
  .circuit-lines {
    position: absolute;
    inset: 0;
    background-image: 
      linear-gradient(90deg, transparent 50%, rgba(78, 205, 196, 0.03) 50%),
      linear-gradient(rgba(78, 205, 196, 0.02) 1px, transparent 1px);
    background-size: 4px 4px, 100% 8px;
  }
  
  .header-content {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 2rem;
    position: relative;
    z-index: 1;
  }
  
  .header-left {
    display: flex;
    align-items: flex-start;
    gap: 1.25rem;
  }
  
  .header-icon {
    padding: 0.75rem;
    background: linear-gradient(135deg, rgba(78, 205, 196, 0.15), rgba(78, 205, 196, 0.05));
    border: 1px solid rgba(78, 205, 196, 0.3);
    border-radius: 12px;
    color: var(--tachi-cyan, #4ecdc4);
    flex-shrink: 0;
  }
  
  .header-icon-img {
    width: var(--icon-size, 48px);
    height: var(--icon-size, 48px);
    padding: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .tachi-icon {
    width: 100%;
    height: 100%;
    object-fit: contain;
    filter: drop-shadow(0 0 8px var(--tachi-cyan, #4ecdc4));
    transition: filter 0.3s ease;
  }
  
  .header-icon-img:hover .tachi-icon {
    filter: drop-shadow(0 0 12px var(--tachi-cyan, #4ecdc4)) drop-shadow(0 0 4px var(--tachi-cyan, #4ecdc4));
  }
  
  .header-text {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .header-tag {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.65rem;
    font-weight: 500;
    color: var(--tachi-cyan, #4ecdc4);
    letter-spacing: 2.5px;
    opacity: 0.9;
    text-transform: uppercase;
  }
  
  .header-title {
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 1.75rem;
    font-weight: 700;
    color: var(--text-primary, #e6edf3);
    letter-spacing: 1.5px;
    margin: 0;
    text-transform: uppercase;
    text-shadow: 0 0 20px var(--tachi-cyan-glow, rgba(78, 205, 196, 0.3));
  }
  
  .header-subtitle {
    font-size: 0.95rem;
    color: var(--text-secondary, rgba(230, 237, 243, 0.7));
    margin: 0.25rem 0 0;
  }
  
  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }
  
  .header-border {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(90deg, 
      transparent, 
      var(--tachi-cyan, #4ecdc4) 20%, 
      var(--tachi-cyan, #4ecdc4) 80%, 
      transparent
    );
    opacity: 0.4;
  }
  
  .header-border::after {
    content: '';
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    bottom: -4px;
    width: 8px;
    height: 8px;
    background: var(--tachi-cyan, #4ecdc4);
    clip-path: polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%);
    box-shadow: 0 0 10px var(--tachi-cyan, #4ecdc4);
  }
  
  :global(.page-header .btn-primary) {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.25rem;
    background: linear-gradient(135deg, var(--tachi-cyan-dark, #2d7a7a), var(--tachi-cyan, #4ecdc4));
    border: 1px solid var(--tachi-cyan, #4ecdc4);
    border-radius: 8px;
    color: var(--bg-primary, #0d1117);
    font-family: var(--font-display, 'Orbitron', sans-serif);
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.5px;
    text-decoration: none;
    text-transform: uppercase;
    cursor: pointer;
    transition: all 0.3s ease;
    box-shadow: 0 0 15px rgba(78, 205, 196, 0.3);
  }
  
  :global(.page-header .btn-primary:hover) {
    background: linear-gradient(135deg, var(--tachi-cyan, #4ecdc4), var(--tachi-cyan-bright, #6ee7df));
    box-shadow: 0 0 25px rgba(78, 205, 196, 0.5);
    transform: translateY(-2px);
  }
  
  :global(.page-header .btn-secondary) {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.25rem;
    background: transparent;
    border: 1px solid var(--border-color, rgba(78, 205, 196, 0.3));
    border-radius: 8px;
    color: var(--text-primary, #e6edf3);
    font-family: var(--font-body, 'Rajdhani', sans-serif);
    font-size: 0.9rem;
    font-weight: 600;
    text-decoration: none;
    cursor: pointer;
    transition: all 0.3s ease;
  }
  
  :global(.page-header .btn-secondary:hover) {
    background: rgba(78, 205, 196, 0.1);
    border-color: var(--tachi-cyan, #4ecdc4);
    color: var(--tachi-cyan, #4ecdc4);
  }
</style>
