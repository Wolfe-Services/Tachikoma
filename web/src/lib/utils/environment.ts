/**
 * Environment detection utilities
 */

export function isTauri(): boolean {
  return typeof window !== 'undefined' && 'tachikoma' in window;
}

export function isBrowser(): boolean {
  return typeof window !== 'undefined';
}

export function isDev(): boolean {
  return import.meta.env.DEV;
}