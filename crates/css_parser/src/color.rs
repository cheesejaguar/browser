//! CSS Color parsing utilities.

use common::color::Color;

/// Parse CSS color string.
pub fn parse_color(input: &str) -> Option<Color> {
    let input = input.trim();

    // Try hex
    if input.starts_with('#') {
        return Color::from_hex(input);
    }

    // Try named color
    if let Some(color) = Color::from_name(input) {
        return Some(color);
    }

    // Try rgb/rgba
    if input.starts_with("rgb") {
        return parse_rgb_color(input);
    }

    // Try hsl/hsla
    if input.starts_with("hsl") {
        return parse_hsl_color(input);
    }

    None
}

/// Parse rgb() or rgba() color.
fn parse_rgb_color(input: &str) -> Option<Color> {
    let has_alpha = input.starts_with("rgba");
    let start = if has_alpha { 5 } else { 4 };
    let end = input.len().saturating_sub(1);

    if end <= start {
        return None;
    }

    let inner = &input[start..end];
    let parts: Vec<&str> = inner.split(|c| c == ',' || c == ' ' || c == '/').collect();
    let parts: Vec<&str> = parts.into_iter().filter(|s| !s.is_empty()).collect();

    if parts.len() < 3 {
        return None;
    }

    let parse_component = |s: &str| -> Option<u8> {
        let s = s.trim();
        if s.ends_with('%') {
            let pct: f32 = s[..s.len() - 1].parse().ok()?;
            Some((pct * 2.55).clamp(0.0, 255.0) as u8)
        } else {
            let val: f32 = s.parse().ok()?;
            Some(val.clamp(0.0, 255.0) as u8)
        }
    };

    let r = parse_component(parts[0])?;
    let g = parse_component(parts[1])?;
    let b = parse_component(parts[2])?;

    let a = if parts.len() > 3 {
        let s = parts[3].trim();
        if s.ends_with('%') {
            let pct: f32 = s[..s.len() - 1].parse().ok()?;
            (pct * 2.55).clamp(0.0, 255.0) as u8
        } else {
            let val: f32 = s.parse().ok()?;
            (val * 255.0).clamp(0.0, 255.0) as u8
        }
    } else {
        255
    };

    Some(Color::rgba(r, g, b, a))
}

/// Parse hsl() or hsla() color.
fn parse_hsl_color(input: &str) -> Option<Color> {
    let has_alpha = input.starts_with("hsla");
    let start = if has_alpha { 5 } else { 4 };
    let end = input.len().saturating_sub(1);

    if end <= start {
        return None;
    }

    let inner = &input[start..end];
    let parts: Vec<&str> = inner.split(|c| c == ',' || c == ' ' || c == '/').collect();
    let parts: Vec<&str> = parts.into_iter().filter(|s| !s.is_empty()).collect();

    if parts.len() < 3 {
        return None;
    }

    let h: f32 = {
        let s = parts[0].trim();
        if s.ends_with("deg") {
            s[..s.len() - 3].parse().ok()?
        } else if s.ends_with("rad") {
            let rad: f32 = s[..s.len() - 3].parse().ok()?;
            rad.to_degrees()
        } else if s.ends_with("turn") {
            let turn: f32 = s[..s.len() - 4].parse().ok()?;
            turn * 360.0
        } else {
            s.parse().ok()?
        }
    };

    let parse_percent = |s: &str| -> Option<f32> {
        let s = s.trim();
        if s.ends_with('%') {
            s[..s.len() - 1].parse::<f32>().ok().map(|v| v / 100.0)
        } else {
            s.parse::<f32>().ok().map(|v| v / 100.0)
        }
    };

    let s = parse_percent(parts[1])?;
    let l = parse_percent(parts[2])?;

    let a = if parts.len() > 3 {
        let s = parts[3].trim();
        if s.ends_with('%') {
            s[..s.len() - 1].parse::<f32>().ok()? / 100.0
        } else {
            s.parse().ok()?
        }
    } else {
        1.0
    };

    Some(hsl_to_rgb(h, s, l, a))
}

/// Convert HSL to RGB.
fn hsl_to_rgb(h: f32, s: f32, l: f32, a: f32) -> Color {
    let h = ((h % 360.0) + 360.0) % 360.0 / 360.0;
    let s = s.clamp(0.0, 1.0);
    let l = l.clamp(0.0, 1.0);
    let a = a.clamp(0.0, 1.0);

    let (r, g, b) = if s == 0.0 {
        (l, l, l)
    } else {
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

        (
            hue_to_rgb(p, q, h + 1.0 / 3.0),
            hue_to_rgb(p, q, h),
            hue_to_rgb(p, q, h - 1.0 / 3.0),
        )
    };

    Color::rgba(
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
        (a * 255.0).round() as u8,
    )
}

/// Convert RGB to HSL.
pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if max == min {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if max == r {
        let mut h = (g - b) / d;
        if g < b {
            h += 6.0;
        }
        h
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } / 6.0;

    (h * 360.0, s, l)
}

/// Color interpolation for transitions/animations.
pub fn interpolate_color(from: Color, to: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);

    // Interpolate in sRGB space (simplified)
    let r = from.r as f32 + (to.r as f32 - from.r as f32) * t;
    let g = from.g as f32 + (to.g as f32 - from.g as f32) * t;
    let b = from.b as f32 + (to.b as f32 - from.b as f32) * t;
    let a = from.a as f32 + (to.a as f32 - from.a as f32) * t;

    Color::rgba(r as u8, g as u8, b as u8, a as u8)
}

/// System colors.
pub fn system_color(name: &str) -> Option<Color> {
    // These would ideally come from the OS
    match name.to_ascii_lowercase().as_str() {
        "canvas" => Some(Color::WHITE),
        "canvastext" => Some(Color::BLACK),
        "linktext" => Some(Color::rgb(0, 0, 238)),
        "visitedtext" => Some(Color::rgb(85, 26, 139)),
        "activetext" => Some(Color::rgb(255, 0, 0)),
        "buttonface" => Some(Color::rgb(240, 240, 240)),
        "buttontext" => Some(Color::BLACK),
        "buttonborder" => Some(Color::rgb(118, 118, 118)),
        "field" => Some(Color::WHITE),
        "fieldtext" => Some(Color::BLACK),
        "highlight" => Some(Color::rgb(0, 120, 215)),
        "highlighttext" => Some(Color::WHITE),
        "selecteditem" => Some(Color::rgb(0, 120, 215)),
        "selecteditemtext" => Some(Color::WHITE),
        "mark" => Some(Color::rgb(255, 255, 0)),
        "marktext" => Some(Color::BLACK),
        "graytext" => Some(Color::rgb(109, 109, 109)),
        "accentcolor" => Some(Color::rgb(0, 120, 215)),
        "accentcolortext" => Some(Color::WHITE),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex() {
        assert_eq!(parse_color("#ff0000"), Some(Color::RED));
        assert_eq!(parse_color("#f00"), Some(Color::RED));
        assert_eq!(parse_color("#00ff00"), Some(Color::rgb(0, 255, 0)));
    }

    #[test]
    fn test_parse_rgb() {
        assert_eq!(parse_color("rgb(255, 0, 0)"), Some(Color::RED));
        assert_eq!(parse_color("rgba(255, 0, 0, 1)"), Some(Color::RED));
        assert_eq!(parse_color("rgb(100%, 0%, 0%)"), Some(Color::RED));
    }

    #[test]
    fn test_parse_hsl() {
        let red = parse_color("hsl(0, 100%, 50%)").unwrap();
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);
        assert_eq!(red.b, 0);
    }

    #[test]
    fn test_interpolate() {
        let white = Color::WHITE;
        let black = Color::BLACK;

        let mid = interpolate_color(white, black, 0.5);
        assert_eq!(mid.r, 127);
        assert_eq!(mid.g, 127);
        assert_eq!(mid.b, 127);
    }
}
