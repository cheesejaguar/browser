#!/bin/bash
# Oxide Browser - macOS Build and Packaging Script
# Builds a universal binary (Apple Silicon + Intel) .app bundle for macOS

set -e

# Configuration
APP_NAME="Oxide Browser"
BUNDLE_ID="com.oxide.browser"
VERSION="0.1.0"
BUILD_DIR="target/release"
PACKAGE_DIR="packaging/macos"
OUTPUT_DIR="dist/macos"
DMG_NAME="OxideBrowser-${VERSION}-macOS"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}==> ${NC}$1"
}

print_success() {
    echo -e "${GREEN}✓ ${NC}$1"
}

print_warning() {
    echo -e "${YELLOW}⚠ ${NC}$1"
}

print_error() {
    echo -e "${RED}✗ ${NC}$1"
}

# Check for required tools
check_requirements() {
    print_step "Checking requirements..."

    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Install from https://rustup.rs"
        exit 1
    fi

    if ! command -v xcrun &> /dev/null; then
        print_error "Xcode Command Line Tools not found. Install with: xcode-select --install"
        exit 1
    fi

    print_success "All requirements met"
}

# Detect architecture and set targets
setup_targets() {
    print_step "Setting up build targets..."

    ARCH=$(uname -m)

    if [[ "$BUILD_UNIVERSAL" == "true" ]]; then
        TARGETS=("aarch64-apple-darwin" "x86_64-apple-darwin")
        print_success "Building universal binary (arm64 + x86_64)"
    elif [[ "$ARCH" == "arm64" ]]; then
        TARGETS=("aarch64-apple-darwin")
        print_success "Building for Apple Silicon (arm64)"
    else
        TARGETS=("x86_64-apple-darwin")
        print_success "Building for Intel (x86_64)"
    fi

    # Ensure targets are installed
    for target in "${TARGETS[@]}"; do
        if ! rustup target list --installed | grep -q "$target"; then
            print_step "Installing target: $target"
            rustup target add "$target"
        fi
    done
}

# Build the application
build_app() {
    print_step "Building Oxide Browser..."

    for target in "${TARGETS[@]}"; do
        print_step "Compiling for $target..."

        RUSTFLAGS="-C link-arg=-Wl,-rpath,@executable_path/../Frameworks" \
        cargo build --release --target "$target" -p browser

        if [[ $? -eq 0 ]]; then
            print_success "Build successful for $target"
        else
            print_error "Build failed for $target"
            exit 1
        fi
    done
}

# Create universal binary if building for multiple architectures
create_universal_binary() {
    if [[ ${#TARGETS[@]} -gt 1 ]]; then
        print_step "Creating universal binary..."

        mkdir -p "${BUILD_DIR}/universal"

        lipo -create \
            "target/aarch64-apple-darwin/release/oxide-browser" \
            "target/x86_64-apple-darwin/release/oxide-browser" \
            -output "${BUILD_DIR}/universal/oxide-browser"

        BINARY_PATH="${BUILD_DIR}/universal/oxide-browser"
        print_success "Universal binary created"
    else
        BINARY_PATH="target/${TARGETS[0]}/release/oxide-browser"
        print_success "Using single-architecture binary"
    fi
}

# Create the .app bundle
create_app_bundle() {
    print_step "Creating application bundle..."

    APP_BUNDLE="${OUTPUT_DIR}/${APP_NAME}.app"
    CONTENTS_DIR="${APP_BUNDLE}/Contents"
    MACOS_DIR="${CONTENTS_DIR}/MacOS"
    RESOURCES_DIR="${CONTENTS_DIR}/Resources"
    FRAMEWORKS_DIR="${CONTENTS_DIR}/Frameworks"

    # Clean and create directory structure
    rm -rf "${APP_BUNDLE}"
    mkdir -p "${MACOS_DIR}"
    mkdir -p "${RESOURCES_DIR}"
    mkdir -p "${FRAMEWORKS_DIR}"

    # Copy executable
    cp "${BINARY_PATH}" "${MACOS_DIR}/oxide-browser"
    chmod +x "${MACOS_DIR}/oxide-browser"

    # Copy Info.plist
    cp "${PACKAGE_DIR}/Info.plist" "${CONTENTS_DIR}/Info.plist"

    # Update version in Info.plist
    /usr/libexec/PlistBuddy -c "Set :CFBundleVersion ${VERSION}" "${CONTENTS_DIR}/Info.plist"
    /usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString ${VERSION}" "${CONTENTS_DIR}/Info.plist"

    # Copy icon if exists
    if [[ -f "${PACKAGE_DIR}/AppIcon.icns" ]]; then
        cp "${PACKAGE_DIR}/AppIcon.icns" "${RESOURCES_DIR}/AppIcon.icns"
    else
        print_warning "No icon file found, using default"
        create_placeholder_icon
    fi

    # Create PkgInfo
    echo "APPL????" > "${CONTENTS_DIR}/PkgInfo"

    print_success "Application bundle created at ${APP_BUNDLE}"
}

# Create a placeholder icon if none exists
create_placeholder_icon() {
    print_step "Generating placeholder icon..."

    # Create a simple icon using sips if available
    ICON_DIR="${OUTPUT_DIR}/icon.iconset"
    mkdir -p "${ICON_DIR}"

    # Generate icon sizes (would need actual icon generation tool)
    # For now, just create the iconset directory structure
    for size in 16 32 64 128 256 512; do
        touch "${ICON_DIR}/icon_${size}x${size}.png"
        touch "${ICON_DIR}/icon_${size}x${size}@2x.png"
    done

    # Convert to icns if iconutil is available
    if command -v iconutil &> /dev/null; then
        iconutil -c icns "${ICON_DIR}" -o "${RESOURCES_DIR}/AppIcon.icns" 2>/dev/null || true
    fi

    rm -rf "${ICON_DIR}"
}

# Code sign the application
code_sign() {
    print_step "Code signing application..."

    ENTITLEMENTS="${PACKAGE_DIR}/Oxide Browser.entitlements"

    if [[ -n "$SIGNING_IDENTITY" ]]; then
        codesign --force --deep --sign "${SIGNING_IDENTITY}" \
            --entitlements "${ENTITLEMENTS}" \
            --options runtime \
            "${OUTPUT_DIR}/${APP_NAME}.app"

        print_success "Application signed with ${SIGNING_IDENTITY}"
    else
        # Ad-hoc signing for local development
        codesign --force --deep --sign - \
            --entitlements "${ENTITLEMENTS}" \
            "${OUTPUT_DIR}/${APP_NAME}.app"

        print_warning "Ad-hoc signed (for development only)"
    fi
}

# Verify the application bundle
verify_bundle() {
    print_step "Verifying application bundle..."

    # Check code signature
    if codesign --verify --deep --strict "${OUTPUT_DIR}/${APP_NAME}.app" 2>/dev/null; then
        print_success "Code signature valid"
    else
        print_warning "Code signature verification failed (may be ad-hoc signed)"
    fi

    # Check Gatekeeper
    if spctl --assess --type exec "${OUTPUT_DIR}/${APP_NAME}.app" 2>/dev/null; then
        print_success "Gatekeeper assessment passed"
    else
        print_warning "Gatekeeper assessment failed (normal for unsigned builds)"
    fi

    # Display bundle info
    print_step "Bundle information:"
    /usr/libexec/PlistBuddy -c "Print :CFBundleIdentifier" "${OUTPUT_DIR}/${APP_NAME}.app/Contents/Info.plist"
    /usr/libexec/PlistBuddy -c "Print :CFBundleVersion" "${OUTPUT_DIR}/${APP_NAME}.app/Contents/Info.plist"

    # Check architectures
    print_step "Binary architectures:"
    lipo -info "${OUTPUT_DIR}/${APP_NAME}.app/Contents/MacOS/oxide-browser"
}

# Create DMG installer
create_dmg() {
    print_step "Creating DMG installer..."

    DMG_PATH="${OUTPUT_DIR}/${DMG_NAME}.dmg"
    DMG_TEMP="${OUTPUT_DIR}/dmg_temp"

    # Clean up
    rm -f "${DMG_PATH}"
    rm -rf "${DMG_TEMP}"

    # Create temp directory with app and Applications symlink
    mkdir -p "${DMG_TEMP}"
    cp -R "${OUTPUT_DIR}/${APP_NAME}.app" "${DMG_TEMP}/"
    ln -s /Applications "${DMG_TEMP}/Applications"

    # Create DMG
    hdiutil create -volname "${APP_NAME}" \
        -srcfolder "${DMG_TEMP}" \
        -ov -format UDZO \
        "${DMG_PATH}"

    # Clean up temp
    rm -rf "${DMG_TEMP}"

    print_success "DMG created at ${DMG_PATH}"
}

# Create ZIP archive
create_zip() {
    print_step "Creating ZIP archive..."

    ZIP_PATH="${OUTPUT_DIR}/${DMG_NAME}.zip"

    cd "${OUTPUT_DIR}"
    rm -f "${DMG_NAME}.zip"
    ditto -c -k --keepParent "${APP_NAME}.app" "${DMG_NAME}.zip"
    cd - > /dev/null

    print_success "ZIP created at ${ZIP_PATH}"
}

# Notarize the application (requires Apple Developer account)
notarize() {
    if [[ -z "$APPLE_ID" ]] || [[ -z "$APPLE_TEAM_ID" ]]; then
        print_warning "Skipping notarization (APPLE_ID and APPLE_TEAM_ID not set)"
        return
    fi

    print_step "Notarizing application..."

    ZIP_PATH="${OUTPUT_DIR}/${DMG_NAME}.zip"

    xcrun notarytool submit "${ZIP_PATH}" \
        --apple-id "${APPLE_ID}" \
        --team-id "${APPLE_TEAM_ID}" \
        --password "${APPLE_PASSWORD}" \
        --wait

    # Staple the notarization ticket
    xcrun stapler staple "${OUTPUT_DIR}/${APP_NAME}.app"

    print_success "Application notarized and stapled"
}

# Print usage
usage() {
    cat << EOF
Oxide Browser macOS Build Script

Usage: $0 [OPTIONS]

Options:
    --universal         Build universal binary (Intel + Apple Silicon)
    --sign IDENTITY     Code sign with specified identity
    --notarize          Notarize the application (requires Apple Developer)
    --dmg               Create DMG installer
    --zip               Create ZIP archive
    --clean             Clean build artifacts before building
    --help              Show this help message

Environment Variables:
    SIGNING_IDENTITY    Code signing identity
    APPLE_ID            Apple ID for notarization
    APPLE_TEAM_ID       Apple Developer Team ID
    APPLE_PASSWORD      App-specific password for notarization

Examples:
    $0                              # Build for current architecture
    $0 --universal --dmg            # Build universal binary with DMG
    $0 --sign "Developer ID" --dmg  # Build and sign with DMG
EOF
}

# Parse arguments
BUILD_UNIVERSAL=false
CREATE_DMG=false
CREATE_ZIP=false
DO_NOTARIZE=false
DO_CLEAN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --universal)
            BUILD_UNIVERSAL=true
            shift
            ;;
        --sign)
            SIGNING_IDENTITY="$2"
            shift 2
            ;;
        --notarize)
            DO_NOTARIZE=true
            shift
            ;;
        --dmg)
            CREATE_DMG=true
            shift
            ;;
        --zip)
            CREATE_ZIP=true
            shift
            ;;
        --clean)
            DO_CLEAN=true
            shift
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Main build process
main() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════╗"
    echo "║           Oxide Browser - macOS Build Script              ║"
    echo "║                    Version ${VERSION}                          ║"
    echo "╚═══════════════════════════════════════════════════════════╝"
    echo ""

    # Navigate to project root
    cd "$(dirname "$0")/../.."

    # Clean if requested
    if [[ "$DO_CLEAN" == "true" ]]; then
        print_step "Cleaning build artifacts..."
        cargo clean
        rm -rf "${OUTPUT_DIR}"
        print_success "Cleaned"
    fi

    # Create output directory
    mkdir -p "${OUTPUT_DIR}"

    check_requirements
    setup_targets
    build_app
    create_universal_binary
    create_app_bundle
    code_sign
    verify_bundle

    if [[ "$CREATE_DMG" == "true" ]]; then
        create_dmg
    fi

    if [[ "$CREATE_ZIP" == "true" ]]; then
        create_zip
    fi

    if [[ "$DO_NOTARIZE" == "true" ]]; then
        notarize
    fi

    echo ""
    echo "╔═══════════════════════════════════════════════════════════╗"
    echo "║                    Build Complete!                        ║"
    echo "╚═══════════════════════════════════════════════════════════╝"
    echo ""
    echo "Application: ${OUTPUT_DIR}/${APP_NAME}.app"

    if [[ "$CREATE_DMG" == "true" ]]; then
        echo "DMG:         ${OUTPUT_DIR}/${DMG_NAME}.dmg"
    fi

    if [[ "$CREATE_ZIP" == "true" ]]; then
        echo "ZIP:         ${OUTPUT_DIR}/${DMG_NAME}.zip"
    fi

    echo ""
    print_success "Oxide Browser is ready for macOS!"
}

main
