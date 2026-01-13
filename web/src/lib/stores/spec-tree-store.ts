import { writable, derived } from 'svelte/store';
import type { SpecTreeNode, TreeState, DragState, SpecStatus } from '$lib/types/spec-tree';
import { ipc } from '$lib/ipc';

function createSpecTreeStore() {
  const initialState: TreeState = {
    nodes: [],
    expandedIds: new Set(['phase-0', 'phase-1']),
    selectedId: null,
    focusedId: null,
    dragState: null,
  };

  const { subscribe, set, update } = writable<TreeState>(initialState);

  return {
    subscribe,

    async loadSpecs(): Promise<void> {
      const specs = await ipc.invoke('spec:list', {});
      const nodes = buildTree(specs);
      update(s => ({ ...s, nodes }));
    },

    toggleExpand(nodeId: string) {
      update(s => {
        const expandedIds = new Set(s.expandedIds);
        if (expandedIds.has(nodeId)) {
          expandedIds.delete(nodeId);
        } else {
          expandedIds.add(nodeId);
        }
        return { ...s, expandedIds };
      });
    },

    expandAll() {
      update(s => {
        const expandedIds = new Set<string>();
        const collect = (nodes: SpecTreeNode[]) => {
          nodes.forEach(n => {
            if (n.children?.length) {
              expandedIds.add(n.id);
              collect(n.children);
            }
          });
        };
        collect(s.nodes);
        return { ...s, expandedIds };
      });
    },

    collapseAll() {
      update(s => ({ ...s, expandedIds: new Set() }));
    },

    select(nodeId: string | null) {
      update(s => ({ ...s, selectedId: nodeId, focusedId: nodeId }));
    },

    setFocus(nodeId: string | null) {
      update(s => ({ ...s, focusedId: nodeId }));
    },

    startDrag(nodeId: string) {
      update(s => ({
        ...s,
        dragState: { nodeId, targetId: null, position: 'after' },
      }));
    },

    updateDrag(targetId: string, position: 'before' | 'after' | 'inside') {
      update(s => {
        if (!s.dragState) return s;
        return {
          ...s,
          dragState: { ...s.dragState, targetId, position },
        };
      });
    },

    endDrag() {
      update(s => ({ ...s, dragState: null }));
    },

    async reorder(nodeId: string, targetId: string, position: 'before' | 'after') {
      // Note: This would need backend support for reordering
      // await ipc.invoke('spec:reorder', { nodeId, targetId, position });
      console.log('Reorder not yet implemented:', { nodeId, targetId, position });
      await this.loadSpecs();
    },
  };
}

function buildTree(specs: any[]): SpecTreeNode[] {
  const phases = new Map<number, SpecTreeNode>();

  specs.forEach(spec => {
    // Extract phase from path or use default
    const phase = extractPhaseFromPath(spec.path) || 0;
    
    if (!phases.has(phase)) {
      phases.set(phase, {
        id: `phase-${phase}`,
        type: 'phase',
        label: getPhaseName(phase),
        number: phase,
        children: [],
        isExpanded: phase <= 1,
      });
    }

    // Extract spec number from name
    const specNumber = extractSpecNumber(spec.name);
    
    phases.get(phase)!.children!.push({
      id: spec.path,
      type: 'spec',
      label: spec.name.replace(/^\d+-/, '').replace(/\.md$/, ''),
      number: specNumber,
      status: spec.status as SpecStatus,
      path: spec.path,
    });
  });

  // Sort specs within each phase by number
  phases.forEach(phase => {
    if (phase.children) {
      phase.children.sort((a, b) => a.number - b.number);
    }
  });

  return Array.from(phases.values()).sort((a, b) => a.number - b.number);
}

function extractPhaseFromPath(path: string): number {
  const match = path.match(/phase-(\d+)/);
  return match ? parseInt(match[1]) : 0;
}

function extractSpecNumber(name: string): number {
  const match = name.match(/^(\d+)/);
  return match ? parseInt(match[1]) : 0;
}

function getPhaseName(phase: number): string {
  const phaseNames: Record<number, string> = {
    0: 'Foundation',
    1: 'Core Features',
    2: 'Advanced Features',
    3: 'Integration',
    4: 'Testing',
    5: 'Documentation',
    6: 'UI/UX',
    7: 'Performance',
    8: 'Security',
    9: 'Deployment',
    10: 'Monitoring',
    11: 'Spec Browser',
    12: 'Future',
  };
  return phaseNames[phase] || `Phase ${phase}`;
}

export const specTreeStore = createSpecTreeStore();

export const flattenedNodes = derived(specTreeStore, $state => {
  const result: (SpecTreeNode & { depth: number })[] = [];
  const flatten = (nodes: SpecTreeNode[], depth = 0) => {
    nodes.forEach(node => {
      result.push({ ...node, depth });
      if (node.children && $state.expandedIds.has(node.id)) {
        flatten(node.children, depth + 1);
      }
    });
  };
  flatten($state.nodes);
  return result;
});