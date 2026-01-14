<script lang="ts">
  export let data: Array<{
    label: string;
    value: number;
    color: string;
  }> = [];
  export let size: number = 120;
  export let showLabels: boolean = true;

  const strokeWidth = 12;
  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  
  $: total = data.reduce((sum, item) => sum + item.value, 0);
  
  $: segments = data.map((item, index) => {
    const percentage = item.value / total;
    const offset = data.slice(0, index).reduce((sum, prev) => sum + (prev.value / total), 0);
    const strokeDasharray = `${percentage * circumference} ${circumference}`;
    const strokeDashoffset = -offset * circumference;
    
    return {
      ...item,
      percentage,
      strokeDasharray,
      strokeDashoffset
    };
  });
</script>

<div class="donut-chart" style="width: {size}px; height: {size}px;">
  <svg width={size} height={size} viewBox="0 0 {size} {size}">
    <g transform="translate({size / 2}, {size / 2})">
      <!-- Background circle -->
      <circle
        cx="0"
        cy="0"
        r={radius}
        fill="none"
        stroke="var(--bg-secondary)"
        stroke-width={strokeWidth}
      />
      
      <!-- Data segments -->
      {#each segments as segment, index}
        <circle
          cx="0"
          cy="0"
          r={radius}
          fill="none"
          stroke={segment.color}
          stroke-width={strokeWidth}
          stroke-dasharray={segment.strokeDasharray}
          stroke-dashoffset={segment.strokeDashoffset}
          transform="rotate(-90)"
          opacity="0.9"
        >
          <title>{segment.label}: {segment.value} ({(segment.percentage * 100).toFixed(1)}%)</title>
        </circle>
      {/each}
    </g>
  </svg>
  
  {#if showLabels && total > 0}
    <div class="center-label">
      <span class="total-value">{total}</span>
      <span class="total-label">Total</span>
    </div>
  {/if}
</div>

<style>
  .donut-chart {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  svg {
    max-width: 100%;
    height: auto;
  }

  circle {
    transition: all 0.3s ease;
  }

  .center-label {
    position: absolute;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
  }

  .total-value {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1;
  }

  .total-label {
    font-size: 0.625rem;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-top: 0.125rem;
  }
</style>