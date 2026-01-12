# Spec 199: Select Component

## Phase
Phase 9: UI Foundation

## Spec ID
199

## Status
Planned

## Dependencies
- Spec 196: Component Library Setup
- Spec 191-195: Design System

## Estimated Context
~10%

---

## Objective

Implement a customizable Select component for Tachikoma with support for single and multiple selection, search/filter functionality, custom option rendering, and full keyboard navigation and accessibility.

---

## Acceptance Criteria

- [ ] Single and multiple selection modes
- [ ] Searchable/filterable options
- [ ] Custom option rendering
- [ ] Grouped options support
- [ ] Clear selection functionality
- [ ] Loading state
- [ ] Disabled options
- [ ] Keyboard navigation (arrow keys, enter, escape)
- [ ] Screen reader accessible
- [ ] Portal-based dropdown positioning

---

## Implementation Details

### src/lib/components/ui/Select/Select.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher, tick } from 'svelte';
  import { fade, slide } from 'svelte/transition';
  import { generateId, cn, clickOutside, trapFocus } from '@utils/component';
  import { Keys } from '@utils/a11y';
  import Icon from '../Icon/Icon.svelte';
  import Spinner from '../Spinner/Spinner.svelte';

  export interface SelectOption {
    value: string;
    label: string;
    disabled?: boolean;
    group?: string;
    icon?: string;
    description?: string;
  }

  type SelectSize = 'sm' | 'md' | 'lg';

  export let options: SelectOption[] = [];
  export let value: string | string[] | null = null;
  export let multiple: boolean = false;
  export let size: SelectSize = 'md';
  export let label: string | undefined = undefined;
  export let placeholder: string = 'Select...';
  export let helperText: string | undefined = undefined;
  export let error: string | boolean = false;
  export let disabled: boolean = false;
  export let required: boolean = false;
  export let searchable: boolean = false;
  export let clearable: boolean = false;
  export let loading: boolean = false;
  export let maxHeight: number = 300;
  let className: string = '';
  export { className as class };

  const dispatch = createEventDispatcher<{
    change: string | string[] | null;
    search: string;
    open: void;
    close: void;
  }>();

  const id = generateId('select');
  let triggerElement: HTMLButtonElement;
  let listboxElement: HTMLUListElement;
  let searchInputElement: HTMLInputElement;
  let isOpen = false;
  let searchQuery = '';
  let highlightedIndex = -1;

  $: hasError = !!error;
  $: errorMessage = typeof error === 'string' ? error : undefined;

  // Filter options based on search
  $: filteredOptions = searchQuery
    ? options.filter(opt =>
        opt.label.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : options;

  // Group options
  $: groupedOptions = groupOptions(filteredOptions);

  // Get selected option(s)
  $: selectedOptions = multiple
    ? options.filter(opt => (value as string[])?.includes(opt.value))
    : options.find(opt => opt.value === value);

  // Display value
  $: displayValue = getDisplayValue(selectedOptions);

  function groupOptions(opts: SelectOption[]): Map<string | null, SelectOption[]> {
    const groups = new Map<string | null, SelectOption[]>();

    opts.forEach(opt => {
      const group = opt.group ?? null;
      if (!groups.has(group)) {
        groups.set(group, []);
      }
      groups.get(group)!.push(opt);
    });

    return groups;
  }

  function getDisplayValue(selected: SelectOption | SelectOption[] | undefined): string {
    if (!selected) return '';

    if (Array.isArray(selected)) {
      if (selected.length === 0) return '';
      if (selected.length === 1) return selected[0].label;
      return `${selected.length} selected`;
    }

    return selected.label;
  }

  async function toggleOpen() {
    if (disabled) return;

    isOpen = !isOpen;

    if (isOpen) {
      dispatch('open');
      await tick();

      if (searchable && searchInputElement) {
        searchInputElement.focus();
      }

      // Highlight current value
      if (value && !multiple) {
        const index = filteredOptions.findIndex(opt => opt.value === value);
        if (index >= 0) highlightedIndex = index;
      }
    } else {
      dispatch('close');
      searchQuery = '';
      highlightedIndex = -1;
      triggerElement?.focus();
    }
  }

  function close() {
    if (isOpen) {
      isOpen = false;
      dispatch('close');
      searchQuery = '';
      highlightedIndex = -1;
    }
  }

  function selectOption(option: SelectOption) {
    if (option.disabled) return;

    if (multiple) {
      const currentValue = (value as string[]) ?? [];
      const newValue = currentValue.includes(option.value)
        ? currentValue.filter(v => v !== option.value)
        : [...currentValue, option.value];

      value = newValue;
      dispatch('change', newValue);
    } else {
      value = option.value;
      dispatch('change', option.value);
      close();
    }
  }

  function clearSelection(event: MouseEvent) {
    event.stopPropagation();
    value = multiple ? [] : null;
    dispatch('change', value);
  }

  function handleSearch(event: Event) {
    searchQuery = (event.target as HTMLInputElement).value;
    highlightedIndex = 0;
    dispatch('search', searchQuery);
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (!isOpen) {
      if ([Keys.Enter, Keys.Space, Keys.ArrowDown, Keys.ArrowUp].includes(event.key as any)) {
        event.preventDefault();
        toggleOpen();
      }
      return;
    }

    const flatOptions = filteredOptions.filter(opt => !opt.disabled);

    switch (event.key) {
      case Keys.ArrowDown:
        event.preventDefault();
        highlightedIndex = Math.min(highlightedIndex + 1, flatOptions.length - 1);
        scrollToHighlighted();
        break;

      case Keys.ArrowUp:
        event.preventDefault();
        highlightedIndex = Math.max(highlightedIndex - 1, 0);
        scrollToHighlighted();
        break;

      case Keys.Enter:
      case Keys.Space:
        if (event.key === Keys.Space && searchable) return;
        event.preventDefault();
        if (highlightedIndex >= 0 && flatOptions[highlightedIndex]) {
          selectOption(flatOptions[highlightedIndex]);
        }
        break;

      case Keys.Escape:
        event.preventDefault();
        close();
        break;

      case Keys.Home:
        event.preventDefault();
        highlightedIndex = 0;
        scrollToHighlighted();
        break;

      case Keys.End:
        event.preventDefault();
        highlightedIndex = flatOptions.length - 1;
        scrollToHighlighted();
        break;
    }
  }

  function scrollToHighlighted() {
    tick().then(() => {
      const highlighted = listboxElement?.querySelector('[data-highlighted="true"]');
      highlighted?.scrollIntoView({ block: 'nearest' });
    });
  }

  function isSelected(option: SelectOption): boolean {
    if (multiple) {
      return (value as string[])?.includes(option.value) ?? false;
    }
    return value === option.value;
  }

  $: wrapperClasses = cn(
    'select-wrapper',
    `select-${size}`,
    isOpen && 'select-open',
    hasError && 'select-error',
    disabled && 'select-disabled',
    className
  );
</script>

<div class="select-field">
  {#if label}
    <label for={id} class="select-label">
      {label}
      {#if required}
        <span class="select-required" aria-hidden="true">*</span>
      {/if}
    </label>
  {/if}

  <div
    class={wrapperClasses}
    use:clickOutside={close}
  >
    <button
      bind:this={triggerElement}
      {id}
      type="button"
      class="select-trigger"
      {disabled}
      aria-haspopup="listbox"
      aria-expanded={isOpen}
      aria-labelledby={label ? `${id}-label` : undefined}
      aria-invalid={hasError}
      on:click={toggleOpen}
      on:keydown={handleKeyDown}
    >
      <span class="select-value" class:placeholder={!displayValue}>
        {displayValue || placeholder}
      </span>

      <span class="select-icons">
        {#if loading}
          <Spinner size={16} />
        {:else}
          {#if clearable && value && !disabled}
            <button
              type="button"
              class="select-clear"
              on:click={clearSelection}
              tabindex="-1"
              aria-label="Clear selection"
            >
              <Icon name="x" size={14} />
            </button>
          {/if}
          <Icon name="chevron-down" size={16} />
        {/if}
      </span>
    </button>

    {#if isOpen}
      <div
        class="select-dropdown"
        transition:slide={{ duration: 150 }}
        style="max-height: {maxHeight}px"
      >
        {#if searchable}
          <div class="select-search">
            <Icon name="search" size={16} />
            <input
              bind:this={searchInputElement}
              type="text"
              class="select-search-input"
              placeholder="Search..."
              value={searchQuery}
              on:input={handleSearch}
              on:keydown={handleKeyDown}
            />
          </div>
        {/if}

        <ul
          bind:this={listboxElement}
          class="select-listbox"
          role="listbox"
          aria-multiselectable={multiple}
          tabindex="-1"
        >
          {#if filteredOptions.length === 0}
            <li class="select-empty">
              {searchQuery ? 'No results found' : 'No options'}
            </li>
          {:else}
            {#each [...groupedOptions] as [groupName, groupOptions], groupIndex}
              {#if groupName}
                <li class="select-group-label">
                  {groupName}
                </li>
              {/if}

              {#each groupOptions as option, optionIndex}
                {@const flatIndex = filteredOptions.indexOf(option)}
                {@const isHighlighted = flatIndex === highlightedIndex}
                {@const selected = isSelected(option)}

                <li
                  role="option"
                  class="select-option"
                  class:highlighted={isHighlighted}
                  class:selected
                  class:disabled={option.disabled}
                  aria-selected={selected}
                  aria-disabled={option.disabled}
                  data-highlighted={isHighlighted}
                  on:click={() => selectOption(option)}
                  on:mouseenter={() => { if (!option.disabled) highlightedIndex = flatIndex; }}
                >
                  {#if multiple}
                    <span class="select-option-checkbox">
                      {#if selected}
                        <Icon name="check" size={14} />
                      {/if}
                    </span>
                  {/if}

                  {#if option.icon}
                    <Icon name={option.icon} size={16} />
                  {/if}

                  <span class="select-option-content">
                    <span class="select-option-label">{option.label}</span>
                    {#if option.description}
                      <span class="select-option-description">{option.description}</span>
                    {/if}
                  </span>

                  {#if !multiple && selected}
                    <Icon name="check" size={16} />
                  {/if}
                </li>
              {/each}
            {/each}
          {/if}
        </ul>
      </div>
    {/if}
  </div>

  {#if helperText || errorMessage}
    <div class="select-footer">
      {#if errorMessage}
        <span class="select-helper select-helper-error" role="alert">
          {errorMessage}
        </span>
      {:else if helperText}
        <span class="select-helper">
          {helperText}
        </span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .select-field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1-5);
    width: 100%;
  }

  .select-label {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-1);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-fg-default);
  }

  .select-required {
    color: var(--color-error-fg);
  }

  .select-wrapper {
    position: relative;
  }

  .select-trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    background-color: var(--color-bg-input);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    transition:
      border-color var(--duration-150) var(--ease-out),
      box-shadow var(--duration-150) var(--ease-out);
  }

  .select-trigger:hover:not(:disabled) {
    border-color: var(--color-border-strong);
  }

  .select-trigger:focus {
    outline: none;
    border-color: var(--tachikoma-500);
    box-shadow: var(--focus-ring);
  }

  .select-wrapper.select-error .select-trigger {
    border-color: var(--color-error-fg);
  }

  .select-wrapper.select-disabled .select-trigger {
    background-color: var(--color-bg-muted);
    cursor: not-allowed;
    opacity: 0.5;
  }

  /* Sizes */
  .select-sm .select-trigger {
    height: var(--spacing-8);
    padding: 0 var(--spacing-2-5);
    font-size: var(--text-sm);
  }

  .select-md .select-trigger {
    height: var(--spacing-10);
    padding: 0 var(--spacing-3);
    font-size: var(--text-sm);
  }

  .select-lg .select-trigger {
    height: var(--spacing-12);
    padding: 0 var(--spacing-4);
    font-size: var(--text-base);
  }

  .select-value {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--color-fg-default);
  }

  .select-value.placeholder {
    color: var(--color-fg-subtle);
  }

  .select-icons {
    display: flex;
    align-items: center;
    gap: var(--spacing-1);
    color: var(--color-fg-muted);
    flex-shrink: 0;
  }

  .select-wrapper.select-open .select-icons :global(svg:last-child) {
    transform: rotate(180deg);
  }

  .select-clear {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-1);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--color-fg-muted);
    cursor: pointer;
  }

  .select-clear:hover {
    color: var(--color-fg-default);
    background-color: var(--color-bg-hover);
  }

  /* Dropdown */
  .select-dropdown {
    position: absolute;
    top: calc(100% + var(--spacing-1));
    left: 0;
    right: 0;
    z-index: var(--z-dropdown);
    background-color: var(--color-bg-overlay);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    box-shadow: var(--dropdown-shadow);
    overflow: hidden;
  }

  .select-search {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2) var(--spacing-3);
    border-bottom: 1px solid var(--color-border-subtle);
    color: var(--color-fg-muted);
  }

  .select-search-input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--color-fg-default);
    font-size: var(--text-sm);
  }

  .select-search-input:focus {
    outline: none;
  }

  .select-search-input::placeholder {
    color: var(--color-fg-subtle);
  }

  .select-listbox {
    list-style: none;
    margin: 0;
    padding: var(--spacing-1);
    overflow-y: auto;
  }

  .select-empty {
    padding: var(--spacing-8) var(--spacing-3);
    text-align: center;
    color: var(--color-fg-muted);
    font-size: var(--text-sm);
  }

  .select-group-label {
    padding: var(--spacing-2) var(--spacing-3);
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: var(--color-fg-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .select-option {
    display: flex;
    align-items: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2) var(--spacing-3);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background-color var(--duration-75) var(--ease-out);
  }

  .select-option.highlighted {
    background-color: var(--color-bg-hover);
  }

  .select-option.selected {
    color: var(--tachikoma-500);
  }

  .select-option.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .select-option-checkbox {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    background-color: var(--color-bg-input);
  }

  .select-option.selected .select-option-checkbox {
    background-color: var(--tachikoma-500);
    border-color: var(--tachikoma-500);
    color: white;
  }

  .select-option-content {
    flex: 1;
    min-width: 0;
  }

  .select-option-label {
    display: block;
    font-size: var(--text-sm);
    color: var(--color-fg-default);
  }

  .select-option-description {
    display: block;
    font-size: var(--text-xs);
    color: var(--color-fg-muted);
    margin-top: var(--spacing-0-5);
  }

  .select-footer {
    min-height: var(--spacing-5);
  }

  .select-helper {
    font-size: var(--text-xs);
    color: var(--color-fg-muted);
  }

  .select-helper-error {
    color: var(--color-error-fg);
  }
</style>
```

---

## Testing Requirements

### Unit Tests

```typescript
// tests/components/Select.test.ts
import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import Select from '@components/ui/Select/Select.svelte';

const options = [
  { value: '1', label: 'Option 1' },
  { value: '2', label: 'Option 2' },
  { value: '3', label: 'Option 3', disabled: true }
];

describe('Select', () => {
  it('should render with placeholder', () => {
    const { getByRole } = render(Select, {
      props: { options, placeholder: 'Select an option' }
    });

    expect(getByRole('button')).toHaveTextContent('Select an option');
  });

  it('should open dropdown on click', async () => {
    const { getByRole, queryByRole } = render(Select, {
      props: { options }
    });

    expect(queryByRole('listbox')).not.toBeInTheDocument();

    await fireEvent.click(getByRole('button'));

    expect(queryByRole('listbox')).toBeInTheDocument();
  });

  it('should select option', async () => {
    const handleChange = vi.fn();
    const { getByRole, getByText, component } = render(Select, {
      props: { options }
    });

    component.$on('change', handleChange);

    await fireEvent.click(getByRole('button'));
    await fireEvent.click(getByText('Option 1'));

    expect(handleChange).toHaveBeenCalledWith(expect.objectContaining({
      detail: '1'
    }));
  });

  it('should handle keyboard navigation', async () => {
    const { getByRole } = render(Select, {
      props: { options }
    });

    const trigger = getByRole('button');
    await fireEvent.keyDown(trigger, { key: 'ArrowDown' });

    // Dropdown should be open
    expect(getByRole('listbox')).toBeInTheDocument();
  });

  it('should support multiple selection', async () => {
    const { getByRole, getByText, component } = render(Select, {
      props: { options, multiple: true, value: [] }
    });

    await fireEvent.click(getByRole('button'));
    await fireEvent.click(getByText('Option 1'));
    await fireEvent.click(getByText('Option 2'));

    expect(component.value).toEqual(['1', '2']);
  });

  it('should filter options when searchable', async () => {
    const { getByRole, getByPlaceholderText, queryByText } = render(Select, {
      props: { options, searchable: true }
    });

    await fireEvent.click(getByRole('button'));

    const searchInput = getByPlaceholderText('Search...');
    await fireEvent.input(searchInput, { target: { value: 'Option 1' } });

    expect(queryByText('Option 1')).toBeInTheDocument();
    expect(queryByText('Option 2')).not.toBeInTheDocument();
  });
});
```

---

## Related Specs

- [196-component-library.md](./196-component-library.md) - Component library setup
- [198-input-component.md](./198-input-component.md) - Input component
- [200-checkbox-toggle.md](./200-checkbox-toggle.md) - Checkbox/Toggle
