/**
 * Color utility functions for Tachikoma
 */

export interface RGB {
  r: number;
  g: number;
  b: number;
}

export interface HSL {
  h: number;
  s: number;
  l: number;
}

/**
 * Parse hex color to RGB
 */
export function hexToRgb(hex: string): RGB {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (!result) throw new Error(`Invalid hex color: ${hex}`);

  return {
    r: parseInt(result[1], 16),
    g: parseInt(result[2], 16),
    b: parseInt(result[3], 16)
  };
}

/**
 * Convert RGB to hex
 */
export function rgbToHex(rgb: RGB): string {
  const toHex = (n: number) => n.toString(16).padStart(2, '0');
  return `#${toHex(rgb.r)}${toHex(rgb.g)}${toHex(rgb.b)}`;
}

/**
 * Convert RGB to HSL
 */
export function rgbToHsl(rgb: RGB): HSL {
  const r = rgb.r / 255;
  const g = rgb.g / 255;
  const b = rgb.b / 255;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  let h = 0;
  let s = 0;
  const l = (max + min) / 2;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

    switch (max) {
      case r: h = ((g - b) / d + (g < b ? 6 : 0)) / 6; break;
      case g: h = ((b - r) / d + 2) / 6; break;
      case b: h = ((r - g) / d + 4) / 6; break;
    }
  }

  return {
    h: Math.round(h * 360),
    s: Math.round(s * 100),
    l: Math.round(l * 100)
  };
}

/**
 * Calculate relative luminance for WCAG contrast
 */
export function getLuminance(rgb: RGB): number {
  const [r, g, b] = [rgb.r, rgb.g, rgb.b].map(v => {
    v /= 255;
    return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/**
 * Calculate WCAG contrast ratio between two colors
 */
export function getContrastRatio(color1: string, color2: string): number {
  const l1 = getLuminance(hexToRgb(color1));
  const l2 = getLuminance(hexToRgb(color2));
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  return (lighter + 0.05) / (darker + 0.05);
}

/**
 * Check if contrast meets WCAG AA standard
 * Normal text: 4.5:1, Large text: 3:1
 */
export function meetsWcagAA(
  foreground: string,
  background: string,
  isLargeText: boolean = false
): boolean {
  const ratio = getContrastRatio(foreground, background);
  return isLargeText ? ratio >= 3 : ratio >= 4.5;
}

/**
 * Check if contrast meets WCAG AAA standard
 * Normal text: 7:1, Large text: 4.5:1
 */
export function meetsWcagAAA(
  foreground: string,
  background: string,
  isLargeText: boolean = false
): boolean {
  const ratio = getContrastRatio(foreground, background);
  return isLargeText ? ratio >= 4.5 : ratio >= 7;
}

/**
 * Generate color with alpha channel
 */
export function withAlpha(color: string, alpha: number): string {
  const rgb = hexToRgb(color);
  return `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alpha})`;
}

/**
 * Lighten a color
 */
export function lighten(color: string, amount: number): string {
  const rgb = hexToRgb(color);
  const hsl = rgbToHsl(rgb);
  hsl.l = Math.min(100, hsl.l + amount);
  return hslToHex(hsl);
}

/**
 * Darken a color
 */
export function darken(color: string, amount: number): string {
  const rgb = hexToRgb(color);
  const hsl = rgbToHsl(rgb);
  hsl.l = Math.max(0, hsl.l - amount);
  return hslToHex(hsl);
}

/**
 * Convert HSL to hex
 */
export function hslToHex(hsl: HSL): string {
  const h = hsl.h / 360;
  const s = hsl.s / 100;
  const l = hsl.l / 100;

  const hue2rgb = (p: number, q: number, t: number) => {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1/6) return p + (q - p) * 6 * t;
    if (t < 1/2) return q;
    if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
    return p;
  };

  let r, g, b;
  if (s === 0) {
    r = g = b = l;
  } else {
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r = hue2rgb(p, q, h + 1/3);
    g = hue2rgb(p, q, h);
    b = hue2rgb(p, q, h - 1/3);
  }

  return rgbToHex({
    r: Math.round(r * 255),
    g: Math.round(g * 255),
    b: Math.round(b * 255)
  });
}

/**
 * Get color from CSS variable
 */
export function getCSSVariable(variable: string): string {
  if (typeof document === 'undefined') return '';
  return getComputedStyle(document.documentElement)
    .getPropertyValue(variable)
    .trim();
}

/**
 * Set CSS variable
 */
export function setCSSVariable(variable: string, value: string): void {
  if (typeof document === 'undefined') return;
  document.documentElement.style.setProperty(variable, value);
}

/**
 * Generate accessible text color for given background
 */
export function getAccessibleTextColor(backgroundColor: string): string {
  const darkText = '#1f2328';
  const lightText = '#e6edf3';
  
  const darkContrast = getContrastRatio(darkText, backgroundColor);
  const lightContrast = getContrastRatio(lightText, backgroundColor);
  
  return darkContrast > lightContrast ? darkText : lightText;
}

/**
 * Check if color is considered dark
 */
export function isDark(color: string): boolean {
  const rgb = hexToRgb(color);
  const luminance = getLuminance(rgb);
  return luminance < 0.5;
}

/**
 * Check if color is considered light
 */
export function isLight(color: string): boolean {
  return !isDark(color);
}

/**
 * Mix two colors
 */
export function mix(color1: string, color2: string, weight: number = 0.5): string {
  const rgb1 = hexToRgb(color1);
  const rgb2 = hexToRgb(color2);
  
  const mixed: RGB = {
    r: Math.round(rgb1.r * (1 - weight) + rgb2.r * weight),
    g: Math.round(rgb1.g * (1 - weight) + rgb2.g * weight),
    b: Math.round(rgb1.b * (1 - weight) + rgb2.b * weight)
  };
  
  return rgbToHex(mixed);
}

/**
 * Generate color palette from base color
 */
export function generatePalette(baseColor: string): Record<string, string> {
  const base = hexToRgb(baseColor);
  const hsl = rgbToHsl(base);
  
  const palette: Record<string, string> = {};
  
  // Generate shades (50-950)
  const shades = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];
  shades.forEach(shade => {
    let lightness: number;
    if (shade === 500) {
      lightness = hsl.l;
    } else if (shade < 500) {
      // Lighter shades
      const factor = (500 - shade) / 450; // 0 to 1
      lightness = hsl.l + (95 - hsl.l) * factor;
    } else {
      // Darker shades
      const factor = (shade - 500) / 450; // 0 to 1
      lightness = hsl.l * (1 - factor * 0.9);
    }
    
    palette[shade.toString()] = hslToHex({
      h: hsl.h,
      s: shade === 50 ? hsl.s * 0.3 : hsl.s, // Reduce saturation for very light shade
      l: Math.max(0, Math.min(100, Math.round(lightness)))
    });
  });
  
  return palette;
}

// Tachikoma blue constants
export const TACHIKOMA_BLUE = '#00d4ff';
export const TACHIKOMA_BLUE_DARK = '#00a8cc';
export const TACHIKOMA_BLUE_LIGHT = '#4ddbff';

// Status color constants
export const SUCCESS_COLOR = '#22c55e';
export const WARNING_COLOR = '#f59e0b';
export const ERROR_COLOR = '#ef4444';
export const INFO_COLOR = '#3b82f6';

/**
 * Color blindness simulation
 */
export function simulateColorBlindness(color: string, type: 'protanopia' | 'deuteranopia' | 'tritanopia'): string {
  const rgb = hexToRgb(color);
  let r = rgb.r / 255;
  let g = rgb.g / 255;
  let b = rgb.b / 255;
  
  // Simplified color blindness simulation matrices
  switch (type) {
    case 'protanopia': // Red blind
      r = 0.567 * r + 0.433 * g;
      g = 0.558 * r + 0.442 * g;
      b = 0.242 * g + 0.758 * b;
      break;
    case 'deuteranopia': // Green blind
      r = 0.625 * r + 0.375 * g;
      g = 0.7 * r + 0.3 * g;
      b = 0.3 * g + 0.7 * b;
      break;
    case 'tritanopia': // Blue blind
      r = 0.95 * r + 0.05 * g;
      g = 0.433 * g + 0.567 * b;
      b = 0.475 * g + 0.525 * b;
      break;
  }
  
  return rgbToHex({
    r: Math.round(Math.max(0, Math.min(255, r * 255))),
    g: Math.round(Math.max(0, Math.min(255, g * 255))),
    b: Math.round(Math.max(0, Math.min(255, b * 255)))
  });
}

/**
 * Get all available theme colors
 */
export function getThemeColors(): Record<string, string> {
  if (typeof document === 'undefined') return {};
  
  const style = getComputedStyle(document.documentElement);
  const colors: Record<string, string> = {};
  
  // Common color variables to extract
  const colorVars = [
    '--tachikoma-500',
    '--color-bg-base',
    '--color-bg-surface',
    '--color-fg-default',
    '--color-fg-muted',
    '--color-success-fg',
    '--color-warning-fg',
    '--color-error-fg',
    '--color-info-fg',
    '--color-accent-fg'
  ];
  
  colorVars.forEach(varName => {
    const value = style.getPropertyValue(varName).trim();
    if (value) {
      colors[varName.replace('--', '')] = value;
    }
  });
  
  return colors;
}