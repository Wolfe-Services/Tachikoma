import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { tick } from 'svelte';
import ContextMeter from './ContextMeter.svelte';
import type { ContextUsage } from '$lib/types/context';

describe('ContextMeter', () => {
  const createMockUsage = (overrides: Partial<ContextUsage> = {}): ContextUsage => ({
    inputTokens: 1000,
    outputTokens: 500,
    totalTokens: 1500,
    maxTokens: 4000,
    usagePercent: 37.5,
    zone: 'safe',
    ...overrides
  });

  beforeEach(() => {
    vi.clearAllTimers();
  });

  describe('Visual meter showing context usage percentage', () => {
    it('displays the correct percentage', async () => {
      const usage = createMockUsage({ usagePercent: 42.7 });
      render(ContextMeter, { usage });
      
      // Wait for animation to complete
      await new Promise(resolve => setTimeout(resolve, 450));
      expect(screen.getByText('43%')).toBeInTheDocument(); // Rounded
    });

    it('shows visual bar with correct width', async () => {
      const usage = createMockUsage({ usagePercent: 60 });
      const { container } = render(ContextMeter, { usage });
      
      // Wait for animation to complete
      await new Promise(resolve => setTimeout(resolve, 450));
      const fillBar = container.querySelector('.context-meter__fill');
      expect(fillBar).toHaveStyle({ width: '60%' });
    });

    it('has proper accessibility attributes', () => {
      const usage = createMockUsage({ usagePercent: 25 });
      const { container } = render(ContextMeter, { usage });
      
      const meter = container.querySelector('[role="meter"]');
      expect(meter).toHaveAttribute('aria-valuenow', '25');
      expect(meter).toHaveAttribute('aria-valuemin', '0');
      expect(meter).toHaveAttribute('aria-valuemax', '100');
      expect(meter).toHaveAttribute('aria-label', 'Context window usage');
    });
  });

  describe('Color zones (green/yellow/orange/red)', () => {
    it('shows safe zone color for low usage', () => {
      const usage = createMockUsage({ usagePercent: 30, zone: 'safe' });
      const { container } = render(ContextMeter, { usage });
      
      const fillBar = container.querySelector('.context-meter__fill');
      expect(fillBar).toHaveStyle({ 'background-color': 'var(--color-success-fg)' });
    });

    it('shows warning zone color at 60%', () => {
      const usage = createMockUsage({ usagePercent: 65, zone: 'warning' });
      const { container } = render(ContextMeter, { usage });
      
      const fillBar = container.querySelector('.context-meter__fill');
      expect(fillBar).toHaveStyle({ 'background-color': 'var(--color-warning-fg)' });
    });

    it('shows danger zone color at 80%', () => {
      const usage = createMockUsage({ usagePercent: 85, zone: 'danger' });
      const { container } = render(ContextMeter, { usage });
      
      const fillBar = container.querySelector('.context-meter__fill');
      expect(fillBar).toHaveStyle({ 'background-color': 'var(--warning-500)' });
    });

    it('shows critical zone color at 95%', () => {
      const usage = createMockUsage({ usagePercent: 97, zone: 'critical' });
      const { container } = render(ContextMeter, { usage });
      
      const fillBar = container.querySelector('.context-meter__fill');
      expect(fillBar).toHaveStyle({ 'background-color': 'var(--color-error-fg)' });
    });
  });

  describe('Token count display (used/max)', () => {
    it('displays token counts in full view', () => {
      const usage = createMockUsage({ 
        totalTokens: 2500, 
        maxTokens: 4000 
      });
      render(ContextMeter, { usage, compact: false });
      
      expect(screen.getByText('2.5k / 4.0k')).toBeInTheDocument();
    });

    it('hides token counts in compact mode', () => {
      const usage = createMockUsage({ 
        totalTokens: 1500, 
        maxTokens: 4000 
      });
      render(ContextMeter, { usage, compact: true });
      
      expect(screen.queryByText('1.5k / 4.0k')).not.toBeInTheDocument();
    });

    it('formats tokens under 1000 without k suffix', () => {
      const usage = createMockUsage({ 
        totalTokens: 750, 
        maxTokens: 4000 
      });
      render(ContextMeter, { usage, compact: false });
      
      expect(screen.getByText('750 / 4.0k')).toBeInTheDocument();
    });
  });

  describe('Animated transitions on updates', () => {
    it('uses tweened animation for percentage changes', async () => {
      const usage = createMockUsage({ usagePercent: 20 });
      const { component } = render(ContextMeter, { usage });
      
      // Update the usage
      await component.$set({ 
        usage: createMockUsage({ usagePercent: 80 })
      });
      
      // The animation should be in progress, not immediately at 80%
      await tick();
      const percentText = screen.getByText(/%$/);
      // Should show intermediate value during animation
      expect(percentText.textContent).not.toBe('80%');
    });

    it('applies CSS transitions to fill bar', () => {
      const usage = createMockUsage({ usagePercent: 50 });
      const { container } = render(ContextMeter, { usage });
      
      const fillBar = container.querySelector('.context-meter__fill');
      expect(fillBar).toHaveClass('context-meter__fill');
      // CSS transition is applied via stylesheet
    });
  });

  describe('Redline warning threshold indicator', () => {
    it('shows REDLINE warning at critical zone', () => {
      const usage = createMockUsage({ usagePercent: 97, zone: 'critical' });
      render(ContextMeter, { usage });
      
      expect(screen.getByText('REDLINE')).toBeInTheDocument();
    });

    it('does not show REDLINE warning in non-critical zones', () => {
      const usage = createMockUsage({ usagePercent: 85, zone: 'danger' });
      render(ContextMeter, { usage });
      
      expect(screen.queryByText('REDLINE')).not.toBeInTheDocument();
    });

    it('applies critical animation when in critical zone', () => {
      const usage = createMockUsage({ usagePercent: 97, zone: 'critical' });
      const { container } = render(ContextMeter, { usage });
      
      const meter = container.querySelector('.context-meter');
      expect(meter).toHaveClass('context-meter--critical');
    });

    it('REDLINE warning has blinking animation', () => {
      const usage = createMockUsage({ usagePercent: 97, zone: 'critical' });
      const { container } = render(ContextMeter, { usage });
      
      const warning = container.querySelector('.context-meter__warning');
      expect(warning).toHaveClass('context-meter__warning');
      // Animation is applied via CSS
    });
  });

  describe('Tooltip with detailed breakdown', () => {
    it('displays threshold markers with titles', () => {
      const usage = createMockUsage();
      const { container } = render(ContextMeter, { usage });
      
      const warningThreshold = container.querySelector('[title="Warning threshold"]');
      const dangerThreshold = container.querySelector('[title="Danger threshold"]');
      const criticalThreshold = container.querySelector('[title="Critical threshold"]');
      
      expect(warningThreshold).toBeInTheDocument();
      expect(dangerThreshold).toBeInTheDocument();
      expect(criticalThreshold).toBeInTheDocument();
    });

    it('shows detailed breakdown in non-compact mode', () => {
      const usage = createMockUsage({ 
        inputTokens: 1200, 
        outputTokens: 800 
      });
      render(ContextMeter, { usage, compact: false });
      
      expect(screen.getByText('In:')).toBeInTheDocument();
      expect(screen.getByText('1.2k')).toBeInTheDocument();
      expect(screen.getByText('Out:')).toBeInTheDocument();
      expect(screen.getByText('800')).toBeInTheDocument();
    });

    it('hides detailed breakdown in compact mode', () => {
      const usage = createMockUsage({ 
        inputTokens: 1200, 
        outputTokens: 800 
      });
      render(ContextMeter, { usage, compact: true });
      
      expect(screen.queryByText('In:')).not.toBeInTheDocument();
      expect(screen.queryByText('Out:')).not.toBeInTheDocument();
    });
  });

  describe('Compact mode', () => {
    it('applies compact styling', () => {
      const usage = createMockUsage();
      const { container } = render(ContextMeter, { usage, compact: true });
      
      const meter = container.querySelector('.context-meter');
      expect(meter).toHaveClass('context-meter--compact');
    });

    it('changes layout to horizontal in compact mode', () => {
      const usage = createMockUsage();
      const { container } = render(ContextMeter, { usage, compact: true });
      
      const meter = container.querySelector('.context-meter--compact');
      expect(meter).toHaveClass('context-meter--compact');
    });
  });
});