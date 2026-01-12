import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';
import { createPersistedStore } from './persistedStore';

export type Theme = 'dark' | 'light' | 'system';
export type ResolvedTheme = 'dark' | 'light';
export type AccessibilityMode = 'default' | 'colorblind' | 'high-contrast';

interface ThemeState {
  theme: Theme;
  resolved: ResolvedTheme;
  accessibilityMode: AccessibilityMode;
}

function getSystemTheme(): ResolvedTheme {
  if (!browser) return 'dark';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function resolveTheme(theme: Theme): ResolvedTheme {
  if (theme === 'system') {
    return getSystemTheme();
  }
  return theme;
}

function createThemeStore() {
  const storedTheme = createPersistedStore<Theme>('dark', { key: 'theme' });
  const storedAccessibility = createPersistedStore<AccessibilityMode>('default', { 
    key: 'accessibility-mode' 
  });
  
  let currentTheme: Theme = 'dark';
  let currentAccessibility: AccessibilityMode = 'default';

  storedTheme.subscribe(t => { currentTheme = t; });
  storedAccessibility.subscribe(a => { currentAccessibility = a; });

  const state = writable<ThemeState>({
    theme: currentTheme,
    resolved: resolveTheme(currentTheme),
    accessibilityMode: currentAccessibility
  });

  // Sync persisted stores with state
  storedTheme.subscribe(theme => {
    state.update(s => ({
      ...s,
      theme,
      resolved: resolveTheme(theme)
    }));
  });

  storedAccessibility.subscribe(accessibilityMode => {
    state.update(s => ({
      ...s,
      accessibilityMode
    }));
  });

  // Watch for system theme changes
  if (browser) {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaQuery.addEventListener('change', () => {
      state.update(s => {
        if (s.theme === 'system') {
          return { ...s, resolved: getSystemTheme() };
        }
        return s;
      });
    });

    // Watch for system accessibility preferences
    const highContrastQuery = window.matchMedia('(prefers-contrast: high)');
    highContrastQuery.addEventListener('change', (e) => {
      if (e.matches && currentAccessibility === 'default') {
        setAccessibilityMode('high-contrast');
      }
    });

    // Apply initial high contrast if system prefers it
    if (highContrastQuery.matches && currentAccessibility === 'default') {
      setAccessibilityMode('high-contrast');
    }
  }

  // Apply theme and accessibility to document
  if (browser) {
    state.subscribe(({ resolved, accessibilityMode }) => {
      document.documentElement.setAttribute('data-theme', resolved);
      document.documentElement.setAttribute('data-accessibility', accessibilityMode);
    });
  }

  function setAccessibilityMode(mode: AccessibilityMode) {
    storedAccessibility.set(mode);
  }

  return {
    subscribe: state.subscribe,

    setTheme: (theme: Theme) => {
      storedTheme.set(theme);
    },

    setAccessibilityMode,

    toggle: () => {
      state.update(s => {
        const newTheme: Theme = s.resolved === 'dark' ? 'light' : 'dark';
        storedTheme.set(newTheme);
        return {
          ...s,
          theme: newTheme,
          resolved: newTheme
        };
      });
    },

    // Get computed theme colors
    getColors: () => {
      if (!browser) return {};
      
      const style = getComputedStyle(document.documentElement);
      return {
        // Brand
        primary: style.getPropertyValue('--color-accent-fg').trim(),
        primaryHover: style.getPropertyValue('--color-accent-emphasis').trim(),
        
        // Backgrounds
        bgBase: style.getPropertyValue('--color-bg-base').trim(),
        bgSurface: style.getPropertyValue('--color-bg-surface').trim(),
        bgElevated: style.getPropertyValue('--color-bg-elevated').trim(),
        bgHover: style.getPropertyValue('--color-bg-hover').trim(),
        
        // Text
        textDefault: style.getPropertyValue('--color-fg-default').trim(),
        textMuted: style.getPropertyValue('--color-fg-muted').trim(),
        textSubtle: style.getPropertyValue('--color-fg-subtle').trim(),
        
        // Borders
        border: style.getPropertyValue('--color-border-default').trim(),
        borderSubtle: style.getPropertyValue('--color-border-subtle').trim(),
        
        // Status
        success: style.getPropertyValue('--color-success-fg').trim(),
        warning: style.getPropertyValue('--color-warning-fg').trim(),
        error: style.getPropertyValue('--color-error-fg').trim(),
        info: style.getPropertyValue('--color-info-fg').trim(),
      };
    }
  };
}

export const themeStore = createThemeStore();
export const currentTheme = derived(themeStore, $theme => $theme.theme);
export const resolvedTheme = derived(themeStore, $theme => $theme.resolved);
export const isDarkMode = derived(themeStore, $theme => $theme.resolved === 'dark');
export const accessibilityMode = derived(themeStore, $theme => $theme.accessibilityMode);

// Convenience functions
export const { setTheme, toggle: toggleTheme, setAccessibilityMode, getColors } = themeStore;