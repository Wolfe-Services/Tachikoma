<script lang="ts">
  import Sidebar from './Sidebar.svelte';
  import Header from './Header.svelte';
  import { onMount } from 'svelte';
  
  let sidebarCollapsed = false;
  let mounted = false;
  
  onMount(() => {
    const stored = localStorage.getItem('sidebar-collapsed');
    if (stored !== null) {
      sidebarCollapsed = stored === 'true';
    }
    // Trigger mount animation
    setTimeout(() => mounted = true, 50);
  });
</script>

<div class="app-shell" class:mounted>
  <Sidebar bind:collapsed={sidebarCollapsed} />
  <div class="content-area">
    <Header />
    <main class="main-content">
      <!-- Cyberpunk city background -->
      <div class="city-bg" aria-hidden="true">
        <img src="/z4nfu0cs7yz11.jpg" alt="" class="city-bg-img" />
      </div>
      
      <!-- Subtle grid overlay -->
      <div class="grid-overlay"></div>
      
      <!-- Large transparent Tachikoma watermark -->
      <div class="tachi-watermark">
        <img src="/tachi.png" alt="" aria-hidden="true" />
      </div>
      
      <!-- Corner decorations -->
      <div class="corner-decor top-left"></div>
      <div class="corner-decor top-right"></div>
      <div class="corner-decor bottom-left"></div>
      <div class="corner-decor bottom-right"></div>
      
      <div class="content-inner">
        <slot />
      </div>
    </main>
  </div>
</div>

<style>
  .app-shell {
    display: grid;
    grid-template-columns: auto 1fr;
    min-height: 100vh;
    background: var(--bg, #0a0c10);
    opacity: 0;
    transition: opacity 0.3s ease;
  }
  
  .app-shell.mounted {
    opacity: 1;
  }
  
  .content-area {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    overflow: hidden;
  }
  
  .main-content {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
    background: var(--bg-primary, #0d1117);
    position: relative;
  }
  
  /* Cyberpunk city background */
  .city-bg {
    position: fixed;
    inset: 0;
    z-index: 0;
    pointer-events: none;
    overflow: hidden;
  }

  .city-bg-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    object-position: center;
    opacity: 0.045;
    filter: saturate(0.7);
  }

  /* Subtle grid overlay */
  .grid-overlay {
    position: fixed;
    inset: 0;
    background-image: 
      linear-gradient(rgba(78, 205, 196, 0.015) 1px, transparent 1px),
      linear-gradient(90deg, rgba(78, 205, 196, 0.015) 1px, transparent 1px);
    background-size: 50px 50px;
    pointer-events: none;
    z-index: 0;
  }
  
  /* Corner decorations */
  .corner-decor {
    position: fixed;
    width: 80px;
    height: 80px;
    pointer-events: none;
    z-index: 0;
    opacity: 0.15;
  }
  
  .corner-decor::before,
  .corner-decor::after {
    content: '';
    position: absolute;
    background: var(--tachi-cyan, #4ecdc4);
  }
  
  .corner-decor::before {
    width: 20px;
    height: 2px;
  }
  
  .corner-decor::after {
    width: 2px;
    height: 20px;
  }
  
  .top-left {
    top: 60px;
    left: 280px;
  }
  
  .top-left::before {
    top: 0;
    left: 0;
  }
  
  .top-left::after {
    top: 0;
    left: 0;
  }
  
  .top-right {
    top: 60px;
    right: 16px;
  }
  
  .top-right::before {
    top: 0;
    right: 0;
  }
  
  .top-right::after {
    top: 0;
    right: 0;
  }
  
  .bottom-left {
    bottom: 16px;
    left: 280px;
  }
  
  .bottom-left::before {
    bottom: 0;
    left: 0;
  }
  
  .bottom-left::after {
    bottom: 0;
    left: 0;
  }
  
  .bottom-right {
    bottom: 16px;
    right: 16px;
  }
  
  .bottom-right::before {
    bottom: 0;
    right: 0;
  }
  
  .bottom-right::after {
    bottom: 0;
    right: 0;
  }
  
  /* Large transparent Tachikoma background */
  .tachi-watermark {
    position: fixed;
    bottom: -10%;
    right: -5%;
    width: 50vw;
    height: 50vw;
    max-width: 600px;
    max-height: 600px;
    pointer-events: none;
    z-index: 0;
    opacity: 0.03;
    filter: grayscale(30%) drop-shadow(0 0 50px rgba(78, 205, 196, 0.2));
    animation: watermarkFloat 20s ease-in-out infinite;
  }
  
  @keyframes watermarkFloat {
    0%, 100% { transform: translateY(0) rotate(0deg); }
    50% { transform: translateY(-20px) rotate(2deg); }
  }
  
  .tachi-watermark img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }
  
  .content-inner {
    position: relative;
    z-index: 1;
    animation: contentFadeIn 0.4s ease-out;
  }
  
  @keyframes contentFadeIn {
    from { 
      opacity: 0; 
      transform: translateY(10px); 
    }
    to { 
      opacity: 1; 
      transform: translateY(0); 
    }
  }
  
  /* Scrollbar within main content */
  .main-content::-webkit-scrollbar {
    width: 6px;
  }
  
  .main-content::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .main-content::-webkit-scrollbar-thumb {
    background: rgba(78, 205, 196, 0.3);
    border-radius: 3px;
  }
  
  .main-content::-webkit-scrollbar-thumb:hover {
    background: rgba(78, 205, 196, 0.5);
  }
</style>
