#!/bin/bash
# Generate macOS app icon from source image
# Requires: ImageMagick (convert) or sips

set -e

SOURCE_IMAGE="${1:-icon.png}"
OUTPUT_DIR="$(dirname "$0")"
ICONSET_DIR="${OUTPUT_DIR}/AppIcon.iconset"

if [[ ! -f "$SOURCE_IMAGE" ]]; then
    echo "Usage: $0 <source-image.png>"
    echo ""
    echo "The source image should be at least 1024x1024 pixels."
    echo "If no image is provided, a placeholder icon will be generated."
    echo ""

    # Generate a simple placeholder icon using Python if available
    if command -v python3 &> /dev/null; then
        echo "Generating placeholder icon..."
        python3 << 'PYTHON_SCRIPT'
import struct
import zlib
import os

def create_png(width, height, filename):
    """Create a simple colored PNG file."""

    def png_chunk(chunk_type, data):
        chunk = chunk_type + data
        return struct.pack('>I', len(data)) + chunk + struct.pack('>I', zlib.crc32(chunk) & 0xffffffff)

    # Header
    header = b'\x89PNG\r\n\x1a\n'

    # IHDR chunk
    ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)  # RGBA
    ihdr = png_chunk(b'IHDR', ihdr_data)

    # Image data - create a gradient with "O" letter shape
    raw_data = []
    cx, cy = width // 2, height // 2
    outer_r = min(width, height) // 2 - 10
    inner_r = outer_r - max(width // 6, 40)

    for y in range(height):
        raw_data.append(0)  # Filter byte
        for x in range(width):
            dx, dy = x - cx, y - cy
            dist = (dx*dx + dy*dy) ** 0.5

            if inner_r < dist < outer_r:
                # Ring for "O"
                r, g, b, a = 66, 133, 244, 255  # Blue
            elif dist <= inner_r:
                # Inner circle - gradient
                t = dist / inner_r
                r = int(255 * (1 - t * 0.3))
                g = int(255 * (1 - t * 0.3))
                b = int(255 * (1 - t * 0.1))
                a = 255
            else:
                # Outside - transparent
                r, g, b, a = 0, 0, 0, 0

            raw_data.extend([r, g, b, a])

    compressed = zlib.compress(bytes(raw_data), 9)
    idat = png_chunk(b'IDAT', compressed)

    # IEND chunk
    iend = png_chunk(b'IEND', b'')

    # Write PNG file
    with open(filename, 'wb') as f:
        f.write(header + ihdr + idat + iend)

    print(f"Created: {filename}")

# Generate icons at all required sizes
sizes = [1024, 512, 256, 128, 64, 32, 16]
output_dir = os.path.dirname(os.path.abspath(__file__)) if '__file__' in dir() else '.'
iconset_dir = os.path.join(output_dir, 'AppIcon.iconset')
os.makedirs(iconset_dir, exist_ok=True)

for size in sizes:
    create_png(size, size, os.path.join(iconset_dir, f'icon_{size}x{size}.png'))
    if size <= 512:
        create_png(size * 2, size * 2, os.path.join(iconset_dir, f'icon_{size}x{size}@2x.png'))

print("\nIconset generated in AppIcon.iconset/")
print("Run 'iconutil -c icns AppIcon.iconset' to create .icns file")
PYTHON_SCRIPT

        # Convert iconset to icns
        if command -v iconutil &> /dev/null; then
            cd "$OUTPUT_DIR"
            iconutil -c icns AppIcon.iconset -o AppIcon.icns
            echo "Created: ${OUTPUT_DIR}/AppIcon.icns"
            rm -rf AppIcon.iconset
        fi
        exit 0
    fi

    echo "Error: No source image provided and Python not available for placeholder generation."
    exit 1
fi

echo "Creating iconset from: $SOURCE_IMAGE"

# Create iconset directory
rm -rf "$ICONSET_DIR"
mkdir -p "$ICONSET_DIR"

# Icon sizes for macOS
SIZES=(16 32 64 128 256 512 1024)

# Check for ImageMagick or sips
if command -v convert &> /dev/null; then
    CONVERTER="imagemagick"
elif command -v sips &> /dev/null; then
    CONVERTER="sips"
else
    echo "Error: Neither ImageMagick nor sips found"
    exit 1
fi

# Generate icons at each size
for size in "${SIZES[@]}"; do
    echo "Generating ${size}x${size}..."

    if [[ "$CONVERTER" == "imagemagick" ]]; then
        convert "$SOURCE_IMAGE" -resize "${size}x${size}" "${ICONSET_DIR}/icon_${size}x${size}.png"

        # @2x version (retina)
        if [[ $size -le 512 ]]; then
            double=$((size * 2))
            convert "$SOURCE_IMAGE" -resize "${double}x${double}" "${ICONSET_DIR}/icon_${size}x${size}@2x.png"
        fi
    else
        sips -z "$size" "$size" "$SOURCE_IMAGE" --out "${ICONSET_DIR}/icon_${size}x${size}.png" > /dev/null

        # @2x version (retina)
        if [[ $size -le 512 ]]; then
            double=$((size * 2))
            sips -z "$double" "$double" "$SOURCE_IMAGE" --out "${ICONSET_DIR}/icon_${size}x${size}@2x.png" > /dev/null
        fi
    fi
done

# Create .icns file
echo "Creating AppIcon.icns..."
iconutil -c icns "$ICONSET_DIR" -o "${OUTPUT_DIR}/AppIcon.icns"

# Clean up
rm -rf "$ICONSET_DIR"

echo ""
echo "Icon created: ${OUTPUT_DIR}/AppIcon.icns"
