export interface DateRange {
  start: Date;
  end: Date;
}

export interface DatePreset {
  id: string;
  label: string;
  getValue: () => DateRange;
}