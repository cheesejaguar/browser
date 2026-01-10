#!/bin/bash
# Quick install script for Oxide Browser on macOS

set -e

INSTALL_DIR="/Applications"
APP_NAME="Oxide Browser.app"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║           Oxide Browser - macOS Installer                 ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Check if we're on macOS
if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "Error: This script is for macOS only."
    exit 1
fi

# Check macOS version
MACOS_VERSION=$(sw_vers -productVersion)
MACOS_MAJOR=$(echo "$MACOS_VERSION" | cut -d. -f1)

echo "Detected macOS version: $MACOS_VERSION"

if [[ "$MACOS_MAJOR" -lt 14 ]]; then
    echo "Warning: Oxide Browser is optimized for macOS 14.0+. You may experience issues."
fi

# Navigate to project root
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

# Check if app bundle exists
if [[ ! -d "dist/macos/${APP_NAME}" ]]; then
    echo ""
    echo "Application bundle not found. Building..."
    echo ""

    # Run build script
    ./packaging/macos/build-macos.sh

    if [[ ! -d "dist/macos/${APP_NAME}" ]]; then
        echo "Error: Build failed. Application bundle not created."
        exit 1
    fi
fi

echo ""
echo "Installing Oxide Browser to ${INSTALL_DIR}..."

# Remove existing installation
if [[ -d "${INSTALL_DIR}/${APP_NAME}" ]]; then
    echo "Removing existing installation..."
    rm -rf "${INSTALL_DIR}/${APP_NAME}"
fi

# Copy to Applications
cp -R "dist/macos/${APP_NAME}" "${INSTALL_DIR}/"

echo ""
echo "✓ Oxide Browser installed successfully!"
echo ""
echo "Launch Oxide Browser from:"
echo "  - Applications folder"
echo "  - Spotlight (Cmd + Space, type 'Oxide')"
echo "  - Terminal: open -a 'Oxide Browser'"
echo ""

# Offer to launch
read -p "Launch Oxide Browser now? [y/N] " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    open -a "Oxide Browser"
fi
