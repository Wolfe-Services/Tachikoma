# Sign all Windows executables and DLLs

param(
    [Parameter(Mandatory=$true)]
    [string]$Directory,

    [Parameter(Mandatory=$false)]
    [string[]]$Extensions = @("*.exe", "*.dll", "*.node")
)

$ErrorActionPreference = "Stop"

Write-Host "=== Windows Batch Signing ===" -ForegroundColor Cyan
Write-Host "Directory: $Directory"
Write-Host "Extensions: $($Extensions -join ', ')"
Write-Host ""

# Validate directory
if (-not (Test-Path $Directory)) {
    throw "Directory not found: $Directory"
}

# Find all files to sign
$FilesToSign = @()
foreach ($Extension in $Extensions) {
    $Files = Get-ChildItem -Path $Directory -Recurse -Include $Extension | Where-Object { -not $_.PSIsContainer }
    $FilesToSign += $Files
}

Write-Host "Found $($FilesToSign.Count) files to sign"

if ($FilesToSign.Count -eq 0) {
    Write-Host "No files found to sign" -ForegroundColor Yellow
    exit 0
}

# Sign each file
$SuccessCount = 0
$FailureCount = 0
$Failures = @()

foreach ($File in $FilesToSign) {
    Write-Host ""
    Write-Host "Signing: $($File.Name)" -ForegroundColor Yellow
    Write-Host "Path: $($File.FullName)"

    try {
        # Call the single file signing script
        $ScriptPath = Join-Path $PSScriptRoot "sign-windows.ps1"
        & $ScriptPath -FilePath $File.FullName

        $SuccessCount++
        Write-Host "✓ Success" -ForegroundColor Green
    } catch {
        $FailureCount++
        $Failures += @{
            File = $File.Name
            Path = $File.FullName
            Error = $_.Exception.Message
        }
        Write-Host "✗ Failed: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Summary
Write-Host ""
Write-Host "=== Batch Signing Summary ===" -ForegroundColor Cyan
Write-Host "Total files: $($FilesToSign.Count)"
Write-Host "Successful: $SuccessCount" -ForegroundColor Green
Write-Host "Failed: $FailureCount" -ForegroundColor Red

if ($FailureCount -gt 0) {
    Write-Host ""
    Write-Host "Failed files:" -ForegroundColor Red
    foreach ($Failure in $Failures) {
        Write-Host "  - $($Failure.File): $($Failure.Error)" -ForegroundColor Red
    }

    # Exit with error if any files failed to sign
    exit 1
} else {
    Write-Host ""
    Write-Host "All files signed successfully!" -ForegroundColor Green
    exit 0
}