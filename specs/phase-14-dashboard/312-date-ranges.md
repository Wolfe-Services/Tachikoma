# 312 - Date Ranges

**Phase:** 14 - Dashboard
**Spec ID:** 312
**Status:** Planned
**Dependencies:** 296-dashboard-layout, 311-dashboard-filters
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Create date range picker components for filtering dashboard data by time periods, including preset ranges, custom date selection, and relative time options.

---

## Acceptance Criteria

- [x] `DateRangePicker.svelte` component created
- [x] Preset date ranges (today, week, month, etc.)
- [x] Custom date range selection
- [x] Calendar view for date picking
- [x] Relative date options (last N days)
- [x] Time zone handling
- [x] Keyboard navigation
- [x] Mobile-friendly interface

---

## Implementation Details

### 1. Date Range Picker Component (web/src/lib/components/dates/DateRangePicker.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fly, fade } from 'svelte/transition';
  import type { DateRange, DatePreset } from '$lib/types/dates';
  import Icon from '$lib/components/common/Icon.svelte';
  import Calendar from './Calendar.svelte';

  export let value: DateRange = {
    start: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000),
    end: new Date()
  };
  export let minDate: Date | null = null;
  export let maxDate: Date | null = null;
  export let showPresets: boolean = true;
  export let showTime: boolean = false;

  const dispatch = createEventDispatcher<{
    change: DateRange;
  }>();

  let open = false;
  let selecting: 'start' | 'end' = 'start';
  let tempStart: Date | null = null;
  let tempEnd: Date | null = null;

  const presets: DatePreset[] = [
    { id: 'today', label: 'Today', getValue: () => ({ start: startOfDay(new Date()), end: endOfDay(new Date()) }) },
    { id: 'yesterday', label: 'Yesterday', getValue: () => {
      const yesterday = new Date(Date.now() - 86400000);
      return { start: startOfDay(yesterday), end: endOfDay(yesterday) };
    }},
    { id: 'last7', label: 'Last 7 Days', getValue: () => ({
      start: startOfDay(new Date(Date.now() - 7 * 86400000)),
      end: endOfDay(new Date())
    })},
    { id: 'last30', label: 'Last 30 Days', getValue: () => ({
      start: startOfDay(new Date(Date.now() - 30 * 86400000)),
      end: endOfDay(new Date())
    })},
    { id: 'thisMonth', label: 'This Month', getValue: () => {
      const now = new Date();
      return {
        start: new Date(now.getFullYear(), now.getMonth(), 1),
        end: endOfDay(new Date())
      };
    }},
    { id: 'lastMonth', label: 'Last Month', getValue: () => {
      const now = new Date();
      return {
        start: new Date(now.getFullYear(), now.getMonth() - 1, 1),
        end: new Date(now.getFullYear(), now.getMonth(), 0, 23, 59, 59)
      };
    }},
    { id: 'thisYear', label: 'This Year', getValue: () => ({
      start: new Date(new Date().getFullYear(), 0, 1),
      end: endOfDay(new Date())
    })},
  ];

  function startOfDay(date: Date): Date {
    return new Date(date.getFullYear(), date.getMonth(), date.getDate(), 0, 0, 0, 0);
  }

  function endOfDay(date: Date): Date {
    return new Date(date.getFullYear(), date.getMonth(), date.getDate(), 23, 59, 59, 999);
  }

  function formatDate(date: Date): string {
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    });
  }

  function formatDateRange(range: DateRange): string {
    const start = formatDate(range.start);
    const end = formatDate(range.end);
    if (start === end) return start;
    return `${start} - ${end}`;
  }

  function applyPreset(preset: DatePreset) {
    value = preset.getValue();
    dispatch('change', value);
    open = false;
  }

  function handleDateSelect(date: Date) {
    if (selecting === 'start') {
      tempStart = date;
      selecting = 'end';
    } else {
      tempEnd = date;
      if (tempStart && tempEnd) {
        const [start, end] = [tempStart, tempEnd].sort((a, b) => a.getTime() - b.getTime());
        value = { start: startOfDay(start), end: endOfDay(end) };
        dispatch('change', value);
      }
      selecting = 'start';
      tempStart = null;
      tempEnd = null;
    }
  }

  function handleApply() {
    open = false;
  }

  function handleClear() {
    tempStart = null;
    tempEnd = null;
    selecting = 'start';
  }

  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.date-range-picker')) {
      open = false;
    }
  }

  $: selectedPreset = presets.find(p => {
    const preset = p.getValue();
    return preset.start.getTime() === value.start.getTime() &&
           preset.end.getTime() === value.end.getTime();
  });
</script>

<svelte:window on:click={handleClickOutside} />

<div class="date-range-picker" class:open>
  <button
    class="picker-trigger"
    on:click|stopPropagation={() => open = !open}
    aria-expanded={open}
  >
    <Icon name="calendar" size={16} />
    <span class="trigger-text">
      {selectedPreset?.label || formatDateRange(value)}
    </span>
    <Icon name="chevron-down" size={14} />
  </button>

  {#if open}
    <div
      class="picker-dropdown"
      on:click|stopPropagation
      transition:fly={{ y: -10, duration: 150 }}
    >
      <div class="dropdown-layout">
        {#if showPresets}
          <div class="presets-panel">
            <h4>Quick Select</h4>
            <ul class="preset-list">
              {#each presets as preset}
                <li>
                  <button
                    class="preset-btn"
                    class:active={selectedPreset?.id === preset.id}
                    on:click={() => applyPreset(preset)}
                  >
                    {preset.label}
                  </button>
                </li>
              {/each}
            </ul>
          </div>
        {/if}

        <div class="calendar-panel">
          <div class="calendar-header">
            <div class="date-inputs">
              <div class="input-group">
                <label>Start Date</label>
                <input
                  type="text"
                  value={tempStart ? formatDate(tempStart) : formatDate(value.start)}
                  readonly
                  class:selecting={selecting === 'start'}
                  on:click={() => selecting = 'start'}
                />
              </div>
              <span class="input-separator">to</span>
              <div class="input-group">
                <label>End Date</label>
                <input
                  type="text"
                  value={tempEnd ? formatDate(tempEnd) : formatDate(value.end)}
                  readonly
                  class:selecting={selecting === 'end'}
                  on:click={() => selecting = 'end'}
                />
              </div>
            </div>
          </div>

          <div class="calendars">
            <Calendar
              selectedStart={tempStart || value.start}
              selectedEnd={tempEnd || value.end}
              {selecting}
              {minDate}
              {maxDate}
              on:select={(e) => handleDateSelect(e.detail)}
            />
          </div>

          {#if showTime}
            <div class="time-inputs">
              <div class="input-group">
                <label>Start Time</label>
                <input type="time" value="00:00" />
              </div>
              <div class="input-group">
                <label>End Time</label>
                <input type="time" value="23:59" />
              </div>
            </div>
          {/if}

          <div class="calendar-footer">
            <button class="btn btn-text" on:click={handleClear}>
              Clear
            </button>
            <button class="btn btn-primary" on:click={handleApply}>
              Apply
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .date-range-picker {
    position: relative;
    display: inline-block;
  }

  .picker-trigger {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color);
    background: var(--bg-primary);
    border-radius: 0.5rem;
    font-size: 0.875rem;
    color: var(--text-primary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .picker-trigger:hover {
    border-color: var(--border-hover);
  }

  .date-range-picker.open .picker-trigger {
    border-color: var(--accent-color);
  }

  .picker-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 0.5rem;
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 0.75rem;
    box-shadow: var(--shadow-xl);
    z-index: 1000;
    overflow: hidden;
  }

  .dropdown-layout {
    display: flex;
  }

  .presets-panel {
    width: 150px;
    padding: 1rem;
    border-right: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  .presets-panel h4 {
    margin: 0 0 0.75rem;
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-tertiary);
  }

  .preset-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .preset-btn {
    display: block;
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: none;
    background: transparent;
    text-align: left;
    font-size: 0.8125rem;
    color: var(--text-primary);
    border-radius: 0.375rem;
    cursor: pointer;
  }

  .preset-btn:hover {
    background: var(--bg-hover);
  }

  .preset-btn.active {
    background: var(--accent-color);
    color: white;
  }

  .calendar-panel {
    padding: 1rem;
  }

  .calendar-header {
    margin-bottom: 1rem;
  }

  .date-inputs {
    display: flex;
    align-items: flex-end;
    gap: 0.75rem;
  }

  .input-group {
    flex: 1;
  }

  .input-group label {
    display: block;
    margin-bottom: 0.25rem;
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--text-tertiary);
  }

  .input-group input {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 0.375rem;
    font-size: 0.8125rem;
    color: var(--text-primary);
    background: var(--bg-primary);
  }

  .input-group input.selecting {
    border-color: var(--accent-color);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
  }

  .input-separator {
    padding-bottom: 0.5rem;
    font-size: 0.8125rem;
    color: var(--text-tertiary);
  }

  .time-inputs {
    display: flex;
    gap: 0.75rem;
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color);
  }

  .time-inputs input[type="time"] {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 0.375rem;
    font-size: 0.8125rem;
  }

  .calendar-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color);
  }

  .btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.8125rem;
    font-weight: 500;
    cursor: pointer;
  }

  .btn-text {
    background: transparent;
    color: var(--text-secondary);
  }

  .btn-text:hover {
    color: var(--text-primary);
  }

  .btn-primary {
    background: var(--accent-color);
    color: white;
  }

  .btn-primary:hover {
    opacity: 0.9;
  }

  @media (max-width: 640px) {
    .dropdown-layout {
      flex-direction: column;
    }

    .presets-panel {
      width: 100%;
      border-right: none;
      border-bottom: 1px solid var(--border-color);
    }

    .preset-list {
      display: flex;
      flex-wrap: wrap;
      gap: 0.25rem;
    }

    .preset-btn {
      width: auto;
    }
  }
</style>
```

### 2. Calendar Component (web/src/lib/components/dates/Calendar.svelte)

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Icon from '$lib/components/common/Icon.svelte';

  export let selectedStart: Date;
  export let selectedEnd: Date;
  export let selecting: 'start' | 'end' = 'start';
  export let minDate: Date | null = null;
  export let maxDate: Date | null = null;

  const dispatch = createEventDispatcher<{
    select: Date;
  }>();

  let viewDate = new Date(selectedStart);

  $: year = viewDate.getFullYear();
  $: month = viewDate.getMonth();
  $: daysInMonth = new Date(year, month + 1, 0).getDate();
  $: firstDayOfMonth = new Date(year, month, 1).getDay();
  $: days = generateDays();

  const weekDays = ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'];
  const monthNames = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December'
  ];

  function generateDays() {
    const days: Array<{ date: Date | null; isCurrentMonth: boolean }> = [];

    // Previous month days
    for (let i = 0; i < firstDayOfMonth; i++) {
      const date = new Date(year, month, -firstDayOfMonth + i + 1);
      days.push({ date, isCurrentMonth: false });
    }

    // Current month days
    for (let i = 1; i <= daysInMonth; i++) {
      days.push({ date: new Date(year, month, i), isCurrentMonth: true });
    }

    // Next month days
    const remaining = 42 - days.length;
    for (let i = 1; i <= remaining; i++) {
      const date = new Date(year, month + 1, i);
      days.push({ date, isCurrentMonth: false });
    }

    return days;
  }

  function prevMonth() {
    viewDate = new Date(year, month - 1, 1);
  }

  function nextMonth() {
    viewDate = new Date(year, month + 1, 1);
  }

  function isSelected(date: Date): boolean {
    return isSameDay(date, selectedStart) || isSameDay(date, selectedEnd);
  }

  function isInRange(date: Date): boolean {
    return date > selectedStart && date < selectedEnd;
  }

  function isSameDay(a: Date, b: Date): boolean {
    return a.getFullYear() === b.getFullYear() &&
           a.getMonth() === b.getMonth() &&
           a.getDate() === b.getDate();
  }

  function isDisabled(date: Date): boolean {
    if (minDate && date < minDate) return true;
    if (maxDate && date > maxDate) return true;
    return false;
  }

  function isToday(date: Date): boolean {
    return isSameDay(date, new Date());
  }

  function handleSelect(date: Date) {
    if (!isDisabled(date)) {
      dispatch('select', date);
    }
  }

  function handleKeydown(event: KeyboardEvent, date: Date) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleSelect(date);
    }
  }
</script>

<div class="calendar">
  <div class="calendar-nav">
    <button class="nav-btn" on:click={prevMonth} aria-label="Previous month">
      <Icon name="chevron-left" size={16} />
    </button>
    <span class="nav-title">{monthNames[month]} {year}</span>
    <button class="nav-btn" on:click={nextMonth} aria-label="Next month">
      <Icon name="chevron-right" size={16} />
    </button>
  </div>

  <div class="calendar-grid">
    <div class="weekdays">
      {#each weekDays as day}
        <div class="weekday">{day}</div>
      {/each}
    </div>

    <div class="days">
      {#each days as { date, isCurrentMonth }}
        {#if date}
          <button
            class="day"
            class:other-month={!isCurrentMonth}
            class:selected={isSelected(date)}
            class:in-range={isInRange(date)}
            class:today={isToday(date)}
            class:disabled={isDisabled(date)}
            disabled={isDisabled(date)}
            on:click={() => handleSelect(date)}
            on:keydown={(e) => handleKeydown(e, date)}
            tabindex={isCurrentMonth ? 0 : -1}
          >
            {date.getDate()}
          </button>
        {:else}
          <div class="day empty" />
        {/if}
      {/each}
    </div>
  </div>
</div>

<style>
  .calendar {
    width: 280px;
  }

  .calendar-nav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.75rem;
  }

  .nav-btn {
    padding: 0.375rem;
    border: none;
    background: transparent;
    border-radius: 0.375rem;
    cursor: pointer;
    color: var(--text-secondary);
  }

  .nav-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .nav-title {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .weekdays {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    margin-bottom: 0.25rem;
  }

  .weekday {
    padding: 0.5rem;
    text-align: center;
    font-size: 0.6875rem;
    font-weight: 600;
    color: var(--text-tertiary);
  }

  .days {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 2px;
  }

  .day {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border: none;
    background: transparent;
    border-radius: 50%;
    font-size: 0.8125rem;
    color: var(--text-primary);
    cursor: pointer;
    transition: all 0.1s ease;
  }

  .day:hover:not(.disabled):not(.selected) {
    background: var(--bg-hover);
  }

  .day.other-month {
    color: var(--text-tertiary);
  }

  .day.selected {
    background: var(--accent-color);
    color: white;
  }

  .day.in-range {
    background: var(--accent-color-light, rgba(59, 130, 246, 0.1));
    border-radius: 0;
  }

  .day.today:not(.selected) {
    border: 1px solid var(--accent-color);
  }

  .day.disabled {
    color: var(--text-tertiary);
    opacity: 0.5;
    cursor: not-allowed;
  }

  .day.empty {
    cursor: default;
  }
</style>
```

### 3. Date Types (web/src/lib/types/dates.ts)

```typescript
export interface DateRange {
  start: Date;
  end: Date;
}

export interface DatePreset {
  id: string;
  label: string;
  getValue: () => DateRange;
}
```

---

## Testing Requirements

1. Preset selection works correctly
2. Custom date range selection works
3. Calendar navigation works
4. Date validation respects min/max
5. Range highlighting displays correctly
6. Keyboard navigation functions
7. Mobile layout is usable

---

## Related Specs

- Depends on: [296-dashboard-layout.md](296-dashboard-layout.md)
- Next: [313-dashboard-refresh.md](313-dashboard-refresh.md)
- Used by: Filter bars, export dialogs
