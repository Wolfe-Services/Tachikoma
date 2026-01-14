<script lang="ts">
  export let size: number = 40;
  export let animated: boolean = true;
  export let showGlow: boolean = true;
  export let variant: 'default' | 'minimal' | 'hero' = 'default';
</script>

<div 
  class="tachikoma-logo" 
  class:animated
  class:show-glow={showGlow}
  class:minimal={variant === 'minimal'}
  class:hero={variant === 'hero'}
  style="--size: {size}px;"
>
  {#if variant === 'hero'}
    <div class="hero-ring ring-outer">
      <div class="ring-segment"></div>
      <div class="ring-segment"></div>
      <div class="ring-segment"></div>
      <div class="ring-segment"></div>
    </div>
    <div class="hero-ring ring-inner"></div>
  {/if}
  
  <div class="logo-container">
    <img 
      src="/tachi.png" 
      alt="Tachikoma"
      width={size}
      height={size}
      class="logo-image"
    />
    
    {#if animated && variant !== 'minimal'}
      <div class="eye-overlay left"></div>
      <div class="eye-overlay right"></div>
    {/if}
  </div>
  
  {#if showGlow}
    <div class="glow-effect"></div>
  {/if}
</div>

<style>
  .tachikoma-logo {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: var(--size);
    height: var(--size);
    transition: all 0.3s ease;
  }
  
  .logo-container {
    position: relative;
    width: 100%;
    height: 100%;
    z-index: 2;
  }
  
  .logo-image {
    width: 100%;
    height: 100%;
    object-fit: contain;
    filter: drop-shadow(0 0 8px rgba(78, 205, 196, 0.5));
    transition: all 0.3s ease;
  }
  
  .tachikoma-logo:hover .logo-image {
    filter: drop-shadow(0 0 15px rgba(78, 205, 196, 0.8));
    transform: scale(1.05);
  }
  
  /* Glow Effect */
  .glow-effect {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 80%;
    height: 80%;
    background: radial-gradient(circle, rgba(78, 205, 196, 0.3) 0%, transparent 70%);
    border-radius: 50%;
    z-index: 1;
    opacity: 0.8;
    transition: opacity 0.3s ease;
  }
  
  .tachikoma-logo:hover .glow-effect {
    opacity: 1;
    animation: glowPulse 1.5s ease-in-out infinite;
  }
  
  @keyframes glowPulse {
    0%, 100% { 
      opacity: 0.8; 
      transform: translate(-50%, -50%) scale(1); 
    }
    50% { 
      opacity: 1; 
      transform: translate(-50%, -50%) scale(1.1); 
    }
  }
  
  /* Eye Overlays - subtle glow effect on the eyes */
  .eye-overlay {
    position: absolute;
    width: 8%;
    height: 8%;
    background: var(--tachi-cyan, #4ecdc4);
    border-radius: 50%;
    opacity: 0;
    transition: opacity 0.3s ease;
    z-index: 3;
    box-shadow: 
      0 0 5px var(--tachi-cyan, #4ecdc4),
      0 0 10px var(--tachi-cyan, #4ecdc4);
  }
  
  .eye-overlay.left {
    top: 38%;
    left: 38%;
  }
  
  .eye-overlay.right {
    top: 38%;
    right: 38%;
  }
  
  .tachikoma-logo:hover .eye-overlay {
    opacity: 0.6;
    animation: eyeGlow 2s ease-in-out infinite;
  }
  
  @keyframes eyeGlow {
    0%, 100% { opacity: 0.6; }
    50% { opacity: 0.9; }
  }
  
  /* Animated Float */
  .animated .logo-image {
    animation: float 3s ease-in-out infinite;
  }
  
  @keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-4px); }
  }
  
  .animated:hover .logo-image {
    animation: none;
    transform: scale(1.05);
  }
  
  /* Minimal Variant */
  .minimal .glow-effect {
    display: none;
  }
  
  .minimal .logo-image {
    filter: drop-shadow(0 0 4px rgba(78, 205, 196, 0.3));
  }
  
  /* Hero Variant */
  .hero {
    width: calc(var(--size) * 1.5);
    height: calc(var(--size) * 1.5);
  }
  
  .hero .logo-container {
    width: var(--size);
    height: var(--size);
  }
  
  .hero-ring {
    position: absolute;
    border-radius: 50%;
    pointer-events: none;
    z-index: 0;
  }
  
  .ring-outer {
    width: 130%;
    height: 130%;
    border: 2px dashed rgba(78, 205, 196, 0.3);
    animation: rotateRing 30s linear infinite;
  }
  
  .ring-inner {
    width: 115%;
    height: 115%;
    border: 1px solid rgba(78, 205, 196, 0.2);
    animation: rotateRing 20s linear infinite reverse;
  }
  
  @keyframes rotateRing {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  
  .ring-segment {
    position: absolute;
    width: 8px;
    height: 8px;
    background: var(--tachi-cyan, #4ecdc4);
    border-radius: 50%;
    box-shadow: 0 0 8px var(--tachi-cyan, #4ecdc4);
  }
  
  .ring-segment:nth-child(1) {
    top: 0;
    left: 50%;
    transform: translateX(-50%);
  }
  
  .ring-segment:nth-child(2) {
    top: 50%;
    right: 0;
    transform: translateY(-50%);
  }
  
  .ring-segment:nth-child(3) {
    bottom: 0;
    left: 50%;
    transform: translateX(-50%);
  }
  
  .ring-segment:nth-child(4) {
    top: 50%;
    left: 0;
    transform: translateY(-50%);
  }
  
  .hero:hover .ring-outer {
    animation-duration: 10s;
  }
  
  .hero:hover .ring-inner {
    animation-duration: 7s;
  }
</style>
