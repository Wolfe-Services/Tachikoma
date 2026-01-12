export const BUILD_INFO = {
  version: __APP_VERSION__,
  commit: __GIT_COMMIT__,
  buildTime: __BUILD_TIME__,
  platform: __PLATFORM__,
  isDev: import.meta.env.DEV
} as const;

// Declare build-time constants
declare const __APP_VERSION__: string;
declare const __GIT_COMMIT__: string;
declare const __BUILD_TIME__: string;
declare const __PLATFORM__: string;