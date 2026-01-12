<script lang="ts">
  export let size: number = 20;
  export let color: string = 'currentColor';
</script>

<svg
  class="spinner"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
  aria-hidden="true"
>
  <circle
    class="spinner-track"
    cx="12"
    cy="12"
    r="10"
    stroke={color}
    stroke-width="3"
    stroke-linecap="round"
  />
  <circle
    class="spinner-head"
    cx="12"
    cy="12"
    r="10"
    stroke={color}
    stroke-width="3"
    stroke-linecap="round"
  />
</svg>

<style>
  .spinner {
    animation: spin 1s linear infinite;
  }

  .spinner-track {
    opacity: 0.2;
  }

  .spinner-head {
    stroke-dasharray: 60;
    stroke-dashoffset: 45;
    transform-origin: center;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>