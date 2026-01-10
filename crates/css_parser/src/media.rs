//! CSS Media Queries.

use crate::values::CssValue;

/// Media type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MediaType {
    All,
    Screen,
    Print,
    Speech,
    Unknown(String),
}

impl MediaType {
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "all" => MediaType::All,
            "screen" => MediaType::Screen,
            "print" => MediaType::Print,
            "speech" => MediaType::Speech,
            other => MediaType::Unknown(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            MediaType::All => "all",
            MediaType::Screen => "screen",
            MediaType::Print => "print",
            MediaType::Speech => "speech",
            MediaType::Unknown(s) => s,
        }
    }

    pub fn matches(&self, context: &MediaContext) -> bool {
        match self {
            MediaType::All => true,
            MediaType::Screen => context.media_type == MediaType::Screen,
            MediaType::Print => context.media_type == MediaType::Print,
            MediaType::Speech => context.media_type == MediaType::Speech,
            MediaType::Unknown(_) => false,
        }
    }
}

impl Default for MediaType {
    fn default() -> Self {
        MediaType::All
    }
}

/// Media feature.
#[derive(Clone, Debug)]
pub struct MediaFeature {
    pub name: String,
    pub value: Option<CssValue>,
}

impl MediaFeature {
    pub fn matches(&self, context: &MediaContext) -> bool {
        let name = self.name.to_ascii_lowercase();

        match name.as_str() {
            "width" => self.matches_length(context.width),
            "min-width" => self.matches_min_length(context.width),
            "max-width" => self.matches_max_length(context.width),
            "height" => self.matches_length(context.height),
            "min-height" => self.matches_min_length(context.height),
            "max-height" => self.matches_max_length(context.height),
            "device-width" => self.matches_length(context.device_width),
            "min-device-width" => self.matches_min_length(context.device_width),
            "max-device-width" => self.matches_max_length(context.device_width),
            "device-height" => self.matches_length(context.device_height),
            "min-device-height" => self.matches_min_length(context.device_height),
            "max-device-height" => self.matches_max_length(context.device_height),
            "aspect-ratio" => self.matches_ratio(context.width / context.height),
            "min-aspect-ratio" => self.matches_min_ratio(context.width / context.height),
            "max-aspect-ratio" => self.matches_max_ratio(context.width / context.height),
            "device-aspect-ratio" => {
                self.matches_ratio(context.device_width / context.device_height)
            }
            "orientation" => {
                let landscape = context.width > context.height;
                match self.value.as_ref().and_then(|v| v.as_string()) {
                    Some("landscape") => landscape,
                    Some("portrait") => !landscape,
                    _ => true,
                }
            }
            "resolution" | "min-resolution" | "max-resolution" => {
                // Would need resolution parsing
                true
            }
            "color" => {
                if self.value.is_some() {
                    self.matches_int(context.color_bits as f32)
                } else {
                    context.color_bits > 0
                }
            }
            "min-color" => self.matches_min_int(context.color_bits as f32),
            "max-color" => self.matches_max_int(context.color_bits as f32),
            "color-index" => {
                if self.value.is_some() {
                    self.matches_int(context.color_index as f32)
                } else {
                    context.color_index > 0
                }
            }
            "monochrome" => {
                if self.value.is_some() {
                    self.matches_int(context.monochrome_bits as f32)
                } else {
                    context.monochrome_bits > 0
                }
            }
            "prefers-color-scheme" => match self.value.as_ref().and_then(|v| v.as_string()) {
                Some("dark") => context.prefers_dark,
                Some("light") => !context.prefers_dark,
                _ => true,
            },
            "prefers-reduced-motion" => match self.value.as_ref().and_then(|v| v.as_string()) {
                Some("reduce") => context.prefers_reduced_motion,
                Some("no-preference") => !context.prefers_reduced_motion,
                _ => true,
            },
            "prefers-contrast" => match self.value.as_ref().and_then(|v| v.as_string()) {
                Some("more") | Some("high") => context.prefers_high_contrast,
                Some("less") | Some("low") => !context.prefers_high_contrast,
                Some("no-preference") => true,
                _ => true,
            },
            "hover" => match self.value.as_ref().and_then(|v| v.as_string()) {
                Some("hover") => context.can_hover,
                Some("none") => !context.can_hover,
                _ => true,
            },
            "pointer" => match self.value.as_ref().and_then(|v| v.as_string()) {
                Some("fine") => context.pointer == PointerType::Fine,
                Some("coarse") => context.pointer == PointerType::Coarse,
                Some("none") => context.pointer == PointerType::None,
                _ => true,
            },
            "scripting" => match self.value.as_ref().and_then(|v| v.as_string()) {
                Some("none") => !context.scripting_enabled,
                Some("initial-only") | Some("enabled") => context.scripting_enabled,
                _ => true,
            },
            "display-mode" => match self.value.as_ref().and_then(|v| v.as_string()) {
                Some(mode) => context.display_mode == mode,
                _ => true,
            },
            // Unknown features match by default (future compat)
            _ => true,
        }
    }

    fn matches_length(&self, actual: f32) -> bool {
        self.value
            .as_ref()
            .and_then(|v| v.as_px())
            .map(|expected| (actual - expected).abs() < 0.5)
            .unwrap_or(true)
    }

    fn matches_min_length(&self, actual: f32) -> bool {
        self.value
            .as_ref()
            .and_then(|v| v.as_px())
            .map(|min| actual >= min)
            .unwrap_or(true)
    }

    fn matches_max_length(&self, actual: f32) -> bool {
        self.value
            .as_ref()
            .and_then(|v| v.as_px())
            .map(|max| actual <= max)
            .unwrap_or(true)
    }

    fn matches_int(&self, actual: f32) -> bool {
        self.value
            .as_ref()
            .and_then(|v| v.as_number())
            .map(|expected| (actual - expected).abs() < 0.5)
            .unwrap_or(true)
    }

    fn matches_min_int(&self, actual: f32) -> bool {
        self.value
            .as_ref()
            .and_then(|v| v.as_number())
            .map(|min| actual >= min)
            .unwrap_or(true)
    }

    fn matches_max_int(&self, actual: f32) -> bool {
        self.value
            .as_ref()
            .and_then(|v| v.as_number())
            .map(|max| actual <= max)
            .unwrap_or(true)
    }

    fn matches_ratio(&self, _actual: f32) -> bool {
        // Would parse ratio from value
        true
    }

    fn matches_min_ratio(&self, _actual: f32) -> bool {
        true
    }

    fn matches_max_ratio(&self, _actual: f32) -> bool {
        true
    }
}

/// Media query.
#[derive(Clone, Debug)]
pub struct MediaQuery {
    pub media_type: Option<MediaType>,
    pub features: Vec<MediaFeature>,
    pub negated: bool,
}

impl MediaQuery {
    pub fn matches(&self, context: &MediaContext) -> bool {
        let type_matches = self
            .media_type
            .as_ref()
            .map(|t| t.matches(context))
            .unwrap_or(true);

        let features_match = self.features.iter().all(|f| f.matches(context));

        let result = type_matches && features_match;

        if self.negated {
            !result
        } else {
            result
        }
    }
}

impl Default for MediaQuery {
    fn default() -> Self {
        Self {
            media_type: None,
            features: Vec::new(),
            negated: false,
        }
    }
}

/// Media query list.
#[derive(Clone, Debug, Default)]
pub struct MediaQueryList {
    pub queries: Vec<MediaQuery>,
}

impl MediaQueryList {
    pub fn new() -> Self {
        Self {
            queries: Vec::new(),
        }
    }

    pub fn all() -> Self {
        Self {
            queries: vec![MediaQuery::default()],
        }
    }

    pub fn matches(&self, context: &MediaContext) -> bool {
        if self.queries.is_empty() {
            return true;
        }
        self.queries.iter().any(|q| q.matches(context))
    }
}

/// Pointer type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PointerType {
    None,
    Coarse,
    #[default]
    Fine,
}

/// Media query evaluation context.
#[derive(Clone, Debug)]
pub struct MediaContext {
    /// Viewport width in CSS pixels.
    pub width: f32,
    /// Viewport height in CSS pixels.
    pub height: f32,
    /// Device width.
    pub device_width: f32,
    /// Device height.
    pub device_height: f32,
    /// Device pixel ratio.
    pub device_pixel_ratio: f32,
    /// Color bits per channel.
    pub color_bits: u32,
    /// Color index (for indexed color).
    pub color_index: u32,
    /// Monochrome bits.
    pub monochrome_bits: u32,
    /// Media type.
    pub media_type: MediaType,
    /// Prefers dark color scheme.
    pub prefers_dark: bool,
    /// Prefers reduced motion.
    pub prefers_reduced_motion: bool,
    /// Prefers high contrast.
    pub prefers_high_contrast: bool,
    /// Device can hover.
    pub can_hover: bool,
    /// Pointer type.
    pub pointer: PointerType,
    /// Scripting enabled.
    pub scripting_enabled: bool,
    /// Display mode (browser, standalone, etc).
    pub display_mode: String,
}

impl Default for MediaContext {
    fn default() -> Self {
        Self {
            width: 1920.0,
            height: 1080.0,
            device_width: 1920.0,
            device_height: 1080.0,
            device_pixel_ratio: 1.0,
            color_bits: 8,
            color_index: 0,
            monochrome_bits: 0,
            media_type: MediaType::Screen,
            prefers_dark: false,
            prefers_reduced_motion: false,
            prefers_high_contrast: false,
            can_hover: true,
            pointer: PointerType::Fine,
            scripting_enabled: true,
            display_mode: "browser".to_string(),
        }
    }
}

impl MediaContext {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            device_width: width,
            device_height: height,
            ..Default::default()
        }
    }

    pub fn screen(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            device_width: width,
            device_height: height,
            media_type: MediaType::Screen,
            ..Default::default()
        }
    }

    pub fn print(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            device_width: width,
            device_height: height,
            media_type: MediaType::Print,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type() {
        let context = MediaContext::screen(1920.0, 1080.0);
        assert!(MediaType::All.matches(&context));
        assert!(MediaType::Screen.matches(&context));
        assert!(!MediaType::Print.matches(&context));
    }

    #[test]
    fn test_min_width() {
        let context = MediaContext::new(800.0, 600.0);

        let feature = MediaFeature {
            name: "min-width".to_string(),
            value: Some(CssValue::Dimension(768.0, "px".to_string())),
        };
        assert!(feature.matches(&context));

        let feature = MediaFeature {
            name: "min-width".to_string(),
            value: Some(CssValue::Dimension(1024.0, "px".to_string())),
        };
        assert!(!feature.matches(&context));
    }

    #[test]
    fn test_prefers_color_scheme() {
        let mut context = MediaContext::default();

        let dark_feature = MediaFeature {
            name: "prefers-color-scheme".to_string(),
            value: Some(CssValue::Ident("dark".to_string())),
        };

        assert!(!dark_feature.matches(&context));

        context.prefers_dark = true;
        assert!(dark_feature.matches(&context));
    }

    #[test]
    fn test_media_query_negation() {
        let context = MediaContext::screen(1920.0, 1080.0);

        let query = MediaQuery {
            media_type: Some(MediaType::Print),
            features: vec![],
            negated: false,
        };
        assert!(!query.matches(&context));

        let query = MediaQuery {
            media_type: Some(MediaType::Print),
            features: vec![],
            negated: true,
        };
        assert!(query.matches(&context));
    }
}
