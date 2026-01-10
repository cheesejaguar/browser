//! CSS Values.

use common::color::Color;
use ordered_float::OrderedFloat;
use std::fmt;

/// CSS value type.
#[derive(Clone, Debug, PartialEq)]
pub enum CssValue {
    /// Keyword/identifier.
    Ident(String),
    /// String value.
    String(String),
    /// Number.
    Number(f32),
    /// Percentage.
    Percentage(f32),
    /// Dimension (value + unit).
    Dimension(f32, String),
    /// Color (hex, name, or function).
    Color(String),
    /// Function call.
    Function(String, Vec<CssValue>),
    /// URL.
    Url(String),
    /// List of values.
    List(Vec<CssValue>),
    /// Operator (/, comma in some contexts).
    Operator(String),
    /// Initial keyword.
    Initial,
    /// Inherit keyword.
    Inherit,
    /// Unset keyword.
    Unset,
    /// Revert keyword.
    Revert,
}

impl CssValue {
    /// Check if this is a keyword.
    pub fn is_keyword(&self, keyword: &str) -> bool {
        matches!(self, CssValue::Ident(s) if s.eq_ignore_ascii_case(keyword))
    }

    /// Get as string if this is an ident or string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            CssValue::Ident(s) | CssValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as number.
    pub fn as_number(&self) -> Option<f32> {
        match self {
            CssValue::Number(n) => Some(*n),
            CssValue::Dimension(n, _) => Some(*n),
            CssValue::Percentage(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as length in pixels (if possible without context).
    pub fn as_px(&self) -> Option<f32> {
        match self {
            CssValue::Number(n) if *n == 0.0 => Some(0.0),
            CssValue::Dimension(n, unit) => {
                match unit.as_str() {
                    "px" => Some(*n),
                    "pt" => Some(n * 96.0 / 72.0),
                    "pc" => Some(n * 96.0 / 6.0),
                    "in" => Some(n * 96.0),
                    "cm" => Some(n * 96.0 / 2.54),
                    "mm" => Some(n * 96.0 / 25.4),
                    "q" => Some(n * 96.0 / 101.6),
                    _ => None, // Relative units need context
                }
            }
            _ => None,
        }
    }

    /// Get as color.
    pub fn as_color(&self) -> Option<Color> {
        match self {
            CssValue::Color(s) => Color::from_hex(s).or_else(|| Color::from_name(s)),
            CssValue::Ident(s) => Color::from_name(s),
            CssValue::Function(name, args) => parse_color_function(name, args),
            _ => None,
        }
    }

    /// Get as URL.
    pub fn as_url(&self) -> Option<&str> {
        match self {
            CssValue::Url(url) => Some(url),
            CssValue::Function(name, args) if name == "url" => {
                args.first().and_then(|v| v.as_string())
            }
            _ => None,
        }
    }

    /// Check if value is initial/inherit/unset/revert.
    pub fn is_css_wide_keyword(&self) -> bool {
        matches!(
            self,
            CssValue::Initial | CssValue::Inherit | CssValue::Unset | CssValue::Revert
        )
    }

    /// Convert to CSS string.
    pub fn to_css_string(&self) -> String {
        match self {
            CssValue::Ident(s) => s.clone(),
            CssValue::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            CssValue::Number(n) => format_number(*n),
            CssValue::Percentage(n) => format!("{}%", format_number(*n)),
            CssValue::Dimension(n, unit) => format!("{}{}", format_number(*n), unit),
            CssValue::Color(s) => s.clone(),
            CssValue::Function(name, args) => {
                let args_str = args
                    .iter()
                    .map(|a| a.to_css_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", name, args_str)
            }
            CssValue::Url(url) => format!("url(\"{}\")", url),
            CssValue::List(values) => values
                .iter()
                .map(|v| v.to_css_string())
                .collect::<Vec<_>>()
                .join(" "),
            CssValue::Operator(op) => op.clone(),
            CssValue::Initial => "initial".to_string(),
            CssValue::Inherit => "inherit".to_string(),
            CssValue::Unset => "unset".to_string(),
            CssValue::Revert => "revert".to_string(),
        }
    }
}

impl Default for CssValue {
    fn default() -> Self {
        CssValue::Initial
    }
}

impl fmt::Display for CssValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_css_string())
    }
}

/// Format number removing unnecessary decimals.
fn format_number(n: f32) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i32)
    } else {
        format!("{:.3}", n).trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Parse color function.
fn parse_color_function(name: &str, args: &[CssValue]) -> Option<Color> {
    match name {
        "rgb" | "rgba" => {
            let (r, g, b, a) = parse_rgb_args(args)?;
            Some(Color::rgba(r, g, b, a))
        }
        "hsl" | "hsla" => {
            let (h, s, l, a) = parse_hsl_args(args)?;
            Some(hsl_to_rgb(h, s, l, a))
        }
        _ => None,
    }
}

fn parse_rgb_args(args: &[CssValue]) -> Option<(u8, u8, u8, u8)> {
    let get_component = |v: &CssValue| -> Option<u8> {
        match v {
            CssValue::Number(n) => Some((*n).clamp(0.0, 255.0) as u8),
            CssValue::Percentage(p) => Some((p * 2.55).clamp(0.0, 255.0) as u8),
            _ => None,
        }
    };

    let r = args.first().and_then(get_component)?;
    let g = args.get(1).and_then(get_component)?;
    let b = args.get(2).and_then(get_component)?;
    let a = args
        .get(3)
        .and_then(|v| match v {
            CssValue::Number(n) => Some((n * 255.0).clamp(0.0, 255.0) as u8),
            CssValue::Percentage(p) => Some((p * 2.55).clamp(0.0, 255.0) as u8),
            _ => None,
        })
        .unwrap_or(255);

    Some((r, g, b, a))
}

fn parse_hsl_args(args: &[CssValue]) -> Option<(f32, f32, f32, f32)> {
    let h = args.first().and_then(|v| v.as_number())?;
    let s = args.get(1).and_then(|v| match v {
        CssValue::Percentage(p) => Some(p / 100.0),
        _ => None,
    })?;
    let l = args.get(2).and_then(|v| match v {
        CssValue::Percentage(p) => Some(p / 100.0),
        _ => None,
    })?;
    let a = args
        .get(3)
        .and_then(|v| match v {
            CssValue::Number(n) => Some(*n),
            CssValue::Percentage(p) => Some(p / 100.0),
            _ => None,
        })
        .unwrap_or(1.0);

    Some((h, s, l, a))
}

fn hsl_to_rgb(h: f32, s: f32, l: f32, a: f32) -> Color {
    let h = h % 360.0 / 360.0;
    let s = s.clamp(0.0, 1.0);
    let l = l.clamp(0.0, 1.0);

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
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    )
}

/// List of CSS values.
#[derive(Clone, Debug, Default)]
pub struct CssValueList {
    pub values: Vec<CssValue>,
}

impl CssValueList {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn push(&mut self, value: CssValue) {
        self.values.push(value);
    }

    pub fn first(&self) -> Option<&CssValue> {
        self.values.first()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &CssValue> {
        self.values.iter()
    }
}

/// Computed value (after resolving relative units, etc.)
#[derive(Clone, Debug)]
pub enum ComputedValue {
    /// Absolute length in pixels.
    Length(f32),
    /// Percentage (relative to containing block).
    Percentage(f32),
    /// Computed color.
    Color(Color),
    /// Computed string.
    String(String),
    /// Computed number.
    Number(f32),
    /// Computed URL.
    Url(String),
    /// Keyword.
    Keyword(String),
    /// List of computed values.
    List(Vec<ComputedValue>),
    /// Auto value.
    Auto,
    /// None value.
    None,
}

impl ComputedValue {
    pub fn as_length(&self) -> Option<f32> {
        match self {
            ComputedValue::Length(l) => Some(*l),
            ComputedValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_color(&self) -> Option<&Color> {
        match self {
            ComputedValue::Color(c) => Some(c),
            _ => None,
        }
    }

    pub fn is_auto(&self) -> bool {
        matches!(self, ComputedValue::Auto)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, ComputedValue::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_value_number() {
        let value = CssValue::Number(42.0);
        assert_eq!(value.as_number(), Some(42.0));
    }

    #[test]
    fn test_css_value_px() {
        let value = CssValue::Dimension(16.0, "px".to_string());
        assert_eq!(value.as_px(), Some(16.0));

        let em = CssValue::Dimension(1.0, "em".to_string());
        assert_eq!(em.as_px(), None); // Needs context
    }

    #[test]
    fn test_css_value_to_string() {
        let value = CssValue::Dimension(16.5, "px".to_string());
        assert_eq!(value.to_css_string(), "16.5px");

        let percentage = CssValue::Percentage(50.0);
        assert_eq!(percentage.to_css_string(), "50%");
    }

    #[test]
    fn test_hsl_conversion() {
        let red = hsl_to_rgb(0.0, 1.0, 0.5, 1.0);
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);
        assert_eq!(red.b, 0);
    }
}
