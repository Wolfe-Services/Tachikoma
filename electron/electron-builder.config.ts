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
      from: '../web/dist',
      to: 'web',
      filter: ['**/*'],
    },
    {
      from: '../target/release/',
      to: 'native/',
      filter: ['**/*.node', '**/tachikoma-*'],
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

  // Artifacts naming - simplified for auto-update compatibility
  artifactName: '${productName}-${version}-${arch}.${ext}',

  // Publish configuration - CDN with GitHub fallback
  publish: [
    // Primary: Custom CDN for global distribution
    {
      provider: 'generic',
      url: 'https://releases.tachikoma.dev/releases/latest',
      channel: 'latest',
    },
    // Fallback: GitHub Releases
    {
      provider: 'github',
      owner: 'tachikoma',
      repo: 'tachikoma',
      releaseType: 'release',  // or 'draft' for manual publishing
      private: false,
      vPrefixedTagName: true,  // Use v1.0.0 format for tags
    },
  ],

  // Generate update manifests for auto-update
  generateUpdatesFilesForAllChannels: true,

  // Force update info generation even without publishing
  forceCodeSigning: false,

  // macOS configuration
  mac: {
    target: [
      {
        target: 'dmg',
        arch: ['x64', 'arm64'],
      },
      {
        target: 'zip',
        arch: ['x64', 'arm64'],  // ZIP files needed for auto-update
      },
    ],
    category: 'public.app-category.developer-tools',
    type: 'distribution',
    icon: 'build/icon.icns',
    darkModeSupport: true,
    
    // Code signing configuration
    identity: process.env.CSC_NAME || process.env.APPLE_IDENTITY || 'Developer ID Application',
    hardenedRuntime: true,
    entitlements: 'build/entitlements.mac.plist',
    entitlementsInherit: 'build/entitlements.mac.inherit.plist',
    
    // Sign all nested code
    signIgnore: [],
    
    // Timestamp server
    timestamp: 'http://timestamp.apple.com/ts01',
    
    // Gatekeeper assess (enabled for verification)
    gatekeeperAssess: true,
    
    // Strict verification
    strictVerify: true,
    
    notarize: {
      teamId: process.env.APPLE_TEAM_ID || '',
    },
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
    artifactName: '${productName}-${version}-${arch}.${ext}',

    // Window configuration
    window: {
      width: 660,
      height: 400,
    },

    // Background
    background: 'build/dmg-background.png',
    backgroundColor: '#1a1a2e',

    // Icon configuration
    icon: 'build/icon.icns',
    iconSize: 128,
    iconTextSize: 14,

    // Contents positioning
    contents: [
      {
        x: 180,
        y: 170,
        type: 'file',
      },
      {
        x: 480,
        y: 170,
        type: 'link',
        path: '/Applications',
      },
    ],

    // Code signing (DMG itself)
    sign: true,

    // Volume title
    title: '${productName} ${version}',

    // Format
    format: 'ULFO', // ULFO = LZFSE compression (fast, good ratio)

    // Internet-enable (allows Safari to auto-open)
    internetEnabled: true,

    // Write update info for Sparkle
    writeUpdateInfo: false,
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
        arch: ['x64'],
      },
      {
        target: 'portable',
        arch: ['x64'],
      },
      {
        target: 'msi',
        arch: ['x64'],
      },
    ],
    icon: 'build/icon.ico',
    publisherName: 'Tachikoma Team',
    publisherDisplayName: 'Tachikoma Team',
    legalTrademarks: 'Tachikoma',
    
    // Code signing configuration
    verifyUpdateCodeSignature: true,
    signAndEditExecutable: true,
    signDlls: true,
    
    // Certificate configuration (via environment variables)
    // CSC_LINK - path to .pfx file or base64-encoded certificate
    // CSC_KEY_PASSWORD - certificate password
    certificateFile: process.env.CSC_LINK,
    certificatePassword: process.env.CSC_KEY_PASSWORD,
    
    // Publisher name (must match certificate subject)
    certificateSubjectName: process.env.WIN_CERT_SUBJECT_NAME || 'Tachikoma Team',
    
    // Signing hash algorithms (modern standard)
    signingHashAlgorithms: ['sha256'],
    
    // Timestamp servers for longevity (RFC 3161 is preferred)
    rfc3161TimeStampServer: 'http://timestamp.digicert.com',
    timeStampServer: 'http://timestamp.comodoca.com', // Fallback
    
    // Custom signing function (optional)
    sign: async (configuration) => {
      // Return undefined to use default electron-builder signing
      // Can be customized for advanced scenarios
      return undefined;
    },
    
    requestedExecutionLevel: 'asInvoker',
    extraFiles: [
      {
        from: 'build/windows',
        to: '.',
        filter: ['**/*'],
      },
    ],
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
    menuCategory: 'Development',
    uninstallDisplayName: '${productName}',
    deleteAppDataOnUninstall: false,
    artifactName: '${productName}-Setup-${version}.${ext}',
    installerIcon: 'build/icon.ico',
    uninstallerIcon: 'build/icon.ico',
    installerHeaderIcon: 'build/icon.ico',
    installerHeader: 'build/windows/installer-header.bmp',
    installerSidebar: 'build/windows/installer-sidebar.bmp',
    uninstallerSidebar: 'build/windows/uninstaller-sidebar.bmp',
    license: '../LICENSE',
    include: 'build/installer.nsh',
    script: 'build/windows/installer-script.nsi',
    installerLanguages: ['en_US'],
    language: '1033', // English
    runAfterFinish: true,
    displayLanguageSelector: false,
    unicode: true,
    warningsAsErrors: false,
    differentialPackage: true,
    packElevateHelper: true,
  },

  // Portable configuration
  portable: {
    artifactName: '${productName}-${version}-portable-${arch}.${ext}',
    requestExecutionLevel: 'user',
  },

  // MSI configuration (for enterprise deployment)
  msi: {
    artifactName: '${productName}-${version}.${ext}',
    createDesktopShortcut: true,
    createStartMenuShortcut: true,
    perMachine: true,  // Install for all users
    runAfterFinish: false,  // MSI best practice
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
    executableName: 'tachikoma',
    desktop: {
      Name: 'Tachikoma',
      GenericName: 'AI Development Environment',
      Comment: 'Autonomous AI-powered development tool',
      Type: 'Application',
      Categories: 'Development;IDE;',
      Keywords: 'ai;development;coding;llm;',
      StartupNotify: true,
      StartupWMClass: 'tachikoma',
      MimeType: 'application/x-tachikoma-spec;x-scheme-handler/tachikoma;',
    },
    fileAssociations: [
      {
        ext: 'tspec',
        name: 'Tachikoma Spec',
        mimeType: 'application/x-tachikoma-spec',
      },
    ],
    maintainer: 'Tachikoma Team <team@tachikoma.io>',
    vendor: 'Tachikoma',
  },

  // AppImage configuration
  appImage: {
    artifactName: '${productName}-${version}-${arch}.${ext}',

    // Include desktop integration
    desktop: {
      entry: {
        Name: 'Tachikoma',
        Exec: 'tachikoma %U',
        Icon: 'tachikoma',
        Type: 'Application',
        Categories: 'Development;',
        MimeType: 'x-scheme-handler/tachikoma;application/x-tachikoma-spec;',
      },
    },

    // License file
    license: '../LICENSE',

    // Synopsis for AppImageHub
    synopsis: 'AI-powered autonomous development environment',

    // Category for AppImageHub
    category: 'Development',
  },

  // Debian package configuration
  deb: {
    artifactName: '${productName}_${version}_${arch}.${ext}',

    // Package metadata
    packageName: 'tachikoma',
    category: 'devel',
    priority: 'optional',
    section: 'devel',

    // Dependencies
    depends: [
      'libgtk-3-0',
      'libnotify4',
      'libnss3',
      'libxss1',
      'libxtst6',
      'xdg-utils',
      'libatspi2.0-0',
      'libuuid1',
      'libsecret-1-0',
    ],

    // Recommended packages
    recommends: [
      'git',
    ],

    // Package scripts
    afterInstall: 'build/linux/after-install.sh',
    afterRemove: 'build/linux/after-remove.sh',

    // Desktop file
    desktop: {
      Name: 'Tachikoma',
      GenericName: 'AI Development Environment',
      Comment: 'AI-powered autonomous development tool',
      Exec: '/opt/Tachikoma/tachikoma %U',
      Icon: 'tachikoma',
      Type: 'Application',
      Categories: 'Development;IDE;',
      MimeType: 'application/x-tachikoma-spec;x-scheme-handler/tachikoma;',
      StartupNotify: 'true',
      StartupWMClass: 'tachikoma',
    },

    // Package maintainer
    maintainer: 'Tachikoma Team <support@tachikoma.dev>',

    // Vendor
    vendor: 'Tachikoma',

    // Homepage
    homepage: 'https://tachikoma.dev',

    // Compression
    compression: 'xz',

    // Fpm options
    fpm: [
      '--deb-priority', 'optional',
    ],
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
      ext: 'tspec',
      name: 'Tachikoma Spec',
      description: 'Tachikoma Specification File',
      mimeType: 'application/x-tachikoma-spec',
      icon: process.platform === 'win32' ? 'build/file-icon.ico' : 'build/file-icon.icns',
      role: 'Editor',
    },
    {
      ext: 'tachi',
      name: 'Tachikoma Project',
      description: 'Tachikoma Project File',
      mimeType: 'application/x-tachikoma',
      icon: process.platform === 'win32' ? 'build/file-icon.ico' : 'build/file-icon.icns',
      role: 'Editor',
    },
    {
      ext: 'tachikoma',
      name: 'Tachikoma Project',
      description: 'Tachikoma Project File',
      mimeType: 'application/x-tachikoma',
      icon: process.platform === 'win32' ? 'build/file-icon.ico' : 'build/file-icon.icns',
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

  afterSign: 'scripts/notarize.js',

  afterPack: async (context) => {
    console.log('Pack complete:', context.outDir);
  },

  afterAllArtifactBuild: async (result) => {
    console.log('All artifacts built:', result.artifactPaths);
    return result.artifactPaths;
  },
};

export default config;