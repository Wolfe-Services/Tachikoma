import type { LayoutLoad } from './$types';

export const prerender = true;
export const ssr = false;
export const trailingSlash = 'never';

export const load: LayoutLoad = async ({ url }) => {
  // Detect if running in Tauri
  const isTauri = typeof window !== 'undefined' && 'tachikoma' in window;
  
  return {
    pathname: url.pathname,
    isTauri
  };
};