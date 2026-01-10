//! Display list for rendering.
//!
//! The display list is an intermediate representation between layout and painting.
//! It contains all the drawing commands needed to render the page.

use common::color::Color;
use common::geometry::{CornerRadii, Point, Rect, Transform};
use layout::layout_box::LayoutBoxId;
use smallvec::SmallVec;
use std::sync::Arc;

/// A display list containing all items to be painted.
#[derive(Clone, Debug, Default)]
pub struct DisplayList {
    /// Display items in paint order.
    items: Vec<DisplayItem>,
    /// Stacking contexts.
    stacking_contexts: Vec<StackingContext>,
}

impl DisplayList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            stacking_contexts: Vec::new(),
        }
    }

    /// Add a display item.
    pub fn push(&mut self, item: DisplayItem) {
        self.items.push(item);
    }

    /// Add multiple display items.
    pub fn extend(&mut self, items: impl IntoIterator<Item = DisplayItem>) {
        self.items.extend(items);
    }

    /// Push a stacking context.
    pub fn push_stacking_context(&mut self, context: StackingContext) {
        self.stacking_contexts.push(context);
    }

    /// Get all items.
    pub fn items(&self) -> &[DisplayItem] {
        &self.items
    }

    /// Get mutable items.
    pub fn items_mut(&mut self) -> &mut Vec<DisplayItem> {
        &mut self.items
    }

    /// Get stacking contexts.
    pub fn stacking_contexts(&self) -> &[StackingContext] {
        &self.stacking_contexts
    }

    /// Clear the display list.
    pub fn clear(&mut self) {
        self.items.clear();
        self.stacking_contexts.clear();
    }

    /// Get number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Sort items by z-index and stacking context.
    pub fn sort_for_painting(&mut self) {
        // Items are already in paint order from display list building
        // Just sort by z-index within stacking contexts
        self.items.sort_by(|a, b| {
            a.stacking_context
                .cmp(&b.stacking_context)
                .then_with(|| a.z_index.cmp(&b.z_index))
        });
    }

    /// Get items within a clip rect.
    pub fn items_in_rect(&self, rect: &Rect) -> impl Iterator<Item = &DisplayItem> {
        self.items.iter().filter(move |item| {
            item.bounds.intersects(rect)
        })
    }
}

/// A single display item.
#[derive(Clone, Debug)]
pub struct DisplayItem {
    /// The type of item.
    pub item_type: DisplayItemType,
    /// Bounding rectangle.
    pub bounds: Rect,
    /// Clip rectangle.
    pub clip: Option<ClipRegion>,
    /// Transform.
    pub transform: Option<Transform>,
    /// Opacity (0.0 - 1.0).
    pub opacity: f32,
    /// Z-index for sorting.
    pub z_index: i32,
    /// Stacking context ID.
    pub stacking_context: usize,
    /// Associated layout box.
    pub layout_box: Option<LayoutBoxId>,
}

impl DisplayItem {
    pub fn new(item_type: DisplayItemType, bounds: Rect) -> Self {
        Self {
            item_type,
            bounds,
            clip: None,
            transform: None,
            opacity: 1.0,
            z_index: 0,
            stacking_context: 0,
            layout_box: None,
        }
    }

    /// Set clip region.
    pub fn with_clip(mut self, clip: ClipRegion) -> Self {
        self.clip = Some(clip);
        self
    }

    /// Set transform.
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);
        self
    }

    /// Set opacity.
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    /// Set z-index.
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// Set stacking context.
    pub fn with_stacking_context(mut self, context: usize) -> Self {
        self.stacking_context = context;
        self
    }

    /// Set layout box.
    pub fn with_layout_box(mut self, layout_box: LayoutBoxId) -> Self {
        self.layout_box = Some(layout_box);
        self
    }
}

/// Types of display items.
#[derive(Clone, Debug)]
pub enum DisplayItemType {
    /// Solid color rectangle.
    SolidColor(SolidColorItem),
    /// Text.
    Text(TextItem),
    /// Image.
    Image(ImageItem),
    /// Border.
    Border(BorderItem),
    /// Box shadow.
    BoxShadow(BoxShadowItem),
    /// Linear gradient.
    LinearGradient(LinearGradientItem),
    /// Radial gradient.
    RadialGradient(RadialGradientItem),
    /// Line (for text decorations, etc.).
    Line(LineItem),
    /// Push clip.
    PushClip(ClipRegion),
    /// Pop clip.
    PopClip,
    /// Push scroll frame.
    PushScrollFrame(ScrollFrame),
    /// Pop scroll frame.
    PopScrollFrame,
}

/// Solid color rectangle.
#[derive(Clone, Debug)]
pub struct SolidColorItem {
    pub color: Color,
    pub radii: Option<CornerRadii>,
}

/// Text display item.
#[derive(Clone, Debug)]
pub struct TextItem {
    /// The text to render.
    pub text: String,
    /// Glyph positions.
    pub glyphs: Vec<GlyphInstance>,
    /// Font key for the font cache.
    pub font_key: FontKey,
    /// Font size in pixels.
    pub font_size: f32,
    /// Text color.
    pub color: Color,
    /// Baseline position.
    pub baseline: f32,
}

/// A single glyph instance.
#[derive(Clone, Debug)]
pub struct GlyphInstance {
    /// Glyph index in the font.
    pub glyph_index: u32,
    /// Position relative to text origin.
    pub point: Point,
}

/// Font key for identifying fonts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FontKey {
    pub family: String,
    pub weight: u16,
    pub style: FontStyle,
}

/// Font style.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

/// Image display item.
#[derive(Clone, Debug)]
pub struct ImageItem {
    /// Image key for the image cache.
    pub image_key: ImageKey,
    /// Source rectangle within the image.
    pub src_rect: Option<Rect>,
    /// How to render the image.
    pub rendering: ImageRendering,
}

/// Image key for identifying images.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageKey(pub u64);

/// Image rendering mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageRendering {
    Auto,
    CrispEdges,
    Pixelated,
}

/// Border display item.
#[derive(Clone, Debug)]
pub struct BorderItem {
    /// Border widths (top, right, bottom, left).
    pub widths: [f32; 4],
    /// Border colors.
    pub colors: [Color; 4],
    /// Border styles.
    pub styles: [BorderStyle; 4],
    /// Corner radii.
    pub radii: Option<CornerRadii>,
}

/// Border style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorderStyle {
    None,
    Solid,
    Dotted,
    Dashed,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
    Hidden,
}

impl From<style::computed::BorderStyle> for BorderStyle {
    fn from(style: style::computed::BorderStyle) -> Self {
        match style {
            style::computed::BorderStyle::None => BorderStyle::None,
            style::computed::BorderStyle::Solid => BorderStyle::Solid,
            style::computed::BorderStyle::Dotted => BorderStyle::Dotted,
            style::computed::BorderStyle::Dashed => BorderStyle::Dashed,
            style::computed::BorderStyle::Double => BorderStyle::Double,
            style::computed::BorderStyle::Groove => BorderStyle::Groove,
            style::computed::BorderStyle::Ridge => BorderStyle::Ridge,
            style::computed::BorderStyle::Inset => BorderStyle::Inset,
            style::computed::BorderStyle::Outset => BorderStyle::Outset,
            style::computed::BorderStyle::Hidden => BorderStyle::Hidden,
        }
    }
}

/// Box shadow display item.
#[derive(Clone, Debug)]
pub struct BoxShadowItem {
    /// Shadow color.
    pub color: Color,
    /// Horizontal offset.
    pub offset_x: f32,
    /// Vertical offset.
    pub offset_y: f32,
    /// Blur radius.
    pub blur_radius: f32,
    /// Spread radius.
    pub spread_radius: f32,
    /// Whether this is an inset shadow.
    pub inset: bool,
    /// Border radii of the box.
    pub radii: Option<CornerRadii>,
}

/// Linear gradient display item.
#[derive(Clone, Debug)]
pub struct LinearGradientItem {
    /// Start point.
    pub start: Point,
    /// End point.
    pub end: Point,
    /// Color stops.
    pub stops: Vec<GradientStop>,
}

/// Radial gradient display item.
#[derive(Clone, Debug)]
pub struct RadialGradientItem {
    /// Center point.
    pub center: Point,
    /// Horizontal radius.
    pub radius_x: f32,
    /// Vertical radius.
    pub radius_y: f32,
    /// Color stops.
    pub stops: Vec<GradientStop>,
}

/// Gradient color stop.
#[derive(Clone, Debug)]
pub struct GradientStop {
    /// Position (0.0 - 1.0).
    pub position: f32,
    /// Color at this stop.
    pub color: Color,
}

/// Line display item.
#[derive(Clone, Debug)]
pub struct LineItem {
    /// Start point.
    pub start: Point,
    /// End point.
    pub end: Point,
    /// Line width.
    pub width: f32,
    /// Line color.
    pub color: Color,
    /// Line style.
    pub style: LineStyle,
}

/// Line style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineStyle {
    Solid,
    Dotted,
    Dashed,
    Wavy,
}

/// Clip region.
#[derive(Clone, Debug)]
pub struct ClipRegion {
    /// Clip rectangle.
    pub rect: Rect,
    /// Corner radii for rounded clips.
    pub radii: Option<CornerRadii>,
    /// Complex clip path.
    pub path: Option<ClipPath>,
}

impl ClipRegion {
    pub fn rect(rect: Rect) -> Self {
        Self {
            rect,
            radii: None,
            path: None,
        }
    }

    pub fn rounded_rect(rect: Rect, radii: CornerRadii) -> Self {
        Self {
            rect,
            radii: Some(radii),
            path: None,
        }
    }
}

/// Complex clip path.
#[derive(Clone, Debug)]
pub enum ClipPath {
    /// Circle clip.
    Circle { center: Point, radius: f32 },
    /// Ellipse clip.
    Ellipse { center: Point, radius_x: f32, radius_y: f32 },
    /// Polygon clip.
    Polygon { points: Vec<Point> },
    /// Path clip (SVG-style).
    Path { commands: Vec<PathCommand> },
}

/// Path command for complex clips.
#[derive(Clone, Debug)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    QuadraticTo { control: Point, to: Point },
    CubicTo { control1: Point, control2: Point, to: Point },
    ArcTo { radii: Point, rotation: f32, large_arc: bool, sweep: bool, to: Point },
    Close,
}

/// Scroll frame for scrollable content.
#[derive(Clone, Debug)]
pub struct ScrollFrame {
    /// Viewport rectangle.
    pub viewport: Rect,
    /// Content rectangle (may be larger than viewport).
    pub content_rect: Rect,
    /// Current scroll offset.
    pub scroll_offset: Point,
    /// Scroll frame ID.
    pub id: ScrollFrameId,
}

/// Scroll frame identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScrollFrameId(pub u64);

/// A stacking context.
#[derive(Clone, Debug)]
pub struct StackingContext {
    /// Stacking context ID.
    pub id: usize,
    /// Parent stacking context.
    pub parent: Option<usize>,
    /// Z-index.
    pub z_index: i32,
    /// Transform.
    pub transform: Option<Transform>,
    /// Opacity.
    pub opacity: f32,
    /// Blend mode.
    pub blend_mode: BlendMode,
    /// Isolation (creates new stacking context).
    pub isolation: bool,
    /// Bounds of this stacking context.
    pub bounds: Rect,
}

/// Blend mode for compositing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_list() {
        let mut list = DisplayList::new();
        assert!(list.is_empty());

        let item = DisplayItem::new(
            DisplayItemType::SolidColor(SolidColorItem {
                color: Color::rgb(255, 0, 0),
                radii: None,
            }),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        );

        list.push(item);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_display_item_builder() {
        let item = DisplayItem::new(
            DisplayItemType::SolidColor(SolidColorItem {
                color: Color::rgb(0, 0, 255),
                radii: None,
            }),
            Rect::new(10.0, 10.0, 50.0, 50.0),
        )
        .with_opacity(0.5)
        .with_z_index(5);

        assert_eq!(item.opacity, 0.5);
        assert_eq!(item.z_index, 5);
    }
}
