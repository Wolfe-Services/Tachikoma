export interface FilterOption {
  value: string;
  label: string;
  icon?: string;
  color?: string;
  count?: number;
}

export interface FilterConfig {
  id: string;
  label: string;
  options: FilterOption[];
  searchable?: boolean;
  multi?: boolean;
}

export interface ActiveFilter {
  id: string;
  label: string;
  values: string[];
}