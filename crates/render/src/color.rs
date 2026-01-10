//! Color utilities for rendering.

use common::color::Color;

/// Color space conversion utilities.
pub struct ColorSpace;

impl ColorSpace {
    /// Convert sRGB to linear RGB.
    pub fn srgb_to_linear(c: u8) -> f32 {
        let c = c as f32 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Convert linear RGB to sRGB.
    pub fn linear_to_srgb(c: f32) -> u8 {
        let c = if c <= 0.0031308 {
            c * 12.92
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (c * 255.0).clamp(0.0, 255.0) as u8
    }

    /// Convert RGB to HSL.
    pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        if (max - min).abs() < f32::EPSILON {
            return (0.0, 0.0, l);
        }

        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };

        let h = if (max - r).abs() < f32::EPSILON {
            (g - b) / d + (if g < b { 6.0 } else { 0.0 })
        } else if (max - g).abs() < f32::EPSILON {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        } / 6.0;

        (h, s, l)
    }

    /// Convert HSL to RGB.
    pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
        if s.abs() < f32::EPSILON {
            let v = (l * 255.0) as u8;
            return (v, v, v);
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        let hue_to_rgb = |p: f32, q: f32, mut t: f32| -> f32 {
            if t < 0.0 {
                t += 1.0;
            }
            if t > 1.0 {
                t -= 1.0;
            }
            if t < 1.0 / 6.0 {
                return p + (q - p) * 6.0 * t;
            }
            if t < 1.0 / 2.0 {
                return q;
            }
            if t < 2.0 / 3.0 {
                return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
            }
            p
        };

        let r = (hue_to_rgb(p, q, h + 1.0 / 3.0) * 255.0) as u8;
        let g = (hue_to_rgb(p, q, h) * 255.0) as u8;
        let b = (hue_to_rgb(p, q, h - 1.0 / 3.0) * 255.0) as u8;

        (r, g, b)
    }

    /// Convert RGB to HSV.
    pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let v = max;

        if (max - min).abs() < f32::EPSILON {
            return (0.0, 0.0, v);
        }

        let d = max - min;
        let s = d / max;

        let h = if (max - r).abs() < f32::EPSILON {
            (g - b) / d + (if g < b { 6.0 } else { 0.0 })
        } else if (max - g).abs() < f32::EPSILON {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        } / 6.0;

        (h, s, v)
    }

    /// Convert HSV to RGB.
    pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
        if s.abs() < f32::EPSILON {
            let val = (v * 255.0) as u8;
            return (val, val, val);
        }

        let i = (h * 6.0).floor();
        let f = h * 6.0 - i;
        let p = v * (1.0 - s);
        let q = v * (1.0 - f * s);
        let t = v * (1.0 - (1.0 - f) * s);

        let (r, g, b) = match (i as i32) % 6 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };

        ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }

    /// Get relative luminance (for WCAG contrast).
    pub fn relative_luminance(r: u8, g: u8, b: u8) -> f32 {
        let r = Self::srgb_to_linear(r);
        let g = Self::srgb_to_linear(g);
        let b = Self::srgb_to_linear(b);

        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Calculate contrast ratio between two colors.
    pub fn contrast_ratio(color1: &Color, color2: &Color) -> f32 {
        let l1 = Self::relative_luminance(color1.r, color1.g, color1.b);
        let l2 = Self::relative_luminance(color2.r, color2.g, color2.b);

        let lighter = l1.max(l2);
        let darker = l1.min(l2);

        (lighter + 0.05) / (darker + 0.05)
    }
}

/// Color manipulation utilities.
pub struct ColorManip;

impl ColorManip {
    /// Lighten a color.
    pub fn lighten(color: &Color, amount: f32) -> Color {
        let (h, s, l) = ColorSpace::rgb_to_hsl(color.r, color.g, color.b);
        let l = (l + amount).clamp(0.0, 1.0);
        let (r, g, b) = ColorSpace::hsl_to_rgb(h, s, l);
        Color::new(r, g, b, color.a)
    }

    /// Darken a color.
    pub fn darken(color: &Color, amount: f32) -> Color {
        let (h, s, l) = ColorSpace::rgb_to_hsl(color.r, color.g, color.b);
        let l = (l - amount).clamp(0.0, 1.0);
        let (r, g, b) = ColorSpace::hsl_to_rgb(h, s, l);
        Color::new(r, g, b, color.a)
    }

    /// Saturate a color.
    pub fn saturate(color: &Color, amount: f32) -> Color {
        let (h, s, l) = ColorSpace::rgb_to_hsl(color.r, color.g, color.b);
        let s = (s + amount).clamp(0.0, 1.0);
        let (r, g, b) = ColorSpace::hsl_to_rgb(h, s, l);
        Color::new(r, g, b, color.a)
    }

    /// Desaturate a color.
    pub fn desaturate(color: &Color, amount: f32) -> Color {
        let (h, s, l) = ColorSpace::rgb_to_hsl(color.r, color.g, color.b);
        let s = (s - amount).clamp(0.0, 1.0);
        let (r, g, b) = ColorSpace::hsl_to_rgb(h, s, l);
        Color::new(r, g, b, color.a)
    }

    /// Adjust hue.
    pub fn adjust_hue(color: &Color, degrees: f32) -> Color {
        let (h, s, l) = ColorSpace::rgb_to_hsl(color.r, color.g, color.b);
        let h = (h + degrees / 360.0).rem_euclid(1.0);
        let (r, g, b) = ColorSpace::hsl_to_rgb(h, s, l);
        Color::new(r, g, b, color.a)
    }

    /// Invert a color.
    pub fn invert(color: &Color) -> Color {
        Color::new(
            255 - color.r,
            255 - color.g,
            255 - color.b,
            color.a,
        )
    }

    /// Get grayscale version.
    pub fn grayscale(color: &Color) -> Color {
        let gray = ((color.r as u32 + color.g as u32 + color.b as u32) / 3) as u8;
        Color::new(gray, gray, gray, color.a)
    }

    /// Get sepia version.
    pub fn sepia(color: &Color) -> Color {
        let r = color.r as f32;
        let g = color.g as f32;
        let b = color.b as f32;

        let new_r = (0.393 * r + 0.769 * g + 0.189 * b).min(255.0) as u8;
        let new_g = (0.349 * r + 0.686 * g + 0.168 * b).min(255.0) as u8;
        let new_b = (0.272 * r + 0.534 * g + 0.131 * b).min(255.0) as u8;

        Color::new(new_r, new_g, new_b, color.a)
    }

    /// Mix two colors.
    pub fn mix(color1: &Color, color2: &Color, weight: f32) -> Color {
        let w = weight.clamp(0.0, 1.0);
        let w1 = 1.0 - w;

        Color::new(
            (color1.r as f32 * w1 + color2.r as f32 * w) as u8,
            (color1.g as f32 * w1 + color2.g as f32 * w) as u8,
            (color1.b as f32 * w1 + color2.b as f32 * w) as u8,
            (color1.a as f32 * w1 + color2.a as f32 * w) as u8,
        )
    }

    /// Get complementary color.
    pub fn complement(color: &Color) -> Color {
        Self::adjust_hue(color, 180.0)
    }

    /// Get triadic colors.
    pub fn triadic(color: &Color) -> (Color, Color, Color) {
        (
            color.clone(),
            Self::adjust_hue(color, 120.0),
            Self::adjust_hue(color, 240.0),
        )
    }

    /// Get analogous colors.
    pub fn analogous(color: &Color) -> (Color, Color, Color) {
        (
            Self::adjust_hue(color, -30.0),
            color.clone(),
            Self::adjust_hue(color, 30.0),
        )
    }
}

/// Premultiplied alpha operations.
pub struct PremultipliedAlpha;

impl PremultipliedAlpha {
    /// Convert to premultiplied alpha.
    pub fn premultiply(color: &Color) -> (u8, u8, u8, u8) {
        let a = color.a as f32 / 255.0;
        (
            (color.r as f32 * a) as u8,
            (color.g as f32 * a) as u8,
            (color.b as f32 * a) as u8,
            color.a,
        )
    }

    /// Convert from premultiplied alpha.
    pub fn unpremultiply(r: u8, g: u8, b: u8, a: u8) -> Color {
        if a == 0 {
            return Color::transparent();
        }

        let a_f = a as f32 / 255.0;
        Color::new(
            (r as f32 / a_f).min(255.0) as u8,
            (g as f32 / a_f).min(255.0) as u8,
            (b as f32 / a_f).min(255.0) as u8,
            a,
        )
    }

    /// Blend premultiplied colors.
    pub fn blend_premultiplied(
        dst_r: u8, dst_g: u8, dst_b: u8, dst_a: u8,
        src_r: u8, src_g: u8, src_b: u8, src_a: u8,
    ) -> (u8, u8, u8, u8) {
        let src_a_f = src_a as f32 / 255.0;
        let inv_src_a = 1.0 - src_a_f;

        (
            (src_r as f32 + dst_r as f32 * inv_src_a) as u8,
            (src_g as f32 + dst_g as f32 * inv_src_a) as u8,
            (src_b as f32 + dst_b as f32 * inv_src_a) as u8,
            (src_a as f32 + dst_a as f32 * inv_src_a) as u8,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_linear_conversion() {
        // Black and white should be the same
        assert!((ColorSpace::srgb_to_linear(0) - 0.0).abs() < 0.001);
        assert!((ColorSpace::srgb_to_linear(255) - 1.0).abs() < 0.001);

        // Round-trip
        for i in 0..=255u8 {
            let linear = ColorSpace::srgb_to_linear(i);
            let back = ColorSpace::linear_to_srgb(linear);
            assert!((i as i32 - back as i32).abs() <= 1);
        }
    }

    #[test]
    fn test_hsl_conversion() {
        // Red
        let (h, s, l) = ColorSpace::rgb_to_hsl(255, 0, 0);
        assert!((h - 0.0).abs() < 0.01);
        assert!((s - 1.0).abs() < 0.01);
        assert!((l - 0.5).abs() < 0.01);

        // Round-trip
        let (r, g, b) = ColorSpace::hsl_to_rgb(h, s, l);
        assert_eq!(r, 255);
        assert!(g < 5);
        assert!(b < 5);
    }

    #[test]
    fn test_contrast_ratio() {
        let white = Color::white();
        let black = Color::black();

        let ratio = ColorSpace::contrast_ratio(&white, &black);
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn test_color_manipulation() {
        let red = Color::rgb(255, 0, 0);

        // Lighten
        let lighter = ColorManip::lighten(&red, 0.2);
        assert!(lighter.r >= red.r || lighter.g > 0 || lighter.b > 0);

        // Complement of red is cyan
        let comp = ColorManip::complement(&red);
        assert!(comp.g > 200);
        assert!(comp.b > 200);
        assert!(comp.r < 50);
    }
}
