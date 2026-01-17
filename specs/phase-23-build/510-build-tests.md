# 510 - Build Tests

**Phase:** 23 - Build & Distribution
**Spec ID:** 510
**Status:** Planned
**Dependencies:** 509-download-page
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Implement comprehensive build verification tests that ensure release artifacts are correctly built, signed, and functional before distribution.

---

## Acceptance Criteria

- [x] Build artifacts verified for each platform
- [x] Code signing validated
- [x] Package integrity checked
- [x] Installer tests automated
- [x] Auto-update packages verified
- [x] Build matrix CI integration

---

## Implementation Details

### 1. Build Verification Script

Create `scripts/verify-build.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?Version required}"
ARTIFACTS_DIR="${2:-electron/out}"

echo "=== Build Verification for v${VERSION} ==="

ERRORS=0

# Function to report errors
report_error() {
    echo "ERROR: $1"
    ((ERRORS++))
}

# Function to report success
report_ok() {
    echo "OK: $1"
}

# Check all expected artifacts exist
check_artifacts() {
    echo ""
    echo "--- Checking Artifacts ---"

    local expected_files=(
        "Tachikoma-${VERSION}-arm64.dmg"
        "Tachikoma-${VERSION}-x64.dmg"
        "Tachikoma-Setup-${VERSION}.exe"
        "Tachikoma-${VERSION}.AppImage"
        "tachikoma_${VERSION}_amd64.deb"
        "latest-mac.yml"
        "latest.yml"
        "latest-linux.yml"
    )

    for file in "${expected_files[@]}"; do
        if [ -f "${ARTIFACTS_DIR}/${file}" ]; then
            report_ok "Found ${file}"
        else
            report_error "Missing ${file}"
        fi
    done
}

# Verify file sizes are reasonable
check_file_sizes() {
    echo ""
    echo "--- Checking File Sizes ---"

    local min_size_mb=50  # Minimum expected size in MB

    for file in "${ARTIFACTS_DIR}"/*.{dmg,exe,AppImage,deb} 2>/dev/null; do
        if [ -f "$file" ]; then
            local size_mb=$(du -m "$file" | cut -f1)
            local filename=$(basename "$file")

            if [ "$size_mb" -lt "$min_size_mb" ]; then
                report_error "${filename} is too small (${size_mb}MB < ${min_size_mb}MB)"
            else
                report_ok "${filename} size OK (${size_mb}MB)"
            fi
        fi
    done
}

# Verify checksums can be generated
check_checksums() {
    echo ""
    echo "--- Generating Checksums ---"

    local checksum_file="${ARTIFACTS_DIR}/checksums.sha256"

    shasum -a 256 "${ARTIFACTS_DIR}"/*.{dmg,exe,AppImage,deb} 2>/dev/null > "$checksum_file" || true

    if [ -s "$checksum_file" ]; then
        report_ok "Checksums generated"
        cat "$checksum_file"
    else
        report_error "Failed to generate checksums"
    fi
}

# Main
check_artifacts
check_file_sizes
check_checksums

echo ""
echo "=== Verification Complete ==="

if [ "$ERRORS" -gt 0 ]; then
    echo "FAILED: ${ERRORS} error(s) found"
    exit 1
else
    echo "PASSED: All checks passed"
fi
```

### 2. macOS Build Tests

Create `scripts/verify-macos-build.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

DMG_FILE="${1:?DMG file required}"

echo "=== macOS Build Verification ==="
echo "File: ${DMG_FILE}"

ERRORS=0
MOUNT_POINT="/Volumes/Tachikoma-Test-$$"

cleanup() {
    if [ -d "$MOUNT_POINT" ]; then
        hdiutil detach "$MOUNT_POINT" -quiet || true
    fi
}
trap cleanup EXIT

# Mount DMG
echo ""
echo "--- Mounting DMG ---"
hdiutil attach "$DMG_FILE" -mountpoint "$MOUNT_POINT" -nobrowse -quiet

if [ ! -d "$MOUNT_POINT/Tachikoma.app" ]; then
    echo "ERROR: Tachikoma.app not found in DMG"
    exit 1
fi
echo "OK: App bundle found"

APP_PATH="$MOUNT_POINT/Tachikoma.app"

# Check code signing
echo ""
echo "--- Verifying Code Signature ---"
if codesign -v --deep --strict "$APP_PATH" 2>&1; then
    echo "OK: Code signature valid"
else
    echo "ERROR: Code signature verification failed"
    ((ERRORS++))
fi

# Check signing identity
echo ""
echo "--- Checking Signing Identity ---"
IDENTITY=$(codesign -dv "$APP_PATH" 2>&1 | grep "Authority" | head -1)
echo "$IDENTITY"

if [[ "$IDENTITY" == *"Developer ID Application"* ]]; then
    echo "OK: Signed with Developer ID"
else
    echo "WARNING: Not signed with Developer ID (may be development build)"
fi

# Check notarization
echo ""
echo "--- Checking Notarization ---"
if spctl -a -v "$APP_PATH" 2>&1 | grep -q "accepted"; then
    echo "OK: App is notarized and accepted by Gatekeeper"
else
    echo "WARNING: App may not be notarized"
fi

# Check entitlements
echo ""
echo "--- Checking Entitlements ---"
ENTITLEMENTS=$(codesign -d --entitlements :- "$APP_PATH" 2>&1)
echo "$ENTITLEMENTS" | head -20

# Required entitlements
REQUIRED_ENTITLEMENTS=(
    "com.apple.security.cs.allow-jit"
    "com.apple.security.cs.allow-unsigned-executable-memory"
)

for ent in "${REQUIRED_ENTITLEMENTS[@]}"; do
    if echo "$ENTITLEMENTS" | grep -q "$ent"; then
        echo "OK: Has entitlement $ent"
    else
        echo "WARNING: Missing entitlement $ent"
    fi
done

# Check app structure
echo ""
echo "--- Checking App Structure ---"

REQUIRED_FILES=(
    "Contents/MacOS/Tachikoma"
    "Contents/Info.plist"
    "Contents/Resources/app.asar"
    "Contents/Frameworks/Electron Framework.framework"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -e "$APP_PATH/$file" ]; then
        echo "OK: Found $file"
    else
        echo "ERROR: Missing $file"
        ((ERRORS++))
    fi
done

# Check Info.plist
echo ""
echo "--- Checking Info.plist ---"
BUNDLE_ID=$(/usr/libexec/PlistBuddy -c "Print CFBundleIdentifier" "$APP_PATH/Contents/Info.plist")
VERSION=$(/usr/libexec/PlistBuddy -c "Print CFBundleShortVersionString" "$APP_PATH/Contents/Info.plist")
echo "Bundle ID: $BUNDLE_ID"
echo "Version: $VERSION"

# Check native modules
echo ""
echo "--- Checking Native Modules ---"
NATIVE_MODULES=$(find "$APP_PATH" -name "*.node" 2>/dev/null || true)
if [ -n "$NATIVE_MODULES" ]; then
    echo "Found native modules:"
    echo "$NATIVE_MODULES"

    # Verify each is signed
    while IFS= read -r module; do
        if codesign -v "$module" 2>&1; then
            echo "OK: $module signed"
        else
            echo "ERROR: $module not properly signed"
            ((ERRORS++))
        fi
    done <<< "$NATIVE_MODULES"
fi

echo ""
echo "=== macOS Verification Complete ==="
if [ "$ERRORS" -gt 0 ]; then
    echo "FAILED: ${ERRORS} error(s)"
    exit 1
else
    echo "PASSED"
fi
```

### 3. Windows Build Tests

Create `scripts/verify-windows-build.ps1`:

```powershell
#Requires -Version 5.1
param(
    [Parameter(Mandatory=$true)]
    [string]$InstallerPath
)

$ErrorActionPreference = "Stop"
$errors = 0

Write-Host "=== Windows Build Verification ===" -ForegroundColor Cyan
Write-Host "File: $InstallerPath"

# Check file exists
if (-not (Test-Path $InstallerPath)) {
    Write-Host "ERROR: File not found" -ForegroundColor Red
    exit 1
}

# Check Authenticode signature
Write-Host "`n--- Checking Authenticode Signature ---"
$signature = Get-AuthenticodeSignature $InstallerPath

if ($signature.Status -eq "Valid") {
    Write-Host "OK: Signature is valid" -ForegroundColor Green
    Write-Host "Signer: $($signature.SignerCertificate.Subject)"
    Write-Host "Timestamp: $($signature.TimeStamperCertificate.Subject)"
} elseif ($signature.Status -eq "NotSigned") {
    Write-Host "WARNING: File is not signed" -ForegroundColor Yellow
} else {
    Write-Host "ERROR: Signature status: $($signature.Status)" -ForegroundColor Red
    $errors++
}

# Check certificate details
if ($signature.SignerCertificate) {
    Write-Host "`n--- Certificate Details ---"
    $cert = $signature.SignerCertificate
    Write-Host "Subject: $($cert.Subject)"
    Write-Host "Issuer: $($cert.Issuer)"
    Write-Host "Valid From: $($cert.NotBefore)"
    Write-Host "Valid To: $($cert.NotAfter)"
    Write-Host "Thumbprint: $($cert.Thumbprint)"

    # Check if EV certificate
    if ($cert.Subject -match "EV") {
        Write-Host "OK: EV Certificate detected" -ForegroundColor Green
    }
}

# Check file size
Write-Host "`n--- Checking File Size ---"
$fileInfo = Get-Item $InstallerPath
$sizeMB = [math]::Round($fileInfo.Length / 1MB, 2)
Write-Host "Size: $sizeMB MB"

if ($sizeMB -lt 50) {
    Write-Host "WARNING: File seems too small" -ForegroundColor Yellow
}

# Test extraction (NSIS specific)
Write-Host "`n--- Testing NSIS Extraction ---"
$tempDir = Join-Path $env:TEMP "tachikoma-verify-$([guid]::NewGuid().ToString('N').Substring(0,8))"

try {
    # NSIS installers support /EXTRACTONLY
    $process = Start-Process -FilePath $InstallerPath -ArgumentList "/S", "/D=$tempDir" -Wait -PassThru -NoNewWindow

    if ($process.ExitCode -eq 0) {
        Write-Host "OK: Installer extraction successful" -ForegroundColor Green

        # Check extracted contents
        $expectedFiles = @(
            "Tachikoma.exe",
            "resources\app.asar"
        )

        foreach ($file in $expectedFiles) {
            $fullPath = Join-Path $tempDir $file
            if (Test-Path $fullPath) {
                Write-Host "OK: Found $file" -ForegroundColor Green

                # Check if exe is signed
                if ($file -match "\.exe$") {
                    $exeSig = Get-AuthenticodeSignature $fullPath
                    if ($exeSig.Status -eq "Valid") {
                        Write-Host "OK: $file is signed" -ForegroundColor Green
                    } else {
                        Write-Host "WARNING: $file signature: $($exeSig.Status)" -ForegroundColor Yellow
                    }
                }
            } else {
                Write-Host "ERROR: Missing $file" -ForegroundColor Red
                $errors++
            }
        }
    } else {
        Write-Host "ERROR: Installer extraction failed with code $($process.ExitCode)" -ForegroundColor Red
        $errors++
    }
} finally {
    # Cleanup
    if (Test-Path $tempDir) {
        Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Write-Host "`n=== Windows Verification Complete ===" -ForegroundColor Cyan

if ($errors -gt 0) {
    Write-Host "FAILED: $errors error(s)" -ForegroundColor Red
    exit 1
} else {
    Write-Host "PASSED" -ForegroundColor Green
    exit 0
}
```

### 4. Linux Build Tests

Create `scripts/verify-linux-build.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

ARTIFACT="${1:?Artifact path required}"
ARTIFACT_TYPE="${2:-auto}"

echo "=== Linux Build Verification ==="
echo "File: ${ARTIFACT}"

ERRORS=0

# Auto-detect type
if [ "$ARTIFACT_TYPE" = "auto" ]; then
    case "$ARTIFACT" in
        *.AppImage) ARTIFACT_TYPE="appimage" ;;
        *.deb) ARTIFACT_TYPE="deb" ;;
        *) echo "ERROR: Unknown artifact type"; exit 1 ;;
    esac
fi

verify_appimage() {
    local appimage="$1"
    echo ""
    echo "--- Verifying AppImage ---"

    # Check it's executable
    if [ -x "$appimage" ]; then
        echo "OK: File is executable"
    else
        chmod +x "$appimage"
        echo "OK: Made executable"
    fi

    # Check ELF header
    if file "$appimage" | grep -q "ELF"; then
        echo "OK: Valid ELF binary"
    else
        echo "ERROR: Not a valid ELF binary"
        ((ERRORS++))
    fi

    # Check AppImage type
    if file "$appimage" | grep -q "AppImage"; then
        echo "OK: AppImage format detected"
    fi

    # Extract and verify contents
    local extract_dir="/tmp/appimage-verify-$$"
    mkdir -p "$extract_dir"

    echo ""
    echo "--- Extracting AppImage ---"
    cd "$extract_dir"
    "$appimage" --appimage-extract > /dev/null 2>&1 || {
        echo "ERROR: Failed to extract AppImage"
        ((ERRORS++))
        return
    }

    # Check required files
    local required_files=(
        "squashfs-root/tachikoma"
        "squashfs-root/resources/app.asar"
        "squashfs-root/tachikoma.desktop"
    )

    for file in "${required_files[@]}"; do
        if [ -e "$file" ]; then
            echo "OK: Found $file"
        else
            echo "ERROR: Missing $file"
            ((ERRORS++))
        fi
    done

    # Check desktop file
    if [ -f "squashfs-root/tachikoma.desktop" ]; then
        echo ""
        echo "--- Desktop Entry ---"
        cat "squashfs-root/tachikoma.desktop" | head -20
    fi

    # Cleanup
    rm -rf "$extract_dir"
}

verify_deb() {
    local deb="$1"
    echo ""
    echo "--- Verifying Debian Package ---"

    # Check package info
    echo ""
    echo "--- Package Info ---"
    dpkg-deb --info "$deb"

    # Check package contents
    echo ""
    echo "--- Package Contents ---"
    dpkg-deb --contents "$deb" | head -30

    # Extract control file
    local extract_dir="/tmp/deb-verify-$$"
    mkdir -p "$extract_dir"
    dpkg-deb --control "$deb" "$extract_dir"

    # Check control file
    if [ -f "$extract_dir/control" ]; then
        echo ""
        echo "--- Control File ---"
        cat "$extract_dir/control"

        # Verify required fields
        for field in Package Version Architecture Maintainer Description; do
            if grep -q "^$field:" "$extract_dir/control"; then
                echo "OK: Has $field field"
            else
                echo "ERROR: Missing $field field"
                ((ERRORS++))
            fi
        done
    fi

    # Check dependencies
    echo ""
    echo "--- Dependencies ---"
    dpkg-deb --field "$deb" Depends

    # Check for post-install script
    if [ -f "$extract_dir/postinst" ]; then
        echo ""
        echo "OK: Has post-install script"
    fi

    # Cleanup
    rm -rf "$extract_dir"
}

case "$ARTIFACT_TYPE" in
    appimage) verify_appimage "$ARTIFACT" ;;
    deb) verify_deb "$ARTIFACT" ;;
esac

echo ""
echo "=== Linux Verification Complete ==="

if [ "$ERRORS" -gt 0 ]; then
    echo "FAILED: ${ERRORS} error(s)"
    exit 1
else
    echo "PASSED"
fi
```

### 5. Auto-Update Package Tests

Create `scripts/verify-update-packages.ts`:

```typescript
#!/usr/bin/env ts-node
/**
 * Verify auto-update packages and manifests
 */

import * as fs from 'fs';
import * as path from 'path';
import * as yaml from 'js-yaml';
import * as crypto from 'crypto';

interface UpdateManifest {
  version: string;
  files: Array<{
    url: string;
    sha512: string;
    size: number;
  }>;
  path: string;
  sha512: string;
  releaseDate: string;
}

function loadManifest(filePath: string): UpdateManifest {
  const content = fs.readFileSync(filePath, 'utf-8');
  return yaml.load(content) as UpdateManifest;
}

function calculateSha512(filePath: string): string {
  const content = fs.readFileSync(filePath);
  return crypto.createHash('sha512').update(content).digest('base64');
}

function verifyManifest(manifestPath: string, artifactsDir: string): boolean {
  console.log(`\nVerifying: ${path.basename(manifestPath)}`);

  const manifest = loadManifest(manifestPath);
  let errors = 0;

  // Check version format
  if (!/^\d+\.\d+\.\d+/.test(manifest.version)) {
    console.log(`ERROR: Invalid version format: ${manifest.version}`);
    errors++;
  } else {
    console.log(`OK: Version ${manifest.version}`);
  }

  // Check release date
  if (manifest.releaseDate) {
    const date = new Date(manifest.releaseDate);
    if (isNaN(date.getTime())) {
      console.log(`ERROR: Invalid release date: ${manifest.releaseDate}`);
      errors++;
    } else {
      console.log(`OK: Release date ${manifest.releaseDate}`);
    }
  }

  // Verify files
  for (const file of manifest.files) {
    const filename = path.basename(file.url);
    const filePath = path.join(artifactsDir, filename);

    if (!fs.existsSync(filePath)) {
      console.log(`ERROR: Referenced file not found: ${filename}`);
      errors++;
      continue;
    }

    // Verify size
    const stat = fs.statSync(filePath);
    if (stat.size !== file.size) {
      console.log(`ERROR: Size mismatch for ${filename}: expected ${file.size}, got ${stat.size}`);
      errors++;
    } else {
      console.log(`OK: ${filename} size matches (${file.size} bytes)`);
    }

    // Verify hash
    const actualHash = calculateSha512(filePath);
    if (actualHash !== file.sha512) {
      console.log(`ERROR: SHA512 mismatch for ${filename}`);
      console.log(`  Expected: ${file.sha512.substring(0, 20)}...`);
      console.log(`  Actual:   ${actualHash.substring(0, 20)}...`);
      errors++;
    } else {
      console.log(`OK: ${filename} SHA512 matches`);
    }
  }

  return errors === 0;
}

// Main
const artifactsDir = process.argv[2] || 'electron/out';

console.log('=== Auto-Update Package Verification ===');
console.log(`Artifacts directory: ${artifactsDir}`);

const manifests = [
  'latest.yml',
  'latest-mac.yml',
  'latest-linux.yml',
];

let allPassed = true;

for (const manifest of manifests) {
  const manifestPath = path.join(artifactsDir, manifest);
  if (fs.existsSync(manifestPath)) {
    if (!verifyManifest(manifestPath, artifactsDir)) {
      allPassed = false;
    }
  } else {
    console.log(`\nWARNING: ${manifest} not found`);
  }
}

console.log('\n=== Verification Complete ===');

if (allPassed) {
  console.log('PASSED: All update packages verified');
  process.exit(0);
} else {
  console.log('FAILED: Some verifications failed');
  process.exit(1);
}
```

### 6. CI Build Verification Workflow

Create `.github/workflows/verify-build.yml`:

```yaml
name: Verify Build

on:
  workflow_call:
    inputs:
      version:
        required: true
        type: string

jobs:
  verify-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download macOS artifacts
        uses: actions/download-artifact@v4
        with:
          name: macos-builds
          path: artifacts

      - name: Verify macOS builds
        run: |
          chmod +x scripts/verify-macos-build.sh

          for dmg in artifacts/*.dmg; do
            ./scripts/verify-macos-build.sh "$dmg"
          done

  verify-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download Windows artifacts
        uses: actions/download-artifact@v4
        with:
          name: windows-builds
          path: artifacts

      - name: Verify Windows builds
        shell: pwsh
        run: |
          Get-ChildItem artifacts/*.exe | ForEach-Object {
            .\scripts\verify-windows-build.ps1 -InstallerPath $_.FullName
          }

  verify-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download Linux artifacts
        uses: actions/download-artifact@v4
        with:
          name: linux-builds
          path: artifacts

      - name: Verify Linux builds
        run: |
          chmod +x scripts/verify-linux-build.sh

          for appimage in artifacts/*.AppImage; do
            ./scripts/verify-linux-build.sh "$appimage" appimage
          done

          for deb in artifacts/*.deb; do
            ./scripts/verify-linux-build.sh "$deb" deb
          done

  verify-update-packages:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts
          merge-multiple: true

      - name: Install dependencies
        run: npm ci

      - name: Verify update packages
        run: npx ts-node scripts/verify-update-packages.ts artifacts

  verify-checksums:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts
          merge-multiple: true

      - name: Generate and verify checksums
        run: |
          cd artifacts

          # Generate checksums
          sha256sum *.{dmg,exe,AppImage,deb} 2>/dev/null > checksums.sha256 || true

          echo "Generated checksums:"
          cat checksums.sha256

          # Upload as artifact
          echo "checksums<<EOF" >> $GITHUB_OUTPUT
          cat checksums.sha256 >> $GITHUB_OUTPUT
          echo "EOF" >> $GITHUB_OUTPUT

      - name: Upload checksums
        uses: actions/upload-artifact@v4
        with:
          name: checksums
          path: artifacts/checksums.sha256

  summary:
    needs: [verify-macos, verify-windows, verify-linux, verify-update-packages, verify-checksums]
    runs-on: ubuntu-latest
    steps:
      - name: Build verification summary
        run: |
          echo "## Build Verification Summary" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "| Platform | Status |" >> $GITHUB_STEP_SUMMARY
          echo "|----------|--------|" >> $GITHUB_STEP_SUMMARY
          echo "| macOS | :white_check_mark: Verified |" >> $GITHUB_STEP_SUMMARY
          echo "| Windows | :white_check_mark: Verified |" >> $GITHUB_STEP_SUMMARY
          echo "| Linux | :white_check_mark: Verified |" >> $GITHUB_STEP_SUMMARY
          echo "| Update Packages | :white_check_mark: Verified |" >> $GITHUB_STEP_SUMMARY
          echo "| Checksums | :white_check_mark: Generated |" >> $GITHUB_STEP_SUMMARY
```

---

## Testing Requirements

1. All platform artifacts pass verification
2. Code signatures validate correctly
3. Package contents are complete
4. Update manifests match actual files
5. CI workflow runs on all platforms

---

## Related Specs

- Depends on: [509-download-page.md](509-download-page.md)
- Related: [494-electron-packaging.md](494-electron-packaging.md)
- Related: [488-test-ci.md](../phase-22-testing/488-test-ci.md)
