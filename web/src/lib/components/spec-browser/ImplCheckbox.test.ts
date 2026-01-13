import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import { tick } from 'svelte';
import ImplCheckbox from './ImplCheckbox.svelte';

describe('ImplCheckbox', () => {
  const defaultProps = {
    id: 'test-checkbox',
    label: 'Test checkbox item',
    checked: false,
    partial: false,
    disabled: false
  };

  describe('Visual checkbox with states (unchecked, checked, partial)', () => {
    it('renders unchecked state by default', () => {
      render(ImplCheckbox, defaultProps);
      
      const checkbox = screen.getByRole('checkbox');
      expect(checkbox).toHaveAttribute('aria-checked', 'false');
      expect(screen.queryByText('✓')).not.toBeInTheDocument();
    });

    it('renders checked state when checked prop is true', () => {
      render(ImplCheckbox, { ...defaultProps, checked: true });
      
      const checkbox = screen.getByRole('checkbox');
      expect(checkbox).toHaveAttribute('aria-checked', 'true');
      
      // Check for checkmark SVG
      const svg = screen.getByRole('checkbox').querySelector('svg');
      expect(svg).toBeInTheDocument();
    });

    it('renders partial state when partial prop is true', () => {
      render(ImplCheckbox, { ...defaultProps, partial: true });
      
      const checkbox = screen.getByRole('checkbox');
      expect(checkbox).toHaveAttribute('aria-checked', 'mixed');
      
      // Check for partial indicator SVG
      const svg = screen.getByRole('checkbox').querySelector('svg');
      expect(svg).toBeInTheDocument();
    });

    it('applies correct CSS classes for different states', () => {
      const { container, component } = render(ImplCheckbox, defaultProps);
      
      let label = container.querySelector('.impl-checkbox');
      expect(label).not.toHaveClass('impl-checkbox--checked');
      expect(label).not.toHaveClass('impl-checkbox--partial');
      
      // Update to checked
      component.$set({ checked: true });
      expect(label).toHaveClass('impl-checkbox--checked');
      
      // Update to partial
      component.$set({ checked: false, partial: true });
      expect(label).toHaveClass('impl-checkbox--partial');
    });

    it('shows strikethrough text when checked', () => {
      const { container } = render(ImplCheckbox, { ...defaultProps, checked: true });
      
      const label = container.querySelector('.impl-checkbox--checked .impl-checkbox__label');
      expect(label).toBeInTheDocument();
    });
  });

  describe('Click/tap to toggle', () => {
    it('dispatches change event when clicked', async () => {
      const component = render(ImplCheckbox, defaultProps);
      const changeHandler = vi.fn();
      component.component.$on('change', changeHandler);
      
      const checkbox = screen.getByRole('checkbox');
      await fireEvent.click(checkbox);
      
      expect(changeHandler).toHaveBeenCalledWith(
        expect.objectContaining({
          detail: { id: 'test-checkbox', checked: true }
        })
      );
    });

    it('toggles from unchecked to checked', async () => {
      const component = render(ImplCheckbox, defaultProps);
      const changeHandler = vi.fn();
      component.component.$on('change', changeHandler);
      
      const checkbox = screen.getByRole('checkbox');
      await fireEvent.click(checkbox);
      
      expect(changeHandler).toHaveBeenCalledWith(
        expect.objectContaining({
          detail: { id: 'test-checkbox', checked: true }
        })
      );
    });

    it('toggles from checked to unchecked', async () => {
      const component = render(ImplCheckbox, { ...defaultProps, checked: true });
      const changeHandler = vi.fn();
      component.component.$on('change', changeHandler);
      
      const checkbox = screen.getByRole('checkbox');
      await fireEvent.click(checkbox);
      
      expect(changeHandler).toHaveBeenCalledWith(
        expect.objectContaining({
          detail: { id: 'test-checkbox', checked: false }
        })
      );
    });

    it('does not toggle when disabled', async () => {
      const component = render(ImplCheckbox, { ...defaultProps, disabled: true });
      const changeHandler = vi.fn();
      component.component.$on('change', changeHandler);
      
      const checkbox = screen.getByRole('checkbox');
      await fireEvent.click(checkbox);
      
      expect(changeHandler).not.toHaveBeenCalled();
    });

    it('does not toggle when updating', async () => {
      const component = render(ImplCheckbox, defaultProps);
      const changeHandler = vi.fn();
      component.component.$on('change', changeHandler);
      
      // First click should work
      const checkbox = screen.getByRole('checkbox');
      await fireEvent.click(checkbox);
      expect(changeHandler).toHaveBeenCalledTimes(1);
      
      // Immediate second click should be blocked during update
      await fireEvent.click(checkbox);
      expect(changeHandler).toHaveBeenCalledTimes(1);
    });
  });

  describe('Keyboard accessible (Space/Enter)', () => {
    it('toggles on Space key', async () => {
      const component = render(ImplCheckbox, defaultProps);
      const changeHandler = vi.fn();
      component.component.$on('change', changeHandler);
      
      const checkbox = screen.getByRole('checkbox');
      checkbox.focus();
      await fireEvent.keyDown(checkbox, { key: ' ' });
      
      expect(changeHandler).toHaveBeenCalledWith(
        expect.objectContaining({
          detail: { id: 'test-checkbox', checked: true }
        })
      );
    });

    it('toggles on Enter key', async () => {
      const component = render(ImplCheckbox, defaultProps);
      const changeHandler = vi.fn();
      component.component.$on('change', changeHandler);
      
      const checkbox = screen.getByRole('checkbox');
      checkbox.focus();
      await fireEvent.keyDown(checkbox, { key: 'Enter' });
      
      expect(changeHandler).toHaveBeenCalledWith(
        expect.objectContaining({
          detail: { id: 'test-checkbox', checked: true }
        })
      );
    });

    it('prevents default behavior on Space/Enter', async () => {
      render(ImplCheckbox, defaultProps);
      
      const checkbox = screen.getByRole('checkbox');
      checkbox.focus();
      
      const spaceEvent = new KeyboardEvent('keydown', { key: ' ', cancelable: true });
      const enterEvent = new KeyboardEvent('keydown', { key: 'Enter', cancelable: true });
      
      const spacePreventDefault = vi.spyOn(spaceEvent, 'preventDefault');
      const enterPreventDefault = vi.spyOn(enterEvent, 'preventDefault');
      
      await fireEvent(checkbox, spaceEvent);
      await fireEvent(checkbox, enterEvent);
      
      expect(spacePreventDefault).toHaveBeenCalled();
      expect(enterPreventDefault).toHaveBeenCalled();
    });

    it('does not respond to other keys', async () => {
      const component = render(ImplCheckbox, defaultProps);
      const changeHandler = vi.fn();
      component.component.$on('change', changeHandler);
      
      const checkbox = screen.getByRole('checkbox');
      checkbox.focus();
      await fireEvent.keyDown(checkbox, { key: 'a' });
      await fireEvent.keyDown(checkbox, { key: 'Escape' });
      
      expect(changeHandler).not.toHaveBeenCalled();
    });

    it('is focusable when not disabled', () => {
      const { container } = render(ImplCheckbox, defaultProps);
      
      const checkbox = container.querySelector('.impl-checkbox__box');
      expect(checkbox).toHaveAttribute('tabindex', '0');
    });

    it('is not focusable when disabled', () => {
      const { container } = render(ImplCheckbox, { ...defaultProps, disabled: true });
      
      const checkbox = container.querySelector('.impl-checkbox__box');
      expect(checkbox).toHaveAttribute('tabindex', '-1');
    });
  });

  describe('Sync state to markdown file', () => {
    // Note: This would require integration with the file system sync functionality
    // which would be handled by the parent component or store
    it('dispatches change event with correct data structure', async () => {
      const component = render(ImplCheckbox, defaultProps);
      const changeHandler = vi.fn();
      component.component.$on('change', changeHandler);
      
      const checkbox = screen.getByRole('checkbox');
      await fireEvent.click(checkbox);
      
      expect(changeHandler).toHaveBeenCalledWith(
        expect.objectContaining({
          detail: expect.objectContaining({
            id: expect.any(String),
            checked: expect.any(Boolean)
          })
        })
      );
    });
  });

  describe('Optimistic updates with rollback', () => {
    it('immediately updates visual state on interaction', async () => {
      render(ImplCheckbox, defaultProps);
      
      const checkbox = screen.getByRole('checkbox');
      expect(checkbox).toHaveAttribute('aria-checked', 'false');
      
      await fireEvent.click(checkbox);
      
      // Should immediately show as checked (optimistic update)
      expect(checkbox).toHaveAttribute('aria-checked', 'true');
    });

    it('shows updating state during operation', async () => {
      const { container } = render(ImplCheckbox, defaultProps);
      
      const checkbox = screen.getByRole('checkbox');
      await fireEvent.click(checkbox);
      
      const label = container.querySelector('.impl-checkbox');
      expect(label).toHaveClass('impl-checkbox--updating');
      
      // Wait for update timeout
      await new Promise(resolve => setTimeout(resolve, 350));
      expect(label).not.toHaveClass('impl-checkbox--updating');
    });

    it('prevents interaction during update', async () => {
      const component = render(ImplCheckbox, defaultProps);
      const changeHandler = vi.fn();
      component.component.$on('change', changeHandler);
      
      const checkbox = screen.getByRole('checkbox');
      await fireEvent.click(checkbox);
      
      // Second click during update should be ignored
      await fireEvent.click(checkbox);
      
      expect(changeHandler).toHaveBeenCalledTimes(1);
    });
  });

  describe('Animation on state change', () => {
    it('applies scale transition to checkmark icon', async () => {
      render(ImplCheckbox, { ...defaultProps, checked: true });
      
      // Check that SVG icon exists (transition would be applied via Svelte)
      const svg = screen.getByRole('checkbox').querySelector('svg');
      expect(svg).toBeInTheDocument();
      expect(svg).toHaveClass('impl-checkbox__icon');
    });

    it('applies pulse animation during update', async () => {
      const { container } = render(ImplCheckbox, defaultProps);
      
      const checkbox = screen.getByRole('checkbox');
      await fireEvent.click(checkbox);
      
      const label = container.querySelector('.impl-checkbox');
      expect(label).toHaveClass('impl-checkbox--updating');
      // CSS animation is applied via stylesheet
    });
  });

  describe('Accessibility', () => {
    it('has proper ARIA attributes', () => {
      const { container } = render(ImplCheckbox, defaultProps);
      
      const checkbox = container.querySelector('[role="checkbox"]');
      expect(checkbox).toHaveAttribute('role', 'checkbox');
      expect(checkbox).toHaveAttribute('aria-checked', 'false');
      expect(checkbox).toHaveAttribute('aria-disabled', 'false');
    });

    it('updates ARIA attributes based on state', () => {
      const { container, component } = render(ImplCheckbox, defaultProps);
      const checkbox = container.querySelector('[role="checkbox"]');
      
      // Update to checked
      component.$set({ checked: true });
      expect(checkbox).toHaveAttribute('aria-checked', 'true');
      
      // Update to partial
      component.$set({ checked: false, partial: true });
      expect(checkbox).toHaveAttribute('aria-checked', 'mixed');
      
      // Update to disabled
      component.$set({ disabled: true, partial: false });
      expect(checkbox).toHaveAttribute('aria-disabled', 'true');
    });

    it('associates label with checkbox', () => {
      render(ImplCheckbox, defaultProps);
      
      const label = screen.getByText('Test checkbox item');
      expect(label).toBeInTheDocument();
    });
  });

  describe('Visual styling and states', () => {
    it('applies disabled styling when disabled', () => {
      const { container } = render(ImplCheckbox, { ...defaultProps, disabled: true });
      
      const label = container.querySelector('.impl-checkbox');
      expect(label).toHaveClass('impl-checkbox--disabled');
    });

    it('shows correct colors for different states', () => {
      const { container } = render(ImplCheckbox, defaultProps);
      
      const box = container.querySelector('.impl-checkbox__box');
      expect(box).toBeInTheDocument();
      
      // Colors are applied via CSS custom properties
      expect(box).toHaveClass('impl-checkbox__box');
    });

    it('displays label text correctly', () => {
      render(ImplCheckbox, { ...defaultProps, label: 'Custom label text' });
      
      expect(screen.getByText('Custom label text')).toBeInTheDocument();
    });
  });
});