//! Canvas API implementation.

use parking_lot::RwLock;
use std::collections::HashMap;

/// Canvas element.
#[derive(Debug)]
pub struct Canvas {
    /// Canvas width.
    width: u32,
    /// Canvas height.
    height: u32,
    /// Pixel data (RGBA).
    data: RwLock<Vec<u8>>,
    /// Context type.
    context_type: RwLock<Option<ContextType>>,
}

impl Canvas {
    /// Create a new canvas.
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            width,
            height,
            data: RwLock::new(vec![0; size]),
            context_type: RwLock::new(None),
        }
    }

    /// Get width.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get height.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get 2D context.
    pub fn get_context_2d(&self) -> Option<CanvasContext2D> {
        let mut ctx_type = self.context_type.write();
        if ctx_type.is_none() {
            *ctx_type = Some(ContextType::Context2D);
        }

        if *ctx_type == Some(ContextType::Context2D) {
            Some(CanvasContext2D::new(self.width, self.height))
        } else {
            None
        }
    }

    /// Get raw pixel data.
    pub fn data(&self) -> Vec<u8> {
        self.data.read().clone()
    }

    /// Set raw pixel data.
    pub fn set_data(&self, data: Vec<u8>) {
        *self.data.write() = data;
    }

    /// To data URL.
    pub fn to_data_url(&self, mime_type: &str) -> String {
        // Simplified - would actually encode image data
        format!("data:{};base64,AAAA", mime_type)
    }

    /// To blob (placeholder).
    pub fn to_blob(&self, _mime_type: &str) -> Vec<u8> {
        self.data.read().clone()
    }
}

/// Context type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextType {
    Context2D,
    WebGL,
    WebGL2,
    ImageBitmapRenderingContext,
}

/// Canvas context trait.
pub trait CanvasContext {
    /// Get canvas width.
    fn canvas_width(&self) -> u32;
    /// Get canvas height.
    fn canvas_height(&self) -> u32;
}

/// 2D canvas rendering context.
#[derive(Debug)]
pub struct CanvasContext2D {
    /// Canvas width.
    width: u32,
    /// Canvas height.
    height: u32,
    /// Pixel data.
    data: Vec<u8>,
    /// Current fill style.
    fill_style: String,
    /// Current stroke style.
    stroke_style: String,
    /// Line width.
    line_width: f64,
    /// Line cap.
    line_cap: LineCap,
    /// Line join.
    line_join: LineJoin,
    /// Global alpha.
    global_alpha: f64,
    /// Global composite operation.
    global_composite_operation: CompositeOperation,
    /// Font.
    font: String,
    /// Text align.
    text_align: TextAlign,
    /// Text baseline.
    text_baseline: TextBaseline,
    /// Shadow color.
    shadow_color: String,
    /// Shadow blur.
    shadow_blur: f64,
    /// Shadow offset X.
    shadow_offset_x: f64,
    /// Shadow offset Y.
    shadow_offset_y: f64,
    /// Current path.
    path: Vec<PathCommand>,
    /// Transform matrix.
    transform: Transform2D,
    /// Save stack.
    save_stack: Vec<ContextState>,
}

impl CanvasContext2D {
    /// Create a new 2D context.
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            width,
            height,
            data: vec![0; size],
            fill_style: "#000000".to_string(),
            stroke_style: "#000000".to_string(),
            line_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            global_alpha: 1.0,
            global_composite_operation: CompositeOperation::SourceOver,
            font: "10px sans-serif".to_string(),
            text_align: TextAlign::Start,
            text_baseline: TextBaseline::Alphabetic,
            shadow_color: "rgba(0,0,0,0)".to_string(),
            shadow_blur: 0.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            path: Vec::new(),
            transform: Transform2D::identity(),
            save_stack: Vec::new(),
        }
    }

    // State methods

    /// Save the current state.
    pub fn save(&mut self) {
        self.save_stack.push(ContextState {
            fill_style: self.fill_style.clone(),
            stroke_style: self.stroke_style.clone(),
            line_width: self.line_width,
            line_cap: self.line_cap,
            line_join: self.line_join,
            global_alpha: self.global_alpha,
            font: self.font.clone(),
            text_align: self.text_align,
            text_baseline: self.text_baseline,
            transform: self.transform,
        });
    }

    /// Restore the previous state.
    pub fn restore(&mut self) {
        if let Some(state) = self.save_stack.pop() {
            self.fill_style = state.fill_style;
            self.stroke_style = state.stroke_style;
            self.line_width = state.line_width;
            self.line_cap = state.line_cap;
            self.line_join = state.line_join;
            self.global_alpha = state.global_alpha;
            self.font = state.font;
            self.text_align = state.text_align;
            self.text_baseline = state.text_baseline;
            self.transform = state.transform;
        }
    }

    // Transform methods

    /// Scale transform.
    pub fn scale(&mut self, x: f64, y: f64) {
        self.transform = self.transform.scale(x, y);
    }

    /// Rotate transform.
    pub fn rotate(&mut self, angle: f64) {
        self.transform = self.transform.rotate(angle);
    }

    /// Translate transform.
    pub fn translate(&mut self, x: f64, y: f64) {
        self.transform = self.transform.translate(x, y);
    }

    /// Set transform.
    pub fn set_transform(&mut self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        self.transform = Transform2D::new(a, b, c, d, e, f);
    }

    /// Reset transform.
    pub fn reset_transform(&mut self) {
        self.transform = Transform2D::identity();
    }

    // Style setters

    /// Set fill style.
    pub fn set_fill_style(&mut self, style: &str) {
        self.fill_style = style.to_string();
    }

    /// Get fill style.
    pub fn fill_style(&self) -> &str {
        &self.fill_style
    }

    /// Set stroke style.
    pub fn set_stroke_style(&mut self, style: &str) {
        self.stroke_style = style.to_string();
    }

    /// Get stroke style.
    pub fn stroke_style(&self) -> &str {
        &self.stroke_style
    }

    /// Set line width.
    pub fn set_line_width(&mut self, width: f64) {
        self.line_width = width;
    }

    /// Get line width.
    pub fn line_width(&self) -> f64 {
        self.line_width
    }

    /// Set global alpha.
    pub fn set_global_alpha(&mut self, alpha: f64) {
        self.global_alpha = alpha.clamp(0.0, 1.0);
    }

    /// Get global alpha.
    pub fn global_alpha(&self) -> f64 {
        self.global_alpha
    }

    /// Set font.
    pub fn set_font(&mut self, font: &str) {
        self.font = font.to_string();
    }

    /// Get font.
    pub fn font(&self) -> &str {
        &self.font
    }

    // Path methods

    /// Begin a new path.
    pub fn begin_path(&mut self) {
        self.path.clear();
    }

    /// Close the current path.
    pub fn close_path(&mut self) {
        self.path.push(PathCommand::ClosePath);
    }

    /// Move to point.
    pub fn move_to(&mut self, x: f64, y: f64) {
        self.path.push(PathCommand::MoveTo(x, y));
    }

    /// Line to point.
    pub fn line_to(&mut self, x: f64, y: f64) {
        self.path.push(PathCommand::LineTo(x, y));
    }

    /// Bezier curve to point.
    pub fn bezier_curve_to(&mut self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        self.path.push(PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y));
    }

    /// Quadratic curve to point.
    pub fn quadratic_curve_to(&mut self, cpx: f64, cpy: f64, x: f64, y: f64) {
        self.path.push(PathCommand::QuadraticCurveTo(cpx, cpy, x, y));
    }

    /// Arc.
    pub fn arc(&mut self, x: f64, y: f64, radius: f64, start_angle: f64, end_angle: f64, anticlockwise: bool) {
        self.path.push(PathCommand::Arc(x, y, radius, start_angle, end_angle, anticlockwise));
    }

    /// Arc to.
    pub fn arc_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, radius: f64) {
        self.path.push(PathCommand::ArcTo(x1, y1, x2, y2, radius));
    }

    /// Rectangle.
    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.path.push(PathCommand::Rect(x, y, width, height));
    }

    // Drawing methods

    /// Fill the current path.
    pub fn fill(&mut self) {
        // Simplified - would actually rasterize the path
        // This is a placeholder implementation
    }

    /// Stroke the current path.
    pub fn stroke(&mut self) {
        // Simplified - would actually rasterize the path
    }

    /// Fill rectangle.
    pub fn fill_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        let color = parse_color(&self.fill_style).unwrap_or([0, 0, 0, 255]);
        self.fill_rect_with_color(x as i32, y as i32, width as u32, height as u32, color);
    }

    fn fill_rect_with_color(&mut self, x: i32, y: i32, width: u32, height: u32, color: [u8; 4]) {
        for py in 0..height as i32 {
            for px in 0..width as i32 {
                let cx = x + px;
                let cy = y + py;
                if cx >= 0 && cy >= 0 && (cx as u32) < self.width && (cy as u32) < self.height {
                    let idx = ((cy as u32 * self.width + cx as u32) * 4) as usize;
                    if idx + 4 <= self.data.len() {
                        // Alpha blending
                        let alpha = color[3] as f64 / 255.0 * self.global_alpha;
                        for i in 0..3 {
                            self.data[idx + i] = ((color[i] as f64 * alpha) + (self.data[idx + i] as f64 * (1.0 - alpha))) as u8;
                        }
                        self.data[idx + 3] = ((alpha * 255.0) + (self.data[idx + 3] as f64 * (1.0 - alpha))) as u8;
                    }
                }
            }
        }
    }

    /// Stroke rectangle.
    pub fn stroke_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.begin_path();
        self.rect(x, y, width, height);
        self.stroke();
    }

    /// Clear rectangle.
    pub fn clear_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        for py in 0..height as u32 {
            for px in 0..width as u32 {
                let cx = x as u32 + px;
                let cy = y as u32 + py;
                if cx < self.width && cy < self.height {
                    let idx = ((cy * self.width + cx) * 4) as usize;
                    if idx + 4 <= self.data.len() {
                        self.data[idx] = 0;
                        self.data[idx + 1] = 0;
                        self.data[idx + 2] = 0;
                        self.data[idx + 3] = 0;
                    }
                }
            }
        }
    }

    /// Fill text.
    pub fn fill_text(&mut self, _text: &str, _x: f64, _y: f64) {
        // Would need font rendering - placeholder
    }

    /// Stroke text.
    pub fn stroke_text(&mut self, _text: &str, _x: f64, _y: f64) {
        // Would need font rendering - placeholder
    }

    /// Measure text.
    pub fn measure_text(&self, text: &str) -> TextMetrics {
        // Simplified - would need actual font metrics
        TextMetrics {
            width: text.len() as f64 * 8.0, // Rough approximation
        }
    }

    // Image methods

    /// Draw image.
    pub fn draw_image(&mut self, _image_data: &[u8], _x: f64, _y: f64) {
        // Would draw image data - placeholder
    }

    /// Get image data.
    pub fn get_image_data(&self, x: u32, y: u32, width: u32, height: u32) -> ImageData {
        let mut data = Vec::with_capacity((width * height * 4) as usize);

        for py in 0..height {
            for px in 0..width {
                let cx = x + px;
                let cy = y + py;
                if cx < self.width && cy < self.height {
                    let idx = ((cy * self.width + cx) * 4) as usize;
                    data.extend_from_slice(&self.data[idx..idx + 4]);
                } else {
                    data.extend_from_slice(&[0, 0, 0, 0]);
                }
            }
        }

        ImageData { width, height, data }
    }

    /// Put image data.
    pub fn put_image_data(&mut self, image_data: &ImageData, x: i32, y: i32) {
        for py in 0..image_data.height as i32 {
            for px in 0..image_data.width as i32 {
                let cx = x + px;
                let cy = y + py;
                if cx >= 0 && cy >= 0 && (cx as u32) < self.width && (cy as u32) < self.height {
                    let src_idx = ((py as u32 * image_data.width + px as u32) * 4) as usize;
                    let dst_idx = ((cy as u32 * self.width + cx as u32) * 4) as usize;
                    if src_idx + 4 <= image_data.data.len() && dst_idx + 4 <= self.data.len() {
                        self.data[dst_idx..dst_idx + 4].copy_from_slice(&image_data.data[src_idx..src_idx + 4]);
                    }
                }
            }
        }
    }

    /// Get pixel data.
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

impl CanvasContext for CanvasContext2D {
    fn canvas_width(&self) -> u32 {
        self.width
    }

    fn canvas_height(&self) -> u32 {
        self.height
    }
}

/// Path command.
#[derive(Clone, Debug)]
pub enum PathCommand {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    BezierCurveTo(f64, f64, f64, f64, f64, f64),
    QuadraticCurveTo(f64, f64, f64, f64),
    Arc(f64, f64, f64, f64, f64, bool),
    ArcTo(f64, f64, f64, f64, f64),
    Rect(f64, f64, f64, f64),
    ClosePath,
}

/// 2D transform matrix.
#[derive(Clone, Copy, Debug)]
pub struct Transform2D {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Transform2D {
    /// Create a new transform.
    pub fn new(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Self {
        Self { a, b, c, d, e, f }
    }

    /// Create identity transform.
    pub fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    }

    /// Scale transform.
    pub fn scale(&self, x: f64, y: f64) -> Self {
        Self::new(self.a * x, self.b * x, self.c * y, self.d * y, self.e, self.f)
    }

    /// Rotate transform.
    pub fn rotate(&self, angle: f64) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self::new(
            self.a * cos + self.c * sin,
            self.b * cos + self.d * sin,
            self.c * cos - self.a * sin,
            self.d * cos - self.b * sin,
            self.e,
            self.f,
        )
    }

    /// Translate transform.
    pub fn translate(&self, x: f64, y: f64) -> Self {
        Self::new(
            self.a,
            self.b,
            self.c,
            self.d,
            self.e + self.a * x + self.c * y,
            self.f + self.b * x + self.d * y,
        )
    }
}

/// Saved context state.
#[derive(Clone, Debug)]
struct ContextState {
    fill_style: String,
    stroke_style: String,
    line_width: f64,
    line_cap: LineCap,
    line_join: LineJoin,
    global_alpha: f64,
    font: String,
    text_align: TextAlign,
    text_baseline: TextBaseline,
    transform: Transform2D,
}

/// Line cap style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

/// Line join style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

/// Composite operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositeOperation {
    SourceOver,
    SourceIn,
    SourceOut,
    SourceAtop,
    DestinationOver,
    DestinationIn,
    DestinationOut,
    DestinationAtop,
    Lighter,
    Copy,
    Xor,
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
}

/// Text align.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Start,
    End,
    Left,
    Right,
    Center,
}

/// Text baseline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextBaseline {
    Top,
    Hanging,
    Middle,
    Alphabetic,
    Ideographic,
    Bottom,
}

/// Text metrics.
#[derive(Clone, Debug)]
pub struct TextMetrics {
    pub width: f64,
}

/// Image data.
#[derive(Clone, Debug)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl ImageData {
    /// Create new image data.
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            width,
            height,
            data: vec![0; size],
        }
    }
}

/// Parse a color string (simplified).
fn parse_color(color: &str) -> Option<[u8; 4]> {
    if color.starts_with('#') {
        let hex = &color[1..];
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some([r, g, b, 255]);
        }
    }

    // Named colors
    match color {
        "black" => Some([0, 0, 0, 255]),
        "white" => Some([255, 255, 255, 255]),
        "red" => Some([255, 0, 0, 255]),
        "green" => Some([0, 128, 0, 255]),
        "blue" => Some([0, 0, 255, 255]),
        "yellow" => Some([255, 255, 0, 255]),
        "transparent" => Some([0, 0, 0, 0]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_creation() {
        let canvas = Canvas::new(800, 600);
        assert_eq!(canvas.width(), 800);
        assert_eq!(canvas.height(), 600);
    }

    #[test]
    fn test_context_2d() {
        let mut ctx = CanvasContext2D::new(100, 100);

        ctx.set_fill_style("#ff0000");
        ctx.fill_rect(10.0, 10.0, 20.0, 20.0);

        let data = ctx.get_data();
        assert!(!data.is_empty());
    }

    #[test]
    fn test_save_restore() {
        let mut ctx = CanvasContext2D::new(100, 100);

        ctx.set_fill_style("#ff0000");
        ctx.save();

        ctx.set_fill_style("#00ff00");
        assert_eq!(ctx.fill_style(), "#00ff00");

        ctx.restore();
        assert_eq!(ctx.fill_style(), "#ff0000");
    }

    #[test]
    fn test_image_data() {
        let mut ctx = CanvasContext2D::new(100, 100);
        ctx.set_fill_style("#ff0000");
        ctx.fill_rect(0.0, 0.0, 10.0, 10.0);

        let image_data = ctx.get_image_data(0, 0, 10, 10);
        assert_eq!(image_data.width, 10);
        assert_eq!(image_data.height, 10);
    }
}
