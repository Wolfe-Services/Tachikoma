/**
 * Theme Management System
 * Provides theme switching capabilities and persistence
 */

import { browser } from '$app/environment';
import { writable, derived } from 'svelte/store';

export type Theme = 'light' | 'dark' | 'system';
export type ResolvedTheme = 'light' | 'dark';

// Store the user's theme preference
export const theme = writable<Theme>('system');

// Store the resolved theme (handles 'system' preference)
export const resolvedTheme = writable<ResolvedTheme>('dark');

// Store system theme preference
export const systemTheme = writable<ResolvedTheme>('dark');

// Derived store that determines if we're in dark mode
export const isDarkMode = derived(
  resolvedTheme,
  ($resolvedTheme) => $resolvedTheme === 'dark'
);

/**
 * Initialize theme system
 */
export function initializeTheme(): void {
  if (!browser) return;

  // Get system preference
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  systemTheme.set(mediaQuery.matches ? 'dark' : 'light');

  // Listen for system theme changes
  mediaQuery.addEventListener('change', (e) => {
    systemTheme.set(e.matches ? 'dark' : 'light');
    updateResolvedTheme();
  });

  // Load saved theme or use system
  const savedTheme = localStorage.getItem('tachikoma-theme') as Theme;
  if (savedTheme && ['light', 'dark', 'system'].includes(savedTheme)) {
    theme.set(savedTheme);
  }

  // Subscribe to theme changes
  theme.subscribe(($theme) => {
    if (browser) {
      localStorage.setItem('tachikoma-theme', $theme);
      updateResolvedTheme();
    }
  });

  // Initial resolution
  updateResolvedTheme();
}

/**
 * Update resolved theme based on current theme and system preference
 */
function updateResolvedTheme(): void {
  if (!browser) return;

  let currentTheme: Theme;
  theme.subscribe(($theme) => {
    currentTheme = $theme;
  })();

  let currentSystemTheme: ResolvedTheme;
  systemTheme.subscribe(($systemTheme) => {
    currentSystemTheme = $systemTheme;
  })();

  const resolved = currentTheme === 'system' ? currentSystemTheme : currentTheme as ResolvedTheme;
  resolvedTheme.set(resolved);

  // Apply theme to document
  document.documentElement.setAttribute('data-theme', resolved);
}

/**
 * Set theme preference
 */
export function setTheme(newTheme: Theme): void {
  theme.set(newTheme);
}

/**
 * Toggle between light and dark themes
 */
export function toggleTheme(): void {
  theme.update(($theme) => {
    if ($theme === 'light') return 'dark';
    if ($theme === 'dark') return 'light';
    // If system, toggle to opposite of current system preference
    let currentSystemTheme: ResolvedTheme;
    systemTheme.subscribe(($systemTheme) => {
      currentSystemTheme = $systemTheme;
    })();
    return currentSystemTheme === 'dark' ? 'light' : 'dark';
  });
}

/**
 * Get current theme values for use in components
 */
export function getThemeValues() {
  if (!browser) return {};

  const computedStyle = getComputedStyle(document.documentElement);
  
  return {
    // Colors
    primary: computedStyle.getPropertyValue('--color-primary').trim(),
    bgBase: computedStyle.getPropertyValue('--color-bg-base').trim(),
    bgSurface: computedStyle.getPropertyValue('--color-bg-surface').trim(),
    textPrimary: computedStyle.getPropertyValue('--color-text-primary').trim(),
    textSecondary: computedStyle.getPropertyValue('--color-text-secondary').trim(),
    border: computedStyle.getPropertyValue('--color-border').trim(),
    
    // Status colors
    success: computedStyle.getPropertyValue('--color-success').trim(),
    warning: computedStyle.getPropertyValue('--color-warning').trim(),
    error: computedStyle.getPropertyValue('--color-error').trim(),
    info: computedStyle.getPropertyValue('--color-info').trim(),
  };
}