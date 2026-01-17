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