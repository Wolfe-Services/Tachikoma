# Sign using Azure Key Vault (for enterprise deployments)

param(
    [Parameter(Mandatory=$true)]
    [string]$FilePath,

    [Parameter(Mandatory=$true)]
    [string]$VaultUrl,

    [Parameter(Mandatory=$true)]
    [string]$CertificateName,

    [Parameter(Mandatory=$false)]
    [string]$TimestampServer = "http://timestamp.digicert.com"
)

$ErrorActionPreference = "Stop"

Write-Host "=== Azure Key Vault Code Signing ===" -ForegroundColor Cyan
Write-Host "File: $FilePath"
Write-Host "Vault: $VaultUrl"
Write-Host "Certificate: $CertificateName"
Write-Host ""

# Check prerequisites
if (-not (Get-Command "az" -ErrorAction SilentlyContinue)) {
    throw "Azure CLI (az) not found. Please install Azure CLI."
}

if (-not (Get-Command "AzureSignTool" -ErrorAction SilentlyContinue)) {
    Write-Host "AzureSignTool not found. Installing..."
    try {
        dotnet tool install --global AzureSignTool
    } catch {
        throw "Failed to install AzureSignTool. Please install manually: dotnet tool install --global AzureSignTool"
    }
}

# Check if file exists
if (-not (Test-Path $FilePath)) {
    throw "File not found: $FilePath"
}

# Get Azure access token
Write-Host "Getting Azure access token..."
try {
    $AccessToken = az account get-access-token --resource https://vault.azure.net --query accessToken -o tsv
    if (-not $AccessToken) {
        throw "Failed to get access token"
    }
} catch {
    throw "Failed to get Azure access token. Please ensure you are logged in with 'az login'"
}

# Sign using AzureSignTool
Write-Host "Signing with Azure Key Vault..."
try {
    AzureSignTool sign `
        --azure-key-vault-url $VaultUrl `
        --azure-key-vault-certificate $CertificateName `
        --azure-key-vault-accesstoken $AccessToken `
        --timestamp-rfc3161 $TimestampServer `
        --timestamp-digest sha256 `
        --file-digest sha256 `
        --verbose `
        $FilePath

    if ($LASTEXITCODE -ne 0) {
        throw "AzureSignTool failed with exit code $LASTEXITCODE"
    }
} catch {
    throw "Azure Key Vault signing failed: $($_.Exception.Message)"
}

Write-Host "Azure Key Vault signing complete!" -ForegroundColor Green

# Verify the signature
Write-Host ""
Write-Host "Verifying signature..."
try {
    $ScriptPath = Join-Path $PSScriptRoot "verify-windows-signature.ps1"
    & $ScriptPath -FilePath $FilePath
} catch {
    Write-Host "Signature verification failed: $($_.Exception.Message)" -ForegroundColor Red
    throw
}