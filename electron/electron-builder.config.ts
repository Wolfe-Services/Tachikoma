// electron-builder.config.ts
import type { Configuration } from 'electron-builder';

const config: Configuration = {
  appId: 'io.tachikoma.app',
  productName: 'Tachikoma',
  copyright: 'Copyright (c) 2024 Tachikoma Team',

  // Directories
  directories: {
    output: 'release/${version}',
    buildResources: 'build',
  },

  // Files to include
  files: [
    'dist/**/*',
    '!dist/**/*.map',
    'node_modules/**/*',
    '!node_modules/**/*.md',
    '!node_modules/**/*.ts',
    '!node_modules/**/test/**',
    '!node_modules/**/tests/**',
    '!node_modules/**/.github/**',
  ],

  // Extra files
  extraFiles: [
    {
      from: '../LICENSE',
      to: 'LICENSE',
    },
  ],

  // Extra resources (native modules, etc.)
  extraResources: [
    {
      from: '../target/release/tachikoma-native.${os}-${arch}.node',
      to: 'native/',
      filter: ['**/*'],
    },
  ],

  // ASAR archive
  asar: true,
  asarUnpack: [
    'node_modules/sharp/**/*',
    'node_modules/@tachikoma/native*/**/*',
  ],

  // Compression
  compression: 'maximum',

  // Remove unneeded locales
  electronLanguages: ['en', 'en-US'],

  // Artifacts naming
  artifactName: '${productName}-${version}-${os}-${arch}.${ext}',

  // Publish configuration
  publish: {
    provider: 'github',
    owner: 'tachikoma',
    repo: 'tachikoma-desktop',
    releaseType: 'release',
  },

  // macOS configuration
  mac: {
    target: [
      {
        target: 'dmg',
        arch: ['x64', 'arm64'],
      },
      {
        target: 'zip',
        arch: ['x64', 'arm64'],
      },
    ],
    category: 'public.app-category.developer-tools',
    icon: 'build/icon.icns',
    darkModeSupport: true,
    hardenedRuntime: true,
    gatekeeperAssess: false,
    entitlements: 'build/entitlements.mac.plist',
    entitlementsInherit: 'build/entitlements.mac.plist',
    notarize: {
      teamId: process.env.APPLE_TEAM_ID,
    },
    extendInfo: {
      NSMicrophoneUsageDescription: 'This app requires microphone access for voice features.',
      NSCameraUsageDescription: 'This app requires camera access for video features.',
      NSAppleEventsUsageDescription: 'This app requires Apple Events access for automation.',
    },
  },

  // DMG configuration
  dmg: {
    contents: [
      {
        x: 130,
        y: 220,
      },
      {
        x: 410,
        y: 220,
        type: 'link',
        path: '/Applications',
      },
    ],
    window: {
      width: 540,
      height: 380,
    },
    background: 'build/dmg-background.png',
    icon: 'build/icon.icns',
    iconSize: 80,
    title: '${productName} ${version}',
  },

  // PKG configuration
  pkg: {
    license: '../LICENSE',
    installLocation: '/Applications',
  },

  // Windows configuration
  win: {
    target: [
      {
        target: 'nsis',
        arch: ['x64', 'arm64'],
      },
      {
        target: 'portable',
        arch: ['x64'],
      },
    ],
    icon: 'build/icon.ico',
    publisherName: 'Tachikoma Team',
    verifyUpdateCodeSignature: true,
    signAndEditExecutable: true,
  },

  // NSIS installer configuration
  nsis: {
    oneClick: false,
    perMachine: false,
    allowToChangeInstallationDirectory: true,
    allowElevation: true,
    createDesktopShortcut: true,
    createStartMenuShortcut: true,
    shortcutName: 'Tachikoma',
    uninstallDisplayName: '${productName}',
    installerIcon: 'build/icon.ico',
    uninstallerIcon: 'build/icon.ico',
    installerHeaderIcon: 'build/icon.ico',
    license: '../LICENSE',
    deleteAppDataOnUninstall: false,
    include: 'build/installer.nsh',
    warningsAsErrors: false,
  },

  // Linux configuration
  linux: {
    target: [
      {
        target: 'AppImage',
        arch: ['x64', 'arm64'],
      },
      {
        target: 'deb',
        arch: ['x64', 'arm64'],
      },
      {
        target: 'rpm',
        arch: ['x64'],
      },
      {
        target: 'snap',
        arch: ['x64'],
      },
    ],
    icon: 'build/icons',
    category: 'Development',
    synopsis: 'Modern development environment',
    description: 'Tachikoma is a modern development environment for building amazing applications.',
    desktop: {
      Name: 'Tachikoma',
      Comment: 'Modern development environment',
      Keywords: 'development;code;editor',
      StartupNotify: 'true',
      StartupWMClass: 'tachikoma',
    },
    maintainer: 'Tachikoma Team <team@tachikoma.io>',
    vendor: 'Tachikoma',
  },

  // AppImage configuration
  appImage: {
    artifactName: '${productName}-${version}-${arch}.${ext}',
    category: 'Development',
    desktop: {
      StartupWMClass: 'tachikoma',
    },
  },

  // Debian package configuration
  deb: {
    depends: ['libgtk-3-0', 'libnotify4', 'libnss3', 'libxss1', 'libxtst6', 'xdg-utils', 'libatspi2.0-0', 'libuuid1'],
    category: 'Development',
    priority: 'optional',
    afterInstall: 'build/linux/after-install.sh',
    afterRemove: 'build/linux/after-remove.sh',
  },

  // RPM package configuration
  rpm: {
    depends: ['gtk3', 'libnotify', 'nss', 'libXScrnSaver', 'libXtst', 'xdg-utils', 'at-spi2-core', 'libuuid'],
    category: 'Development',
  },

  // Snap configuration
  snap: {
    confinement: 'strict',
    grade: 'stable',
    summary: 'Modern development environment',
    plugs: ['desktop', 'desktop-legacy', 'home', 'x11', 'unity7', 'browser-support', 'network', 'gsettings', 'opengl'],
  },

  // File associations
  fileAssociations: [
    {
      ext: 'tachi',
      name: 'Tachikoma Project',
      description: 'Tachikoma Project File',
      mimeType: 'application/x-tachikoma',
      icon: 'build/file-icon.icns',
      role: 'Editor',
    },
    {
      ext: 'tachikoma',
      name: 'Tachikoma Project',
      description: 'Tachikoma Project File',
      mimeType: 'application/x-tachikoma',
      icon: 'build/file-icon.icns',
      role: 'Editor',
    },
  ],

  // Protocol handlers
  protocols: [
    {
      name: 'Tachikoma',
      schemes: ['tachikoma'],
    },
  ],

  // Hooks
  beforeBuild: async (context) => {
    console.log('Building for:', context.platform.nodeName, context.arch);
    // Run any pre-build scripts
  },

  afterSign: async (context) => {
    // Run notarization for macOS
    if (context.electronPlatformName === 'darwin') {
      console.log('Notarizing application...');
    }
  },

  afterPack: async (context) => {
    console.log('Pack complete:', context.outDir);
  },

  afterAllArtifactBuild: async (result) => {
    console.log('All artifacts built:', result.artifactPaths);
    return result.artifactPaths;
  },
};

export default config;