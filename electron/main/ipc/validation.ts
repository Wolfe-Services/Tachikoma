// src/electron/main/ipc/validation.ts
import { z } from 'zod';

// Validation schemas for IPC arguments
export const schemas = {
  path: z.string().min(1).max(4096),
  title: z.string().max(256),
  message: z.string().max(4096),
  url: z.string().url(),

  pathRequest: z.object({
    path: z.string().min(1).max(4096),
  }),

  titleRequest: z.object({
    title: z.string().max(256),
  }),

  fileOptions: z.object({
    title: z.string().max(256).optional(),
    filters: z
      .array(
        z.object({
          name: z.string(),
          extensions: z.array(z.string()),
        })
      )
      .optional(),
    multiSelect: z.boolean().optional(),
  }),

  saveFileOptions: z.object({
    title: z.string().max(256).optional(),
    defaultPath: z.string().max(4096).optional(),
    filters: z
      .array(
        z.object({
          name: z.string(),
          extensions: z.array(z.string()),
        })
      )
      .optional(),
  }),

  writeFileArgs: z.object({
    path: z.string().min(1).max(4096),
    data: z.string(),
    options: z
      .object({
        encoding: z.string().optional(),
        atomic: z.boolean().optional(),
      })
      .optional(),
  }),

  createDirArgs: z.object({
    path: z.string().min(1).max(4096),
    recursive: z.boolean().optional(),
  }),

  watchArgs: z.object({
    path: z.string().min(1).max(4096),
    options: z
      .object({
        recursive: z.boolean().optional(),
      })
      .optional(),
  }),

  unwatchArgs: z.object({
    watchId: z.string().min(1),
  }),

  messageBoxArgs: z.object({
    type: z.enum(['info', 'error', 'warning', 'question']).optional(),
    title: z.string().max(256).optional(),
    message: z.string().max(4096),
    detail: z.string().max(4096).optional(),
    buttons: z.array(z.string()).max(10).optional(),
  }),

  confirmArgs: z.object({
    message: z.string().max(4096),
    detail: z.string().max(4096).optional(),
  }),

  errorArgs: z.object({
    message: z.string().max(4096),
    detail: z.string().max(4096).optional(),
  }),

  menuUpdateArgs: z.object({
    menuState: z.record(z.boolean()),
  }),

  contextMenuArgs: z.object({
    x: z.number(),
    y: z.number(),
    template: z.any().optional(),
  }),

  updateCheckArgs: z.object({
    silent: z.boolean().optional(),
  }),

  notificationArgs: z.object({
    title: z.string().max(256),
    body: z.string().max(1024),
    icon: z.string().optional(),
    silent: z.boolean().optional(),
    urgency: z.enum(['normal', 'critical', 'low']).optional(),
  }),

  exceptionArgs: z.object({
    error: z.instanceof(Error),
  }),

  rejectionArgs: z.object({
    reason: z.any(),
  }),

  appPathArgs: z.object({
    name: z.string(),
  }),
};

export function validate<T>(schema: z.ZodSchema<T>, data: unknown): T {
  try {
    return schema.parse(data);
  } catch (error) {
    if (error instanceof z.ZodError) {
      throw new Error(`Validation failed: ${error.errors.map(e => e.message).join(', ')}`);
    }
    throw error;
  }
}

export function validateAsync<T>(
  schema: z.ZodSchema<T>,
  data: unknown
): Promise<T> {
  return schema.parseAsync(data);
}

// Sanitization functions
export function sanitizePath(path: string): string {
  // Remove any dangerous path traversal attempts
  return path.replace(/\.\.\//g, '').replace(/\.\.\\/g, '');
}

export function sanitizeString(input: string, maxLength = 1024): string {
  return input.slice(0, maxLength).replace(/[\x00-\x1F\x7F]/g, '');
}

export function sanitizeFilename(filename: string): string {
  // Remove or replace invalid filename characters
  return filename.replace(/[<>:"/\\|?*\x00-\x1f]/g, '_').slice(0, 255);
}