import { describe, it, expect } from 'vitest';
import {
  hexToRgb,
  rgbToHex,
  rgbToHsl,
  hslToHex,
  getLuminance,
  getContrastRatio,
  meetsWcagAA,
  meetsWcagAAA,
  withAlpha,
  lighten,
  darken,
  getAccessibleTextColor,
  isDark,
  isLight,
  mix,
  generatePalette,
  simulateColorBlindness,
  TACHIKOMA_BLUE,
  TACHIKOMA_BLUE_DARK,
  TACHIKOMA_BLUE_LIGHT,
  SUCCESS_COLOR,
  WARNING_COLOR,
  ERROR_COLOR,
  INFO_COLOR
} from './colors';

describe('Color Utilities', () => {
  describe('Color Conversion', () => {
    it('should convert hex to RGB', () => {
      expect(hexToRgb('#00d4ff')).toEqual({ r: 0, g: 212, b: 255 });
      expect(hexToRgb('#ffffff')).toEqual({ r: 255, g: 255, b: 255 });
      expect(hexToRgb('#000000')).toEqual({ r: 0, g: 0, b: 0 });
      expect(hexToRgb('00d4ff')).toEqual({ r: 0, g: 212, b: 255 });
    });

    it('should convert RGB to hex', () => {
      expect(rgbToHex({ r: 0, g: 212, b: 255 })).toBe('#00d4ff');
      expect(rgbToHex({ r: 255, g: 255, b: 255 })).toBe('#ffffff');
      expect(rgbToHex({ r: 0, g: 0, b: 0 })).toBe('#000000');
    });

    it('should convert RGB to HSL', () => {
      const hsl = rgbToHsl({ r: 0, g: 212, b: 255 });
      expect(hsl.h).toBeCloseTo(190, 0);
      expect(hsl.s).toBe(100);
      expect(hsl.l).toBe(50);
    });

    it('should convert HSL to hex', () => {
      expect(hslToHex({ h: 190, s: 100, l: 50 })).toBe('#00d4ff');
      expect(hslToHex({ h: 0, s: 0, l: 100 })).toBe('#ffffff');
      expect(hslToHex({ h: 0, s: 0, l: 0 })).toBe('#000000');
    });

    it('should handle invalid hex colors', () => {
      expect(() => hexToRgb('#invalid')).toThrow('Invalid hex color: #invalid');
      expect(() => hexToRgb('xyz')).toThrow('Invalid hex color: xyz');
    });
  });

  describe('WCAG Compliance', () => {
    it('should calculate luminance correctly', () => {
      const whiteLuminance = getLuminance({ r: 255, g: 255, b: 255 });
      const blackLuminance = getLuminance({ r: 0, g: 0, b: 0 });
      expect(whiteLuminance).toBeCloseTo(1, 2);
      expect(blackLuminance).toBeCloseTo(0, 2);
    });

    it('should calculate contrast ratio', () => {
      const ratio = getContrastRatio('#ffffff', '#000000');
      expect(ratio).toBeCloseTo(21, 0);
      
      const tachikoma = getContrastRatio(TACHIKOMA_BLUE, '#000000');
      expect(tachikoma).toBeGreaterThan(1);
    });

    it('should check WCAG AA compliance', () => {
      expect(meetsWcagAA('#000000', '#ffffff')).toBe(true);
      expect(meetsWcagAA('#ffffff', '#000000')).toBe(true);
      expect(meetsWcagAA('#777777', '#ffffff')).toBe(false);
      
      // Large text has lower requirement (3:1)
      expect(meetsWcagAA('#777777', '#ffffff', true)).toBe(true);
    });

    it('should check WCAG AAA compliance', () => {
      expect(meetsWcagAAA('#000000', '#ffffff')).toBe(true);
      expect(meetsWcagAAA('#666666', '#ffffff')).toBe(false);
      
      // Large text has lower requirement (4.5:1)
      expect(meetsWcagAAA('#777777', '#ffffff', true)).toBe(false);
    });
  });

  describe('Color Manipulation', () => {
    it('should generate color with alpha', () => {
      expect(withAlpha('#00d4ff', 0.5)).toBe('rgba(0, 212, 255, 0.5)');
      expect(withAlpha('#ffffff', 0)).toBe('rgba(255, 255, 255, 0)');
      expect(withAlpha('#000000', 1)).toBe('rgba(0, 0, 0, 1)');
    });

    it('should lighten colors', () => {
      const lightened = lighten('#00d4ff', 20);
      const originalHsl = rgbToHsl(hexToRgb('#00d4ff'));
      const lightenedHsl = rgbToHsl(hexToRgb(lightened));
      
      expect(lightenedHsl.l).toBeGreaterThan(originalHsl.l);
    });

    it('should darken colors', () => {
      const darkened = darken('#00d4ff', 20);
      const originalHsl = rgbToHsl(hexToRgb('#00d4ff'));
      const darkenedHsl = rgbToHsl(hexToRgb(darkened));
      
      expect(darkenedHsl.l).toBeLessThan(originalHsl.l);
    });

    it('should mix colors', () => {
      const mixed = mix('#ff0000', '#0000ff', 0.5);
      expect(mixed).toBe('#800080'); // Purple
      
      const weighted = mix('#ffffff', '#000000', 0.25);
      const weightedRgb = hexToRgb(weighted);
      expect(weightedRgb.r).toBe(191); // Closer to white
    });
  });

  describe('Color Analysis', () => {
    it('should identify dark colors', () => {
      expect(isDark('#000000')).toBe(true);
      expect(isDark('#ffffff')).toBe(false);
      expect(isDark(TACHIKOMA_BLUE)).toBe(false); // Bright blue
    });

    it('should identify light colors', () => {
      expect(isLight('#ffffff')).toBe(true);
      expect(isLight('#000000')).toBe(false);
      expect(isLight('#f0f0f0')).toBe(true);
    });

    it('should get accessible text color', () => {
      expect(getAccessibleTextColor('#ffffff')).toBe('#1f2328'); // Dark text on light bg
      expect(getAccessibleTextColor('#000000')).toBe('#e6edf3'); // Light text on dark bg
    });
  });

  describe('Palette Generation', () => {
    it('should generate color palette', () => {
      const palette = generatePalette(TACHIKOMA_BLUE);
      
      expect(palette['500']).toBe(TACHIKOMA_BLUE);
      expect(Object.keys(palette)).toHaveLength(11); // 50-950
      
      // Should be ordered from light to dark
      const lightness50 = rgbToHsl(hexToRgb(palette['50'])).l;
      const lightness900 = rgbToHsl(hexToRgb(palette['900'])).l;
      expect(lightness50).toBeGreaterThan(lightness900);
    });
  });

  describe('Color Blindness Simulation', () => {
    it('should simulate protanopia', () => {
      const simulated = simulateColorBlindness('#ff0000', 'protanopia');
      expect(simulated).toMatch(/^#[0-9a-fA-F]{6}$/);
      expect(simulated).not.toBe('#ff0000'); // Should be different
    });

    it('should simulate deuteranopia', () => {
      const simulated = simulateColorBlindness('#00ff00', 'deuteranopia');
      expect(simulated).toMatch(/^#[0-9a-fA-F]{6}$/);
      expect(simulated).not.toBe('#00ff00');
    });

    it('should simulate tritanopia', () => {
      const simulated = simulateColorBlindness('#0000ff', 'tritanopia');
      expect(simulated).toMatch(/^#[0-9a-fA-F]{6}$/);
      expect(simulated).not.toBe('#0000ff');
    });
  });

  describe('Color Constants', () => {
    it('should have correct Tachikoma blue constants', () => {
      expect(TACHIKOMA_BLUE).toBe('#00d4ff');
      expect(TACHIKOMA_BLUE_DARK).toBe('#00a8cc');
      expect(TACHIKOMA_BLUE_LIGHT).toBe('#4ddbff');
    });

    it('should have status color constants', () => {
      expect(SUCCESS_COLOR).toBe('#22c55e');
      expect(WARNING_COLOR).toBe('#f59e0b');
      expect(ERROR_COLOR).toBe('#ef4444');
      expect(INFO_COLOR).toBe('#3b82f6');
    });

    it('should have valid hex format for all constants', () => {
      const colors = [
        TACHIKOMA_BLUE,
        TACHIKOMA_BLUE_DARK,
        TACHIKOMA_BLUE_LIGHT,
        SUCCESS_COLOR,
        WARNING_COLOR,
        ERROR_COLOR,
        INFO_COLOR
      ];
      
      colors.forEach(color => {
        expect(color).toMatch(/^#[0-9a-fA-F]{6}$/);
      });
    });
  });

  describe('Edge Cases', () => {
    it('should handle extreme lightening/darkening', () => {
      const maxLightened = lighten('#000000', 100);
      const maxDarkened = darken('#ffffff', 100);
      
      expect(rgbToHsl(hexToRgb(maxLightened)).l).toBe(100);
      expect(rgbToHsl(hexToRgb(maxDarkened)).l).toBe(0);
    });

    it('should handle alpha bounds', () => {
      expect(withAlpha('#ff0000', -1)).toBe('rgba(255, 0, 0, -1)'); // No clamping in function
      expect(withAlpha('#ff0000', 2)).toBe('rgba(255, 0, 0, 2)');
    });

    it('should handle grayscale colors', () => {
      const grayHsl = rgbToHsl({ r: 128, g: 128, b: 128 });
      expect(grayHsl.s).toBe(0);
      
      const grayHex = hslToHex({ h: 0, s: 0, l: 50 });
      expect(grayHex).toBe('#808080');
    });
  });

  describe('Consistency Tests', () => {
    it('should maintain consistency in hex-RGB-hex conversion', () => {
      const originalHex = '#a3b5c7';
      const rgb = hexToRgb(originalHex);
      const convertedHex = rgbToHex(rgb);
      expect(convertedHex).toBe(originalHex);
    });

    it('should maintain consistency in RGB-HSL-RGB conversion', () => {
      const originalRgb = { r: 163, g: 181, b: 199 };
      const hsl = rgbToHsl(originalRgb);
      const hex = hslToHex(hsl);
      const convertedRgb = hexToRgb(hex);
      
      // Allow for small rounding differences
      expect(convertedRgb.r).toBeCloseTo(originalRgb.r, 0);
      expect(convertedRgb.g).toBeCloseTo(originalRgb.g, 0);
      expect(convertedRgb.b).toBeCloseTo(originalRgb.b, 0);
    });

    it('should have symmetric contrast ratios', () => {
      const ratio1 = getContrastRatio('#ff0000', '#0000ff');
      const ratio2 = getContrastRatio('#0000ff', '#ff0000');
      expect(ratio1).toBe(ratio2);
    });
  });
});