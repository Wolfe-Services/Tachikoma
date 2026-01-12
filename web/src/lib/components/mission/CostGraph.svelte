<script lang="ts">
  export let data: { timestamp: string; cost: number }[] = [];

  let svgElement: SVGSVGElement;
  
  const width = 280;
  const height = 120;
  const margin = { top: 20, right: 20, bottom: 30, left: 40 };
  const chartWidth = width - margin.left - margin.right;
  const chartHeight = height - margin.top - margin.bottom;

  $: if (data.length === 0) {
    data = [];
  }

  $: maxCost = data.length ? Math.max(...data.map(d => d.cost)) : 0;
  $: minCost = data.length ? Math.min(...data.map(d => d.cost)) : 0;
  $: costRange = maxCost - minCost || 1; // Avoid division by zero

  $: pathData = data.length > 1 ? data.map((d, i) => {
    const x = (i / (data.length - 1)) * chartWidth;
    const y = chartHeight - ((d.cost - minCost) / costRange) * chartHeight;
    return `${i === 0 ? 'M' : 'L'} ${x} ${y}`;
  }).join(' ') : '';

  function formatCost(amount: number): string {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 3,
      maximumFractionDigits: 3,
    }).format(amount);
  }

  function formatTime(timestamp: string): string {
    return new Date(timestamp).toLocaleTimeString('en-US', { 
      hour: '2-digit', 
      minute: '2-digit' 
    });
  }
</script>

<div class="cost-graph">
  <h4 class="cost-graph__title">Cost History</h4>
  
  {#if data.length === 0}
    <div class="cost-graph__empty">
      <p>No cost history available</p>
    </div>
  {:else if data.length === 1}
    <div class="cost-graph__single">
      <p>Started at {formatCost(data[0].cost)}</p>
      <span class="cost-graph__time">{formatTime(data[0].timestamp)}</span>
    </div>
  {:else}
    <div class="cost-graph__chart">
      <svg bind:this={svgElement} {width} {height} viewBox="0 0 {width} {height}">
        <g transform="translate({margin.left}, {margin.top})">
          <!-- Grid lines -->
          <defs>
            <pattern id="grid" width="1" height="1" patternUnits="userSpaceOnUse">
              <path d="M 1 0 L 0 0 0 1" fill="none" stroke="var(--border)" stroke-width="0.5" opacity="0.3"/>
            </pattern>
          </defs>
          
          <!-- Y-axis -->
          <line x1="0" y1="0" x2="0" y2="{chartHeight}" stroke="var(--border)" stroke-width="1"/>
          
          <!-- X-axis -->
          <line x1="0" y1="{chartHeight}" x2="{chartWidth}" y2="{chartHeight}" stroke="var(--border)" stroke-width="1"/>
          
          <!-- Cost line -->
          {#if pathData}
            <path
              d={pathData}
              fill="none"
              stroke="var(--accent)"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          {/if}
          
          <!-- Data points -->
          {#each data as point, i}
            {#if data.length > 1}
              {@const x = (i / (data.length - 1)) * chartWidth}
              {@const y = chartHeight - ((point.cost - minCost) / costRange) * chartHeight}
              <circle
                cx={x}
                cy={y}
                r="3"
                fill="var(--accent)"
                stroke="var(--bg)"
                stroke-width="1"
              >
                <title>{formatCost(point.cost)} at {formatTime(point.timestamp)}</title>
              </circle>
            {/if}
          {/each}
          
          <!-- Y-axis labels -->
          <text x="-8" y="4" text-anchor="end" fill="var(--text-muted)" font-size="11">
            {formatCost(maxCost)}
          </text>
          {#if minCost !== maxCost}
            <text x="-8" y="{chartHeight + 4}" text-anchor="end" fill="var(--text-muted)" font-size="11">
              {formatCost(minCost)}
            </text>
          {/if}
        </g>
      </svg>
    </div>
    
    <div class="cost-graph__summary">
      <span class="cost-graph__stat">
        <span class="cost-graph__stat-label">Started:</span>
        <span class="cost-graph__stat-value">{formatCost(data[0].cost)}</span>
      </span>
      <span class="cost-graph__stat">
        <span class="cost-graph__stat-label">Current:</span>
        <span class="cost-graph__stat-value">{formatCost(data[data.length - 1].cost)}</span>
      </span>
      <span class="cost-graph__stat">
        <span class="cost-graph__stat-label">Growth:</span>
        <span class="cost-graph__stat-value cost-graph__stat-value--accent">
          +{formatCost(data[data.length - 1].cost - data[0].cost)}
        </span>
      </span>
    </div>
  {/if}
</div>

<style>
  .cost-graph {
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
  }

  .cost-graph__title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
    margin: 0 0 12px 0;
  }

  .cost-graph__empty,
  .cost-graph__single {
    text-align: center;
    padding: 20px;
    color: var(--text-muted);
    font-size: 13px;
  }

  .cost-graph__single p {
    margin: 0 0 4px 0;
    color: var(--text);
  }

  .cost-graph__time {
    font-size: 11px;
    color: var(--text-muted);
  }

  .cost-graph__chart {
    display: flex;
    justify-content: center;
    margin-bottom: 12px;
  }

  .cost-graph__summary {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    padding-top: 8px;
    border-top: 1px solid var(--border);
  }

  .cost-graph__stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
    text-align: center;
  }

  .cost-graph__stat-label {
    color: var(--text-muted);
  }

  .cost-graph__stat-value {
    color: var(--text);
    font-weight: 600;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
  }

  .cost-graph__stat-value--accent {
    color: var(--accent);
  }

  svg {
    background: transparent;
  }
</style>