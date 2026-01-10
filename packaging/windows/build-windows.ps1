# Oxide Browser - Windows Build Script
# Builds installer and portable package for Windows 11 (64-bit)

param(
    [switch]$Portable,
    [switch]$Installer,
    [switch]$All,
    [switch]$Clean,
    [string]$SignCert,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

# Configuration
$AppName = "Oxide Browser"
$AppExe = "oxide-browser.exe"
$Version = "0.1.0"
$Publisher = "Oxide Browser Team"
$BuildDir = "target\release"
$OutputDir = "dist\windows"
$PackageDir = "packaging\windows"

# Colors
function Write-Step { param($msg) Write-Host "==> " -ForegroundColor Blue -NoNewline; Write-Host $msg }
function Write-Success { param($msg) Write-Host "[OK] " -ForegroundColor Green -NoNewline; Write-Host $msg }
function Write-Warning { param($msg) Write-Host "[!] " -ForegroundColor Yellow -NoNewline; Write-Host $msg }
function Write-Error { param($msg) Write-Host "[X] " -ForegroundColor Red -NoNewline; Write-Host $msg }

function Show-Help {
    Write-Host @"

Oxide Browser - Windows Build Script

Usage: .\build-windows.ps1 [OPTIONS]

Options:
    -Portable       Create portable ZIP package
    -Installer      Create MSI installer (requires WiX Toolset)
    -All            Create both portable and installer
    -Clean          Clean build artifacts before building
    -SignCert       Path to code signing certificate (.pfx)
    -Help           Show this help message

Examples:
    .\build-windows.ps1 -Portable
    .\build-windows.ps1 -All
    .\build-windows.ps1 -Installer -SignCert "cert.pfx"

Requirements:
    - Rust toolchain (rustup.rs)
    - WiX Toolset 3.x or 4.x (for installer)
    - Windows SDK (for signtool)

"@
}

function Test-Requirements {
    Write-Step "Checking requirements..."

    # Check Rust
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Error "Rust/Cargo not found. Install from https://rustup.rs"
        exit 1
    }

    # Check target
    $targets = rustup target list --installed
    if ($targets -notcontains "x86_64-pc-windows-msvc") {
        Write-Step "Installing Windows MSVC target..."
        rustup target add x86_64-pc-windows-msvc
    }

    Write-Success "All requirements met"
}

function Build-Application {
    Write-Step "Building Oxide Browser..."

    $env:RUSTFLAGS = "-C target-feature=+crt-static"

    cargo build --release --target x86_64-pc-windows-msvc -p browser

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed"
        exit 1
    }

    Write-Success "Build successful"
}

function New-PortablePackage {
    Write-Step "Creating portable package..."

    $portableDir = "$OutputDir\portable"
    $zipName = "OxideBrowser-$Version-windows-x64-portable.zip"

    # Clean and create directory
    if (Test-Path $portableDir) { Remove-Item -Recurse -Force $portableDir }
    New-Item -ItemType Directory -Force -Path $portableDir | Out-Null
    New-Item -ItemType Directory -Force -Path "$portableDir\$AppName" | Out-Null

    # Copy executable
    Copy-Item "target\x86_64-pc-windows-msvc\release\$AppExe" "$portableDir\$AppName\"

    # Copy runtime files
    if (Test-Path "$PackageDir\resources") {
        Copy-Item -Recurse "$PackageDir\resources\*" "$portableDir\$AppName\"
    }

    # Create portable marker file
    Set-Content -Path "$portableDir\$AppName\portable.txt" -Value "This is a portable installation"

    # Create README
    Set-Content -Path "$portableDir\$AppName\README.txt" -Value @"
Oxide Browser - Portable Edition
Version: $Version

This is a portable version that stores all data in the application folder.

To run:
  Double-click oxide-browser.exe

For command-line options:
  oxide-browser.exe --help

Website: https://github.com/oxide-browser/oxide
"@

    # Create ZIP
    if (Test-Path "$OutputDir\$zipName") { Remove-Item "$OutputDir\$zipName" }
    Compress-Archive -Path "$portableDir\$AppName" -DestinationPath "$OutputDir\$zipName"

    # Clean temp directory
    Remove-Item -Recurse -Force $portableDir

    Write-Success "Portable package created: $OutputDir\$zipName"
}

function New-Installer {
    Write-Step "Creating installer..."

    # Check for WiX
    $wixPath = $null
    $wixPaths = @(
        "${env:ProgramFiles(x86)}\WiX Toolset v3.11\bin",
        "${env:ProgramFiles}\WiX Toolset v3.11\bin",
        "${env:ProgramFiles(x86)}\WiX Toolset v3.14\bin",
        "${env:ProgramFiles}\WiX Toolset v3.14\bin"
    )

    foreach ($path in $wixPaths) {
        if (Test-Path "$path\candle.exe") {
            $wixPath = $path
            break
        }
    }

    # Try WiX 4.x (dotnet tool)
    $useWix4 = $false
    if (-not $wixPath) {
        if (Get-Command wix -ErrorAction SilentlyContinue) {
            $useWix4 = $true
            Write-Step "Using WiX 4.x"
        } else {
            Write-Warning "WiX Toolset not found. Skipping installer creation."
            Write-Warning "Install WiX from: https://wixtoolset.org/"
            return
        }
    }

    $installerDir = "$OutputDir\installer"
    $msiName = "OxideBrowser-$Version-windows-x64.msi"

    # Create installer directory
    if (Test-Path $installerDir) { Remove-Item -Recurse -Force $installerDir }
    New-Item -ItemType Directory -Force -Path $installerDir | Out-Null

    # Copy files for installer
    Copy-Item "target\x86_64-pc-windows-msvc\release\$AppExe" "$installerDir\"

    if ($useWix4) {
        # WiX 4.x build
        wix build "$PackageDir\Product.wxs" `
            -d "ProductVersion=$Version" `
            -d "SourceDir=$installerDir" `
            -o "$OutputDir\$msiName"
    } else {
        # WiX 3.x build
        & "$wixPath\candle.exe" "$PackageDir\Product.wxs" `
            -dProductVersion="$Version" `
            -dSourceDir="$installerDir" `
            -out "$installerDir\Product.wixobj"

        & "$wixPath\light.exe" "$installerDir\Product.wixobj" `
            -out "$OutputDir\$msiName" `
            -ext WixUIExtension
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Installer creation failed"
        return
    }

    # Clean temp directory
    Remove-Item -Recurse -Force $installerDir

    Write-Success "Installer created: $OutputDir\$msiName"
}

function Sign-Files {
    param($FilePath)

    if (-not $SignCert) { return }

    Write-Step "Signing $FilePath..."

    $signtool = Get-Command signtool -ErrorAction SilentlyContinue
    if (-not $signtool) {
        # Try to find in Windows SDK
        $sdkPaths = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue
        if ($sdkPaths) {
            $signtool = $sdkPaths | Sort-Object | Select-Object -Last 1
        }
    }

    if (-not $signtool) {
        Write-Warning "signtool not found. Skipping signing."
        return
    }

    & $signtool sign /f $SignCert /tr http://timestamp.digicert.com /td sha256 /fd sha256 $FilePath

    if ($LASTEXITCODE -eq 0) {
        Write-Success "Signed: $FilePath"
    } else {
        Write-Warning "Signing failed for: $FilePath"
    }
}

# Main
if ($Help) {
    Show-Help
    exit 0
}

Write-Host ""
Write-Host "=========================================="
Write-Host "  Oxide Browser - Windows Build Script"
Write-Host "  Version: $Version"
Write-Host "=========================================="
Write-Host ""

# Navigate to project root
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location (Join-Path $scriptDir "..\..")

# Clean if requested
if ($Clean) {
    Write-Step "Cleaning build artifacts..."
    cargo clean
    if (Test-Path $OutputDir) { Remove-Item -Recurse -Force $OutputDir }
    Write-Success "Cleaned"
}

# Create output directory
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# Build
Test-Requirements
Build-Application

# Sign executable
if ($SignCert) {
    Sign-Files "target\x86_64-pc-windows-msvc\release\$AppExe"
}

# Package
if ($All -or (-not $Portable -and -not $Installer)) {
    $Portable = $true
    $Installer = $true
}

if ($Portable) {
    New-PortablePackage
}

if ($Installer) {
    New-Installer
    if ($SignCert -and (Test-Path "$OutputDir\OxideBrowser-$Version-windows-x64.msi")) {
        Sign-Files "$OutputDir\OxideBrowser-$Version-windows-x64.msi"
    }
}

Write-Host ""
Write-Host "=========================================="
Write-Host "  Build Complete!"
Write-Host "=========================================="
Write-Host ""
Write-Host "Output directory: $OutputDir"
Get-ChildItem $OutputDir | ForEach-Object { Write-Host "  - $($_.Name)" }
Write-Host ""
