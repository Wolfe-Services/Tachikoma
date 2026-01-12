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
        arch: ['universal'],
      },
      {
        target: 'zip',
        arch: ['universal'],
      },
      {
        target: 'pkg',
        arch: ['universal'],
      },
    ],
    category: 'public.app-category.developer-tools',
    type: 'distribution',
    icon: 'build/icon.icns',
    darkModeSupport: true,
    hardenedRuntime: true,
    gatekeeperAssess: false,
    entitlements: 'build/entitlements.mac.plist',
    entitlementsInherit: 'build/entitlements.mac.inherit.plist',
    notarize: {
      teamId: process.env.APPLE_TEAM_ID || '',
    },
    identity: process.env.APPLE_IDENTITY || 'Developer ID Application',
    bundleVersion: process.env.BUILD_NUMBER || '1',
    minimumSystemVersion: '10.15.0',
    x64ArchFiles: '*',
    mergeASARs: true,
    binaries: [
      'Contents/Frameworks/Tachikoma Helper.app/Contents/MacOS/Tachikoma Helper',
      'Contents/Frameworks/Tachikoma Helper (GPU).app/Contents/MacOS/Tachikoma Helper (GPU)',
      'Contents/Frameworks/Tachikoma Helper (Renderer).app/Contents/MacOS/Tachikoma Helper (Renderer)',
    ],
    extendInfo: {
      CFBundleDocumentTypes: [
        {
          CFBundleTypeName: 'Tachikoma Project',
          CFBundleTypeRole: 'Editor',
          CFBundleTypeExtensions: ['tachi', 'tachikoma'],
          CFBundleTypeIconFile: 'file-icon.icns',
          LSHandlerRank: 'Owner',
          LSItemContentTypes: ['io.tachikoma.project'],
        },
      ],
      UTExportedTypeDeclarations: [
        {
          UTTypeIdentifier: 'io.tachikoma.project',
          UTTypeDescription: 'Tachikoma Project',
          UTTypeConformsTo: ['public.data', 'public.content'],
          UTTypeTagSpecification: {
            'public.filename-extension': ['tachi', 'tachikoma'],
            'public.mime-type': ['application/x-tachikoma'],
          },
        },
      ],
      NSMicrophoneUsageDescription: 'Tachikoma requires microphone access for voice input features.',
      NSCameraUsageDescription: 'Tachikoma requires camera access for video features.',
      NSAppleEventsUsageDescription: 'Tachikoma requires Apple Events access for automation.',
      NSDocumentsFolderUsageDescription: 'Tachikoma requires access to your Documents folder.',
      NSDesktopFolderUsageDescription: 'Tachikoma requires access to your Desktop.',
      NSDownloadsFolderUsageDescription: 'Tachikoma requires access to your Downloads folder.',
      LSMinimumSystemVersion: '10.15.0',
      NSSupportsAutomaticGraphicsSwitching: true,
      NSHighResolutionCapable: true,
    },
  },

  // DMG configuration
  dmg: {
    window: {
      width: 540,
      height: 380,
    },
    contents: [
      {
        x: 130,
        y: 220,
        type: 'file',
      },
      {
        x: 410,
        y: 220,
        type: 'link',
        path: '/Applications',
      },
    ],
    background: 'build/dmg-background.png',
    backgroundColor: '#1a1a1a',
    icon: 'build/icon.icns',
    iconSize: 80,
    title: '${productName} ${version}',
    format: 'ULFO',
    sign: true,
    writeUpdateInfo: true,
  },

  // PKG configuration
  pkg: {
    license: '../LICENSE',
    installLocation: '/Applications',
    allowAnywhere: false,
    allowCurrentUserHome: false,
    allowRootDirectory: false,
    identity: process.env.APPLE_INSTALLER_IDENTITY || 'Developer ID Installer',
    scripts: 'build/pkg-scripts',
  },

  // Mac App Store configuration
  mas: {
    category: 'public.app-category.developer-tools',
    type: 'distribution',
    entitlements: 'build/entitlements.mas.plist',
    entitlementsInherit: 'build/entitlements.mas.inherit.plist',
    provisioningProfile: 'build/embedded.provisionprofile',
    hardenedRuntime: false,
    identity: process.env.APPLE_MAS_IDENTITY || '3rd Party Mac Developer Application',
    extendInfo: {
      CFBundleDocumentTypes: [
        {
          CFBundleTypeName: 'Tachikoma Project',
          CFBundleTypeRole: 'Editor',
          CFBundleTypeExtensions: ['tachi', 'tachikoma'],
          CFBundleTypeIconFile: 'file-icon.icns',
          LSHandlerRank: 'Owner',
          LSItemContentTypes: ['io.tachikoma.project'],
        },
      ],
      UTExportedTypeDeclarations: [
        {
          UTTypeIdentifier: 'io.tachikoma.project',
          UTTypeDescription: 'Tachikoma Project',
          UTTypeConformsTo: ['public.data', 'public.content'],
          UTTypeTagSpecification: {
            'public.filename-extension': ['tachi', 'tachikoma'],
            'public.mime-type': ['application/x-tachikoma'],
          },
        },
      ],
      NSMicrophoneUsageDescription: 'Tachikoma requires microphone access for voice input features.',
      NSCameraUsageDescription: 'Tachikoma requires camera access for video features.',
      NSAppleEventsUsageDescription: 'Tachikoma requires Apple Events access for automation.',
      LSMinimumSystemVersion: '10.15.0',
      NSSupportsAutomaticGraphicsSwitching: true,
      NSHighResolutionCapable: true,
    },
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