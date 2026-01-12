# Spec 176: Code Signing

## Phase
8 - Electron Shell

## Spec ID
176

## Status
Planned

## Dependencies
- Spec 175 (Build Configuration)

## Estimated Context
~8%

---

## Objective

Implement code signing for all platforms to ensure application integrity and enable distribution through official channels. This includes macOS notarization, Windows Authenticode signing, and Linux package signing.

---

## Acceptance Criteria

- [ ] macOS code signing with Developer ID certificate
- [ ] macOS notarization for Gatekeeper approval
- [ ] Windows Authenticode signing with EV certificate
- [ ] Linux GPG signing for packages
- [ ] Automated signing in CI/CD pipeline
- [ ] Signature verification tools
- [ ] Certificate management documentation
- [ ] Timestamping for long-term validity

---

## Implementation Details

### macOS Signing Configuration

```typescript
// scripts/sign-macos.ts
import { execSync } from 'child_process';
import { join } from 'path';

interface SigningConfig {
  identity: string;
  entitlements: string;
  hardenedRuntime: boolean;
  timestamp: boolean;
}

export async function signMacOSApp(
  appPath: string,
  config: SigningConfig
): Promise<void> {
  const { identity, entitlements, hardenedRuntime, timestamp } = config;

  // Build codesign arguments
  const args: string[] = [
    '--force',
    '--deep',
    '--sign',
    identity,
    '--entitlements',
    entitlements,
  ];

  if (hardenedRuntime) {
    args.push('--options', 'runtime');
  }

  if (timestamp) {
    args.push('--timestamp');
  }

  args.push(appPath);

  console.log('Signing application...');
  execSync(`codesign ${args.join(' ')}`, { stdio: 'inherit' });

  // Verify signature
  console.log('Verifying signature...');
  execSync(`codesign --verify --deep --strict --verbose=2 "${appPath}"`, {
    stdio: 'inherit',
  });

  console.log('Checking entitlements...');
  execSync(`codesign -d --entitlements :- "${appPath}"`, { stdio: 'inherit' });
}

export async function signMacOSFrameworks(appPath: string, identity: string): Promise<void> {
  const frameworksPath = join(appPath, 'Contents', 'Frameworks');

  // Sign each framework
  const frameworks = [
    'Electron Framework.framework',
    'Mantle.framework',
    'ReactiveCocoa.framework',
    'Squirrel.framework',
  ];

  for (const framework of frameworks) {
    const frameworkPath = join(frameworksPath, framework);
    try {
      execSync(
        `codesign --force --deep --sign "${identity}" --options runtime "${frameworkPath}"`,
        { stdio: 'inherit' }
      );
    } catch {
      console.log(`Framework ${framework} not found, skipping...`);
    }
  }

  // Sign helper apps
  const helpers = [
    'Tachikoma Helper.app',
    'Tachikoma Helper (GPU).app',
    'Tachikoma Helper (Renderer).app',
    'Tachikoma Helper (Plugin).app',
  ];

  for (const helper of helpers) {
    const helperPath = join(frameworksPath, helper);
    try {
      execSync(
        `codesign --force --deep --sign "${identity}" --options runtime "${helperPath}"`,
        { stdio: 'inherit' }
      );
    } catch {
      console.log(`Helper ${helper} not found, skipping...`);
    }
  }
}
```

### macOS Notarization

```typescript
// scripts/notarize-macos.ts
import { notarize } from '@electron/notarize';
import { join } from 'path';

interface NotarizeConfig {
  appBundleId: string;
  appPath: string;
  appleId: string;
  appleIdPassword: string;
  teamId: string;
}

export async function notarizeMacOSApp(config: NotarizeConfig): Promise<void> {
  const { appBundleId, appPath, appleId, appleIdPassword, teamId } = config;

  console.log('Starting notarization...');
  console.log(`App path: ${appPath}`);
  console.log(`Bundle ID: ${appBundleId}`);

  try {
    await notarize({
      tool: 'notarytool',
      appBundleId,
      appPath,
      appleId,
      appleIdPassword,
      teamId,
    });

    console.log('Notarization complete!');

    // Verify notarization
    const { execSync } = require('child_process');
    console.log('Verifying notarization...');
    execSync(`spctl --assess --type execute --verbose "${appPath}"`, {
      stdio: 'inherit',
    });
  } catch (error) {
    console.error('Notarization failed:', error);
    throw error;
  }
}

// electron-builder afterSign hook
export async function afterSignHook(context: any): Promise<void> {
  const { electronPlatformName, appOutDir } = context;

  if (electronPlatformName !== 'darwin') {
    return;
  }

  if (!process.env.APPLE_ID || !process.env.APPLE_APP_SPECIFIC_PASSWORD) {
    console.warn('Skipping notarization: credentials not provided');
    return;
  }

  const appName = context.packager.appInfo.productFilename;
  const appPath = join(appOutDir, `${appName}.app`);

  await notarizeMacOSApp({
    appBundleId: 'io.tachikoma.app',
    appPath,
    appleId: process.env.APPLE_ID,
    appleIdPassword: process.env.APPLE_APP_SPECIFIC_PASSWORD,
    teamId: process.env.APPLE_TEAM_ID || '',
  });
}
```

### Windows Signing Configuration

```typescript
// scripts/sign-windows.ts
import { execSync } from 'child_process';
import { join } from 'path';

interface WindowsSigningConfig {
  certificatePath: string;
  certificatePassword: string;
  timestampServer: string;
  description: string;
  descriptionUrl: string;
}

export async function signWindowsExecutable(
  exePath: string,
  config: WindowsSigningConfig
): Promise<void> {
  const { certificatePath, certificatePassword, timestampServer, description, descriptionUrl } = config;

  // Use signtool from Windows SDK
  const signtool = process.env.SIGNTOOL_PATH || 'signtool';

  const args: string[] = [
    'sign',
    '/f',
    certificatePath,
    '/p',
    certificatePassword,
    '/fd',
    'sha256',
    '/tr',
    timestampServer,
    '/td',
    'sha256',
    '/d',
    `"${description}"`,
    '/du',
    descriptionUrl,
    `"${exePath}"`,
  ];

  console.log(`Signing: ${exePath}`);
  execSync(`${signtool} ${args.join(' ')}`, { stdio: 'inherit' });

  // Verify signature
  console.log('Verifying signature...');
  execSync(`${signtool} verify /pa "${exePath}"`, { stdio: 'inherit' });
}

export async function signWindowsInstaller(
  installerPath: string,
  config: WindowsSigningConfig
): Promise<void> {
  // Sign the installer itself
  await signWindowsExecutable(installerPath, config);

  // If it's an NSIS installer, also sign the uninstaller
  const uninstallerPath = installerPath.replace('.exe', '-uninstaller.exe');
  try {
    await signWindowsExecutable(uninstallerPath, config);
  } catch {
    console.log('No separate uninstaller found');
  }
}

// electron-builder sign hook for Windows
export async function windowsSignHook(configuration: any): Promise<void> {
  const { path } = configuration;

  if (!process.env.WIN_CSC_LINK || !process.env.WIN_CSC_KEY_PASSWORD) {
    console.warn('Skipping Windows signing: credentials not provided');
    return;
  }

  await signWindowsExecutable(path, {
    certificatePath: process.env.WIN_CSC_LINK,
    certificatePassword: process.env.WIN_CSC_KEY_PASSWORD,
    timestampServer: 'http://timestamp.digicert.com',
    description: 'Tachikoma',
    descriptionUrl: 'https://tachikoma.io',
  });
}
```

### Linux Package Signing

```bash
#!/bin/bash
# scripts/sign-linux.sh

set -e

# Configuration
GPG_KEY_ID="${GPG_KEY_ID:-}"
PACKAGE_DIR="${1:-release}"

if [ -z "$GPG_KEY_ID" ]; then
    echo "GPG_KEY_ID not set, skipping signing"
    exit 0
fi

echo "Signing Linux packages..."

# Sign DEB packages
for deb in "$PACKAGE_DIR"/*.deb; do
    if [ -f "$deb" ]; then
        echo "Signing: $deb"
        dpkg-sig -k "$GPG_KEY_ID" --sign builder "$deb"
        dpkg-sig --verify "$deb"
    fi
done

# Sign RPM packages
for rpm in "$PACKAGE_DIR"/*.rpm; do
    if [ -f "$rpm" ]; then
        echo "Signing: $rpm"
        rpm --addsign "$rpm"
        rpm --checksig "$rpm"
    fi
done

# Create checksums
echo "Creating checksums..."
cd "$PACKAGE_DIR"
sha256sum *.AppImage *.deb *.rpm > SHA256SUMS.txt 2>/dev/null || true
gpg --armor --detach-sign SHA256SUMS.txt

echo "Linux package signing complete!"
```

### Certificate Management Script

```typescript
// scripts/manage-certs.ts
import { execSync } from 'child_process';
import { existsSync, mkdirSync, writeFileSync, readFileSync } from 'fs';
import { join } from 'path';

const CERTS_DIR = join(__dirname, '..', '.certs');

interface CertInfo {
  platform: 'mac' | 'win' | 'linux';
  type: 'signing' | 'distribution';
  expiresAt: Date;
  thumbprint: string;
}

export function setupCertsDirectory(): void {
  if (!existsSync(CERTS_DIR)) {
    mkdirSync(CERTS_DIR, { mode: 0o700 });
  }

  // Create .gitignore in certs directory
  writeFileSync(join(CERTS_DIR, '.gitignore'), '*\n!.gitignore\n');
}

export function importMacOSCertificate(p12Path: string, password: string): void {
  // Create a temporary keychain
  const keychainPath = join(CERTS_DIR, 'build.keychain');
  const keychainPassword = 'build-keychain-password';

  // Create keychain
  execSync(`security create-keychain -p "${keychainPassword}" "${keychainPath}"`, {
    stdio: 'inherit',
  });

  // Set keychain settings
  execSync(`security set-keychain-settings -t 3600 -u "${keychainPath}"`, {
    stdio: 'inherit',
  });

  // Unlock keychain
  execSync(`security unlock-keychain -p "${keychainPassword}" "${keychainPath}"`, {
    stdio: 'inherit',
  });

  // Import certificate
  execSync(
    `security import "${p12Path}" -k "${keychainPath}" -P "${password}" -T /usr/bin/codesign -T /usr/bin/productsign`,
    { stdio: 'inherit' }
  );

  // Add keychain to search list
  execSync(
    `security list-keychains -s "${keychainPath}" $(security list-keychains | tr -d '"')`,
    { stdio: 'inherit' }
  );

  // Set key partition list
  execSync(
    `security set-key-partition-list -S apple-tool:,apple: -s -k "${keychainPassword}" "${keychainPath}"`,
    { stdio: 'inherit' }
  );

  console.log('macOS certificate imported successfully');
}

export function importWindowsCertificate(pfxPath: string): void {
  // Write to certs directory
  const destPath = join(CERTS_DIR, 'windows-cert.pfx');

  if (existsSync(pfxPath)) {
    execSync(`cp "${pfxPath}" "${destPath}"`, { stdio: 'inherit' });
    console.log('Windows certificate copied to:', destPath);
  }
}

export function listCertificates(): void {
  console.log('\n=== macOS Certificates ===');
  try {
    execSync('security find-identity -v -p codesigning', { stdio: 'inherit' });
  } catch {
    console.log('No macOS signing certificates found');
  }

  console.log('\n=== Certificate Files ===');
  try {
    execSync(`ls -la "${CERTS_DIR}"`, { stdio: 'inherit' });
  } catch {
    console.log('Certificates directory not found');
  }
}

export function verifyCertificates(): Record<string, boolean> {
  const results: Record<string, boolean> = {
    macOSCodeSign: false,
    windowsCodeSign: false,
    linuxGPG: false,
  };

  // Check macOS
  try {
    const output = execSync('security find-identity -v -p codesigning', {
      encoding: 'utf-8',
    });
    results.macOSCodeSign = output.includes('Developer ID Application');
  } catch {
    // No macOS certificates
  }

  // Check Windows
  results.windowsCodeSign = existsSync(join(CERTS_DIR, 'windows-cert.pfx'));

  // Check Linux GPG
  try {
    execSync('gpg --list-secret-keys', { encoding: 'utf-8' });
    results.linuxGPG = true;
  } catch {
    // No GPG keys
  }

  return results;
}
```

### CI/CD Signing Integration

```yaml
# .github/workflows/sign.yml (partial)
env:
  # macOS
  CSC_LINK: ${{ secrets.MAC_CERTS_BASE64 }}
  CSC_KEY_PASSWORD: ${{ secrets.MAC_CERTS_PASSWORD }}
  APPLE_ID: ${{ secrets.APPLE_ID }}
  APPLE_APP_SPECIFIC_PASSWORD: ${{ secrets.APPLE_APP_SPECIFIC_PASSWORD }}
  APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}

  # Windows
  WIN_CSC_LINK: ${{ secrets.WIN_CERTS_BASE64 }}
  WIN_CSC_KEY_PASSWORD: ${{ secrets.WIN_CERTS_PASSWORD }}

  # Linux
  GPG_PRIVATE_KEY: ${{ secrets.GPG_PRIVATE_KEY }}
  GPG_KEY_ID: ${{ secrets.GPG_KEY_ID }}

jobs:
  sign-macos:
    runs-on: macos-latest
    steps:
      - name: Decode certificate
        run: |
          echo "$CSC_LINK" | base64 --decode > certificate.p12

      - name: Import certificate
        run: |
          security create-keychain -p "" build.keychain
          security default-keychain -s build.keychain
          security unlock-keychain -p "" build.keychain
          security import certificate.p12 -k build.keychain -P "$CSC_KEY_PASSWORD" -T /usr/bin/codesign
          security set-key-partition-list -S apple-tool:,apple: -s -k "" build.keychain

      - name: Build and sign
        run: npm run build:mac

  sign-windows:
    runs-on: windows-latest
    steps:
      - name: Decode certificate
        shell: powershell
        run: |
          [IO.File]::WriteAllBytes("certificate.pfx", [Convert]::FromBase64String($env:WIN_CSC_LINK))

      - name: Build and sign
        run: npm run build:win

  sign-linux:
    runs-on: ubuntu-latest
    steps:
      - name: Import GPG key
        run: |
          echo "$GPG_PRIVATE_KEY" | gpg --import

      - name: Build
        run: npm run build:linux

      - name: Sign packages
        run: ./scripts/sign-linux.sh release
```

---

## Testing Requirements

### Signature Verification Tests

```typescript
// scripts/verify-signatures.ts
import { execSync } from 'child_process';
import { existsSync } from 'fs';

interface VerificationResult {
  path: string;
  signed: boolean;
  valid: boolean;
  details: string;
}

export function verifyMacOSSignature(appPath: string): VerificationResult {
  try {
    const output = execSync(
      `codesign --verify --deep --strict --verbose=2 "${appPath}" 2>&1`,
      { encoding: 'utf-8' }
    );

    return {
      path: appPath,
      signed: true,
      valid: true,
      details: output,
    };
  } catch (error: any) {
    return {
      path: appPath,
      signed: error.stdout?.includes('valid on disk'),
      valid: false,
      details: error.stderr || error.message,
    };
  }
}

export function verifyWindowsSignature(exePath: string): VerificationResult {
  try {
    const output = execSync(`signtool verify /pa "${exePath}" 2>&1`, {
      encoding: 'utf-8',
    });

    return {
      path: exePath,
      signed: true,
      valid: output.includes('Successfully verified'),
      details: output,
    };
  } catch (error: any) {
    return {
      path: exePath,
      signed: false,
      valid: false,
      details: error.stderr || error.message,
    };
  }
}

export function verifyDebSignature(debPath: string): VerificationResult {
  try {
    const output = execSync(`dpkg-sig --verify "${debPath}" 2>&1`, {
      encoding: 'utf-8',
    });

    return {
      path: debPath,
      signed: true,
      valid: output.includes('GOODSIG'),
      details: output,
    };
  } catch (error: any) {
    return {
      path: debPath,
      signed: false,
      valid: false,
      details: error.stderr || error.message,
    };
  }
}
```

---

## Related Specs

- Spec 175: Build Configuration
- Spec 177: macOS Build
- Spec 178: Windows Build
- Spec 179: Linux Build
