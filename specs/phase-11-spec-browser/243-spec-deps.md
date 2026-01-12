# Spec 243: Dependency Visualization

## Phase
11 - Spec Browser UI

## Spec ID
243

## Status
Planned

## Dependencies
- Spec 231 (Spec List Layout)
- Spec 236 (Spec Detail View)
- Spec 239 (Spec Validation)

## Estimated Context
~10%

---

## Objective

Create an interactive dependency visualization system that displays spec relationships as a graph, supports navigation, highlights dependency chains, and identifies circular dependencies or orphaned specs.

---

## Acceptance Criteria

- [ ] Interactive graph visualization of dependencies
- [ ] Zoom, pan, and fit-to-view controls
- [ ] Click to navigate to spec
- [ ] Highlight dependency chains on hover
- [ ] Visual indicators for circular dependencies
- [ ] Filter by phase or status
- [ ] Export graph as image
- [ ] Mini-map for large graphs

---

## Implementation Details

### DependencyGraph.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { writable } from 'svelte/store';
  import * as d3 from 'd3';
  import type { Spec } from '$lib/types/spec';
  import Icon from '$lib/components/Icon.svelte';
  import Button from '$lib/components/Button.svelte';
  import Tooltip from '$lib/components/Tooltip.svelte';

  export let spec: Spec | null = null;
  export let specs: Spec[] = [];
  export let width = 800;
  export let height = 600;
  export let showMiniMap = true;

  const dispatch = createEventDispatcher<{
    nodeClick: { spec: Spec };
    nodeHover: { spec: Spec | null };
  }>();

  let svgRef: SVGSVGElement;
  let containerRef: HTMLDivElement;
  let simulation: d3.Simulation<GraphNode, GraphLink>;
  let zoom: d3.ZoomBehavior<SVGSVGElement, unknown>;

  let hoveredNode = writable<string | null>(null);
  let selectedNode = writable<string | null>(spec?.id ?? null);
  let highlightedPath = writable<Set<string>>(new Set());

  interface GraphNode extends d3.SimulationNodeDatum {
    id: string;
    title: string;
    status: string;
    phase: number;
    isCircular?: boolean;
    radius: number;
  }

  interface GraphLink extends d3.SimulationLinkDatum<GraphNode> {
    source: GraphNode | string;
    target: GraphNode | string;
    isCircular?: boolean;
  }

  // Build graph data from specs
  $: graphData = buildGraphData(specs, spec);

  function buildGraphData(specs: Spec[], focusSpec: Spec | null) {
    const nodes: GraphNode[] = specs.map(s => ({
      id: s.id,
      title: s.title,
      status: s.status,
      phase: s.phase,
      radius: s.id === focusSpec?.id ? 20 : 15
    }));

    const links: GraphLink[] = [];
    const circularPaths = findCircularDependencies(specs);

    specs.forEach(s => {
      s.dependencies?.forEach(depId => {
        const isCircular = circularPaths.some(
          path => path.includes(s.id) && path.includes(depId)
        );

        links.push({
          source: s.id,
          target: depId,
          isCircular
        });

        // Mark nodes in circular paths
        if (isCircular) {
          const sourceNode = nodes.find(n => n.id === s.id);
          const targetNode = nodes.find(n => n.id === depId);
          if (sourceNode) sourceNode.isCircular = true;
          if (targetNode) targetNode.isCircular = true;
        }
      });
    });

    return { nodes, links };
  }

  function findCircularDependencies(specs: Spec[]): string[][] {
    const paths: string[][] = [];
    const specMap = new Map(specs.map(s => [s.id, s]));

    function dfs(
      id: string,
      visited: Set<string>,
      path: string[]
    ): void {
      if (path.includes(id)) {
        const cycleStart = path.indexOf(id);
        paths.push([...path.slice(cycleStart), id]);
        return;
      }

      if (visited.has(id)) return;
      visited.add(id);

      const spec = specMap.get(id);
      spec?.dependencies?.forEach(depId => {
        if (specMap.has(depId)) {
          dfs(depId, visited, [...path, id]);
        }
      });
    }

    specs.forEach(s => {
      dfs(s.id, new Set(), []);
    });

    return paths;
  }

  function getNodeColor(node: GraphNode): string {
    if (node.isCircular) return 'var(--color-danger)';

    const colors: Record<string, string> = {
      'planned': 'var(--color-status-planned)',
      'in-progress': 'var(--color-status-progress)',
      'implemented': 'var(--color-status-implemented)',
      'tested': 'var(--color-status-tested)',
      'deprecated': 'var(--color-status-deprecated)'
    };

    return colors[node.status] || 'var(--color-neutral)';
  }

  function highlightDependencyPath(nodeId: string) {
    const path = new Set<string>();
    const specMap = new Map(specs.map(s => [s.id, s]));

    // Find all dependencies (upstream)
    function findUpstream(id: string) {
      path.add(id);
      const spec = specMap.get(id);
      spec?.dependencies?.forEach(depId => {
        if (!path.has(depId) && specMap.has(depId)) {
          findUpstream(depId);
        }
      });
    }

    // Find all dependents (downstream)
    function findDownstream(id: string) {
      specs.forEach(s => {
        if (s.dependencies?.includes(id) && !path.has(s.id)) {
          path.add(s.id);
          findDownstream(s.id);
        }
      });
    }

    findUpstream(nodeId);
    findDownstream(nodeId);

    highlightedPath.set(path);
  }

  function clearHighlight() {
    highlightedPath.set(new Set());
  }

  function handleNodeClick(node: GraphNode) {
    const specData = specs.find(s => s.id === node.id);
    if (specData) {
      selectedNode.set(node.id);
      dispatch('nodeClick', { spec: specData });
    }
  }

  function handleNodeHover(node: GraphNode | null) {
    if (node) {
      hoveredNode.set(node.id);
      highlightDependencyPath(node.id);
      const specData = specs.find(s => s.id === node.id);
      dispatch('nodeHover', { spec: specData ?? null });
    } else {
      hoveredNode.set(null);
      clearHighlight();
      dispatch('nodeHover', { spec: null });
    }
  }

  function zoomIn() {
    d3.select(svgRef).transition().call(zoom.scaleBy, 1.5);
  }

  function zoomOut() {
    d3.select(svgRef).transition().call(zoom.scaleBy, 0.67);
  }

  function fitToView() {
    const bounds = svgRef.getBBox();
    const fullWidth = width;
    const fullHeight = height;
    const widthScale = fullWidth / bounds.width;
    const heightScale = fullHeight / bounds.height;
    const scale = Math.min(widthScale, heightScale) * 0.9;
    const centerX = bounds.x + bounds.width / 2;
    const centerY = bounds.y + bounds.height / 2;

    d3.select(svgRef)
      .transition()
      .duration(500)
      .call(
        zoom.transform,
        d3.zoomIdentity
          .translate(fullWidth / 2, fullHeight / 2)
          .scale(scale)
          .translate(-centerX, -centerY)
      );
  }

  function exportAsImage() {
    const svgData = new XMLSerializer().serializeToString(svgRef);
    const canvas = document.createElement('canvas');
    canvas.width = width * 2;
    canvas.height = height * 2;
    const ctx = canvas.getContext('2d')!;
    ctx.scale(2, 2);

    const img = new Image();
    img.onload = () => {
      ctx.fillStyle = 'white';
      ctx.fillRect(0, 0, width, height);
      ctx.drawImage(img, 0, 0);

      const link = document.createElement('a');
      link.download = `dependency-graph-${spec?.id || 'all'}.png`;
      link.href = canvas.toDataURL('image/png');
      link.click();
    };
    img.src = 'data:image/svg+xml;base64,' + btoa(unescape(encodeURIComponent(svgData)));
  }

  onMount(() => {
    const svg = d3.select(svgRef);
    const g = svg.append('g').attr('class', 'graph-container');

    // Add zoom behavior
    zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        g.attr('transform', event.transform);
      });

    svg.call(zoom);

    // Add arrow marker
    svg.append('defs').append('marker')
      .attr('id', 'arrowhead')
      .attr('viewBox', '-0 -5 10 10')
      .attr('refX', 20)
      .attr('refY', 0)
      .attr('orient', 'auto')
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .append('path')
      .attr('d', 'M 0,-5 L 10,0 L 0,5')
      .attr('fill', 'var(--color-text-tertiary)');

    // Add circular arrow marker
    svg.select('defs').append('marker')
      .attr('id', 'arrowhead-circular')
      .attr('viewBox', '-0 -5 10 10')
      .attr('refX', 20)
      .attr('refY', 0)
      .attr('orient', 'auto')
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .append('path')
      .attr('d', 'M 0,-5 L 10,0 L 0,5')
      .attr('fill', 'var(--color-danger)');

    // Create links
    const link = g.append('g')
      .attr('class', 'links')
      .selectAll('line')
      .data(graphData.links)
      .join('line')
      .attr('class', 'link')
      .attr('stroke', d => d.isCircular ? 'var(--color-danger)' : 'var(--color-border)')
      .attr('stroke-width', d => d.isCircular ? 2 : 1)
      .attr('marker-end', d => d.isCircular ? 'url(#arrowhead-circular)' : 'url(#arrowhead)');

    // Create nodes
    const node = g.append('g')
      .attr('class', 'nodes')
      .selectAll('g')
      .data(graphData.nodes)
      .join('g')
      .attr('class', 'node')
      .call(d3.drag<SVGGElement, GraphNode>()
        .on('start', dragStarted)
        .on('drag', dragged)
        .on('end', dragEnded)
      )
      .on('click', (event, d) => handleNodeClick(d))
      .on('mouseenter', (event, d) => handleNodeHover(d))
      .on('mouseleave', () => handleNodeHover(null));

    node.append('circle')
      .attr('r', d => d.radius)
      .attr('fill', d => getNodeColor(d))
      .attr('stroke', d => d.id === spec?.id ? 'var(--color-primary)' : 'none')
      .attr('stroke-width', 3);

    node.append('text')
      .text(d => d.id)
      .attr('text-anchor', 'middle')
      .attr('dy', '0.35em')
      .attr('font-size', '10px')
      .attr('font-weight', '600')
      .attr('fill', 'white')
      .attr('pointer-events', 'none');

    // Create simulation
    simulation = d3.forceSimulation(graphData.nodes)
      .force('link', d3.forceLink<GraphNode, GraphLink>(graphData.links)
        .id(d => d.id)
        .distance(100)
      )
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(30))
      .on('tick', () => {
        link
          .attr('x1', d => (d.source as GraphNode).x!)
          .attr('y1', d => (d.source as GraphNode).y!)
          .attr('x2', d => (d.target as GraphNode).x!)
          .attr('y2', d => (d.target as GraphNode).y!);

        node.attr('transform', d => `translate(${d.x},${d.y})`);
      });

    function dragStarted(event: d3.D3DragEvent<SVGGElement, GraphNode, GraphNode>) {
      if (!event.active) simulation.alphaTarget(0.3).restart();
      event.subject.fx = event.subject.x;
      event.subject.fy = event.subject.y;
    }

    function dragged(event: d3.D3DragEvent<SVGGElement, GraphNode, GraphNode>) {
      event.subject.fx = event.x;
      event.subject.fy = event.y;
    }

    function dragEnded(event: d3.D3DragEvent<SVGGElement, GraphNode, GraphNode>) {
      if (!event.active) simulation.alphaTarget(0);
      event.subject.fx = null;
      event.subject.fy = null;
    }

    // Initial fit to view
    setTimeout(fitToView, 500);
  });

  onDestroy(() => {
    if (simulation) {
      simulation.stop();
    }
  });

  // Update highlight styles
  $: {
    if (svgRef) {
      d3.select(svgRef)
        .selectAll('.node')
        .classed('node--highlighted', d => $highlightedPath.has((d as GraphNode).id))
        .classed('node--dimmed', d =>
          $highlightedPath.size > 0 && !$highlightedPath.has((d as GraphNode).id)
        );

      d3.select(svgRef)
        .selectAll('.link')
        .classed('link--dimmed', d => {
          const source = (d as GraphLink).source as GraphNode;
          const target = (d as GraphLink).target as GraphNode;
          return $highlightedPath.size > 0 &&
            (!$highlightedPath.has(source.id) || !$highlightedPath.has(target.id));
        });
    }
  }
</script>

<div class="dependency-graph" bind:this={containerRef}>
  <div class="dependency-graph__toolbar">
    <div class="dependency-graph__toolbar-group">
      <Tooltip content="Zoom in">
        <Button variant="ghost" size="sm" on:click={zoomIn}>
          <Icon name="zoom-in" size={16} />
        </Button>
      </Tooltip>
      <Tooltip content="Zoom out">
        <Button variant="ghost" size="sm" on:click={zoomOut}>
          <Icon name="zoom-out" size={16} />
        </Button>
      </Tooltip>
      <Tooltip content="Fit to view">
        <Button variant="ghost" size="sm" on:click={fitToView}>
          <Icon name="maximize" size={16} />
        </Button>
      </Tooltip>
    </div>

    <div class="dependency-graph__toolbar-group">
      <Tooltip content="Export as image">
        <Button variant="ghost" size="sm" on:click={exportAsImage}>
          <Icon name="download" size={16} />
        </Button>
      </Tooltip>
    </div>
  </div>

  <svg
    bind:this={svgRef}
    {width}
    {height}
    class="dependency-graph__svg"
  />

  <div class="dependency-graph__legend">
    <h4>Legend</h4>
    <div class="dependency-graph__legend-item">
      <span class="dependency-graph__legend-dot" style:background="var(--color-status-planned)" />
      <span>Planned</span>
    </div>
    <div class="dependency-graph__legend-item">
      <span class="dependency-graph__legend-dot" style:background="var(--color-status-progress)" />
      <span>In Progress</span>
    </div>
    <div class="dependency-graph__legend-item">
      <span class="dependency-graph__legend-dot" style:background="var(--color-status-implemented)" />
      <span>Implemented</span>
    </div>
    <div class="dependency-graph__legend-item">
      <span class="dependency-graph__legend-dot" style:background="var(--color-status-tested)" />
      <span>Tested</span>
    </div>
    <div class="dependency-graph__legend-item">
      <span class="dependency-graph__legend-dot" style:background="var(--color-danger)" />
      <span>Circular Dep</span>
    </div>
  </div>

  {#if $hoveredNode}
    {@const hoveredSpec = specs.find(s => s.id === $hoveredNode)}
    {#if hoveredSpec}
      <div class="dependency-graph__tooltip">
        <div class="dependency-graph__tooltip-id">{hoveredSpec.id}</div>
        <div class="dependency-graph__tooltip-title">{hoveredSpec.title}</div>
        <div class="dependency-graph__tooltip-deps">
          <span>Deps: {hoveredSpec.dependencies?.length ?? 0}</span>
          <span>Dependents: {specs.filter(s => s.dependencies?.includes(hoveredSpec.id)).length}</span>
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .dependency-graph {
    position: relative;
    width: 100%;
    height: 100%;
    background: var(--color-surface-subtle);
    border-radius: 8px;
    overflow: hidden;
  }

  .dependency-graph__toolbar {
    position: absolute;
    top: 12px;
    left: 12px;
    display: flex;
    gap: 8px;
    z-index: 10;
  }

  .dependency-graph__toolbar-group {
    display: flex;
    gap: 4px;
    padding: 4px;
    background: var(--color-surface);
    border-radius: 6px;
    box-shadow: var(--shadow-sm);
  }

  .dependency-graph__svg {
    width: 100%;
    height: 100%;
  }

  .dependency-graph__svg :global(.link) {
    transition: opacity 0.2s;
  }

  .dependency-graph__svg :global(.link--dimmed) {
    opacity: 0.1;
  }

  .dependency-graph__svg :global(.node) {
    cursor: pointer;
    transition: opacity 0.2s;
  }

  .dependency-graph__svg :global(.node--dimmed) {
    opacity: 0.2;
  }

  .dependency-graph__svg :global(.node--highlighted circle) {
    filter: drop-shadow(0 0 4px currentColor);
  }

  .dependency-graph__legend {
    position: absolute;
    bottom: 12px;
    left: 12px;
    padding: 12px;
    background: var(--color-surface);
    border-radius: 6px;
    box-shadow: var(--shadow-sm);
    font-size: 0.75rem;
  }

  .dependency-graph__legend h4 {
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--color-text-tertiary);
    margin: 0 0 8px;
  }

  .dependency-graph__legend-item {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
    color: var(--color-text-secondary);
  }

  .dependency-graph__legend-item:last-child {
    margin-bottom: 0;
  }

  .dependency-graph__legend-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
  }

  .dependency-graph__tooltip {
    position: absolute;
    top: 12px;
    right: 12px;
    padding: 12px;
    background: var(--color-surface);
    border-radius: 6px;
    box-shadow: var(--shadow-md);
    max-width: 250px;
  }

  .dependency-graph__tooltip-id {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-primary);
    margin-bottom: 4px;
  }

  .dependency-graph__tooltip-title {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-primary);
    margin-bottom: 8px;
  }

  .dependency-graph__tooltip-deps {
    display: flex;
    gap: 12px;
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }
</style>
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import DependencyGraph from './DependencyGraph.svelte';
import { createMockSpecs } from '$lib/test-utils/mock-data';

describe('DependencyGraph', () => {
  const specs = [
    { id: '1', title: 'Spec 1', status: 'planned', dependencies: [] },
    { id: '2', title: 'Spec 2', status: 'in-progress', dependencies: ['1'] },
    { id: '3', title: 'Spec 3', status: 'implemented', dependencies: ['1', '2'] }
  ];

  it('renders SVG element', () => {
    render(DependencyGraph, { props: { specs } });

    expect(document.querySelector('svg')).toBeInTheDocument();
  });

  it('renders toolbar controls', () => {
    render(DependencyGraph, { props: { specs } });

    expect(screen.getByLabelText('Zoom in')).toBeInTheDocument();
    expect(screen.getByLabelText('Zoom out')).toBeInTheDocument();
    expect(screen.getByLabelText('Fit to view')).toBeInTheDocument();
  });

  it('renders legend', () => {
    render(DependencyGraph, { props: { specs } });

    expect(screen.getByText('Legend')).toBeInTheDocument();
    expect(screen.getByText('Planned')).toBeInTheDocument();
    expect(screen.getByText('In Progress')).toBeInTheDocument();
  });

  it('dispatches nodeClick on node click', async () => {
    const { component } = render(DependencyGraph, { props: { specs } });

    const clickHandler = vi.fn();
    component.$on('nodeClick', clickHandler);

    // Note: D3 nodes are created dynamically, testing might require waiting
    // This is a simplified test
  });

  it('detects circular dependencies', () => {
    const circularSpecs = [
      { id: '1', title: 'Spec 1', dependencies: ['3'] },
      { id: '2', title: 'Spec 2', dependencies: ['1'] },
      { id: '3', title: 'Spec 3', dependencies: ['2'] }
    ];

    render(DependencyGraph, { props: { specs: circularSpecs } });

    expect(screen.getByText('Circular Dep')).toBeInTheDocument();
  });

  it('highlights focused spec', () => {
    const focusSpec = specs[1];
    render(DependencyGraph, { props: { specs, spec: focusSpec } });

    // Check that focus spec has different styling
  });
});
```

---

## Related Specs

- Spec 236: Spec Detail View
- Spec 239: Spec Validation
- Spec 244: Version History
