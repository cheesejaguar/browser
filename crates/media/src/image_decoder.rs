//! Image decoding.

use std::io::Cursor;
use thiserror::Error;

/// Image decoder.
#[derive(Debug)]
pub struct ImageDecoder {
    /// Maximum image dimensions.
    max_width: u32,
    max_height: u32,
    /// Maximum memory usage.
    max_memory: usize,
}

impl ImageDecoder {
    /// Create a new image decoder.
    pub fn new() -> Self {
        Self {
            max_width: 16384,
            max_height: 16384,
            max_memory: 256 * 1024 * 1024, // 256MB
        }
    }

    /// Set maximum dimensions.
    pub fn set_max_dimensions(&mut self, width: u32, height: u32) {
        self.max_width = width;
        self.max_height = height;
    }

    /// Set maximum memory.
    pub fn set_max_memory(&mut self, bytes: usize) {
        self.max_memory = bytes;
    }

    /// Decode an image from bytes.
    pub fn decode(&self, data: &[u8]) -> Result<DecodedImage, ImageError> {
        let format = ImageFormat::detect(data).ok_or(ImageError::UnknownFormat)?;
        self.decode_with_format(data, format)
    }

    /// Decode an image with a known format.
    pub fn decode_with_format(&self, data: &[u8], format: ImageFormat) -> Result<DecodedImage, ImageError> {
        match format {
            ImageFormat::Png => self.decode_png(data),
            ImageFormat::Jpeg => self.decode_jpeg(data),
            ImageFormat::Gif => self.decode_gif(data),
            ImageFormat::WebP => self.decode_webp(data),
            ImageFormat::Bmp => self.decode_bmp(data),
            ImageFormat::Ico => self.decode_ico(data),
            ImageFormat::Svg => Err(ImageError::UnsupportedFormat("SVG requires different handling".to_string())),
        }
    }

    fn decode_png(&self, data: &[u8]) -> Result<DecodedImage, ImageError> {
        let decoder = png::Decoder::new(Cursor::new(data));
        let mut reader = decoder.read_info().map_err(|e| ImageError::DecodingError(e.to_string()))?;

        let info = reader.info();
        self.check_dimensions(info.width, info.height)?;

        let mut buf = vec![0; reader.output_buffer_size()];
        let output_info = reader.next_frame(&mut buf).map_err(|e| ImageError::DecodingError(e.to_string()))?;

        let (width, height) = (output_info.width, output_info.height);
        let color_type = output_info.color_type;

        // Convert to RGBA
        let pixels = match color_type {
            png::ColorType::Rgba => buf[..output_info.buffer_size()].to_vec(),
            png::ColorType::Rgb => {
                let rgb = &buf[..output_info.buffer_size()];
                let mut rgba = Vec::with_capacity((width * height * 4) as usize);
                for chunk in rgb.chunks(3) {
                    rgba.extend_from_slice(chunk);
                    rgba.push(255);
                }
                rgba
            }
            png::ColorType::GrayscaleAlpha => {
                let ga = &buf[..output_info.buffer_size()];
                let mut rgba = Vec::with_capacity((width * height * 4) as usize);
                for chunk in ga.chunks(2) {
                    rgba.push(chunk[0]);
                    rgba.push(chunk[0]);
                    rgba.push(chunk[0]);
                    rgba.push(chunk[1]);
                }
                rgba
            }
            png::ColorType::Grayscale => {
                let g = &buf[..output_info.buffer_size()];
                let mut rgba = Vec::with_capacity((width * height * 4) as usize);
                for &gray in g {
                    rgba.push(gray);
                    rgba.push(gray);
                    rgba.push(gray);
                    rgba.push(255);
                }
                rgba
            }
            png::ColorType::Indexed => {
                return Err(ImageError::UnsupportedFormat("Indexed PNG".to_string()));
            }
        };

        Ok(DecodedImage {
            width,
            height,
            format: ImageFormat::Png,
            pixels,
            has_alpha: matches!(color_type, png::ColorType::Rgba | png::ColorType::GrayscaleAlpha),
        })
    }

    fn decode_jpeg(&self, data: &[u8]) -> Result<DecodedImage, ImageError> {
        let mut decoder = jpeg_decoder::Decoder::new(Cursor::new(data));
        let pixels_rgb = decoder.decode().map_err(|e| ImageError::DecodingError(e.to_string()))?;
        let info = decoder.info().ok_or(ImageError::DecodingError("No JPEG info".to_string()))?;

        self.check_dimensions(info.width as u32, info.height as u32)?;

        // Convert RGB to RGBA
        let mut pixels = Vec::with_capacity((info.width as usize * info.height as usize) * 4);
        for chunk in pixels_rgb.chunks(3) {
            pixels.extend_from_slice(chunk);
            pixels.push(255);
        }

        Ok(DecodedImage {
            width: info.width as u32,
            height: info.height as u32,
            format: ImageFormat::Jpeg,
            pixels,
            has_alpha: false,
        })
    }

    fn decode_gif(&self, data: &[u8]) -> Result<DecodedImage, ImageError> {
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);

        let mut decoder = options.read_info(Cursor::new(data))
            .map_err(|e| ImageError::DecodingError(e.to_string()))?;

        let width = decoder.width() as u32;
        let height = decoder.height() as u32;
        self.check_dimensions(width, height)?;

        // Decode first frame
        if let Some(frame) = decoder.read_next_frame().map_err(|e| ImageError::DecodingError(e.to_string()))? {
            let pixels = frame.buffer.to_vec();
            Ok(DecodedImage {
                width,
                height,
                format: ImageFormat::Gif,
                pixels,
                has_alpha: true,
            })
        } else {
            Err(ImageError::DecodingError("No frames in GIF".to_string()))
        }
    }

    fn decode_webp(&self, data: &[u8]) -> Result<DecodedImage, ImageError> {
        let decoder = webp::Decoder::new(data);
        let image = decoder.decode().ok_or(ImageError::DecodingError("WebP decode failed".to_string()))?;

        let width = image.width();
        let height = image.height();
        self.check_dimensions(width, height)?;

        let pixels = image.to_image().to_rgba8().into_raw();

        Ok(DecodedImage {
            width,
            height,
            format: ImageFormat::WebP,
            pixels,
            has_alpha: image.is_alpha(),
        })
    }

    fn decode_bmp(&self, data: &[u8]) -> Result<DecodedImage, ImageError> {
        let img = image::load_from_memory_with_format(data, image::ImageFormat::Bmp)
            .map_err(|e| ImageError::DecodingError(e.to_string()))?;

        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();
        self.check_dimensions(width, height)?;

        Ok(DecodedImage {
            width,
            height,
            format: ImageFormat::Bmp,
            pixels: rgba.into_raw(),
            has_alpha: false,
        })
    }

    fn decode_ico(&self, data: &[u8]) -> Result<DecodedImage, ImageError> {
        let img = image::load_from_memory_with_format(data, image::ImageFormat::Ico)
            .map_err(|e| ImageError::DecodingError(e.to_string()))?;

        let rgba = img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();
        self.check_dimensions(width, height)?;

        Ok(DecodedImage {
            width,
            height,
            format: ImageFormat::Ico,
            pixels: rgba.into_raw(),
            has_alpha: true,
        })
    }

    fn check_dimensions(&self, width: u32, height: u32) -> Result<(), ImageError> {
        if width > self.max_width || height > self.max_height {
            return Err(ImageError::DimensionsTooLarge {
                width,
                height,
                max_width: self.max_width,
                max_height: self.max_height,
            });
        }

        let memory = (width as usize) * (height as usize) * 4;
        if memory > self.max_memory {
            return Err(ImageError::MemoryLimitExceeded {
                required: memory,
                limit: self.max_memory,
            });
        }

        Ok(())
    }
}

impl Default for ImageDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Decoded image.
#[derive(Clone, Debug)]
pub struct DecodedImage {
    /// Image width.
    pub width: u32,
    /// Image height.
    pub height: u32,
    /// Original format.
    pub format: ImageFormat,
    /// RGBA pixel data.
    pub pixels: Vec<u8>,
    /// Whether image has alpha channel.
    pub has_alpha: bool,
}

impl DecodedImage {
    /// Get pixel at coordinates.
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let idx = ((y * self.width + x) * 4) as usize;
        if idx + 4 > self.pixels.len() {
            return None;
        }

        Some([
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ])
    }

    /// Set pixel at coordinates.
    pub fn set_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }

        let idx = ((y * self.width + x) * 4) as usize;
        if idx + 4 > self.pixels.len() {
            return;
        }

        self.pixels[idx] = rgba[0];
        self.pixels[idx + 1] = rgba[1];
        self.pixels[idx + 2] = rgba[2];
        self.pixels[idx + 3] = rgba[3];
    }

    /// Create a scaled version of the image.
    pub fn scale(&self, new_width: u32, new_height: u32) -> DecodedImage {
        let mut new_pixels = vec![0u8; (new_width * new_height * 4) as usize];

        let x_ratio = self.width as f32 / new_width as f32;
        let y_ratio = self.height as f32 / new_height as f32;

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f32 * x_ratio) as u32;
                let src_y = (y as f32 * y_ratio) as u32;

                if let Some(pixel) = self.get_pixel(src_x, src_y) {
                    let idx = ((y * new_width + x) * 4) as usize;
                    new_pixels[idx..idx + 4].copy_from_slice(&pixel);
                }
            }
        }

        DecodedImage {
            width: new_width,
            height: new_height,
            format: self.format,
            pixels: new_pixels,
            has_alpha: self.has_alpha,
        }
    }

    /// Memory usage in bytes.
    pub fn memory_usage(&self) -> usize {
        self.pixels.len()
    }
}

/// Image format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFormat {
    /// PNG.
    Png,
    /// JPEG.
    Jpeg,
    /// GIF.
    Gif,
    /// WebP.
    WebP,
    /// BMP.
    Bmp,
    /// ICO.
    Ico,
    /// SVG.
    Svg,
}

impl ImageFormat {
    /// Detect image format from magic bytes.
    pub fn detect(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }

        // PNG: 89 50 4E 47
        if data.starts_with(&[0x89, b'P', b'N', b'G']) {
            return Some(ImageFormat::Png);
        }

        // JPEG: FF D8 FF
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some(ImageFormat::Jpeg);
        }

        // GIF: GIF87a or GIF89a
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return Some(ImageFormat::Gif);
        }

        // WebP: RIFF....WEBP
        if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
            return Some(ImageFormat::WebP);
        }

        // BMP: BM
        if data.starts_with(b"BM") {
            return Some(ImageFormat::Bmp);
        }

        // ICO: 00 00 01 00
        if data.starts_with(&[0x00, 0x00, 0x01, 0x00]) {
            return Some(ImageFormat::Ico);
        }

        // SVG: Check for XML/SVG
        if data.starts_with(b"<?xml") || data.starts_with(b"<svg") {
            return Some(ImageFormat::Svg);
        }

        None
    }

    /// Get MIME type for format.
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Gif => "image/gif",
            ImageFormat::WebP => "image/webp",
            ImageFormat::Bmp => "image/bmp",
            ImageFormat::Ico => "image/x-icon",
            ImageFormat::Svg => "image/svg+xml",
        }
    }

    /// Get file extension.
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::WebP => "webp",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Ico => "ico",
            ImageFormat::Svg => "svg",
        }
    }
}

/// Image decoding error.
#[derive(Debug, Error)]
pub enum ImageError {
    #[error("Unknown image format")]
    UnknownFormat,

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Decoding error: {0}")]
    DecodingError(String),

    #[error("Image dimensions too large: {width}x{height} (max: {max_width}x{max_height})")]
    DimensionsTooLarge {
        width: u32,
        height: u32,
        max_width: u32,
        max_height: u32,
    },

    #[error("Memory limit exceeded: required {required} bytes, limit is {limit} bytes")]
    MemoryLimitExceeded { required: usize, limit: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(
            ImageFormat::detect(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]),
            Some(ImageFormat::Png)
        );

        assert_eq!(
            ImageFormat::detect(&[0xFF, 0xD8, 0xFF, 0xE0]),
            Some(ImageFormat::Jpeg)
        );

        assert_eq!(
            ImageFormat::detect(b"GIF89a...."),
            Some(ImageFormat::Gif)
        );

        assert_eq!(
            ImageFormat::detect(b"RIFF....WEBP"),
            Some(ImageFormat::WebP)
        );

        assert_eq!(
            ImageFormat::detect(b"BM...."),
            Some(ImageFormat::Bmp)
        );
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(ImageFormat::Png.mime_type(), "image/png");
        assert_eq!(ImageFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(ImageFormat::Gif.mime_type(), "image/gif");
    }
}
