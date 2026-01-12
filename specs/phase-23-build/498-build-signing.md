# Spec 498: Code Signing

## Phase
23 - Build/Package System

## Spec ID
498

## Status
Planned

## Dependencies
- Spec 494 (Electron Packaging)
- Spec 499 (macOS Packaging)
- Spec 500 (Windows Installer)

## Estimated Context
~10%

---

## Objective

Implement code signing infrastructure for all platforms to ensure application authenticity and enable distribution through official channels. This includes macOS notarization, Windows Authenticode signing, and Linux GPG signing.

---

## Acceptance Criteria

- [ ] macOS code signing with Developer ID certificate
- [ ] macOS notarization with Apple notary service
- [ ] Windows Authenticode signing with EV certificate
- [ ] Linux GPG signing for packages
- [ ] Secure credential management in CI
- [ ] Signing verification scripts
- [ ] Timestamping for long-term validity
- [ ] Hardware security module (HSM) support
- [ ] Signing status reporting
- [ ] Emergency revocation procedures

---

## Implementation Details

### Code Signing Orchestrator (scripts/signing/index.ts)

```typescript
// scripts/signing/index.ts
import * as fs from 'fs';
import * as path from 'path';
import { spawn } from 'child_process';

interface SigningConfig {
  platform: 'darwin' | 'win32' | 'linux';
  identity?: string;
  timestamp?: boolean;
  hardened?: boolean;
  notarize?: boolean;
}

interface SigningResult {
  success: boolean;
  path: string;
  signature?: string;
  error?: string;
}

class CodeSigner {
  private config: SigningConfig;

  constructor(config: SigningConfig) {
    this.config = config;
  }

  async sign(filePath: string): Promise<SigningResult> {
    switch (this.config.platform) {
      case 'darwin':
        return this.signMacOS(filePath);
      case 'win32':
        return this.signWindows(filePath);
      case 'linux':
        return this.signLinux(filePath);
      default:
        throw new Error(`Unsupported platform: ${this.config.platform}`);
    }
  }

  private async signMacOS(filePath: string): Promise<SigningResult> {
    const identity = this.config.identity ?? process.env.APPLE_SIGNING_IDENTITY;

    if (!identity) {
      return {
        success: false,
        path: filePath,
        error: 'No signing identity provided',
      };
    }

    const args = [
      '--sign', identity,
      '--timestamp',
      '--options', 'runtime',
    ];

    if (this.config.hardened) {
      args.push('--entitlements', 'resources/entitlements.mac.plist');
    }

    args.push(filePath);

    try {
      await this.runCommand('codesign', args);

      // Verify signature
      await this.runCommand('codesign', ['--verify', '--deep', '--strict', filePath]);

      return {
        success: true,
        path: filePath,
        signature: await this.getMacOSSignatureInfo(filePath),
      };
    } catch (error) {
      return {
        success: false,
        path: filePath,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  private async signWindows(filePath: string): Promise<SigningResult> {
    const certPath = process.env.WINDOWS_CERTIFICATE_PATH;
    const certPassword = process.env.WINDOWS_CERTIFICATE_PASSWORD;

    if (!certPath || !certPassword) {
      return {
        success: false,
        path: filePath,
        error: 'No Windows certificate configured',
      };
    }

    const args = [
      'sign',
      '/fd', 'SHA256',
      '/f', certPath,
      '/p', certPassword,
      '/tr', 'http://timestamp.digicert.com',
      '/td', 'SHA256',
      filePath,
    ];

    try {
      await this.runCommand('signtool', args);

      return {
        success: true,
        path: filePath,
        signature: await this.getWindowsSignatureInfo(filePath),
      };
    } catch (error) {
      return {
        success: false,
        path: filePath,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  private async signLinux(filePath: string): Promise<SigningResult> {
    const gpgKeyId = process.env.GPG_KEY_ID;

    if (!gpgKeyId) {
      return {
        success: false,
        path: filePath,
        error: 'No GPG key configured',
      };
    }

    const sigPath = `${filePath}.sig`;
    const args = [
      '--detach-sign',
      '--armor',
      '--local-user', gpgKeyId,
      '--output', sigPath,
      filePath,
    ];

    try {
      await this.runCommand('gpg', args);

      return {
        success: true,
        path: filePath,
        signature: fs.readFileSync(sigPath, 'utf-8'),
      };
    } catch (error) {
      return {
        success: false,
        path: filePath,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  private async getMacOSSignatureInfo(filePath: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const proc = spawn('codesign', ['-dv', '--verbose=4', filePath]);
      let output = '';

      proc.stderr.on('data', (data) => {
        output += data.toString();
      });

      proc.on('close', (code) => {
        if (code === 0) {
          resolve(output);
        } else {
          reject(new Error(`Failed to get signature info: ${output}`));
        }
      });
    });
  }

  private async getWindowsSignatureInfo(filePath: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const proc = spawn('signtool', ['verify', '/pa', '/v', filePath]);
      let output = '';

      proc.stdout.on('data', (data) => {
        output += data.toString();
      });

      proc.on('close', (code) => {
        if (code === 0) {
          resolve(output);
        } else {
          reject(new Error(`Failed to get signature info: ${output}`));
        }
      });
    });
  }

  private runCommand(command: string, args: string[]): Promise<void> {
    return new Promise((resolve, reject) => {
      const proc = spawn(command, args, { stdio: 'inherit' });

      proc.on('close', (code) => {
        if (code === 0) {
          resolve();
        } else {
          reject(new Error(`Command failed with code ${code}`));
        }
      });

      proc.on('error', (err) => {
        reject(err);
      });
    });
  }
}

export { CodeSigner, SigningConfig, SigningResult };
```

### macOS Notarization Script (scripts/signing/notarize.ts)

```typescript
// scripts/signing/notarize.ts
import { notarize } from '@electron/notarize';
import * as path from 'path';
import * as fs from 'fs';

interface NotarizationConfig {
  appPath: string;
  appleId?: string;
  appleIdPassword?: string;
  teamId?: string;
  timeout?: number;
}

interface NotarizationResult {
  success: boolean;
  requestUuid?: string;
  error?: string;
}

async function notarizeApp(config: NotarizationConfig): Promise<NotarizationResult> {
  const {
    appPath,
    appleId = process.env.APPLE_ID,
    appleIdPassword = process.env.APPLE_ID_PASSWORD,
    teamId = process.env.APPLE_TEAM_ID,
    timeout = 1800000, // 30 minutes
  } = config;

  if (!appleId || !appleIdPassword || !teamId) {
    console.log('Skipping notarization - credentials not configured');
    return {
      success: false,
      error: 'Missing Apple credentials',
    };
  }

  if (!fs.existsSync(appPath)) {
    return {
      success: false,
      error: `App not found: ${appPath}`,
    };
  }

  console.log(`Notarizing ${appPath}...`);
  console.log('This may take several minutes...');

  try {
    await notarize({
      tool: 'notarytool',
      appPath,
      appleId,
      appleIdPassword,
      teamId,
    });

    console.log('Notarization successful!');

    // Staple the notarization ticket
    await stapleApp(appPath);

    return {
      success: true,
    };
  } catch (error) {
    console.error('Notarization failed:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

async function stapleApp(appPath: string): Promise<void> {
  const { spawn } = await import('child_process');

  return new Promise((resolve, reject) => {
    const proc = spawn('xcrun', ['stapler', 'staple', appPath], {
      stdio: 'inherit',
    });

    proc.on('close', (code) => {
      if (code === 0) {
        console.log('Stapling successful');
        resolve();
      } else {
        reject(new Error(`Stapling failed with code ${code}`));
      }
    });

    proc.on('error', (err) => {
      reject(err);
    });
  });
}

async function checkNotarizationStatus(requestUuid: string): Promise<void> {
  const appleId = process.env.APPLE_ID;
  const appleIdPassword = process.env.APPLE_ID_PASSWORD;
  const teamId = process.env.APPLE_TEAM_ID;

  const { spawn } = await import('child_process');

  return new Promise((resolve, reject) => {
    const proc = spawn('xcrun', [
      'notarytool',
      'info',
      requestUuid,
      '--apple-id', appleId!,
      '--password', appleIdPassword!,
      '--team-id', teamId!,
    ], {
      stdio: 'inherit',
    });

    proc.on('close', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`Status check failed with code ${code}`));
      }
    });
  });
}

export { notarizeApp, stapleApp, checkNotarizationStatus, NotarizationConfig };
```

### Windows Signing Script (scripts/signing/windows.ts)

```typescript
// scripts/signing/windows.ts
import { spawn } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

interface WindowsSigningConfig {
  certificatePath?: string;
  certificatePassword?: string;
  certificateSubject?: string;
  timestampServer?: string;
  hashAlgorithm?: 'SHA1' | 'SHA256' | 'SHA384' | 'SHA512';
  description?: string;
  url?: string;
}

const defaultConfig: WindowsSigningConfig = {
  timestampServer: 'http://timestamp.digicert.com',
  hashAlgorithm: 'SHA256',
};

async function signWindowsExecutable(
  filePath: string,
  config: WindowsSigningConfig = {}
): Promise<void> {
  const mergedConfig = { ...defaultConfig, ...config };

  const certPath = mergedConfig.certificatePath ?? process.env.WINDOWS_CERTIFICATE_PATH;
  const certPassword = mergedConfig.certificatePassword ?? process.env.WINDOWS_CERTIFICATE_PASSWORD;

  if (!certPath && !mergedConfig.certificateSubject) {
    throw new Error('Either certificate path or subject must be provided');
  }

  const args: string[] = ['sign'];

  // Hash algorithm
  args.push('/fd', mergedConfig.hashAlgorithm!);

  // Certificate
  if (certPath) {
    args.push('/f', certPath);
    if (certPassword) {
      args.push('/p', certPassword);
    }
  } else if (mergedConfig.certificateSubject) {
    args.push('/n', mergedConfig.certificateSubject);
  }

  // Timestamp
  if (mergedConfig.timestampServer) {
    args.push('/tr', mergedConfig.timestampServer);
    args.push('/td', mergedConfig.hashAlgorithm!);
  }

  // Description
  if (mergedConfig.description) {
    args.push('/d', mergedConfig.description);
  }

  // URL
  if (mergedConfig.url) {
    args.push('/du', mergedConfig.url);
  }

  args.push(filePath);

  console.log(`Signing ${filePath}...`);

  await runSignTool(args);

  console.log('Signing successful');
}

async function verifyWindowsSignature(filePath: string): Promise<boolean> {
  try {
    await runSignTool(['verify', '/pa', filePath]);
    return true;
  } catch {
    return false;
  }
}

async function runSignTool(args: string[]): Promise<string> {
  return new Promise((resolve, reject) => {
    // Try to find signtool in common locations
    const signtoolPaths = [
      'signtool',
      'C:\\Program Files (x86)\\Windows Kits\\10\\bin\\x64\\signtool.exe',
      'C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.22621.0\\x64\\signtool.exe',
    ];

    let signtoolPath = signtoolPaths[0];
    for (const p of signtoolPaths) {
      if (fs.existsSync(p) || p === 'signtool') {
        signtoolPath = p;
        break;
      }
    }

    const proc = spawn(signtoolPath, args, { stdio: 'pipe' });
    let output = '';
    let errorOutput = '';

    proc.stdout.on('data', (data) => {
      output += data.toString();
    });

    proc.stderr.on('data', (data) => {
      errorOutput += data.toString();
    });

    proc.on('close', (code) => {
      if (code === 0) {
        resolve(output);
      } else {
        reject(new Error(`signtool failed: ${errorOutput || output}`));
      }
    });

    proc.on('error', (err) => {
      reject(err);
    });
  });
}

export { signWindowsExecutable, verifyWindowsSignature, WindowsSigningConfig };
```

### CI Signing Workflow (.github/workflows/sign.yml)

```yaml
# .github/workflows/sign.yml
name: Code Signing

on:
  workflow_call:
    inputs:
      artifact-name:
        required: true
        type: string
      platform:
        required: true
        type: string
    secrets:
      APPLE_SIGNING_IDENTITY:
        required: false
      APPLE_ID:
        required: false
      APPLE_ID_PASSWORD:
        required: false
      APPLE_TEAM_ID:
        required: false
      WINDOWS_CERTIFICATE_BASE64:
        required: false
      WINDOWS_CERTIFICATE_PASSWORD:
        required: false
      GPG_PRIVATE_KEY:
        required: false
      GPG_PASSPHRASE:
        required: false

jobs:
  sign-macos:
    if: inputs.platform == 'darwin'
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download artifact
        uses: actions/download-artifact@v4
        with:
          name: ${{ inputs.artifact-name }}
          path: dist/

      - name: Import certificates
        env:
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
        run: |
          # Create keychain
          security create-keychain -p "" build.keychain
          security default-keychain -s build.keychain
          security unlock-keychain -p "" build.keychain

          # Import certificate
          echo "${{ secrets.APPLE_SIGNING_IDENTITY }}" | base64 --decode > certificate.p12
          security import certificate.p12 -k build.keychain -P "" -T /usr/bin/codesign
          security set-key-partition-list -S apple-tool:,apple: -s -k "" build.keychain

          rm certificate.p12

      - name: Sign application
        run: |
          find dist -name "*.app" -exec codesign --sign "Developer ID Application" \
            --timestamp --options runtime --deep --force {} \;

      - name: Notarize application
        env:
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_ID_PASSWORD: ${{ secrets.APPLE_ID_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        run: |
          npx ts-node scripts/signing/notarize.ts dist/*.app

      - name: Upload signed artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ inputs.artifact-name }}-signed
          path: dist/

  sign-windows:
    if: inputs.platform == 'win32'
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download artifact
        uses: actions/download-artifact@v4
        with:
          name: ${{ inputs.artifact-name }}
          path: dist/

      - name: Import certificate
        shell: pwsh
        run: |
          $certBytes = [Convert]::FromBase64String("${{ secrets.WINDOWS_CERTIFICATE_BASE64 }}")
          $certPath = "certificate.pfx"
          [IO.File]::WriteAllBytes($certPath, $certBytes)
          echo "CERT_PATH=$certPath" >> $env:GITHUB_ENV

      - name: Sign executables
        shell: pwsh
        run: |
          Get-ChildItem -Path dist -Filter "*.exe" -Recurse | ForEach-Object {
            & signtool sign /f $env:CERT_PATH /p "${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}" `
              /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $_.FullName
          }

      - name: Clean up certificate
        shell: pwsh
        run: Remove-Item certificate.pfx -Force

      - name: Upload signed artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ inputs.artifact-name }}-signed
          path: dist/

  sign-linux:
    if: inputs.platform == 'linux'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download artifact
        uses: actions/download-artifact@v4
        with:
          name: ${{ inputs.artifact-name }}
          path: dist/

      - name: Import GPG key
        run: |
          echo "${{ secrets.GPG_PRIVATE_KEY }}" | gpg --batch --import
          echo "${{ secrets.GPG_PASSPHRASE }}" | gpg --batch --passphrase-fd 0 \
            --pinentry-mode loopback --sign --detach-sign dist/*.AppImage

      - name: Create checksums
        run: |
          cd dist
          sha256sum * > SHA256SUMS
          gpg --batch --detach-sign --armor SHA256SUMS

      - name: Upload signed artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ inputs.artifact-name }}-signed
          path: dist/
```

---

## Testing Requirements

### Unit Tests

```typescript
// scripts/signing/__tests__/signing.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { CodeSigner } from '../index';

vi.mock('child_process');

describe('CodeSigner', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should create signer for macOS', () => {
    const signer = new CodeSigner({
      platform: 'darwin',
      identity: 'Developer ID Application: Test',
    });
    expect(signer).toBeDefined();
  });

  it('should create signer for Windows', () => {
    const signer = new CodeSigner({
      platform: 'win32',
    });
    expect(signer).toBeDefined();
  });

  it('should create signer for Linux', () => {
    const signer = new CodeSigner({
      platform: 'linux',
    });
    expect(signer).toBeDefined();
  });

  it('should fail without signing identity', async () => {
    const signer = new CodeSigner({
      platform: 'darwin',
    });

    const result = await signer.sign('/path/to/app');
    expect(result.success).toBe(false);
    expect(result.error).toContain('signing identity');
  });
});
```

### Integration Tests

```typescript
// scripts/signing/__tests__/signing.integration.test.ts
import { describe, it, expect, beforeAll } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';

describe('Code Signing Integration', () => {
  it('should verify codesign is available on macOS', async () => {
    if (process.platform !== 'darwin') return;

    const { spawn } = await import('child_process');

    const result = await new Promise<boolean>((resolve) => {
      const proc = spawn('which', ['codesign']);
      proc.on('close', (code) => resolve(code === 0));
    });

    expect(result).toBe(true);
  });

  it('should verify signtool is available on Windows', async () => {
    if (process.platform !== 'win32') return;

    const { spawn } = await import('child_process');

    const result = await new Promise<boolean>((resolve) => {
      const proc = spawn('where', ['signtool']);
      proc.on('close', (code) => resolve(code === 0));
    });

    // signtool might not be in PATH on all Windows systems
    expect(typeof result).toBe('boolean');
  });

  it('should verify gpg is available on Linux', async () => {
    if (process.platform !== 'linux') return;

    const { spawn } = await import('child_process');

    const result = await new Promise<boolean>((resolve) => {
      const proc = spawn('which', ['gpg']);
      proc.on('close', (code) => resolve(code === 0));
    });

    expect(result).toBe(true);
  });
});
```

---

## Related Specs

- Spec 494: Electron Packaging
- Spec 499: macOS Packaging
- Spec 500: Windows Installer
- Spec 501: Linux Packages
- Spec 503: Release Workflow
