import type { ImplementationPlan, PlanSection, PlanItem, PlanProgress } from '$lib/types/impl-plan';

export function parseImplementationPlan(content: string, specId: string): ImplementationPlan | null {
  const lines = content.split('\n');
  const sections: PlanSection[] = [];
  let currentSection: PlanSection | null = null;
  let title = '';
  
  // Extract spec title
  const titleMatch = content.match(/^#\s+(.+)/m);
  if (titleMatch) {
    title = titleMatch[1];
  }

  // Look for acceptance criteria section
  let inAcceptanceCriteria = false;
  let inImplementationDetails = false;
  
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    
    // Look for "Acceptance Criteria" section
    if (line.match(/^##\s+Acceptance Criteria/i)) {
      inAcceptanceCriteria = true;
      inImplementationDetails = false;
      currentSection = {
        id: 'acceptance-criteria',
        title: 'Acceptance Criteria',
        items: []
      };
      sections.push(currentSection);
      continue;
    }
    
    // Look for "Implementation Details" section 
    if (line.match(/^##\s+Implementation Details/i)) {
      inImplementationDetails = true;
      inAcceptanceCriteria = false;
      currentSection = null;
      continue;
    }

    // Stop at next major section
    if (line.match(/^##\s+/) && !line.match(/^##\s+(Acceptance Criteria|Implementation Details)/i)) {
      inAcceptanceCriteria = false;
      inImplementationDetails = false;
      currentSection = null;
      continue;
    }

    // Parse subsections within Implementation Details
    if (inImplementationDetails && line.match(/^###\s+(.+)/)) {
      const sectionTitle = line.match(/^###\s+(.+)/)?.[1] || 'Unknown Section';
      currentSection = {
        id: generateId(sectionTitle),
        title: sectionTitle,
        items: []
      };
      sections.push(currentSection);
      continue;
    }
    
    // Parse checklist items
    if (currentSection && line.match(/^-\s+\[[ x]\]/)) {
      const item = parseChecklistItem(line, i + 1);
      if (item) {
        currentSection.items.push(item);
      }
    }
  }

  // If no sections were found, return null
  if (sections.length === 0) {
    return null;
  }

  const progress = calculateProgress(sections);
  
  return {
    specId,
    title,
    sections,
    progress
  };
}

function parseChecklistItem(line: string, lineNumber: number): PlanItem | null {
  const match = line.match(/^-\s+\[([x\s])\]\s+(.+)$/);
  if (!match) return null;
  
  const [, checkState, text] = match;
  const completed = checkState.toLowerCase() === 'x';
  
  return {
    id: generateId(`${lineNumber}-${text}`),
    text: text.trim(),
    completed,
    lineNumber
  };
}

function generateId(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\w\s-]/g, '')
    .replace(/\s+/g, '-')
    .substring(0, 50);
}

function calculateProgress(sections: PlanSection[]): PlanProgress {
  let completed = 0;
  let total = 0;
  
  for (const section of sections) {
    for (const item of section.items) {
      total++;
      if (item.completed) {
        completed++;
      }
      
      if (item.subItems) {
        for (const subItem of item.subItems) {
          total++;
          if (subItem.completed) {
            completed++;
          }
        }
      }
    }
  }
  
  const percentage = total === 0 ? 0 : Math.round((completed / total) * 100);
  
  return {
    completed,
    total,
    percentage
  };
}

export function updateCheckboxInContent(content: string, lineNumber: number, checked: boolean): string {
  const lines = content.split('\n');
  
  if (lineNumber <= 0 || lineNumber > lines.length) {
    return content;
  }
  
  const line = lines[lineNumber - 1];
  const checkboxMatch = line.match(/^(\s*-\s+\[)([x\s])(\]\s+.+)$/);
  
  if (checkboxMatch) {
    const [, prefix, , suffix] = checkboxMatch;
    const newCheckState = checked ? 'x' : ' ';
    lines[lineNumber - 1] = `${prefix}${newCheckState}${suffix}`;
  }
  
  return lines.join('\n');
}