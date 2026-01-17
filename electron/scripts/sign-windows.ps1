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

Write-Host "=== Windows Code Signing ===" -ForegroundColor Cyan
Write-Host "Signing: $FilePath"

# Check if file exists
if (-not (Test-Path $FilePath)) {
    throw "File not found: $FilePath"
}

# Find signtool.exe
$SignToolPath = $null
$PossiblePaths = @(
    "C:\Program Files (x86)\Windows Kits\10\bin\*\x64\signtool.exe",
    "C:\Program Files (x86)\Microsoft SDKs\Windows\*\bin\x64\signtool.exe",
    "C:\Program Files\Microsoft SDKs\Windows\*\bin\x64\signtool.exe"
)

foreach ($Pattern in $PossiblePaths) {
    $Found = Get-ChildItem $Pattern -ErrorAction SilentlyContinue | Sort-Object -Descending | Select-Object -First 1
    if ($Found) {
        $SignToolPath = $Found.FullName
        break
    }
}

if (-not $SignToolPath) {
    throw "signtool.exe not found. Please install Windows SDK."
}

Write-Host "Using signtool: $SignToolPath"

# Determine signing method
if ($CertPath -and (Test-Path $CertPath)) {
    # Sign with PFX file
    Write-Host "Signing with certificate file: $CertPath"

    $args = @(
        "sign",
        "/f", $CertPath,
        "/p", $CertPassword,
        "/fd", "sha256",           # File digest algorithm
        "/tr", $TimestampServer,   # RFC 3161 timestamp server
        "/td", "sha256",           # Timestamp digest algorithm
        "/v",                      # Verbose output
        $FilePath
    )

    Write-Host "Executing: signtool $($args -join ' ')" -ForegroundColor Yellow
    & $SignToolPath $args

    if ($LASTEXITCODE -ne 0) {
        throw "Signing with certificate file failed with exit code $LASTEXITCODE"
    }
} elseif ($env:WIN_CERT_SUBJECT_NAME) {
    # Sign with certificate from Windows certificate store (for EV certificates)
    Write-Host "Signing with certificate from store: $env:WIN_CERT_SUBJECT_NAME"

    $args = @(
        "sign",
        "/n", $env:WIN_CERT_SUBJECT_NAME,  # Certificate subject name
        "/fd", "sha256",                    # File digest algorithm
        "/tr", $TimestampServer,            # RFC 3161 timestamp server
        "/td", "sha256",                    # Timestamp digest algorithm
        "/v",                               # Verbose output
        $FilePath
    )

    Write-Host "Executing: signtool $($args -join ' ')" -ForegroundColor Yellow
    & $SignToolPath $args

    if ($LASTEXITCODE -ne 0) {
        throw "Signing with certificate from store failed with exit code $LASTEXITCODE"
    }
} else {
    # Auto-select best available certificate
    Write-Host "Auto-selecting certificate from store..."

    $args = @(
        "sign",
        "/a",                      # Auto-select certificate
        "/fd", "sha256",           # File digest algorithm
        "/tr", $TimestampServer,   # RFC 3161 timestamp server
        "/td", "sha256",           # Timestamp digest algorithm
        "/v",                      # Verbose output
        $FilePath
    )

    Write-Host "Executing: signtool $($args -join ' ')" -ForegroundColor Yellow
    & $SignToolPath $args

    if ($LASTEXITCODE -ne 0) {
        throw "Auto-signing failed with exit code $LASTEXITCODE"
    }
}

Write-Host "Signing complete!" -ForegroundColor Green

# Verify signature
Write-Host ""
Write-Host "=== Verifying Signature ===" -ForegroundColor Cyan
$verifyArgs = @("verify", "/pa", "/v", $FilePath)

Write-Host "Executing: signtool $($verifyArgs -join ' ')" -ForegroundColor Yellow
& $SignToolPath $verifyArgs

if ($LASTEXITCODE -ne 0) {
    throw "Signature verification failed with exit code $LASTEXITCODE"
}

Write-Host "Verification passed!" -ForegroundColor Green