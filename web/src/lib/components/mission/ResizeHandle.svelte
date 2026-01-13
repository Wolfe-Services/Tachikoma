<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let direction: 'horizontal' | 'vertical' = 'horizontal';
  export let position: 'left' | 'right' | 'top' | 'bottom' = 'right';
  export let cursor: string = direction === 'horizontal' ? 'col-resize' : 'row-resize';

  const dispatch = createEventDispatcher<{
    resize: { delta: number };
  }>();

  let isDragging = false;
  let startX = 0;
  let startY = 0;
  let startWidth = 0;
  let startHeight = 0;

  function handleMouseDown(event: MouseEvent) {
    isDragging = true;
    startX = event.clientX;
    startY = event.clientY;

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    event.preventDefault();
  }

  function handleMouseMove(event: MouseEvent) {
    if (!isDragging) return;

    const deltaX = event.clientX - startX;
    const deltaY = event.clientY - startY;

    if (direction === 'horizontal') {
      let delta = deltaX;
      if (position === 'left') delta = -delta;
      dispatch('resize', { delta });
    } else {
      let delta = deltaY;
      if (position === 'top') delta = -delta;
      dispatch('resize', { delta });
    }
  }

  function handleMouseUp() {
    isDragging = false;
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  }
</script>

<div
  class="resize-handle"
  class:horizontal={direction === 'horizontal'}
  class:vertical={direction === 'vertical'}
  class:left={position === 'left'}
  class:right={position === 'right'}
  class:top={position === 'top'}
  class:bottom={position === 'bottom'}
  class:dragging={isDragging}
  style="cursor: {cursor}"
  role="separator"
  aria-orientation={direction === 'horizontal' ? 'vertical' : 'horizontal'}
  on:mousedown={handleMouseDown}
  on:keydown={(e) => {
    if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
      e.preventDefault();
      const delta = e.key === 'ArrowLeft' ? -10 : 10;
      dispatch('resize', { delta: position === 'left' ? -delta : delta });
    }
  }}
  tabindex="0"
/>

<style>
  .resize-handle {
    position: absolute;
    background: transparent;
    z-index: 10;
    transition: background-color 0.15s ease;
  }

  .resize-handle:hover,
  .resize-handle:focus-visible {
    background-color: var(--color-accent-muted);
  }

  .resize-handle:focus-visible {
    outline: 2px solid var(--color-accent-fg);
    outline-offset: -1px;
  }

  .resize-handle.dragging {
    background-color: var(--color-accent-fg);
  }

  .horizontal {
    top: 0;
    bottom: 0;
    width: 4px;
  }

  .horizontal.left {
    left: -2px;
  }

  .horizontal.right {
    right: -2px;
  }

  .vertical {
    left: 0;
    right: 0;
    height: 4px;
  }

  .vertical.top {
    top: -2px;
  }

  .vertical.bottom {
    bottom: -2px;
  }

  /* Visual indicator on hover */
  .resize-handle::before {
    content: '';
    position: absolute;
    background: var(--color-accent-fg);
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .resize-handle:hover::before,
  .resize-handle:focus-visible::before,
  .resize-handle.dragging::before {
    opacity: 1;
  }

  .horizontal::before {
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 2px;
    height: 20px;
    border-radius: 1px;
  }

  .vertical::before {
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 20px;
    height: 2px;
    border-radius: 1px;
  }
</style>