# 301 - Token Charts

**Phase:** 14 - Dashboard
**Spec ID:** 301
**Status:** Planned
**Dependencies:** 296-dashboard-layout
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create chart components for visualizing token usage across missions, including bar charts, pie charts, and stacked area charts for token consumption analysis.

---

## Acceptance Criteria

- [x] `TokenBarChart.svelte` component created
- [x] `TokenPieChart.svelte` for distribution
- [x] `TokenAreaChart.svelte` for time series
- [x] Input vs output token comparison
- [x] Model-specific breakdowns
- [x] Interactive tooltips
- [x] Responsive chart sizing
- [x] Animation on data updates

---

## Implementation Details

### 1. Token Bar Chart (web/src/lib/components/charts/TokenBarChart.svelte)

```svelte
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
    color: var(--text-secondary);
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
    stroke: var(--border-color);
    stroke-dasharray: 2, 2;
  }

  .axis-label {
    font-size: 0.6875rem;
    fill: var(--text-tertiary);
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
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.375rem;
    box-shadow: var(--shadow-lg);
    pointer-events: none;
    z-index: 1000;
  }

  .tooltip-title {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .tooltip-value {
    font-size: 0.8125rem;
    color: var(--text-secondary);
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
```

### 2. Token Pie Chart (web/src/lib/components/charts/TokenPieChart.svelte)

```svelte
<script lang="ts">
  import { fade } from 'svelte/transition';

  export let data: Array<{
    label: string;
    value: number;
    color: string;
  }> = [];
  export let size: number = 200;
  export let donut: boolean = true;
  export let showLabels: boolean = true;

  let hoveredIndex: number | null = null;

  $: total = data.reduce((sum, d) => sum + d.value, 0);
  $: innerRadius = donut ? size / 4 : 0;
  $: outerRadius = size / 2 - 10;

  $: slices = calculateSlices(data, total);

  function calculateSlices(items: typeof data, sum: number) {
    let currentAngle = -Math.PI / 2;
    return items.map(item => {
      const angle = (item.value / sum) * 2 * Math.PI;
      const slice = {
        ...item,
        startAngle: currentAngle,
        endAngle: currentAngle + angle,
        percent: (item.value / sum) * 100
      };
      currentAngle += angle;
      return slice;
    });
  }

  function polarToCartesian(angle: number, radius: number) {
    return {
      x: size / 2 + radius * Math.cos(angle),
      y: size / 2 + radius * Math.sin(angle)
    };
  }

  function describeArc(startAngle: number, endAngle: number, inner: number, outer: number) {
    const start = polarToCartesian(startAngle, outer);
    const end = polarToCartesian(endAngle, outer);
    const innerStart = polarToCartesian(endAngle, inner);
    const innerEnd = polarToCartesian(startAngle, inner);
    const largeArc = endAngle - startAngle > Math.PI ? 1 : 0;

    if (inner === 0) {
      return [
        `M ${size / 2} ${size / 2}`,
        `L ${start.x} ${start.y}`,
        `A ${outer} ${outer} 0 ${largeArc} 1 ${end.x} ${end.y}`,
        'Z'
      ].join(' ');
    }

    return [
      `M ${start.x} ${start.y}`,
      `A ${outer} ${outer} 0 ${largeArc} 1 ${end.x} ${end.y}`,
      `L ${innerStart.x} ${innerStart.y}`,
      `A ${inner} ${inner} 0 ${largeArc} 0 ${innerEnd.x} ${innerEnd.y}`,
      'Z'
    ].join(' ');
  }

  function formatNumber(n: number): string {
    if (n >= 1000000) return `${(n / 1000000).toFixed(1)}M`;
    if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
    return n.toString();
  }
</script>

<div class="token-pie-chart">
  <svg width={size} height={size}>
    {#each slices as slice, i}
      <path
        class="slice"
        class:hovered={hoveredIndex === i}
        d={describeArc(slice.startAngle, slice.endAngle, innerRadius, outerRadius)}
        fill={slice.color}
        on:mouseenter={() => hoveredIndex = i}
        on:mouseleave={() => hoveredIndex = null}
      />
    {/each}

    {#if donut}
      <text
        class="center-label"
        x={size / 2}
        y={size / 2 - 5}
        text-anchor="middle"
        dominant-baseline="middle"
      >
        {formatNumber(total)}
      </text>
      <text
        class="center-sublabel"
        x={size / 2}
        y={size / 2 + 12}
        text-anchor="middle"
        dominant-baseline="middle"
      >
        tokens
      </text>
    {/if}
  </svg>

  {#if showLabels}
    <ul class="pie-legend">
      {#each slices as slice, i}
        <li
          class="legend-item"
          class:hovered={hoveredIndex === i}
          on:mouseenter={() => hoveredIndex = i}
          on:mouseleave={() => hoveredIndex = null}
        >
          <span class="legend-color" style="background: {slice.color}" />
          <span class="legend-label">{slice.label}</span>
          <span class="legend-value">{formatNumber(slice.value)}</span>
          <span class="legend-percent">{slice.percent.toFixed(1)}%</span>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .token-pie-chart {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
  }

  .slice {
    cursor: pointer;
    transition: transform 0.15s ease, opacity 0.15s ease;
    transform-origin: center;
  }

  .slice:hover,
  .slice.hovered {
    transform: scale(1.03);
    opacity: 0.9;
  }

  .center-label {
    font-size: 1.25rem;
    font-weight: 700;
    fill: var(--text-primary);
  }

  .center-sublabel {
    font-size: 0.75rem;
    fill: var(--text-tertiary);
  }

  .pie-legend {
    list-style: none;
    padding: 0;
    margin: 0;
    width: 100%;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
    border-radius: 0.375rem;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .legend-item:hover,
  .legend-item.hovered {
    background: var(--bg-hover);
  }

  .legend-color {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: 0.125rem;
    flex-shrink: 0;
  }

  .legend-label {
    flex: 1;
    font-size: 0.8125rem;
    color: var(--text-primary);
  }

  .legend-value {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .legend-percent {
    width: 3rem;
    text-align: right;
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }
</style>
```

### 3. Token Area Chart (web/src/lib/components/charts/TokenAreaChart.svelte)

```svelte
<script lang="ts">
  import { fade } from 'svelte/transition';

  export let data: Array<{
    date: string;
    inputTokens: number;
    outputTokens: number;
  }> = [];
  export let height: number = 250;
  export let stacked: boolean = true;

  let containerWidth = 0;
  let hoveredIndex: number | null = null;

  const padding = { top: 20, right: 20, bottom: 30, left: 50 };

  $: chartWidth = containerWidth - padding.left - padding.right;
  $: chartHeight = height - padding.top - padding.bottom;

  $: maxValue = stacked
    ? Math.max(...data.map(d => d.inputTokens + d.outputTokens))
    : Math.max(...data.flatMap(d => [d.inputTokens, d.outputTokens]));

  $: xScale = (index: number) => (index / (data.length - 1)) * chartWidth;
  $: yScale = (value: number) => chartHeight - (value / maxValue) * chartHeight;

  $: inputPath = generatePath(data.map(d => d.inputTokens));
  $: outputPath = stacked
    ? generateStackedPath(data.map(d => d.inputTokens), data.map(d => d.outputTokens))
    : generatePath(data.map(d => d.outputTokens));

  $: inputAreaPath = generateAreaPath(data.map(d => d.inputTokens));
  $: outputAreaPath = stacked
    ? generateStackedAreaPath(data.map(d => d.inputTokens), data.map(d => d.outputTokens))
    : generateAreaPath(data.map(d => d.outputTokens));

  function generatePath(values: number[]): string {
    return values
      .map((v, i) => `${i === 0 ? 'M' : 'L'} ${xScale(i)} ${yScale(v)}`)
      .join(' ');
  }

  function generateAreaPath(values: number[]): string {
    const line = generatePath(values);
    return `${line} L ${xScale(values.length - 1)} ${chartHeight} L 0 ${chartHeight} Z`;
  }

  function generateStackedPath(base: number[], top: number[]): string {
    return top
      .map((v, i) => `${i === 0 ? 'M' : 'L'} ${xScale(i)} ${yScale(base[i] + v)}`)
      .join(' ');
  }

  function generateStackedAreaPath(base: number[], top: number[]): string {
    const topLine = top.map((v, i) => `${xScale(i)} ${yScale(base[i] + v)}`).join(' L ');
    const bottomLine = [...base].reverse().map((v, i) => `${xScale(base.length - 1 - i)} ${yScale(v)}`).join(' L ');
    return `M ${topLine} L ${bottomLine} Z`;
  }

  function formatNumber(n: number): string {
    if (n >= 1000000) return `${(n / 1000000).toFixed(1)}M`;
    if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
    return n.toString();
  }
</script>

<div class="token-area-chart" bind:clientWidth={containerWidth}>
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

  <svg width={containerWidth} {height}>
    <defs>
      <linearGradient id="inputGradient" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="var(--blue-500)" stop-opacity="0.3" />
        <stop offset="100%" stop-color="var(--blue-500)" stop-opacity="0.05" />
      </linearGradient>
      <linearGradient id="outputGradient" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" stop-color="var(--purple-500)" stop-opacity="0.3" />
        <stop offset="100%" stop-color="var(--purple-500)" stop-opacity="0.05" />
      </linearGradient>
    </defs>

    <g transform="translate({padding.left}, {padding.top})">
      <!-- Grid lines -->
      {#each Array.from({ length: 5 }, (_, i) => i) as i}
        <line
          class="grid-line"
          x1="0"
          y1={(chartHeight / 4) * i}
          x2={chartWidth}
          y2={(chartHeight / 4) * i}
        />
        <text
          class="axis-label"
          x="-10"
          y={(chartHeight / 4) * i}
          text-anchor="end"
          dominant-baseline="middle"
        >
          {formatNumber(maxValue - (maxValue / 4) * i)}
        </text>
      {/each}

      <!-- Output area (rendered first if stacked) -->
      <path
        class="area output"
        d={outputAreaPath}
        fill="url(#outputGradient)"
      />
      <path
        class="line output"
        d={outputPath}
        fill="none"
        stroke="var(--purple-500)"
        stroke-width="2"
      />

      <!-- Input area -->
      <path
        class="area input"
        d={inputAreaPath}
        fill="url(#inputGradient)"
      />
      <path
        class="line input"
        d={inputPath}
        fill="none"
        stroke="var(--blue-500)"
        stroke-width="2"
      />

      <!-- Data points and hover areas -->
      {#each data as item, i}
        <g class="data-point" transform="translate({xScale(i)}, 0)">
          <rect
            class="hover-area"
            x={-chartWidth / data.length / 2}
            y="0"
            width={chartWidth / data.length}
            height={chartHeight}
            on:mouseenter={() => hoveredIndex = i}
            on:mouseleave={() => hoveredIndex = null}
          />

          {#if hoveredIndex === i}
            <line
              class="hover-line"
              x1="0"
              y1="0"
              x2="0"
              y2={chartHeight}
              transition:fade={{ duration: 100 }}
            />
            <circle
              class="point input"
              cx="0"
              cy={yScale(item.inputTokens)}
              r="4"
              transition:fade={{ duration: 100 }}
            />
            <circle
              class="point output"
              cx="0"
              cy={stacked ? yScale(item.inputTokens + item.outputTokens) : yScale(item.outputTokens)}
              r="4"
              transition:fade={{ duration: 100 }}
            />
          {/if}
        </g>

        <!-- X-axis labels -->
        {#if i % Math.ceil(data.length / 6) === 0}
          <text
            class="axis-label x-axis"
            x={xScale(i)}
            y={chartHeight + 20}
            text-anchor="middle"
          >
            {item.date}
          </text>
        {/if}
      {/each}
    </g>
  </svg>

  {#if hoveredIndex !== null}
    <div class="hover-tooltip" style="left: {xScale(hoveredIndex) + padding.left}px;">
      <div class="tooltip-date">{data[hoveredIndex].date}</div>
      <div class="tooltip-row input">
        <span>Input:</span>
        <span>{formatNumber(data[hoveredIndex].inputTokens)}</span>
      </div>
      <div class="tooltip-row output">
        <span>Output:</span>
        <span>{formatNumber(data[hoveredIndex].outputTokens)}</span>
      </div>
      <div class="tooltip-row total">
        <span>Total:</span>
        <span>{formatNumber(data[hoveredIndex].inputTokens + data[hoveredIndex].outputTokens)}</span>
      </div>
    </div>
  {/if}
</div>

<style>
  .token-area-chart {
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
    color: var(--text-secondary);
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
    stroke: var(--border-color);
    stroke-dasharray: 2, 2;
  }

  .axis-label {
    font-size: 0.6875rem;
    fill: var(--text-tertiary);
  }

  .hover-area {
    fill: transparent;
    cursor: crosshair;
  }

  .hover-line {
    stroke: var(--border-color);
    stroke-dasharray: 4, 4;
  }

  .point.input {
    fill: var(--blue-500);
  }

  .point.output {
    fill: var(--purple-500);
  }

  .hover-tooltip {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    padding: 0.75rem;
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.5rem;
    box-shadow: var(--shadow-lg);
    pointer-events: none;
    z-index: 10;
  }

  .tooltip-date {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 0.5rem;
  }

  .tooltip-row {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    font-size: 0.75rem;
    padding: 0.125rem 0;
  }

  .tooltip-row.input span:first-child {
    color: var(--blue-500);
  }

  .tooltip-row.output span:first-child {
    color: var(--purple-500);
  }

  .tooltip-row.total {
    border-top: 1px solid var(--border-color);
    margin-top: 0.25rem;
    padding-top: 0.375rem;
    font-weight: 500;
  }
</style>
```

---

## Testing Requirements

1. Bar chart renders correct heights
2. Pie chart slices sum to 100%
3. Area chart paths are correct
4. Hover states show tooltips
5. Animations play on mount
6. Charts resize responsively
7. Stacked mode calculates correctly

---

## Related Specs

- Depends on: [296-dashboard-layout.md](296-dashboard-layout.md)
- Next: [302-success-rate.md](302-success-rate.md)
- Used by: Token analytics views
