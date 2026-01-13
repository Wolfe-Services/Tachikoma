import { describe, it, expect } from 'vitest';
import { parseImplementationPlan, updateCheckboxInContent } from '$lib/utils/impl-plan';

describe('parseImplementationPlan', () => {
  it('should parse acceptance criteria section', () => {
    const markdown = `# Test Spec

## Acceptance Criteria

- [ ] Parse implementation section from markdown
- [x] Display checkboxes with status
- [ ] Update checkbox state

## Other Section

Some other content.
`;

    const plan = parseImplementationPlan(markdown, 'test-spec');
    
    expect(plan).not.toBeNull();
    expect(plan!.specId).toBe('test-spec');
    expect(plan!.title).toBe('Test Spec');
    expect(plan!.sections).toHaveLength(1);
    
    const section = plan!.sections[0];
    expect(section.title).toBe('Acceptance Criteria');
    expect(section.items).toHaveLength(3);
    
    expect(section.items[0].text).toBe('Parse implementation section from markdown');
    expect(section.items[0].completed).toBe(false);
    
    expect(section.items[1].text).toBe('Display checkboxes with status');
    expect(section.items[1].completed).toBe(true);
    
    expect(section.items[2].text).toBe('Update checkbox state');
    expect(section.items[2].completed).toBe(false);
  });

  it('should parse implementation details sections', () => {
    const markdown = `# Test Spec

## Implementation Details

### 1. Types

- [ ] Create type definitions
- [x] Export interfaces

### 2. Components

- [ ] Create main component
- [ ] Add styling
`;

    const plan = parseImplementationPlan(markdown, 'test-spec');
    
    expect(plan).not.toBeNull();
    expect(plan!.sections).toHaveLength(2);
    
    expect(plan!.sections[0].title).toBe('1. Types');
    expect(plan!.sections[0].items).toHaveLength(2);
    
    expect(plan!.sections[1].title).toBe('2. Components');
    expect(plan!.sections[1].items).toHaveLength(2);
  });

  it('should calculate progress correctly', () => {
    const markdown = `# Test Spec

## Acceptance Criteria

- [ ] Task 1
- [x] Task 2
- [x] Task 3
- [ ] Task 4
`;

    const plan = parseImplementationPlan(markdown, 'test-spec');
    
    expect(plan).not.toBeNull();
    expect(plan!.progress.completed).toBe(2);
    expect(plan!.progress.total).toBe(4);
    expect(plan!.progress.percentage).toBe(50);
  });

  it('should return null for markdown without checklists', () => {
    const markdown = `# Test Spec

Just some regular content without any checkboxes.
`;

    const plan = parseImplementationPlan(markdown, 'test-spec');
    expect(plan).toBeNull();
  });
});

describe('updateCheckboxInContent', () => {
  it('should update checkbox state from unchecked to checked', () => {
    const content = `# Test

## Acceptance Criteria

- [ ] First task
- [x] Second task
- [ ] Third task
`;

    const updated = updateCheckboxInContent(content, 5, true);
    expect(updated).toContain('- [x] First task');
  });

  it('should update checkbox state from checked to unchecked', () => {
    const content = `# Test

## Acceptance Criteria

- [ ] First task
- [x] Second task
- [ ] Third task
`;

    const updated = updateCheckboxInContent(content, 6, false);
    expect(updated).toContain('- [ ] Second task');
  });

  it('should handle invalid line numbers gracefully', () => {
    const content = `# Test

- [ ] Task
`;

    const updated = updateCheckboxInContent(content, 999, true);
    expect(updated).toBe(content);
  });

  it('should only update actual checkbox lines', () => {
    const content = `# Test

Regular text line
- [ ] Checkbox line
More regular text
`;

    const updated = updateCheckboxInContent(content, 3, true);
    expect(updated).toBe(content); // Should not change non-checkbox lines
  });
});