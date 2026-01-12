/**
 * Shared component type definitions
 */

// Size variants
export type Size = 'xs' | 'sm' | 'md' | 'lg' | 'xl';
export type ButtonSize = 'sm' | 'md' | 'lg';
export type InputSize = 'sm' | 'md' | 'lg';

// Color/variant types
export type ColorVariant = 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info';
export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'outline' | 'danger';
export type StatusVariant = 'success' | 'warning' | 'error' | 'info' | 'neutral';

// Common prop interfaces
export interface BaseProps {
  class?: string;
  id?: string;
  'data-testid'?: string;
}

export interface DisableableProps {
  disabled?: boolean;
}

export interface LoadingProps {
  loading?: boolean;
}

export interface SizeProps<T extends string = Size> {
  size?: T;
}

export interface VariantProps<T extends string = ColorVariant> {
  variant?: T;
}

// Form-related types
export interface FormFieldProps {
  name?: string;
  value?: string;
  required?: boolean;
  disabled?: boolean;
  readonly?: boolean;
}

export interface ValidationProps {
  error?: string | boolean;
  success?: boolean;
  helperText?: string;
}

// Event types
export interface ClickEvent extends MouseEvent {
  currentTarget: EventTarget & HTMLElement;
}

export interface ChangeEvent extends Event {
  currentTarget: EventTarget & HTMLInputElement;
}

export interface FocusEvent extends globalThis.FocusEvent {
  currentTarget: EventTarget & HTMLElement;
}

export interface KeyboardEvent extends globalThis.KeyboardEvent {
  currentTarget: EventTarget & HTMLElement;
}