import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { expect, test, describe, vi } from 'vitest';
import RedlineWarning from './RedlineWarning.svelte';
import type { RedlineWarning as RedlineWarningType } from '$lib/types/redline';

describe('RedlineWarning Component', () => {
  const mockWarning: RedlineWarningType = {
    level: 'yellow',
    contextPercent: 75,
    message: 'Context usage is approaching the warning threshold',
    recommendations: [
      {
        id: '1',
        title: 'Create Checkpoint',
        description: 'Save current progress and reduce context',
        action: 'create_checkpoint',
        impact: 'Reduces context by ~50%'
      },
      {
        id: '2',
        title: 'Summarize Context',
        description: 'Summarize recent changes',
        action: 'summarize_context',
        impact: 'Reduces context by ~30%'
      }
    ],
    canDismiss: true
  };

  test('displays progressive warning levels correctly', async () => {
    // Test yellow warning
    const { rerender } = render(RedlineWarning, { props: { warning: mockWarning } });
    
    expect(screen.getByText('Context Warning')).toBeInTheDocument();
    expect(screen.getByText('⚠️')).toBeInTheDocument();
    expect(screen.getByText('75%')).toBeInTheDocument();

    // Test orange warning
    await rerender({ 
      warning: { 
        ...mockWarning, 
        level: 'orange' as const,
        contextPercent: 85
      } 
    });
    
    expect(screen.getByText('High Context Usage')).toBeInTheDocument();
    expect(screen.getByText('🔶')).toBeInTheDocument();
    expect(screen.getByText('85%')).toBeInTheDocument();

    // Test red warning
    await rerender({ 
      warning: { 
        ...mockWarning, 
        level: 'red' as const,
        contextPercent: 95
      } 
    });
    
    expect(screen.getByText('Context Redline')).toBeInTheDocument();
    expect(screen.getByText('🔴')).toBeInTheDocument();
    expect(screen.getByText('95%')).toBeInTheDocument();
  });

  test('displays animated alert appearance', () => {
    render(RedlineWarning, { props: { warning: mockWarning } });
    
    const warningElement = screen.getByRole('alert');
    expect(warningElement).toBeInTheDocument();
    expect(warningElement).toHaveClass('redline-warning');
  });

  test('shows red warning with animation', () => {
    const redWarning = { ...mockWarning, level: 'red' as const };
    render(RedlineWarning, { props: { warning: redWarning } });
    
    const warningElement = screen.getByRole('alert');
    expect(warningElement).toHaveClass('redline-warning--red');
  });

  test('displays actionable recommendations', () => {
    render(RedlineWarning, { props: { warning: mockWarning } });
    
    expect(screen.getByText('Create Checkpoint')).toBeInTheDocument();
    expect(screen.getByText('Reduces context by ~50%')).toBeInTheDocument();
    expect(screen.getByText('Summarize Context')).toBeInTheDocument();
    expect(screen.getByText('Reduces context by ~30%')).toBeInTheDocument();
  });

  test('handles quick actions correctly', async () => {
    const user = userEvent.setup();
    const component = render(RedlineWarning, { props: { warning: mockWarning } });
    
    const checkpointButton = screen.getByText('Create Checkpoint');
    
    let actionDispatched = false;
    let actionType = '';
    
    component.component.$on('action', (event) => {
      actionDispatched = true;
      actionType = event.detail;
    });
    
    await user.click(checkpointButton);
    
    expect(actionDispatched).toBe(true);
    expect(actionType).toBe('create_checkpoint');
  });

  test('is dismissible when canDismiss is true', async () => {
    const user = userEvent.setup();
    const component = render(RedlineWarning, { props: { warning: mockWarning } });
    
    const dismissButton = screen.getByLabelText('Dismiss warning');
    expect(dismissButton).toBeInTheDocument();
    
    let dismissed = false;
    component.component.$on('dismiss', () => {
      dismissed = true;
    });
    
    await user.click(dismissButton);
    expect(dismissed).toBe(true);
  });

  test('does not show dismiss button when canDismiss is false', () => {
    const nonDismissibleWarning = { ...mockWarning, canDismiss: false };
    render(RedlineWarning, { props: { warning: nonDismissibleWarning } });
    
    expect(screen.queryByLabelText('Dismiss warning')).not.toBeInTheDocument();
  });

  test('handles snooze options correctly', async () => {
    const user = userEvent.setup();
    const component = render(RedlineWarning, { props: { warning: mockWarning } });
    
    const snooze5Button = screen.getByText('5m');
    const snooze15Button = screen.getByText('15m');
    const snooze30Button = screen.getByText('30m');
    
    expect(snooze5Button).toBeInTheDocument();
    expect(snooze15Button).toBeInTheDocument();
    expect(snooze30Button).toBeInTheDocument();
    
    let snoozeMinutes = 0;
    component.component.$on('snooze', (event) => {
      snoozeMinutes = event.detail.minutes;
    });
    
    await user.click(snooze15Button);
    expect(snoozeMinutes).toBe(15);
  });

  test('has accessible announcements with appropriate aria-live', () => {
    // Test regular warning
    const { rerender } = render(RedlineWarning, { props: { warning: mockWarning } });
    
    let warningElement = screen.getByRole('alert');
    expect(warningElement).toHaveAttribute('aria-live', 'polite');
    
    // Test critical red warning
    const redWarning = { ...mockWarning, level: 'red' as const };
    rerender({ warning: redWarning });
    
    warningElement = screen.getByRole('alert');
    expect(warningElement).toHaveAttribute('aria-live', 'assertive');
  });

  test('can be hidden when show is false', () => {
    render(RedlineWarning, { props: { warning: mockWarning, show: false } });
    
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  test('shows warning message', () => {
    render(RedlineWarning, { props: { warning: mockWarning } });
    
    expect(screen.getByText('Context usage is approaching the warning threshold')).toBeInTheDocument();
  });

  test('displays recommendation descriptions in tooltips', () => {
    render(RedlineWarning, { props: { warning: mockWarning } });
    
    const checkpointButton = screen.getByText('Create Checkpoint').closest('button');
    expect(checkpointButton).toHaveAttribute('title', 'Save current progress and reduce context');
  });
});