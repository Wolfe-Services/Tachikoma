<script lang="ts">
  export let data: number[] = [];
  export let width: number = 60;
  export let height: number = 20;
  export let color: string = 'var(--accent-color)';
  export let strokeWidth: number = 1.5;

  $: maxValue = data.length ? Math.max(...data) : 0;
  $: minValue = data.length ? Math.min(...data) : 0;
  $: range = maxValue - minValue || 1;

  $: pathData = data.length > 1 ? data.map((value, index) => {
    const x = (index / (data.length - 1)) * width;
    const y = height - ((value - minValue) / range) * height;
    return `${index === 0 ? 'M' : 'L'} ${x} ${y}`;
  }).join(' ') : '';
</script>

<svg {width} {height} viewBox="0 0 {width} {height}" class="sparkline">
  {#if pathData}
    <path
      d={pathData}
      fill="none"
      stroke={color}
      stroke-width={strokeWidth}
      stroke-linecap="round"
      stroke-linejoin="round"
      vector-effect="non-scaling-stroke"
    />
  {/if}
</svg>

<style>
  .sparkline {
    display: block;
  }
</style>