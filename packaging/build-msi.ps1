# Build MSI package for LucAstra
# Requires WiX Toolset: https://wixtoolset.org/

param(
    [string]$Version = "1.0.0"
)

$ErrorActionPreference = "Stop"

Write-Host "Building MSI package for LucAstra v$Version" -ForegroundColor Green

# Check for WiX toolset
$wixPath = "${env:WIX}bin"
if (-not (Test-Path $wixPath)) {
    Write-Error "WiX Toolset not found. Please install from https://wixtoolset.org/"
    exit 1
}

# Add WiX to PATH
$env:PATH = "$wixPath;$env:PATH"

# Ensure release binary exists
if (-not (Test-Path "..\target\release\app.exe")) {
    Write-Error "Release binary not found. Run 'cargo build --release' first."
    exit 1
}

# Create placeholder icon if doesn't exist
$iconPath = "..\assets\icon.ico"
if (-not (Test-Path $iconPath)) {
    Write-Warning "Icon not found at $iconPath, creating placeholder"
    New-Item -ItemType Directory -Force -Path "..\assets" | Out-Null
    # For now, skip icon requirement or use default
}

# Compile WiX source
Write-Host "Compiling WiX source..." -ForegroundColor Cyan
& candle.exe -nologo lucastra.wxs -ext WixUIExtension

if ($LASTEXITCODE -ne 0) {
    Write-Error "WiX compilation failed"
    exit 1
}

# Link to create MSI
Write-Host "Linking MSI..." -ForegroundColor Cyan
& light.exe -nologo -out "lucastra-${Version}-x64.msi" lucastra.wixobj -ext WixUIExtension

if ($LASTEXITCODE -ne 0) {
    Write-Error "MSI linking failed"
    exit 1
}

# Clean up intermediate files
Remove-Item lucastra.wixobj -ErrorAction SilentlyContinue
Remove-Item lucastra.wixpdb -ErrorAction SilentlyContinue

Write-Host "`nMSI package created: lucastra-${Version}-x64.msi" -ForegroundColor Green
Write-Host "Install with: msiexec /i lucastra-${Version}-x64.msi" -ForegroundColor Yellow
