import type { PageServerLoad } from './$types';
import { env } from '$env/dynamic/private';
import { readFileSync, existsSync } from 'fs';
import { resolve } from 'path';

// Load .env from repo root if not already loaded
function loadEnvFromRoot(): Record<string, string> {
  const envVars: Record<string, string> = {};
  
  // Try multiple possible locations for the .env file
  const possiblePaths = [
    resolve(process.cwd(), '.env'),
    resolve(process.cwd(), '..', '.env'),
    resolve(import.meta.dirname || '', '..', '..', '..', '..', '.env'),
  ];
  
  for (const envPath of possiblePaths) {
    if (existsSync(envPath)) {
      try {
        const content = readFileSync(envPath, 'utf-8');
        for (const line of content.split('\n')) {
          const trimmed = line.trim();
          if (trimmed && !trimmed.startsWith('#')) {
            const [key, ...valueParts] = trimmed.split('=');
            if (key && valueParts.length > 0) {
              envVars[key.trim()] = valueParts.join('=').trim();
            }
          }
        }
        break; // Found and parsed, stop looking
      } catch (e) {
        console.warn(`Could not read .env from ${envPath}:`, e);
      }
    }
  }
  
  return envVars;
}

export const load: PageServerLoad = async () => {
  // Get env vars from both SvelteKit's env and manual loading from .env
  const manualEnv = loadEnvFromRoot();
  // Merge SvelteKit env with manually loaded env (manual takes precedence for missing keys)
  const anthropicKey = env.ANTHROPIC_API_KEY || manualEnv.ANTHROPIC_API_KEY || '';
  const openaiKey = env.OPENAI_API_KEY || manualEnv.OPENAI_API_KEY || '';
  const googleKey = env.GEMINI_API_KEY || env.GOOGLE_API_KEY || manualEnv.GEMINI_API_KEY || manualEnv.GOOGLE_API_KEY || '';

  // We mask keys for security - only show first 4 and last 4 chars
  const maskKey = (key: string | undefined): string => {
    if (!key || key.length < 8) return key || '';
    return key.substring(0, 4) + '...' + key.substring(key.length - 4);
  };

  return {
    envConfig: {
      anthropicKey,
      openaiKey,
      googleKey,
      // Pass masked versions for display
      anthropicKeyMasked: maskKey(anthropicKey),
      openaiKeyMasked: maskKey(openaiKey),
      googleKeyMasked: maskKey(googleKey),
      // Flags to know if keys are set
      hasAnthropicKey: !!anthropicKey,
      hasOpenaiKey: !!openaiKey,
      hasGoogleKey: !!googleKey,
    }
  };
};
