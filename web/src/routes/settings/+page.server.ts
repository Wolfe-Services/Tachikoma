import type { PageServerLoad } from './$types';
import { env } from '$env/dynamic/private';

export const load: PageServerLoad = async () => {
  // Read API keys from environment variables
  // We mask them for security - only show first 4 and last 4 chars
  const maskKey = (key: string | undefined): string => {
    if (!key || key.length < 8) return key || '';
    return key.substring(0, 4) + '...' + key.substring(key.length - 4);
  };

  return {
    envConfig: {
      anthropicKey: env.ANTHROPIC_API_KEY || '',
      openaiKey: env.OPENAI_API_KEY || '',
      googleKey: env.GEMINI_API_KEY || env.GOOGLE_API_KEY || '',
      // Pass masked versions for display
      anthropicKeyMasked: maskKey(env.ANTHROPIC_API_KEY),
      openaiKeyMasked: maskKey(env.OPENAI_API_KEY),
      googleKeyMasked: maskKey(env.GEMINI_API_KEY || env.GOOGLE_API_KEY),
      // Flags to know if keys are set
      hasAnthropicKey: !!env.ANTHROPIC_API_KEY,
      hasOpenaiKey: !!env.OPENAI_API_KEY,
      hasGoogleKey: !!(env.GEMINI_API_KEY || env.GOOGLE_API_KEY),
    }
  };
};
