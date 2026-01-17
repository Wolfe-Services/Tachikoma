# Tachikoma Build Pipeline Architecture

## Overview

The Tachikoma build system implements a sophisticated multi-stage pipeline that orchestrates builds across Rust, TypeScript/Svelte, and Electron components. The pipeline is designed for both development efficiency and production optimization.

## Pipeline Architecture

```
Source Code Repository
         │
    ┌────┴────┐
    │   Git   │
    └────┬────┘
         │
    ┌────▼────┐
    │  Build  │
    │ System  │ ──── Central Configuration (build.config.ts)
    └────┬────┘      Platform Matrix, Options, Security
         │
    ┌────▼────┐
    │ Stage 1 │ ──── Rust Workspace Build
    │  Rust   │      ├─► tachikoma-common-core
    └────┬────┘      ├─► tachikoma-common-config
         │           ├─► tachikoma-primitives
         │           ├─► tachikoma-backends
         │           ├─► tachikoma-loop
         │           ├─► tachikoma-vcs
         │           ├─► tachikoma-forge-types
         │           └─► tachikoma-native (NAPI binding)
    ┌────▼────┐
    │ Stage 2 │ ──── Web Frontend Build  
    │  Web    │      ├─► TypeScript Compilation
    └────┬────┘      ├─► Svelte Component Processing
         │           ├─► Vite Asset Bundling
         │           ├─► CSS Processing & Optimization
         │           ├─► Source Map Generation
         │           └─► Asset Optimization (images, fonts)
    ┌────▼────┐
    │ Stage 3 │ ──── Electron Application Build
    │Electron │      ├─► Main Process Compilation
    └────┬────┘      ├─► Preload Script Bundle
         │           ├─► Renderer Process Integration
         │           ├─► Native Module Linking
         │           ├─► IPC Type Generation
         │           └─► Security Configuration (CSP, isolation)
    ┌────▼────┐
    │ Stage 4 │ ──── Platform Packaging
    │Package │       ├─► macOS (.dmg, .app, universal binaries)
    └────┬────┘      ├─► Windows (.exe, .msi, NSIS installer)
         │           ├─► Linux (.AppImage, .deb, .rpm)
         │           ├─► Code Signing & Notarization
         │           └─► Auto-update Feed Generation
    ┌────▼────┐
    │ Stage 5 │ ──── Distribution & Deployment
    │Distribute│     ├─► GitHub Releases
    └─────────┘     ├─► Update Server
                    ├─► Artifact Storage (CDN)
                    └─► Release Notes Generation
```

## Build Modes

### Development Mode
- **Fast incremental compilation**: Only changed modules rebuilt
- **Hot Module Replacement**: Live code updates without app restart
- **Debug symbols**: Full debugging information included
- **Source maps**: Maintained for frontend debugging
- **Live reload**: File watcher triggers automatic rebuilds
- **Relaxed security**: Development-friendly CSP policies

**Performance Targets:**
- Initial build: < 30 seconds
- Incremental rebuild: < 5 seconds
- Hot reload: < 1 second

### Production Mode  
- **Full optimization passes**: All code optimized for size and speed
- **Asset minification**: JavaScript, CSS, and HTML compressed
- **Tree shaking**: Dead code elimination across all modules
- **Debug symbol stripping**: Reduced binary size
- **Code signing**: All executables digitally signed
- **Security hardening**: Strict CSP, context isolation

**Optimization Strategies:**
- Rust: `--release` flag with LTO (Link Time Optimization)
- Web: Vite production build with code splitting
- Native modules: Release builds with symbol stripping
- Assets: Compression, format conversion (WebP), lazy loading

### CI/CD Mode
- **Matrix builds**: Parallel builds across all platforms
- **Automated testing**: Full test suite execution
- **Artifact validation**: Build verification and size checks
- **Security scanning**: Dependency vulnerability checks  
- **Release automation**: Tag-based deployment triggers

## Platform-Specific Considerations

### macOS Build Pipeline
```
Source → Rust (universal) → Web Bundle → Electron → Code Sign → Notarize → DMG
                                                        ↓
                                                  entitlements.plist
                                                  Apple Developer ID
```

**macOS-Specific Steps:**
1. **Universal Binary Creation**: Combine x64 and arm64 Rust builds
2. **Code Signing**: Sign all executables and native modules
3. **Notarization**: Submit to Apple for security verification
4. **DMG Creation**: Installer with background image and layout
5. **Gatekeeper Compatibility**: Proper bundle structure and metadata

### Windows Build Pipeline  
```
Source → Rust (x64/arm64) → Web Bundle → Electron → Sign → NSIS → MSI
                                                     ↓
                                               Code Signing Cert
                                               Windows SDK Tools
```

**Windows-Specific Steps:**
1. **Multi-architecture**: Separate builds for x64 and arm64
2. **Code Signing**: Authenticode signature with trusted certificate
3. **NSIS Installer**: Custom installer with registry entries
4. **MSI Package**: Enterprise deployment format
5. **Windows Store**: UWP packaging for store distribution

### Linux Build Pipeline
```
Source → Rust (x64/arm64) → Web Bundle → Electron → Package → Verify
                                                     ↓
                                            AppImage/deb/rpm/snap
                                            Desktop integration
```

**Linux-Specific Steps:**
1. **Multi-format Packaging**: AppImage, deb, rpm, snap formats
2. **Desktop Integration**: .desktop files and MIME associations
3. **Dependency Management**: Proper library linking and bundling
4. **Permission Model**: Snap confinement and AppArmor policies
5. **Distribution**: Repository metadata and signing

## Build Orchestration

### Master Build Script (`scripts/build.ts`)

Coordinates the entire build process with:
- **Pre-flight checks**: Verify prerequisites and environment
- **Dependency resolution**: Ensure all native modules are compatible
- **Parallel execution**: Optimize build times with concurrent stages
- **Error handling**: Graceful failure recovery and detailed logging
- **Progress reporting**: Real-time build status and metrics

### Configuration System

Central build configuration in `build.config.ts`:

```typescript
interface BuildConfig {
  version: string;           // Semantic version from package.json
  buildNumber: string;       // CI build number or 'dev'
  gitCommit: string;         // Git SHA for traceability
  
  platforms: PlatformConfig[];  // Target platform matrix
  options: BuildOptions;        // Optimization and feature flags
  
  paths: {
    rootDir: string;         // Project root
    outputDir: string;       // Build artifacts
    cacheDir: string;        // Build cache
  };
}
```

### Caching Strategy

Multi-level caching for build performance:

1. **Cargo Cache**: Rust dependency compilation cache
2. **Node Modules Cache**: NPM dependency cache  
3. **Vite Cache**: Frontend build artifact cache
4. **Native Module Cache**: NAPI-RS build cache
5. **Asset Cache**: Processed image and font cache

**Cache Invalidation:**
- File-based: Content hashing for precise invalidation
- Dependency-based: Cargo.lock and package-lock.json changes
- Configuration-based: build.config.ts modifications

## Native Module Integration

### NAPI-RS Build Process

```
Rust Source → NAPI-RS → Native Module → Electron Integration
     ↓              ↓           ↓              ↓
   Cargo         Bindings   .node file    require()
```

**Build Steps:**
1. **Rust Compilation**: Native code compiled to shared library
2. **NAPI Binding**: JavaScript-compatible interface generation
3. **Platform Targeting**: Architecture-specific builds (x64, arm64)
4. **Electron ABI**: Compatibility with specific Electron versions
5. **Security**: Signed native modules for production

### Cross-Compilation

Support for building native modules on different platforms:
- **macOS Host**: Can build for darwin-x64, darwin-arm64
- **Linux Host**: Can build for linux-x64, linux-arm64, win32-x64 (via MinGW)
- **Windows Host**: Can build for win32-x64, win32-arm64

## Security in Build Pipeline

### Code Signing
- **macOS**: Apple Developer ID with notarization
- **Windows**: Authenticode with trusted certificate authority
- **Linux**: GPG signing for package repositories

### Build Environment Security
- **Sandboxed builds**: Isolated build environments
- **Dependency verification**: Lock file integrity checks
- **Supply chain security**: SBOM (Software Bill of Materials) generation
- **Vulnerability scanning**: Automated security audit of dependencies

### Artifact Integrity
- **Checksums**: SHA256 hashes for all build artifacts
- **Reproducible builds**: Deterministic output for security verification
- **Provenance**: Complete build environment documentation

## Performance Monitoring

### Build Metrics
- **Build duration**: Total and per-stage timing
- **Artifact sizes**: Tracking size regression over time
- **Cache hit rates**: Build cache effectiveness
- **Resource usage**: CPU, memory, and disk utilization

### Optimization Tracking
- **Bundle analysis**: JavaScript bundle size breakdown
- **Dependency impact**: Size contribution of each dependency
- **Performance regression**: Automated detection of slowdowns

## Testing Integration

### Build Verification
- **Smoke tests**: Basic functionality verification
- **Integration tests**: Cross-component communication
- **Performance tests**: Startup time and memory usage
- **Security tests**: Permissions and sandboxing verification

### Automated Quality Gates
- **Build success**: All stages complete without errors
- **Test passage**: Full test suite execution
- **Size limits**: Artifact size within acceptable bounds
- **Security scan**: No high-severity vulnerabilities

This comprehensive build pipeline ensures reliable, secure, and performant distribution of Tachikoma across all supported platforms while maintaining developer productivity and code quality standards.