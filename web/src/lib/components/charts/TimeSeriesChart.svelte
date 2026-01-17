<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import type { TimeSeriesData, TimeSeriesPoint } from '$lib/types/charts';

  export let data: TimeSeriesData[] = [];
  export let height: number = 300;
  export let showLegend: boolean = true;
  export let showBrush: boolean = true;
  export let enableZoom: boolean = true;
  export let timeFormat: 'hour' | 'day' | 'week' | 'month' = 'day';

  const dispatch = createEventDispatcher<{
    rangeChange: { start: Date; end: Date };
  }>();

  let containerWidth = 0;
  let hoveredIndex: number | null = null;
  let hoveredSeries: string | null = null;
  let isDragging = false;
  let brushStart: number | null = null;
  let brushEnd: number | null = null;
  let zoomLevel = 1;
  let panOffset = 0;

  const padding = { top: 20, right: 20, bottom: 60, left: 60 };
  const brushHeight = 40;

  $: mainHeight = showBrush ? height - brushHeight - 20 : height;
  $: chartWidth = containerWidth - padding.left - padding.right;
  $: chartHeight = mainHeight - padding.top - padding.bottom;

  $: allPoints = data.flatMap(series =>
    series.points.map(p => ({ ...p, seriesId: series.id, seriesLabel: series.label, color: series.color }))
  );

  $: timeExtent = getTimeExtent(allPoints);
  $: valueExtent = getValueExtent(allPoints);

  $: visibleTimeExtent = applyZoomPan(timeExtent, zoomLevel, panOffset);

  $: xScale = (time: number) => {
    const [start, end] = visibleTimeExtent;
    return ((time - start) / (end - start)) * chartWidth;
  };

  $: yScale = (value: number) => {
    const [min, max] = valueExtent;
    return chartHeight - ((value - min) / (max - min)) * chartHeight;
  };

  function getTimeExtent(points: TimeSeriesPoint[]): [number, number] {
    if (points.length === 0) return [Date.now() - 86400000, Date.now()];
    const times = points.map(p => new Date(p.timestamp).getTime());
    return [Math.min(...times), Math.max(...times)];
  }

  function getValueExtent(points: TimeSeriesPoint[]): [number, number] {
    if (points.length === 0) return [0, 100];
    const values = points.map(p => p.value);
    const min = Math.min(...values);
    const max = Math.max(...values);
    const padding = (max - min) * 0.1;
    return [Math.max(0, min - padding), max + padding];
  }

  function applyZoomPan(extent: [number, number], zoom: number, pan: number): [number, number] {
    const [start, end] = extent;
    const range = end - start;
    const zoomedRange = range / zoom;
    const center = start + range / 2 + pan;
    return [center - zoomedRange / 2, center + zoomedRange / 2];
  }

  function generatePath(points: TimeSeriesPoint[]): string {
    if (points.length === 0) return '';
    return points
      .filter(p => {
        const time = new Date(p.timestamp).getTime();
        return time >= visibleTimeExtent[0] && time <= visibleTimeExtent[1];
      })
      .map((p, i) => {
        const x = xScale(new Date(p.timestamp).getTime());
        const y = yScale(p.value);
        return `${i === 0 ? 'M' : 'L'} ${x} ${y}`;
      })
      .join(' ');
  }

  function generateAreaPath(points: TimeSeriesPoint[]): string {
    if (points.length === 0) return '';
    const visiblePoints = points.filter(p => {
      const time = new Date(p.timestamp).getTime();
      return time >= visibleTimeExtent[0] && time <= visibleTimeExtent[1];
    });
    
    if (visiblePoints.length === 0) return '';
    
    const linePath = generatePath(points);
    const lastPoint = visiblePoints[visiblePoints.length - 1];
    const firstPoint = visiblePoints[0];
    return `${linePath} L ${xScale(new Date(lastPoint.timestamp).getTime())} ${chartHeight} L ${xScale(new Date(firstPoint.timestamp).getTime())} ${chartHeight} Z`;
  }

  function formatTimestamp(time: number): string {
    const date = new Date(time);
    switch (timeFormat) {
      case 'hour':
        return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      case 'day':
        return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
      case 'week':
        return `Week ${Math.ceil(date.getDate() / 7)}`;
      case 'month':
        return date.toLocaleDateString([], { month: 'short', year: '2-digit' });
      default:
        return date.toLocaleDateString();
    }
  }

  function formatValue(value: number): string {
    if (value >= 1000000) return `${(value / 1000000).toFixed(1)}M`;
    if (value >= 1000) return `${(value / 1000).toFixed(1)}K`;
    return value.toFixed(0);
  }

  function handleBrushStart(event: MouseEvent) {
    if (!showBrush) return;
    isDragging = true;
    const rect = (event.currentTarget as SVGElement).getBoundingClientRect();
    brushStart = event.clientX - rect.left - padding.left;
    brushEnd = brushStart;
  }

  function handleBrushMove(event: MouseEvent) {
    if (!isDragging || brushStart === null) return;
    const rect = (event.currentTarget as SVGElement).closest('svg')!.getBoundingClientRect();
    brushEnd = Math.max(0, Math.min(chartWidth, event.clientX - rect.left - padding.left));
  }

  function handleBrushEnd() {
    if (brushStart !== null && brushEnd !== null && brushStart !== brushEnd) {
      const [start, end] = [brushStart, brushEnd].sort((a, b) => a - b);
      const [timeStart, timeEnd] = timeExtent;
      const range = timeEnd - timeStart;

      const newStart = new Date(timeStart + (start / chartWidth) * range);
      const newEnd = new Date(timeStart + (end / chartWidth) * range);

      dispatch('rangeChange', { start: newStart, end: newEnd });
    }
    isDragging = false;
    brushStart = null;
    brushEnd = null;
  }

  function handleWheel(event: WheelEvent) {
    if (!enableZoom) return;
    event.preventDefault();

    const delta = event.deltaY > 0 ? 0.9 : 1.1;
    zoomLevel = Math.max(1, Math.min(10, zoomLevel * delta));
  }

  function resetZoom() {
    zoomLevel = 1;
    panOffset = 0;
  }

  function findNearestPoint(mouseX: number): { index: number; series: string } | null {
    let nearestIndex = -1;
    let nearestSeries = '';
    let minDistance = Infinity;

    data.forEach(series => {
      series.points.forEach((point, index) => {
        const pointX = xScale(new Date(point.timestamp).getTime());
        const distance = Math.abs(pointX - mouseX);
        if (distance < minDistance) {
          minDistance = distance;
          nearestIndex = index;
          nearestSeries = series.id;
        }
      });
    });

    return nearestIndex >= 0 ? { index: nearestIndex, series: nearestSeries } : null;
  }

  // Generate time axis ticks
  $: timeTicks = generateTimeTicks(visibleTimeExtent, 6);

  function generateTimeTicks(extent: [number, number], count: number): number[] {
    const [start, end] = extent;
    const step = (end - start) / (count - 1);
    return Array.from({ length: count }, (_, i) => start + step * i);
  }

  // Generate value axis ticks
  $: valueTicks = generateValueTicks(valueExtent, 5);

  function generateValueTicks(extent: [number, number], count: number): number[] {
    const [min, max] = extent;
    const step = (max - min) / (count - 1);
    return Array.from({ length: count }, (_, i) => min + step * i);
  }
</script>

<div
  class="time-series-chart"
  bind:clientWidth={containerWidth}
  on:wheel={handleWheel}
  role="img"
  aria-label="Time series chart"
>
  {#if showLegend && data.length > 1}
    <div class="chart-legend">
      {#each data as series}
        <div class="legend-item">
          <span class="legend-color" style="background: {series.color}" />
          <span>{series.label}</span>
        </div>
      {/each}
    </div>
  {/if}

  {#if containerWidth > 0}
    <svg width={containerWidth} height={mainHeight}>
      <defs>
        {#each data as series}
          <linearGradient id="gradient-{series.id}" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color={series.color} stop-opacity="0.3" />
            <stop offset="100%" stop-color={series.color} stop-opacity="0.05" />
          </linearGradient>
        {/each}
        <clipPath id="chart-clip">
          <rect x="0" y="0" width={chartWidth} height={chartHeight} />
        </clipPath>
      </defs>

      <g transform="translate({padding.left}, {padding.top})">
        <!-- Grid lines -->
        {#each valueTicks as tick}
          <line
            class="grid-line"
            x1="0"
            y1={yScale(tick)}
            x2={chartWidth}
            y2={yScale(tick)}
          />
          <text
            class="axis-label"
            x="-10"
            y={yScale(tick)}
            text-anchor="end"
            dominant-baseline="middle"
          >
            {formatValue(tick)}
          </text>
        {/each}

        <!-- Chart area with clipping -->
        <g clip-path="url(#chart-clip)">
          {#each data as series}
            <!-- Area fill -->
            <path
              class="series-area"
              d={generateAreaPath(series.points)}
              fill="url(#gradient-{series.id})"
            />

            <!-- Line -->
            <path
              class="series-line"
              d={generatePath(series.points)}
              fill="none"
              stroke={series.color}
              stroke-width="2"
            />

            <!-- Data points on hover -->
            {#if hoveredIndex !== null && hoveredSeries === series.id}
              {@const point = series.points[hoveredIndex]}
              {#if point}
                <circle
                  class="data-point"
                  cx={xScale(new Date(point.timestamp).getTime())}
                  cy={yScale(point.value)}
                  r="4"
                  fill={series.color}
                  transition:fade={{ duration: 100 }}
                />
              {/if}
            {/if}
          {/each}

          <!-- Hover detection overlay -->
          <rect
            class="hover-overlay"
            x="0"
            y="0"
            width={chartWidth}
            height={chartHeight}
            on:mousemove={(e) => {
              const rect = e.currentTarget.getBoundingClientRect();
              const mouseX = e.clientX - rect.left;
              const nearest = findNearestPoint(mouseX);
              if (nearest) {
                hoveredIndex = nearest.index;
                hoveredSeries = nearest.series;
              }
            }}
            on:mouseleave={() => {
              hoveredIndex = null;
              hoveredSeries = null;
            }}
          />

          <!-- Hover line -->
          {#if hoveredIndex !== null && hoveredSeries !== null}
            {@const series = data.find(s => s.id === hoveredSeries)}
            {#if series && series.points[hoveredIndex]}
              {@const point = series.points[hoveredIndex]}
              <line
                class="hover-line"
                x1={xScale(new Date(point.timestamp).getTime())}
                y1="0"
                x2={xScale(new Date(point.timestamp).getTime())}
                y2={chartHeight}
                transition:fade={{ duration: 100 }}
              />
            {/if}
          {/if}
        </g>

        <!-- X-axis labels -->
        {#each timeTicks as tick, i}
          <text
            class="axis-label x-axis"
            x={xScale(tick)}
            y={chartHeight + 20}
            text-anchor="middle"
          >
            {formatTimestamp(tick)}
          </text>
        {/each}
      </g>
    </svg>
  {/if}

  {#if showBrush && containerWidth > 0}
    <svg
      class="brush-chart"
      width={containerWidth}
      height={brushHeight}
      on:mousedown={handleBrushStart}
      on:mousemove={handleBrushMove}
      on:mouseup={handleBrushEnd}
      on:mouseleave={handleBrushEnd}
      role="slider"
      aria-label="Time range selector"
    >
      <g transform="translate({padding.left}, 5)">
        <!-- Mini chart lines -->
        {#each data as series}
          <path
            class="brush-line"
            d={generatePath(series.points)}
            fill="none"
            stroke={series.color}
            stroke-width="1"
            opacity="0.5"
          />
        {/each}

        <!-- Brush selection -->
        {#if brushStart !== null && brushEnd !== null && isDragging}
          <rect
            class="brush-selection"
            x={Math.min(brushStart, brushEnd)}
            y="0"
            width={Math.abs(brushEnd - brushStart)}
            height={brushHeight - 10}
            transition:fade={{ duration: 100 }}
          />
        {/if}
      </g>
    </svg>
  {/if}

  {#if enableZoom && zoomLevel > 1}
    <button class="reset-zoom" on:click={resetZoom}>
      Reset Zoom
    </button>
  {/if}

  <!-- Tooltip -->
  {#if hoveredIndex !== null && hoveredSeries !== null && containerWidth > 0}
    {@const series = data.find(s => s.id === hoveredSeries)}
    {#if series && series.points[hoveredIndex]}
      {@const point = series.points[hoveredIndex]}
      <div
        class="chart-tooltip"
        style="left: {xScale(new Date(point.timestamp).getTime()) + padding.left}px; top: {yScale(point.value) + padding.top}px;"
        transition:fly={{ y: 5, duration: 100 }}
      >
        <div class="tooltip-time">{new Date(point.timestamp).toLocaleString()}</div>
        <div class="tooltip-value" style="color: {series.color}">
          {series.label}: {formatValue(point.value)}
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .time-series-chart {
    width: 100%;
  }

  .legend {
    display: flex;
    gap: 1rem;
    margin-bottom: 0.75rem;
    flex-wrap: wrap;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }

  .legend-color {
    width: 12px;
    height: 12px;
    border-radius: 2px;
  }

  .legend-label {
    font-size: 0.8125rem;
    color: var(--text);
  }

  .chart-container {
    width: 100%;
    overflow-x: auto;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--text-muted);
    font-size: 0.875rem;
  }

  svg {
    background: transparent;
    max-width: 100%;
  }
</style>