<script lang="ts">
  export let data: Array<{
    id: string;
    label: string;
    color: string;
    points: Array<{ timestamp: string; value: number }>;
  }> = [];
  export let height: number = 200;
  export let showLegend: boolean = true;
  export let showBrush: boolean = true;

  let svgElement: SVGSVGElement;
  
  const width = 600;
  const margin = { top: 20, right: 20, bottom: 40, left: 50 };
  const chartWidth = width - margin.left - margin.right;
  const chartHeight = height - margin.top - margin.bottom;

  $: allPoints = data.flatMap(d => d.points);
  $: maxValue = allPoints.length ? Math.max(...allPoints.map(p => p.value)) : 0;
  $: minValue = allPoints.length ? Math.min(...allPoints.map(p => p.value)) : 0;
  $: valueRange = maxValue - minValue || 1;

  $: timeRange = allPoints.length ? {
    start: new Date(Math.min(...allPoints.map(p => new Date(p.timestamp).getTime()))),
    end: new Date(Math.max(...allPoints.map(p => new Date(p.timestamp).getTime())))
  } : { start: new Date(), end: new Date() };

  $: timeDuration = timeRange.end.getTime() - timeRange.start.getTime() || 1;

  function getPathData(points: Array<{ timestamp: string; value: number }>): string {
    if (points.length < 2) return '';
    
    return points.map((p, i) => {
      const x = ((new Date(p.timestamp).getTime() - timeRange.start.getTime()) / timeDuration) * chartWidth;
      const y = chartHeight - ((p.value - minValue) / valueRange) * chartHeight;
      return `${i === 0 ? 'M' : 'L'} ${x} ${y}`;
    }).join(' ');
  }

  function formatValue(value: number): string {
    return value.toFixed(0);
  }

  function formatTime(timestamp: string): string {
    return new Date(timestamp).toLocaleDateString();
  }
</script>

<div class="time-series-chart">
  {#if showLegend && data.length > 0}
    <div class="legend">
      {#each data as series}
        <div class="legend-item">
          <div class="legend-color" style="background-color: {series.color}"></div>
          <span class="legend-label">{series.label}</span>
        </div>
      {/each}
    </div>
  {/if}

  <div class="chart-container">
    {#if allPoints.length === 0}
      <div class="empty-state">
        <p>No data available</p>
      </div>
    {:else}
      <svg bind:this={svgElement} {width} {height} viewBox="0 0 {width} {height}">
        <g transform="translate({margin.left}, {margin.top})">
          <!-- Y-axis -->
          <line x1="0" y1="0" x2="0" y2="{chartHeight}" stroke="var(--border)" stroke-width="1"/>
          
          <!-- X-axis -->
          <line x1="0" y1="{chartHeight}" x2="{chartWidth}" y2="{chartHeight}" stroke="var(--border)" stroke-width="1"/>
          
          <!-- Grid lines -->
          {#each [0, 0.25, 0.5, 0.75, 1] as ratio}
            {#if ratio > 0}
              <line
                x1="0"
                y1="{chartHeight * (1 - ratio)}"
                x2="{chartWidth}"
                y2="{chartHeight * (1 - ratio)}"
                stroke="var(--border)"
                stroke-width="0.5"
                opacity="0.3"
              />
            {/if}
          {/each}
          
          <!-- Data lines -->
          {#each data as series}
            {@const pathData = getPathData(series.points)}
            {#if pathData}
              <path
                d={pathData}
                fill="none"
                stroke={series.color}
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              
              <!-- Data points -->
              {#each series.points as point}
                {@const x = ((new Date(point.timestamp).getTime() - timeRange.start.getTime()) / timeDuration) * chartWidth}
                {@const y = chartHeight - ((point.value - minValue) / valueRange) * chartHeight}
                <circle
                  cx={x}
                  cy={y}
                  r="3"
                  fill={series.color}
                  stroke="var(--bg-card)"
                  stroke-width="1"
                >
                  <title>{formatValue(point.value)} at {formatTime(point.timestamp)}</title>
                </circle>
              {/each}
            {/if}
          {/each}
          
          <!-- Y-axis labels -->
          <text x="-8" y="4" text-anchor="end" fill="var(--text-muted)" font-size="11">
            {formatValue(maxValue)}
          </text>
          {#if minValue !== maxValue}
            <text x="-8" y="{chartHeight + 4}" text-anchor="end" fill="var(--text-muted)" font-size="11">
              {formatValue(minValue)}
            </text>
          {/if}
        </g>
      </svg>
    {/if}
  </div>
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