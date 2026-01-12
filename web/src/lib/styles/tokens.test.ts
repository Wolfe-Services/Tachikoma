import { describe, it, expect } from 'vitest';
import { colors, spacing, fontSize, fontWeight, borderRadius, zIndex, transition } from './tokens';

describe('Design Tokens', () => {
  describe('Color System', () => {
    it('should have complete blue color scale', () => {
      expect(colors.blue[500]).toBe('#00d4ff');
      expect(Object.keys(colors.blue)).toHaveLength(10);
      
      // Test that all blue shades exist
      const expectedShades = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900];
      expectedShades.forEach(shade => {
        expect(colors.blue[shade]).toBeDefined();
        expect(colors.blue[shade]).toMatch(/^#[0-9a-fA-F]{6}$/);
      });
    });

    it('should have complete gray color scale', () => {
      expect(colors.gray[500]).toBe('#64748b');
      expect(Object.keys(colors.gray)).toHaveLength(11); // includes 950
      
      // Test gray scale progression
      expect(colors.gray[50]).toBe('#f8fafc');
      expect(colors.gray[950]).toBe('#020617');
    });

    it('should have status colors', () => {
      expect(colors.status.success).toBe('#22c55e');
      expect(colors.status.warning).toBe('#eab308');
      expect(colors.status.error).toBe('#ef4444');
      expect(colors.status.info).toBe('#00d4ff');
    });

    it('should have valid hex color format', () => {
      const allColors = [
        ...Object.values(colors.blue),
        ...Object.values(colors.gray),
        ...Object.values(colors.status)
      ];
      
      allColors.forEach(color => {
        expect(color).toMatch(/^#[0-9a-fA-F]{6}$/);
      });
    });
  });

  describe('Spacing System', () => {
    it('should have consistent spacing scale', () => {
      expect(spacing[4]).toBe('1rem');
      expect(spacing[8]).toBe('2rem');
      expect(spacing[16]).toBe('4rem');
    });

    it('should follow 4px base increment', () => {
      expect(spacing[1]).toBe('0.25rem'); // 4px
      expect(spacing[2]).toBe('0.5rem');  // 8px
      expect(spacing[3]).toBe('0.75rem'); // 12px
      expect(spacing[4]).toBe('1rem');    // 16px
    });

    it('should have zero and pixel values', () => {
      expect(spacing[0]).toBe('0');
      expect(spacing.px).toBe('1px');
    });

    it('should support fractional values', () => {
      expect(spacing[0.5]).toBe('0.125rem'); // 2px
      expect(spacing[1.5]).toBe('0.375rem'); // 6px
      expect(spacing[2.5]).toBe('0.625rem'); // 10px
    });
  });

  describe('Typography System', () => {
    it('should have typography scale', () => {
      expect(fontSize.base).toBe('1rem');
      expect(fontSize.sm).toBe('0.875rem');
      expect(fontSize.lg).toBe('1.125rem');
    });

    it('should have extended size scale', () => {
      expect(fontSize.xs).toBe('0.75rem');
      expect(fontSize['2xl']).toBe('1.5rem');
      expect(fontSize['3xl']).toBe('1.875rem');
      expect(fontSize['4xl']).toBe('2.25rem');
      expect(fontSize['5xl']).toBe('3rem');
    });

    it('should have complete font weight range', () => {
      expect(fontWeight.thin).toBe(100);
      expect(fontWeight.normal).toBe(400);
      expect(fontWeight.bold).toBe(700);
      expect(fontWeight.black).toBe(900);
      
      // Test all weights exist
      const expectedWeights = [100, 200, 300, 400, 500, 600, 700, 800, 900];
      expectedWeights.forEach(weight => {
        const weightKey = Object.keys(fontWeight).find(key => fontWeight[key] === weight);
        expect(weightKey).toBeDefined();
      });
    });
  });

  describe('Border Radius System', () => {
    it('should have border radius scale', () => {
      expect(borderRadius.none).toBe('0');
      expect(borderRadius.sm).toBe('0.25rem');
      expect(borderRadius.md).toBe('0.375rem');
      expect(borderRadius.lg).toBe('0.5rem');
      expect(borderRadius.full).toBe('9999px');
    });

    it('should have extended radius options', () => {
      expect(borderRadius.xl).toBe('0.75rem');
      expect(borderRadius['2xl']).toBe('1rem');
      expect(borderRadius['3xl']).toBe('1.5rem');
    });
  });

  describe('Z-Index System', () => {
    it('should have ordered z-index scale', () => {
      expect(zIndex[10] < zIndex[20]).toBe(true);
      expect(zIndex[20] < zIndex[30]).toBe(true);
      expect(zIndex.dropdown < zIndex.modal).toBe(true);
      expect(zIndex.modal < zIndex.tooltip).toBe(true);
    });

    it('should have semantic z-index values', () => {
      expect(zIndex.dropdown).toBe(1000);
      expect(zIndex.modal).toBe(1050);
      expect(zIndex.tooltip).toBe(1070);
      expect(zIndex.toast).toBe(1080);
    });
  });

  describe('Transition System', () => {
    it('should have duration scale', () => {
      expect(transition.duration[150]).toBe('150ms');
      expect(transition.duration[300]).toBe('300ms');
      expect(transition.duration[500]).toBe('500ms');
    });

    it('should have timing functions', () => {
      expect(transition.timing.linear).toBe('linear');
      expect(transition.timing.easeIn).toBe('cubic-bezier(0.4, 0, 1, 1)');
      expect(transition.timing.easeOut).toBe('cubic-bezier(0, 0, 0.2, 1)');
      expect(transition.timing.easeInOut).toBe('cubic-bezier(0.4, 0, 0.2, 1)');
      expect(transition.timing.bounce).toBe('cubic-bezier(0.68, -0.55, 0.265, 1.55)');
    });
  });

  describe('Token Consistency', () => {
    it('should use consistent naming conventions', () => {
      // Color keys should be numbers for scales
      Object.keys(colors.blue).forEach(key => {
        expect(Number(key)).toBeGreaterThan(0);
      });
      
      // Spacing should use consistent format
      Object.values(spacing).forEach(value => {
        expect(value).toMatch(/^(0|1px|\d+(\.\d+)?rem)$/);
      });
    });

    it('should maintain semantic relationships', () => {
      // Primary blue should be 500 shade
      expect(colors.blue[500]).toBe('#00d4ff');
      
      // Base font size should be 1rem
      expect(fontSize.base).toBe('1rem');
      
      // Normal font weight should be 400
      expect(fontWeight.normal).toBe(400);
    });
  });
});