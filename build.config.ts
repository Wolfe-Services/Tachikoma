/**
 * Tachikoma Build Configuration
 *
 * Central configuration for all build processes.
 */

export interface BuildConfig {
  // Version info
  version: string;
  buildNumber: string;
  gitCommit: string;

  // Paths
  rootDir: string;
  outputDir: string;
  cacheDir: string;

  // Platform targets
  platforms: PlatformConfig[];

  // Build options
  options: BuildOptions;
}

export interface PlatformConfig {
  name: 'darwin' | 'win32' | 'linux';
  arch: 'x64' | 'arm64';
  enabled: boolean;
}

export interface BuildOptions {
  // Optimization level
  release: boolean;

  // Code signing
  sign: boolean;
  notarize: boolean;

  // Output formats
  formats: {
    dmg: boolean;
    pkg: boolean;
    nsis: boolean;
    msi: boolean;
    appimage: boolean;
    deb: boolean;
    rpm: boolean;
  };

  // Features
  includeSourceMaps: boolean;
  stripDebugSymbols: boolean;
  compressAssets: boolean;
}

export function loadBuildConfig(): BuildConfig {
  const pkg = require('./package.json');

  return {
    version: pkg.version,
    buildNumber: process.env.BUILD_NUMBER || 'dev',
    gitCommit: process.env.GIT_COMMIT || 'unknown',

    rootDir: process.cwd(),
    outputDir: 'dist',
    cacheDir: '.build-cache',

    platforms: [
      { name: 'darwin', arch: 'x64', enabled: true },
      { name: 'darwin', arch: 'arm64', enabled: true },
      { name: 'win32', arch: 'x64', enabled: true },
      { name: 'linux', arch: 'x64', enabled: true },
    ],

    options: {
      release: process.env.NODE_ENV === 'production',
      sign: !!process.env.CSC_LINK,
      notarize: !!process.env.APPLE_ID,

      formats: {
        dmg: true,
        pkg: false,
        nsis: true,
        msi: false,
        appimage: true,
        deb: true,
        rpm: false,
      },

      includeSourceMaps: process.env.NODE_ENV !== 'production',
      stripDebugSymbols: process.env.NODE_ENV === 'production',
      compressAssets: true,
    },
  };
}