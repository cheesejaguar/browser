//! Color representation and manipulation.

use serde::{Deserialize, Serialize};
use std::fmt;

/// RGBA color with 8-bit components.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const RED: Color = Color::rgb(255, 0, 0);
    pub const GREEN: Color = Color::rgb(0, 128, 0);
    pub const BLUE: Color = Color::rgb(0, 0, 255);

    #[inline]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    #[inline]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create color from floating point values (0.0 - 1.0).
    pub fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0) as u8,
            a: (a.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }

    /// Parse color from hex string (e.g., "#ff0000", "#f00", "#ff000080").
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);

        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                Some(Self::rgb(r * 17, g * 17, b * 17))
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                let a = u8::from_str_radix(&hex[3..4], 16).ok()?;
                Some(Self::rgba(r * 17, g * 17, b * 17, a * 17))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::rgba(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Get named CSS color.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "transparent" => Some(Self::TRANSPARENT),
            "black" => Some(Self::BLACK),
            "white" => Some(Self::WHITE),
            "red" => Some(Self::RED),
            "green" => Some(Self::GREEN),
            "blue" => Some(Self::BLUE),
            "yellow" => Some(Self::rgb(255, 255, 0)),
            "cyan" | "aqua" => Some(Self::rgb(0, 255, 255)),
            "magenta" | "fuchsia" => Some(Self::rgb(255, 0, 255)),
            "gray" | "grey" => Some(Self::rgb(128, 128, 128)),
            "silver" => Some(Self::rgb(192, 192, 192)),
            "maroon" => Some(Self::rgb(128, 0, 0)),
            "olive" => Some(Self::rgb(128, 128, 0)),
            "lime" => Some(Self::rgb(0, 255, 0)),
            "teal" => Some(Self::rgb(0, 128, 128)),
            "navy" => Some(Self::rgb(0, 0, 128)),
            "purple" => Some(Self::rgb(128, 0, 128)),
            "orange" => Some(Self::rgb(255, 165, 0)),
            "pink" => Some(Self::rgb(255, 192, 203)),
            "brown" => Some(Self::rgb(165, 42, 42)),
            "coral" => Some(Self::rgb(255, 127, 80)),
            "crimson" => Some(Self::rgb(220, 20, 60)),
            "darkblue" => Some(Self::rgb(0, 0, 139)),
            "darkgray" | "darkgrey" => Some(Self::rgb(169, 169, 169)),
            "darkgreen" => Some(Self::rgb(0, 100, 0)),
            "darkred" => Some(Self::rgb(139, 0, 0)),
            "gold" => Some(Self::rgb(255, 215, 0)),
            "indigo" => Some(Self::rgb(75, 0, 130)),
            "ivory" => Some(Self::rgb(255, 255, 240)),
            "khaki" => Some(Self::rgb(240, 230, 140)),
            "lavender" => Some(Self::rgb(230, 230, 250)),
            "lightblue" => Some(Self::rgb(173, 216, 230)),
            "lightgray" | "lightgrey" => Some(Self::rgb(211, 211, 211)),
            "lightgreen" => Some(Self::rgb(144, 238, 144)),
            "lightyellow" => Some(Self::rgb(255, 255, 224)),
            "mintcream" => Some(Self::rgb(245, 255, 250)),
            "mistyrose" => Some(Self::rgb(255, 228, 225)),
            "moccasin" => Some(Self::rgb(255, 228, 181)),
            "oldlace" => Some(Self::rgb(253, 245, 230)),
            "orangered" => Some(Self::rgb(255, 69, 0)),
            "orchid" => Some(Self::rgb(218, 112, 214)),
            "plum" => Some(Self::rgb(221, 160, 221)),
            "salmon" => Some(Self::rgb(250, 128, 114)),
            "skyblue" => Some(Self::rgb(135, 206, 235)),
            "slateblue" => Some(Self::rgb(106, 90, 205)),
            "slategray" | "slategrey" => Some(Self::rgb(112, 128, 144)),
            "snow" => Some(Self::rgb(255, 250, 250)),
            "steelblue" => Some(Self::rgb(70, 130, 180)),
            "tan" => Some(Self::rgb(210, 180, 140)),
            "thistle" => Some(Self::rgb(216, 191, 216)),
            "tomato" => Some(Self::rgb(255, 99, 71)),
            "turquoise" => Some(Self::rgb(64, 224, 208)),
            "violet" => Some(Self::rgb(238, 130, 238)),
            "wheat" => Some(Self::rgb(245, 222, 179)),
            "whitesmoke" => Some(Self::rgb(245, 245, 245)),
            "yellowgreen" => Some(Self::rgb(154, 205, 50)),
            _ => None,
        }
    }

    /// Convert to f32 array for GPU.
    #[inline]
    pub fn to_f32_array(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    /// Convert to packed u32 (RGBA).
    #[inline]
    pub fn to_u32(&self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }

    /// Blend with another color using alpha compositing.
    pub fn blend_over(&self, background: Color) -> Color {
        let fg_a = self.a as f32 / 255.0;
        let bg_a = background.a as f32 / 255.0;

        let out_a = fg_a + bg_a * (1.0 - fg_a);

        if out_a == 0.0 {
            return Color::TRANSPARENT;
        }

        let blend = |fg: u8, bg: u8| -> u8 {
            let fg = fg as f32 / 255.0;
            let bg = bg as f32 / 255.0;
            let out = (fg * fg_a + bg * bg_a * (1.0 - fg_a)) / out_a;
            (out * 255.0) as u8
        };

        Color::rgba(
            blend(self.r, background.r),
            blend(self.g, background.g),
            blend(self.b, background.b),
            (out_a * 255.0) as u8,
        )
    }

    /// Lighten or darken the color.
    pub fn adjust_lightness(&self, factor: f32) -> Color {
        let adjust = |c: u8| -> u8 {
            if factor > 0.0 {
                (c as f32 + (255.0 - c as f32) * factor).min(255.0) as u8
            } else {
                (c as f32 * (1.0 + factor)).max(0.0) as u8
            }
        };

        Color::rgba(adjust(self.r), adjust(self.g), adjust(self.b), self.a)
    }
}

impl fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 255 {
            write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            write!(f, "#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex() {
        assert_eq!(Color::from_hex("#ff0000"), Some(Color::RED));
        assert_eq!(Color::from_hex("#f00"), Some(Color::RED));
        assert_eq!(Color::from_hex("00ff00"), Some(Color::rgb(0, 255, 0)));
        assert_eq!(Color::from_hex("#ffffff80"), Some(Color::rgba(255, 255, 255, 128)));
    }

    #[test]
    fn test_from_name() {
        assert_eq!(Color::from_name("red"), Some(Color::RED));
        assert_eq!(Color::from_name("WHITE"), Some(Color::WHITE));
        assert_eq!(Color::from_name("transparent"), Some(Color::TRANSPARENT));
    }
}
