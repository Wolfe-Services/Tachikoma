// src/shared/protocol-urls.ts

export const PROTOCOL_SCHEMES = {
  APP: 'tachikoma',
  ASSET: 'tachikoma-asset',
} as const;

export function createAppUrl(path: string): string {
  const cleanPath = path.startsWith('/') ? path : `/${path}`;
  return `${PROTOCOL_SCHEMES.APP}://app${cleanPath}`;
}

export function createApiUrl(endpoint: string): string {
  const cleanEndpoint = endpoint.startsWith('/') ? endpoint : `/${endpoint}`;
  return `${PROTOCOL_SCHEMES.APP}://api${cleanEndpoint}`;
}

export function createResourceUrl(resourcePath: string): string {
  const cleanPath = resourcePath.startsWith('/') ? resourcePath : `/${resourcePath}`;
  return `${PROTOCOL_SCHEMES.APP}://resource${cleanPath}`;
}

export function createAssetUrl(filePath: string): string {
  // Encode the file path
  const encodedPath = encodeURIComponent(filePath);
  return `${PROTOCOL_SCHEMES.ASSET}://${encodedPath}`;
}

export function isProtocolUrl(url: string): boolean {
  return (
    url.startsWith(`${PROTOCOL_SCHEMES.APP}://`) ||
    url.startsWith(`${PROTOCOL_SCHEMES.ASSET}://`)
  );
}

export function parseProtocolUrl(url: string): {
  scheme: string;
  host: string;
  path: string;
  query: Record<string, string>;
} | null {
  try {
    const parsed = new URL(url);
    const query: Record<string, string> = {};
    parsed.searchParams.forEach((v, k) => (query[k] = v));

    return {
      scheme: parsed.protocol.replace(':', ''),
      host: parsed.hostname,
      path: parsed.pathname,
      query,
    };
  } catch {
    return null;
  }
}