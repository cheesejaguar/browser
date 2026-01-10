//! Text layout and shaping.

use common::geometry::{Point, Rect};
use style::computed::ComputedStyle;
use unicode_segmentation::UnicodeSegmentation;

/// Text run (shaped text ready for rendering).
#[derive(Clone, Debug)]
pub struct TextRun {
    /// Original text content.
    pub text: String,
    /// Shaped glyphs.
    pub glyphs: Vec<ShapedGlyph>,
    /// Total width.
    pub width: f32,
    /// Total height (line height).
    pub height: f32,
    /// Ascent from baseline.
    pub ascent: f32,
    /// Descent from baseline.
    pub descent: f32,
    /// Font size used.
    pub font_size: f32,
    /// Line breaks.
    pub line_breaks: Vec<usize>,
}

impl TextRun {
    pub fn empty() -> Self {
        Self {
            text: String::new(),
            glyphs: Vec::new(),
            width: 0.0,
            height: 0.0,
            ascent: 0.0,
            descent: 0.0,
            font_size: 16.0,
            line_breaks: Vec::new(),
        }
    }

    /// Get number of glyphs.
    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    /// Get baseline position for a line.
    pub fn baseline(&self) -> f32 {
        self.ascent
    }

    /// Find glyph at x position.
    pub fn glyph_at_x(&self, x: f32) -> Option<usize> {
        let mut current_x = 0.0;
        for (i, glyph) in self.glyphs.iter().enumerate() {
            if x < current_x + glyph.advance / 2.0 {
                return Some(i);
            }
            current_x += glyph.advance;
        }
        Some(self.glyphs.len())
    }

    /// Get x position of glyph.
    pub fn x_of_glyph(&self, index: usize) -> f32 {
        self.glyphs.iter().take(index).map(|g| g.advance).sum()
    }

    /// Get text range for glyphs.
    pub fn text_range(&self, start: usize, end: usize) -> &str {
        let start_byte = self.glyphs.get(start).map(|g| g.cluster as usize).unwrap_or(0);
        let end_byte = self.glyphs.get(end).map(|g| g.cluster as usize).unwrap_or(self.text.len());
        &self.text[start_byte..end_byte]
    }
}

/// Shaped glyph.
#[derive(Clone, Debug)]
pub struct ShapedGlyph {
    /// Glyph ID.
    pub glyph_id: u32,
    /// X offset from pen position.
    pub x_offset: f32,
    /// Y offset from pen position.
    pub y_offset: f32,
    /// Advance width.
    pub advance: f32,
    /// Character cluster index.
    pub cluster: u32,
    /// Unicode character.
    pub character: char,
}

/// Text shaper.
pub struct TextShaper {
    /// Default font metrics.
    default_ascent: f32,
    default_descent: f32,
}

impl TextShaper {
    pub fn new() -> Self {
        Self {
            default_ascent: 0.8,
            default_descent: 0.2,
        }
    }

    /// Shape text into a text run.
    pub fn shape_text(&self, text: &str, style: &ComputedStyle) -> TextRun {
        let font_size = style.font_size;
        let line_height = self.compute_line_height(style);

        // Simple text shaping - real implementation would use HarfBuzz
        let mut glyphs = Vec::new();
        let mut width = 0.0;
        let mut cluster = 0u32;

        for grapheme in text.graphemes(true) {
            let c = grapheme.chars().next().unwrap_or(' ');
            let advance = self.char_width(c, font_size);

            glyphs.push(ShapedGlyph {
                glyph_id: c as u32,
                x_offset: 0.0,
                y_offset: 0.0,
                advance,
                cluster,
                character: c,
            });

            width += advance;
            cluster += grapheme.len() as u32;
        }

        let ascent = font_size * self.default_ascent;
        let descent = font_size * self.default_descent;

        TextRun {
            text: text.to_string(),
            glyphs,
            width,
            height: line_height,
            ascent,
            descent,
            font_size,
            line_breaks: Vec::new(),
        }
    }

    /// Compute line height.
    fn compute_line_height(&self, style: &ComputedStyle) -> f32 {
        use style::computed::LineHeight;

        match &style.line_height {
            LineHeight::Normal => style.font_size * 1.2,
            LineHeight::Number(n) => style.font_size * n,
            LineHeight::Length(l) => *l,
            LineHeight::Percentage(p) => style.font_size * p / 100.0,
        }
    }

    /// Get character width (simplified).
    fn char_width(&self, c: char, font_size: f32) -> f32 {
        // Simplified character width calculation
        // Real implementation would use font metrics
        let width_factor = match c {
            ' ' => 0.25,
            'i' | 'l' | '!' | '|' | '.' | ',' | ':' | ';' | '\'' => 0.3,
            'm' | 'w' | 'M' | 'W' => 0.7,
            _ if c.is_ascii_alphanumeric() => 0.5,
            _ if c.is_ascii() => 0.5,
            _ => 1.0, // CJK and other wide characters
        };

        font_size * width_factor
    }

    /// Break text into lines.
    pub fn break_into_lines(&self, text_run: &mut TextRun, max_width: f32) {
        let mut line_breaks = Vec::new();
        let mut line_width = 0.0;
        let mut last_break = 0;
        let mut last_space = 0;

        for (i, glyph) in text_run.glyphs.iter().enumerate() {
            line_width += glyph.advance;

            // Track spaces for soft breaks
            if glyph.character.is_whitespace() {
                last_space = i + 1;
            }

            // Check for line break
            if line_width > max_width && i > last_break {
                if last_space > last_break {
                    // Break at last space
                    line_breaks.push(last_space);
                    line_width = text_run.glyphs[last_space..=i]
                        .iter()
                        .map(|g| g.advance)
                        .sum();
                    last_break = last_space;
                } else {
                    // Force break at current position
                    line_breaks.push(i);
                    line_width = glyph.advance;
                    last_break = i;
                }
            }
        }

        text_run.line_breaks = line_breaks;
    }

    /// Measure text width.
    pub fn measure_width(&self, text: &str, style: &ComputedStyle) -> f32 {
        let font_size = style.font_size;
        text.chars().map(|c| self.char_width(c, font_size)).sum()
    }
}

impl Default for TextShaper {
    fn default() -> Self {
        Self::new()
    }
}

/// Line box for inline layout.
#[derive(Clone, Debug)]
pub struct LineBox {
    /// Rect of the line box.
    pub rect: Rect,
    /// Baseline y position.
    pub baseline: f32,
    /// Fragments in this line.
    pub fragments: Vec<LineFragment>,
}

impl LineBox {
    pub fn new(x: f32, y: f32, width: f32) -> Self {
        Self {
            rect: Rect::new(x, y, width, 0.0),
            baseline: 0.0,
            fragments: Vec::new(),
        }
    }

    /// Add fragment to line.
    pub fn add_fragment(&mut self, fragment: LineFragment) {
        // Update line height
        let fragment_height = fragment.ascent + fragment.descent;
        if fragment_height > self.rect.height {
            self.rect.height = fragment_height;
        }

        // Update baseline
        if fragment.ascent > self.baseline {
            self.baseline = fragment.ascent;
        }

        self.fragments.push(fragment);
    }

    /// Get used width.
    pub fn used_width(&self) -> f32 {
        self.fragments.iter().map(|f| f.width).sum()
    }

    /// Get remaining width.
    pub fn remaining_width(&self) -> f32 {
        self.rect.width - self.used_width()
    }
}

/// Fragment within a line box.
#[derive(Clone, Debug)]
pub struct LineFragment {
    /// Position within line.
    pub x: f32,
    /// Width.
    pub width: f32,
    /// Ascent.
    pub ascent: f32,
    /// Descent.
    pub descent: f32,
    /// Layout box this belongs to.
    pub layout_box: crate::layout_box::LayoutBoxId,
    /// Glyph range in text run.
    pub glyph_start: usize,
    pub glyph_end: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_shaper() {
        let shaper = TextShaper::new();
        let style = ComputedStyle::default_style();
        let run = shaper.shape_text("Hello", &style);

        assert_eq!(run.text, "Hello");
        assert_eq!(run.glyphs.len(), 5);
        assert!(run.width > 0.0);
    }

    #[test]
    fn test_line_breaking() {
        let shaper = TextShaper::new();
        let style = ComputedStyle::default_style();
        let mut run = shaper.shape_text("Hello World", &style);

        shaper.break_into_lines(&mut run, 50.0);
        // Should have at least one line break if width is small enough
    }
}
