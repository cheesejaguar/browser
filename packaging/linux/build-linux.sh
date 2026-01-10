#!/bin/bash
# Oxide Browser - Linux Build Script
# Builds DEB package and AppImage for Ubuntu 24+ and other Linux distributions

set -e

# Configuration
APP_NAME="Oxide Browser"
APP_ID="com.oxide.browser"
BINARY_NAME="oxide-browser"
VERSION="0.1.0"
ARCH="amd64"
BUILD_DIR="target/release"
OUTPUT_DIR="dist/linux"
PACKAGE_DIR="packaging/linux"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_step() { echo -e "${BLUE}==> ${NC}$1"; }
print_success() { echo -e "${GREEN}✓ ${NC}$1"; }
print_warning() { echo -e "${YELLOW}⚠ ${NC}$1"; }
print_error() { echo -e "${RED}✗ ${NC}$1"; }

# Parse arguments
BUILD_DEB=false
BUILD_APPIMAGE=false
BUILD_TAR=false
DO_CLEAN=false

usage() {
    cat << EOF
Oxide Browser - Linux Build Script

Usage: $0 [OPTIONS]

Options:
    --deb           Build Debian/Ubuntu package
    --appimage      Build AppImage
    --tar           Build tarball
    --all           Build all package formats
    --clean         Clean build artifacts before building
    --help          Show this help message

Examples:
    $0 --deb                    # Build .deb package
    $0 --appimage               # Build AppImage
    $0 --all                    # Build all formats
EOF
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --deb) BUILD_DEB=true; shift ;;
        --appimage) BUILD_APPIMAGE=true; shift ;;
        --tar) BUILD_TAR=true; shift ;;
        --all) BUILD_DEB=true; BUILD_APPIMAGE=true; BUILD_TAR=true; shift ;;
        --clean) DO_CLEAN=true; shift ;;
        --help) usage; exit 0 ;;
        *) print_error "Unknown option: $1"; usage; exit 1 ;;
    esac
done

# Default to all if nothing specified
if ! $BUILD_DEB && ! $BUILD_APPIMAGE && ! $BUILD_TAR; then
    BUILD_DEB=true
    BUILD_APPIMAGE=true
    BUILD_TAR=true
fi

check_requirements() {
    print_step "Checking requirements..."

    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Install from https://rustup.rs"
        exit 1
    fi

    if $BUILD_DEB && ! command -v dpkg-deb &> /dev/null; then
        print_warning "dpkg-deb not found. Install with: sudo apt install dpkg"
        BUILD_DEB=false
    fi

    print_success "Requirements check complete"
}

build_application() {
    print_step "Building Oxide Browser..."

    # Build dependencies
    sudo apt-get update -qq || true
    sudo apt-get install -y -qq \
        build-essential \
        pkg-config \
        libssl-dev \
        libx11-dev \
        libxcb1-dev \
        libxkbcommon-dev \
        libwayland-dev \
        libgtk-3-dev \
        libglib2.0-dev \
        libasound2-dev \
        2>/dev/null || true

    cargo build --release -p browser

    if [[ $? -eq 0 ]]; then
        print_success "Build successful"
    else
        print_error "Build failed"
        exit 1
    fi
}

create_desktop_file() {
    local dest="$1"
    cat > "$dest" << EOF
[Desktop Entry]
Name=${APP_NAME}
GenericName=Web Browser
Comment=A high-performance web browser written in Rust
Exec=${BINARY_NAME} %U
Icon=${APP_ID}
Terminal=false
Type=Application
Categories=Network;WebBrowser;
MimeType=text/html;text/xml;application/xhtml+xml;application/xml;application/rss+xml;application/rdf+xml;x-scheme-handler/http;x-scheme-handler/https;
StartupNotify=true
StartupWMClass=oxide-browser
Actions=new-window;new-private-window;

[Desktop Action new-window]
Name=New Window
Exec=${BINARY_NAME} --new-window

[Desktop Action new-private-window]
Name=New Private Window
Exec=${BINARY_NAME} --private-window
EOF
}

create_appdata_file() {
    local dest="$1"
    cat > "$dest" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>${APP_ID}</id>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT</project_license>
  <name>${APP_NAME}</name>
  <summary>A high-performance web browser written in Rust</summary>
  <description>
    <p>
      Oxide Browser is a modern, high-performance web browser built from the ground up in Rust.
      It features GPU-accelerated rendering, a powerful JavaScript engine, and comprehensive
      web standards support.
    </p>
    <p>Features:</p>
    <ul>
      <li>GPU-accelerated rendering with wgpu</li>
      <li>Full HTML5 and CSS3 support</li>
      <li>JavaScript engine (Boa)</li>
      <li>Multi-tab browsing</li>
      <li>Privacy-focused design</li>
      <li>Cross-platform support</li>
    </ul>
  </description>
  <launchable type="desktop-id">${APP_ID}.desktop</launchable>
  <url type="homepage">https://github.com/oxide-browser/oxide</url>
  <url type="bugtracker">https://github.com/oxide-browser/oxide/issues</url>
  <developer_name>Oxide Browser Team</developer_name>
  <screenshots>
    <screenshot type="default">
      <caption>Oxide Browser main window</caption>
      <image>https://raw.githubusercontent.com/oxide-browser/oxide/main/docs/screenshot.png</image>
    </screenshot>
  </screenshots>
  <releases>
    <release version="${VERSION}" date="2024-01-01">
      <description>
        <p>Initial release</p>
      </description>
    </release>
  </releases>
  <content_rating type="oars-1.1" />
  <supports>
    <control>pointing</control>
    <control>keyboard</control>
    <control>touch</control>
  </supports>
  <requires>
    <display_length compare="ge">768</display_length>
  </requires>
</component>
EOF
}

build_deb_package() {
    print_step "Building Debian package..."

    local DEB_DIR="${OUTPUT_DIR}/deb"
    local DEB_NAME="oxide-browser_${VERSION}_${ARCH}.deb"
    local PKG_ROOT="${DEB_DIR}/oxide-browser_${VERSION}_${ARCH}"

    # Clean and create directory structure
    rm -rf "$PKG_ROOT"
    mkdir -p "${PKG_ROOT}/DEBIAN"
    mkdir -p "${PKG_ROOT}/usr/bin"
    mkdir -p "${PKG_ROOT}/usr/share/applications"
    mkdir -p "${PKG_ROOT}/usr/share/icons/hicolor/256x256/apps"
    mkdir -p "${PKG_ROOT}/usr/share/icons/hicolor/128x128/apps"
    mkdir -p "${PKG_ROOT}/usr/share/icons/hicolor/64x64/apps"
    mkdir -p "${PKG_ROOT}/usr/share/icons/hicolor/48x48/apps"
    mkdir -p "${PKG_ROOT}/usr/share/icons/hicolor/32x32/apps"
    mkdir -p "${PKG_ROOT}/usr/share/icons/hicolor/16x16/apps"
    mkdir -p "${PKG_ROOT}/usr/share/metainfo"
    mkdir -p "${PKG_ROOT}/usr/share/doc/oxide-browser"

    # Copy binary
    cp "${BUILD_DIR}/${BINARY_NAME}" "${PKG_ROOT}/usr/bin/"
    chmod 755 "${PKG_ROOT}/usr/bin/${BINARY_NAME}"

    # Create desktop file
    create_desktop_file "${PKG_ROOT}/usr/share/applications/${APP_ID}.desktop"

    # Create appdata
    create_appdata_file "${PKG_ROOT}/usr/share/metainfo/${APP_ID}.appdata.xml"

    # Copy icons (use placeholder if not available)
    if [[ -f "${PACKAGE_DIR}/icons/256x256.png" ]]; then
        cp "${PACKAGE_DIR}/icons/256x256.png" "${PKG_ROOT}/usr/share/icons/hicolor/256x256/apps/${APP_ID}.png"
    fi

    # Create control file
    local INSTALLED_SIZE=$(du -sk "${PKG_ROOT}" | cut -f1)
    cat > "${PKG_ROOT}/DEBIAN/control" << EOF
Package: oxide-browser
Version: ${VERSION}
Section: web
Priority: optional
Architecture: ${ARCH}
Depends: libc6 (>= 2.35), libssl3, libx11-6, libxcb1, libxkbcommon0, libwayland-client0, libgtk-3-0
Recommends: fonts-noto, fonts-noto-color-emoji
Suggests: hunspell
Installed-Size: ${INSTALLED_SIZE}
Maintainer: Oxide Browser Team <oxide@example.com>
Homepage: https://github.com/oxide-browser/oxide
Description: A high-performance web browser written in Rust
 Oxide Browser is a modern web browser built from scratch in Rust,
 featuring GPU-accelerated rendering, full HTML5/CSS3 support,
 and a powerful JavaScript engine.
 .
 Features include:
  - GPU-accelerated rendering with wgpu
  - HTML5 parser and DOM implementation
  - CSS3 with flexbox and grid layout
  - JavaScript engine (Boa)
  - Multi-tab browsing
  - Privacy-focused design
EOF

    # Create postinst script
    cat > "${PKG_ROOT}/DEBIAN/postinst" << 'EOF'
#!/bin/sh
set -e

# Update desktop database
if command -v update-desktop-database > /dev/null; then
    update-desktop-database -q /usr/share/applications || true
fi

# Update icon cache
if command -v gtk-update-icon-cache > /dev/null; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
fi

# Update MIME database
if command -v update-mime-database > /dev/null; then
    update-mime-database /usr/share/mime || true
fi

exit 0
EOF
    chmod 755 "${PKG_ROOT}/DEBIAN/postinst"

    # Create postrm script
    cat > "${PKG_ROOT}/DEBIAN/postrm" << 'EOF'
#!/bin/sh
set -e

if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    # Update desktop database
    if command -v update-desktop-database > /dev/null; then
        update-desktop-database -q /usr/share/applications || true
    fi

    # Update icon cache
    if command -v gtk-update-icon-cache > /dev/null; then
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
    fi
fi

exit 0
EOF
    chmod 755 "${PKG_ROOT}/DEBIAN/postrm"

    # Create copyright file
    cat > "${PKG_ROOT}/usr/share/doc/oxide-browser/copyright" << EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: Oxide Browser
Upstream-Contact: oxide@example.com
Source: https://github.com/oxide-browser/oxide

Files: *
Copyright: 2024 Oxide Browser Team
License: MIT

License: MIT
 Permission is hereby granted, free of charge, to any person obtaining a copy
 of this software and associated documentation files (the "Software"), to deal
 in the Software without restriction, including without limitation the rights
 to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 copies of the Software, and to permit persons to whom the Software is
 furnished to do so, subject to the following conditions:
 .
 The above copyright notice and this permission notice shall be included in all
 copies or substantial portions of the Software.
 .
 THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 SOFTWARE.
EOF

    # Build the package
    dpkg-deb --build --root-owner-group "${PKG_ROOT}" "${OUTPUT_DIR}/${DEB_NAME}"

    # Clean up
    rm -rf "${PKG_ROOT}"

    print_success "DEB package created: ${OUTPUT_DIR}/${DEB_NAME}"
}

build_appimage() {
    print_step "Building AppImage..."

    local APPIMAGE_DIR="${OUTPUT_DIR}/appimage"
    local APPDIR="${APPIMAGE_DIR}/Oxide_Browser.AppDir"
    local APPIMAGE_NAME="Oxide_Browser-${VERSION}-x86_64.AppImage"

    # Download AppImage tools if needed
    if [[ ! -f "${APPIMAGE_DIR}/appimagetool-x86_64.AppImage" ]]; then
        mkdir -p "${APPIMAGE_DIR}"
        print_step "Downloading AppImage tools..."
        wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage" \
            -O "${APPIMAGE_DIR}/appimagetool-x86_64.AppImage"
        chmod +x "${APPIMAGE_DIR}/appimagetool-x86_64.AppImage"
    fi

    # Clean and create AppDir structure
    rm -rf "${APPDIR}"
    mkdir -p "${APPDIR}/usr/bin"
    mkdir -p "${APPDIR}/usr/share/applications"
    mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"
    mkdir -p "${APPDIR}/usr/share/metainfo"

    # Copy binary
    cp "${BUILD_DIR}/${BINARY_NAME}" "${APPDIR}/usr/bin/"
    chmod 755 "${APPDIR}/usr/bin/${BINARY_NAME}"

    # Create desktop file
    create_desktop_file "${APPDIR}/usr/share/applications/${APP_ID}.desktop"
    cp "${APPDIR}/usr/share/applications/${APP_ID}.desktop" "${APPDIR}/${APP_ID}.desktop"

    # Create appdata
    create_appdata_file "${APPDIR}/usr/share/metainfo/${APP_ID}.appdata.xml"

    # Create/copy icon
    if [[ -f "${PACKAGE_DIR}/icons/256x256.png" ]]; then
        cp "${PACKAGE_DIR}/icons/256x256.png" "${APPDIR}/${APP_ID}.png"
        cp "${PACKAGE_DIR}/icons/256x256.png" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/${APP_ID}.png"
    else
        # Create placeholder icon
        if command -v convert &> /dev/null; then
            convert -size 256x256 xc:steelblue -fill white -gravity center \
                -pointsize 72 -annotate 0 "O" "${APPDIR}/${APP_ID}.png"
        fi
    fi

    # Create AppRun
    cat > "${APPDIR}/AppRun" << 'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS}"
exec "${HERE}/usr/bin/oxide-browser" "$@"
EOF
    chmod +x "${APPDIR}/AppRun"

    # Build AppImage
    ARCH=x86_64 "${APPIMAGE_DIR}/appimagetool-x86_64.AppImage" \
        --no-appstream "${APPDIR}" "${OUTPUT_DIR}/${APPIMAGE_NAME}"

    # Clean up
    rm -rf "${APPDIR}"

    print_success "AppImage created: ${OUTPUT_DIR}/${APPIMAGE_NAME}"
}

build_tarball() {
    print_step "Building tarball..."

    local TAR_NAME="oxide-browser-${VERSION}-linux-x86_64.tar.gz"
    local TAR_DIR="${OUTPUT_DIR}/tar/oxide-browser-${VERSION}"

    # Clean and create directory
    rm -rf "${OUTPUT_DIR}/tar"
    mkdir -p "${TAR_DIR}"

    # Copy files
    cp "${BUILD_DIR}/${BINARY_NAME}" "${TAR_DIR}/"
    chmod 755 "${TAR_DIR}/${BINARY_NAME}"

    # Create desktop file
    create_desktop_file "${TAR_DIR}/${APP_ID}.desktop"

    # Create README
    cat > "${TAR_DIR}/README.txt" << EOF
Oxide Browser ${VERSION}
=======================

A high-performance web browser written in Rust.

Installation:
  1. Copy oxide-browser to /usr/local/bin/ or ~/bin/
  2. Copy ${APP_ID}.desktop to ~/.local/share/applications/
  3. Run: oxide-browser

Command-line options:
  oxide-browser --help

Website: https://github.com/oxide-browser/oxide
EOF

    # Create install script
    cat > "${TAR_DIR}/install.sh" << 'EOF'
#!/bin/bash
set -e

INSTALL_DIR="${HOME}/.local"

echo "Installing Oxide Browser..."

mkdir -p "${INSTALL_DIR}/bin"
mkdir -p "${INSTALL_DIR}/share/applications"

cp oxide-browser "${INSTALL_DIR}/bin/"
chmod +x "${INSTALL_DIR}/bin/oxide-browser"

cp com.oxide.browser.desktop "${INSTALL_DIR}/share/applications/"
sed -i "s|Exec=oxide-browser|Exec=${INSTALL_DIR}/bin/oxide-browser|g" \
    "${INSTALL_DIR}/share/applications/com.oxide.browser.desktop"

echo "Installed to ${INSTALL_DIR}/bin/oxide-browser"
echo "Add ${INSTALL_DIR}/bin to your PATH if not already present"
EOF
    chmod +x "${TAR_DIR}/install.sh"

    # Create tarball
    cd "${OUTPUT_DIR}/tar"
    tar -czvf "../${TAR_NAME}" "oxide-browser-${VERSION}"
    cd - > /dev/null

    # Clean up
    rm -rf "${OUTPUT_DIR}/tar"

    print_success "Tarball created: ${OUTPUT_DIR}/${TAR_NAME}"
}

# Main
echo ""
echo "=========================================="
echo "  Oxide Browser - Linux Build Script"
echo "  Version: ${VERSION}"
echo "=========================================="
echo ""

# Navigate to project root
cd "$(dirname "$0")/../.."

# Clean if requested
if $DO_CLEAN; then
    print_step "Cleaning build artifacts..."
    cargo clean
    rm -rf "${OUTPUT_DIR}"
    print_success "Cleaned"
fi

# Create output directory
mkdir -p "${OUTPUT_DIR}"

# Build
check_requirements
build_application

# Create packages
if $BUILD_DEB; then
    build_deb_package
fi

if $BUILD_APPIMAGE; then
    build_appimage
fi

if $BUILD_TAR; then
    build_tarball
fi

echo ""
echo "=========================================="
echo "  Build Complete!"
echo "=========================================="
echo ""
echo "Output directory: ${OUTPUT_DIR}"
ls -la "${OUTPUT_DIR}"
echo ""
