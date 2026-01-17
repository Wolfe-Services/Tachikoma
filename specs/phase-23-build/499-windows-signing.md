# 499 - Windows Code Signing

**Phase:** 23 - Build & Distribution
**Spec ID:** 499
**Status:** Planned
**Dependencies:** 498-windows-installer
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Configure Windows code signing using EV (Extended Validation) or standard code signing certificates to establish publisher identity and avoid SmartScreen warnings.

---

## Acceptance Criteria

- [x] Executable files signed with Authenticode
- [x] Installer signed
- [x] Timestamp server used for longevity
- [x] SmartScreen reputation established
- [x] CI/CD signing automated
- [x] Signature verification passes

---

## Implementation Details

### 1. Signing Configuration

Update `electron/electron-builder.config.js`:

```javascript
win: {
  // ... other config

  // Code signing
  signingHashAlgorithms: ['sha256'],
  sign: async (configuration) => {
    // Custom signing logic if needed
    // Return undefined to use default signing
    return undefined;
  },

  // Certificate configuration (via environment)
  // CSC_LINK - path to .pfx or base64-encoded certificate
  // CSC_KEY_PASSWORD - certificate password

  // Timestamp server (required for long-term validity)
  rfc3161TimeStampServer: 'http://timestamp.digicert.com',

  // Publisher name (must match certificate)
  publisherName: 'Your Company Name',

  // Request certificate from Windows store (for EV)
  certificateSubjectName: 'Your Company Name',
},
```

### 2. Manual Signing Script

Create `electron/scripts/sign-windows.ps1`:

```powershell
# Windows Code Signing Script

param(
    [Parameter(Mandatory=$true)]
    [string]$FilePath,

    [Parameter(Mandatory=$false)]
    [string]$CertPath = $env:CSC_LINK,

    [Parameter(Mandatory=$false)]
    [string]$CertPassword = $env:CSC_KEY_PASSWORD,

    [Parameter(Mandatory=$false)]
    [string]$TimestampServer = "http://timestamp.digicert.com"
)

$ErrorActionPreference = "Stop"

Write-Host "Signing: $FilePath"

# Check if file exists
if (-not (Test-Path $FilePath)) {
    throw "File not found: $FilePath"
}

# Determine signing method
if ($CertPath) {
    # Sign with PFX file
    Write-Host "Signing with certificate file..."

    $SignToolPath = (Get-ChildItem "C:\Program Files (x86)\Windows Kits\*\bin\*\x64\signtool.exe" |
                     Sort-Object -Descending | Select-Object -First 1).FullName

    if (-not $SignToolPath) {
        throw "signtool.exe not found. Install Windows SDK."
    }

    # Build arguments
    $args = @(
        "sign",
        "/f", $CertPath,
        "/p", $CertPassword,
        "/fd", "sha256",
        "/tr", $TimestampServer,
        "/td", "sha256",
        "/v",
        $FilePath
    )

    & $SignToolPath $args

    if ($LASTEXITCODE -ne 0) {
        throw "Signing failed with exit code $LASTEXITCODE"
    }
} else {
    # Sign with certificate from Windows store (for EV certificates)
    Write-Host "Signing with certificate from store..."

    $SignToolPath = (Get-ChildItem "C:\Program Files (x86)\Windows Kits\*\bin\*\x64\signtool.exe" |
                     Sort-Object -Descending | Select-Object -First 1).FullName

    $args = @(
        "sign",
        "/a",  # Auto-select certificate
        "/fd", "sha256",
        "/tr", $TimestampServer,
        "/td", "sha256",
        "/v",
        $FilePath
    )

    & $SignToolPath $args

    if ($LASTEXITCODE -ne 0) {
        throw "Signing failed with exit code $LASTEXITCODE"
    }
}

Write-Host "Signing complete!"

# Verify signature
Write-Host "Verifying signature..."
& $SignToolPath verify /pa /v $FilePath

if ($LASTEXITCODE -ne 0) {
    throw "Signature verification failed"
}

Write-Host "Verification passed!"
```

### 3. Batch Signing Script

Create `electron/scripts/sign-all-windows.ps1`:

```powershell
# Sign all Windows executables

param(
    [Parameter(Mandatory=$true)]
    [string]$Directory
)

$ErrorActionPreference = "Stop"

# Find all executables
$exeFiles = Get-ChildItem -Path $Directory -Recurse -Include "*.exe","*.dll" |
            Where-Object { -not $_.PSIsContainer }

Write-Host "Found $($exeFiles.Count) files to sign"

foreach ($file in $exeFiles) {
    Write-Host "Signing: $($file.FullName)"

    try {
        & "$PSScriptRoot\sign-windows.ps1" -FilePath $file.FullName
    } catch {
        Write-Warning "Failed to sign $($file.Name): $_"
    }
}

Write-Host "Batch signing complete!"
```

### 4. CI/CD Signing

Create `.github/workflows/sign-windows.yml`:

```yaml
name: Sign Windows

on:
  workflow_call:
    inputs:
      artifact-name:
        required: true
        type: string
    secrets:
      WINDOWS_CERTIFICATE:
        required: true
      WINDOWS_CERTIFICATE_PASSWORD:
        required: true

jobs:
  sign:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download artifact
        uses: actions/download-artifact@v4
        with:
          name: ${{ inputs.artifact-name }}
          path: ./unsigned

      - name: Import certificate
        shell: powershell
        env:
          CERTIFICATE: ${{ secrets.WINDOWS_CERTIFICATE }}
          PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
        run: |
          $certBytes = [Convert]::FromBase64String($env:CERTIFICATE)
          $certPath = "$env:TEMP\certificate.pfx"
          [IO.File]::WriteAllBytes($certPath, $certBytes)
          $env:CSC_LINK = $certPath
          $env:CSC_KEY_PASSWORD = $env:PASSWORD

      - name: Sign executables
        shell: powershell
        run: |
          Get-ChildItem -Path ./unsigned -Recurse -Include "*.exe" | ForEach-Object {
            Write-Host "Signing: $($_.FullName)"
            signtool sign /f "$env:TEMP\certificate.pfx" /p "$env:WINDOWS_CERTIFICATE_PASSWORD" /fd sha256 /tr http://timestamp.digicert.com /td sha256 $_.FullName
          }

      - name: Verify signatures
        shell: powershell
        run: |
          Get-ChildItem -Path ./unsigned -Recurse -Include "*.exe" | ForEach-Object {
            Write-Host "Verifying: $($_.FullName)"
            signtool verify /pa $_.FullName
          }

      - name: Upload signed artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ inputs.artifact-name }}-signed
          path: ./unsigned
```

### 5. Azure Key Vault Signing (Enterprise)

For enterprise deployments using Azure Key Vault:

Create `electron/scripts/sign-azure-kv.ps1`:

```powershell
# Sign using Azure Key Vault

param(
    [Parameter(Mandatory=$true)]
    [string]$FilePath,

    [Parameter(Mandatory=$true)]
    [string]$VaultUrl,

    [Parameter(Mandatory=$true)]
    [string]$CertificateName
)

$ErrorActionPreference = "Stop"

# Requires Azure CLI and AzureSignTool
# Install: dotnet tool install --global AzureSignTool

Write-Host "Signing with Azure Key Vault..."
Write-Host "Vault: $VaultUrl"
Write-Host "Certificate: $CertificateName"

# Sign using AzureSignTool
AzureSignTool sign `
    --azure-key-vault-url $VaultUrl `
    --azure-key-vault-certificate $CertificateName `
    --azure-key-vault-accesstoken (az account get-access-token --resource https://vault.azure.net --query accessToken -o tsv) `
    --timestamp-rfc3161 http://timestamp.digicert.com `
    --timestamp-digest sha256 `
    --file-digest sha256 `
    --verbose `
    $FilePath

if ($LASTEXITCODE -ne 0) {
    throw "Azure signing failed"
}

Write-Host "Azure Key Vault signing complete!"
```

### 6. Signature Verification Script

Create `electron/scripts/verify-windows-signature.ps1`:

```powershell
# Verify Windows code signature

param(
    [Parameter(Mandatory=$true)]
    [string]$FilePath
)

Write-Host "=== Windows Signature Verification ===" -ForegroundColor Cyan
Write-Host "File: $FilePath"
Write-Host ""

# Get signature info
$sig = Get-AuthenticodeSignature $FilePath

Write-Host "Status: $($sig.Status)"
Write-Host "Signer: $($sig.SignerCertificate.Subject)"
Write-Host "Issuer: $($sig.SignerCertificate.Issuer)"
Write-Host "Valid From: $($sig.SignerCertificate.NotBefore)"
Write-Host "Valid To: $($sig.SignerCertificate.NotAfter)"

if ($sig.TimeStamperCertificate) {
    Write-Host ""
    Write-Host "Timestamp: $($sig.TimeStamperCertificate.Subject)"
}

Write-Host ""
if ($sig.Status -eq "Valid") {
    Write-Host "SIGNATURE VALID" -ForegroundColor Green
    exit 0
} else {
    Write-Host "SIGNATURE INVALID: $($sig.StatusMessage)" -ForegroundColor Red
    exit 1
}
```

---

## Testing Requirements

1. Executables have valid Authenticode signatures
2. Timestamp is properly applied
3. SmartScreen doesn't show warnings
4. Signature survives Windows updates
5. CI signing works correctly

---

## Related Specs

- Depends on: [498-windows-installer.md](498-windows-installer.md)
- Next: [500-linux-appimage.md](500-linux-appimage.md)
- Related: [496-macos-signing.md](496-macos-signing.md)
