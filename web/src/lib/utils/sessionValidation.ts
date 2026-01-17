import type { SessionDraft, ValidationResult } from '$lib/types/forge';

export async function validateSessionConfig(
  draft: SessionDraft,
  stepId?: string
): Promise<ValidationResult> {
  const errors: string[] = [];
  const warnings: string[] = [];

  if (!stepId || stepId === 'goal') {
    if (!draft.name || draft.name.trim().length < 3) {
      errors.push('Session name must be at least 3 characters');
    }
    if (!draft.goal || draft.goal.trim().length < 10) {
      errors.push('Goal description must be at least 10 characters');
    }
    if (draft.goal && draft.goal.length > 5000) {
      errors.push('Goal description exceeds maximum length of 5000 characters');
    }
  }

  if (!stepId || stepId === 'participants') {
    if (draft.participants.length < 2) {
      errors.push('At least 2 participants are required');
    }
    if (draft.participants.length > 10) {
      warnings.push('Having more than 10 participants may increase costs significantly');
    }

    const uniqueIds = new Set(draft.participants.map(p => p.id));
    if (uniqueIds.size !== draft.participants.length) {
      errors.push('Duplicate participants are not allowed');
    }
  }

  // Handle both legacy step IDs and new combined oracle-config step
  if (!stepId || stepId === 'oracle' || stepId === 'oracle-config') {
    if (!draft.oracle) {
      errors.push('An oracle must be selected');
    }

    if (draft.oracle && draft.participants.some(p => p.id === draft.oracle?.id)) {
      warnings.push('Oracle is also a participant - this may affect deliberation dynamics');
    }
  }

  if (!stepId || stepId === 'config' || stepId === 'oracle-config') {
    if (draft.config.maxRounds < 1 || draft.config.maxRounds > 20) {
      errors.push('Maximum rounds must be between 1 and 20');
    }
    if (draft.config.convergenceThreshold < 0.5 || draft.config.convergenceThreshold > 1) {
      errors.push('Convergence threshold must be between 0.5 and 1.0');
    }
    if (draft.config.timeoutMinutes < 5 || draft.config.timeoutMinutes > 480) {
      errors.push('Timeout must be between 5 minutes and 8 hours');
    }
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings
  };
}