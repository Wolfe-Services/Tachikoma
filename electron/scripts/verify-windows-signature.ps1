# Verify Windows code signature

param(
    [Parameter(Mandatory=$true)]
    [string]$FilePath
)

Write-Host "=== Windows Signature Verification ===" -ForegroundColor Cyan
Write-Host "File: $FilePath"
Write-Host ""

# Check if file exists
if (-not (Test-Path $FilePath)) {
    Write-Host "File not found: $FilePath" -ForegroundColor Red
    exit 1
}

try {
    # Get signature info using PowerShell
    $sig = Get-AuthenticodeSignature $FilePath

    Write-Host "Status: $($sig.Status)"
    
    if ($sig.SignerCertificate) {
        Write-Host "Signer: $($sig.SignerCertificate.Subject)"
        Write-Host "Issuer: $($sig.SignerCertificate.Issuer)"
        Write-Host "Valid From: $($sig.SignerCertificate.NotBefore)"
        Write-Host "Valid To: $($sig.SignerCertificate.NotAfter)"
        Write-Host "Thumbprint: $($sig.SignerCertificate.Thumbprint)"
        
        # Check if certificate is expired
        $now = Get-Date
        if ($sig.SignerCertificate.NotAfter -lt $now) {
            Write-Host "⚠ Certificate is expired!" -ForegroundColor Yellow
        }
        
        if ($sig.SignerCertificate.NotBefore -gt $now) {
            Write-Host "⚠ Certificate is not yet valid!" -ForegroundColor Yellow
        }
    }

    if ($sig.TimeStamperCertificate) {
        Write-Host ""
        Write-Host "Timestamp: $($sig.TimeStamperCertificate.Subject)"
        Write-Host "Timestamp Valid From: $($sig.TimeStamperCertificate.NotBefore)"
        Write-Host "Timestamp Valid To: $($sig.TimeStamperCertificate.NotAfter)"
    } else {
        Write-Host "⚠ No timestamp found - signature may expire with certificate!" -ForegroundColor Yellow
    }

    # Additional verification using signtool
    Write-Host ""
    Write-Host "=== Detailed Verification ===" -ForegroundColor Cyan

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

    if ($SignToolPath) {
        Write-Host "Using signtool: $SignToolPath"
        Write-Host ""
        
        # Verify with signtool
        & $SignToolPath verify /pa /v $FilePath
        $SignToolResult = $LASTEXITCODE
        
        Write-Host ""
        if ($SignToolResult -eq 0) {
            Write-Host "signtool verification: PASSED" -ForegroundColor Green
        } else {
            Write-Host "signtool verification: FAILED" -ForegroundColor Red
        }
    } else {
        Write-Host "signtool.exe not found - skipping detailed verification" -ForegroundColor Yellow
    }

    Write-Host ""
    if ($sig.Status -eq "Valid") {
        Write-Host "SIGNATURE VALID ✓" -ForegroundColor Green
        exit 0
    } else {
        Write-Host "SIGNATURE INVALID ✗" -ForegroundColor Red
        Write-Host "Status Message: $($sig.StatusMessage)"
        exit 1
    }

} catch {
    Write-Host "Verification failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}