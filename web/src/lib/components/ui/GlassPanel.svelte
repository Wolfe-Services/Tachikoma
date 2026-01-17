<script lang="ts">
  export let as: keyof HTMLElementTagNameMap = 'section';
  export let padded: boolean = true;
  export let subtle: boolean = false;
  export let accent: 'cyan' | 'red' | 'yellow' | 'green' | 'blue' | 'purple' | 'none' = 'cyan';
  export let className: string = '';

  const accents = {
    cyan: 'var(--tachi-cyan, #4ecdc4)',
    red: 'var(--tachi-red, #ff6b6b)',
    yellow: 'var(--tachi-yellow, #ffd93d)',
    green: 'var(--success-color, #3fb950)',
    blue: 'var(--info-color, #58a6ff)',
    purple: 'var(--color-accent, #8b5cf6)',
    none: 'transparent'
  } as const;

  $: accentColor = accents[accent];
</script>

<svelte:element
  this={as}
  class={`glass-panel ${subtle ? 'subtle' : ''} ${padded ? 'padded' : ''} ${className}`.trim()}
  style={`--glass-accent: ${accentColor};`}
  {...$$restProps}
>
  <slot />
</svelte:element>

<style>
  .glass-panel {
    position: relative;
    overflow: hidden;
    border-radius: 16px;

    /* Polyglass base */
    background:
      linear-gradient(135deg, rgba(255, 255, 255, 0.06), rgba(255, 255, 255, 0.015)),
      rgba(22, 27, 34, 0.6);
    border: 1px solid color-mix(in srgb, var(--glass-accent) 35%, rgba(78, 205, 196, 0.1));
    box-shadow:
      0 0 0 1px rgba(0, 0, 0, 0.35) inset,
      0 10px 40px rgba(0, 0, 0, 0.35),
      0 0 30px color-mix(in srgb, var(--glass-accent) 18%, transparent);

    -webkit-backdrop-filter: blur(14px) saturate(1.25);
    backdrop-filter: blur(14px) saturate(1.25);
  }

  .glass-panel.subtle {
    background:
      linear-gradient(135deg, rgba(255, 255, 255, 0.04), rgba(255, 255, 255, 0.01)),
      rgba(13, 17, 23, 0.45);
    border-color: rgba(78, 205, 196, 0.12);
    box-shadow:
      0 0 0 1px rgba(0, 0, 0, 0.35) inset,
      0 10px 30px rgba(0, 0, 0, 0.25);
  }

  .glass-panel.padded {
    padding: 1.25rem;
  }

  /* Top accent line */
  .glass-panel::before {
    content: '';
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
    height: 2px;
    background: linear-gradient(90deg, transparent, var(--glass-accent), transparent);
    opacity: 0.55;
    pointer-events: none;
  }

  /* Polyglass polygon shimmer */
  .glass-panel::after {
    content: '';
    position: absolute;
    inset: -40% -20%;
    background:
      linear-gradient(
        115deg,
        transparent 0%,
        color-mix(in srgb, var(--glass-accent) 18%, transparent) 35%,
        rgba(255, 255, 255, 0.06) 50%,
        transparent 65%
      );
    clip-path: polygon(0 10%, 60% 0, 100% 45%, 70% 100%, 10% 80%);
    transform: translate3d(-10%, -8%, 0) rotate(-6deg);
    opacity: 0.6;
    pointer-events: none;
    filter: blur(0.5px);
    animation: glassSheen 12s linear infinite;
  }

  @keyframes glassSheen {
    0% {
      transform: translate3d(-14%, -10%, 0) rotate(-6deg);
      opacity: 0.35;
    }
    50% {
      transform: translate3d(4%, 2%, 0) rotate(-6deg);
      opacity: 0.75;
    }
    100% {
      transform: translate3d(18%, 10%, 0) rotate(-6deg);
      opacity: 0.35;
    }
  }
</style>

