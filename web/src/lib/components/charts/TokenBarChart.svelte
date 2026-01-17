<script lang="ts">
  import { onMount, afterUpdate } from 'svelte';
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import { fade } from 'svelte/transition';

  export let data: Array<{
    label: string;
    inputTokens: number;
    outputTokens: number;
  }> = [];
  export let height: number = 300;
  export let showLegend: boolean = true;
  export let animated: boolean = true;

  let containerWidth = 0;
  let hoveredIndex: number | null = null;
  let tooltip = { show: false, x: 0, y: 0, data: null as any };

  const padding = { top: 20, right: 20, bottom: 40, left: 60 };

  $: chartWidth = containerWidth - padding.left - padding.right;
  $: chartHeight = height - padding.top - padding.bottom;

  $: maxValue = Math.max(
    ...data.flatMap(d => [d.inputTokens, d.outputTokens])
  );

  $: barWidth = Math.min(40, (chartWidth / data.length - 10) / 2);
  $: groupWidth = barWidth * 2 + 4;

  $: yScale = (value: number) => chartHeight - (value / maxValue) * chartHeight;
  $: xScale = (index: number) => (index * chartWidth) / data.length + chartWidth / data.length / 2;

  function formatNumber(n: number): string {
    if (n >= 1000000) return `${(n / 1000000).toFixed(1)}M`;
    if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
    return n.toString();
  }

  function handleBarHover(event: MouseEvent, index: number, type: 'input' | 'output') {
    const rect = (event.target as SVGElement).getBoundingClientRect();
    tooltip = {
      show: true,
      x: rect.left + rect.width / 2,
      y: rect.top,
      data: {
        label: data[index].label,
        type,
        value: type === 'input' ? data[index].inputTokens : data[index].outputTokens
      }
    };
    hoveredIndex = index;
  }

  function handleBarLeave() {
    tooltip.show = false;
    hoveredIndex = null;
  }

  // Generate Y-axis ticks
  $: yTicks = Array.from({ length: 5 }, (_, i) => Math.round((maxValue / 4) * i));
</script>

<div class="token-bar-chart" bind:clientWidth={containerWidth}>
  {#if showLegend}
    <div class="chart-legend">
      <div class="legend-item">
        <span class="legend-color input" />
        <span>Input Tokens</span>
      </div>
      <div class="legend-item">
        <span class="legend-color output" />
        <span>Output Tokens</span>
      </div>
    </div>
  {/if}

  <svg width={containerWidth} {height}>
    <g transform="translate({padding.left}, {padding.top})">
      <!-- Y-axis grid lines -->
      {#each yTicks as tick}
        <line
          class="grid-line"
          x1="0"
          y1={yScale(tick)}
          x2={chartWidth}
          y2={yScale(tick)}
        />
        <text
          class="axis-label y-axis"
          x="-10"
          y={yScale(tick)}
          text-anchor="end"
          dominant-baseline="middle"
        >
          {formatNumber(tick)}
        </text>
      {/each}

      <!-- Bars -->
      {#each data as item, i}
        <g
          class="bar-group"
          class:hovered={hoveredIndex === i}
          transform="translate({xScale(i) - groupWidth / 2}, 0)"
        >
          <!-- Input bar -->
          <rect
            class="bar input"
            x="0"
            y={animated ? chartHeight : yScale(item.inputTokens)}
            width={barWidth}
            height={animated ? 0 : chartHeight - yScale(item.inputTokens)}
            rx="2"
            on:mouseenter={(e) => handleBarHover(e, i, 'input')}
            on:mouseleave={handleBarLeave}
          >
            {#if animated}
              <animate
                attributeName="y"
                from={chartHeight}
                to={yScale(item.inputTokens)}
                dur="0.5s"
                fill="freeze"
                begin="{i * 0.05}s"
              />
              <animate
                attributeName="height"
                from="0"
                to={chartHeight - yScale(item.inputTokens)}
                dur="0.5s"
                fill="freeze"
                begin="{i * 0.05}s"
              />
            {/if}
          </rect>

          <!-- Output bar -->
          <rect
            class="bar output"
            x={barWidth + 4}
            y={animated ? chartHeight : yScale(item.outputTokens)}
            width={barWidth}
            height={animated ? 0 : chartHeight - yScale(item.outputTokens)}
            rx="2"
            on:mouseenter={(e) => handleBarHover(e, i, 'output')}
            on:mouseleave={handleBarLeave}
          >
            {#if animated}
              <animate
                attributeName="y"
                from={chartHeight}
                to={yScale(item.outputTokens)}
                dur="0.5s"
                fill="freeze"
                begin="{i * 0.05 + 0.1}s"
              />
              <animate
                attributeName="height"
                from="0"
                to={chartHeight - yScale(item.outputTokens)}
                dur="0.5s"
                fill="freeze"
                begin="{i * 0.05 + 0.1}s"
              />
            {/if}
          </rect>

          <!-- X-axis label -->
          <text
            class="axis-label x-axis"
            x={groupWidth / 2}
            y={chartHeight + 20}
            text-anchor="middle"
          >
            {item.label}
          </text>
        </g>
      {/each}
    </g>
  </svg>

  {#if tooltip.show}
    <div
      class="tooltip"
      style="left: {tooltip.x}px; top: {tooltip.y}px;"
      transition:fade={{ duration: 100 }}
    >
      <div class="tooltip-title">{tooltip.data.label}</div>
      <div class="tooltip-value">
        <span class="tooltip-type {tooltip.data.type}">{tooltip.data.type}</span>
        {formatNumber(tooltip.data.value)} tokens
      </div>
    </div>
  {/if}
</div>

<style>
  .token-bar-chart {
    position: relative;
    width: 100%;
  }

  .chart-legend {
    display: flex;
    gap: 1.5rem;
    justify-content: center;
    margin-bottom: 0.5rem;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .legend-color {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: 0.125rem;
  }

  .legend-color.input {
    background: var(--blue-500);
  }

  .legend-color.output {
    background: var(--purple-500);
  }

  .grid-line {
    stroke: var(--color-border);
    stroke-dasharray: 2, 2;
  }

  .axis-label {
    font-size: 0.6875rem;
    fill: var(--color-text-muted);
  }

  .bar {
    cursor: pointer;
    transition: opacity 0.15s ease;
  }

  .bar.input {
    fill: var(--blue-500);
  }

  .bar.output {
    fill: var(--purple-500);
  }

  .bar-group:hover .bar {
    opacity: 0.8;
  }

  .bar-group.hovered .bar {
    opacity: 1;
  }

  .tooltip {
    position: fixed;
    transform: translate(-50%, -100%);
    margin-top: -8px;
    padding: 0.5rem 0.75rem;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    box-shadow: var(--shadow-lg);
    pointer-events: none;
    z-index: 1000;
  }

  .tooltip-title {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .tooltip-value {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin-top: 0.25rem;
  }

  .tooltip-type {
    font-weight: 500;
    text-transform: capitalize;
  }

  .tooltip-type.input {
    color: var(--blue-500);
  }

  .tooltip-type.output {
    color: var(--purple-500);
  }
</style>