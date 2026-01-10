//! Rasterizer for converting display lists to pixels.

use crate::display_list::{
    BlendMode, BorderItem, BorderStyle, BoxShadowItem, ClipRegion, DisplayItem, DisplayItemType,
    DisplayList, GradientStop, ImageItem, ImageKey, LinearGradientItem, LineItem, LineStyle,
    RadialGradientItem, SolidColorItem, TextItem,
};
use crate::font::FontCache;
use crate::image_cache::ImageCache;
use common::color::Color;
use common::geometry::{CornerRadii, Point, Rect};
use rayon::prelude::*;
use std::sync::Arc;

/// Pixel buffer for rasterization output.
pub struct PixelBuffer {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// RGBA pixel data (4 bytes per pixel).
    pub data: Vec<u8>,
}

impl PixelBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            width,
            height,
            data: vec![0; size],
        }
    }

    /// Fill with a color.
    pub fn fill(&mut self, color: Color) {
        for chunk in self.data.chunks_exact_mut(4) {
            chunk[0] = color.r;
            chunk[1] = color.g;
            chunk[2] = color.b;
            chunk[3] = color.a;
        }
    }

    /// Clear to transparent.
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    /// Get pixel at position.
    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        if x >= self.width || y >= self.height {
            return Color::transparent();
        }

        let offset = ((y * self.width + x) * 4) as usize;
        Color::new(
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
        )
    }

    /// Set pixel at position.
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let offset = ((y * self.width + x) * 4) as usize;
        self.data[offset] = color.r;
        self.data[offset + 1] = color.g;
        self.data[offset + 2] = color.b;
        self.data[offset + 3] = color.a;
    }

    /// Blend pixel at position.
    pub fn blend_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height || color.a == 0 {
            return;
        }

        let existing = self.get_pixel(x, y);
        let blended = existing.blend(&color);
        self.set_pixel(x, y, blended);
    }

    /// Get as raw bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable bytes.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

/// Software rasterizer.
pub struct Rasterizer {
    /// Font cache.
    font_cache: Arc<FontCache>,
    /// Image cache.
    image_cache: Arc<ImageCache>,
    /// Current clip stack.
    clip_stack: Vec<ClipRegion>,
}

impl Rasterizer {
    pub fn new(font_cache: Arc<FontCache>, image_cache: Arc<ImageCache>) -> Self {
        Self {
            font_cache,
            image_cache,
            clip_stack: Vec::new(),
        }
    }

    /// Rasterize a display list to a pixel buffer.
    pub fn rasterize(&mut self, display_list: &DisplayList, buffer: &mut PixelBuffer) {
        // Clear buffer
        buffer.fill(Color::white());

        // Rasterize each item
        for item in display_list.items() {
            self.rasterize_item(item, buffer);
        }
    }

    /// Rasterize a single display item.
    fn rasterize_item(&mut self, item: &DisplayItem, buffer: &mut PixelBuffer) {
        // Apply opacity
        let opacity = item.opacity;

        // Handle clip
        if let Some(clip) = &item.clip {
            self.clip_stack.push(clip.clone());
        }

        match &item.item_type {
            DisplayItemType::SolidColor(solid) => {
                self.rasterize_solid_color(solid, &item.bounds, opacity, buffer);
            }
            DisplayItemType::Text(text) => {
                self.rasterize_text(text, &item.bounds, opacity, buffer);
            }
            DisplayItemType::Image(image) => {
                self.rasterize_image(image, &item.bounds, buffer);
            }
            DisplayItemType::Border(border) => {
                self.rasterize_border(border, &item.bounds, buffer);
            }
            DisplayItemType::BoxShadow(shadow) => {
                self.rasterize_box_shadow(shadow, &item.bounds, buffer);
            }
            DisplayItemType::LinearGradient(gradient) => {
                self.rasterize_linear_gradient(gradient, &item.bounds, buffer);
            }
            DisplayItemType::RadialGradient(gradient) => {
                self.rasterize_radial_gradient(gradient, &item.bounds, buffer);
            }
            DisplayItemType::Line(line) => {
                self.rasterize_line(line, buffer);
            }
            DisplayItemType::PushClip(clip) => {
                self.clip_stack.push(clip.clone());
            }
            DisplayItemType::PopClip => {
                self.clip_stack.pop();
            }
            DisplayItemType::PushScrollFrame(_) => {}
            DisplayItemType::PopScrollFrame => {}
        }

        if item.clip.is_some() {
            self.clip_stack.pop();
        }
    }

    /// Rasterize a solid color rectangle.
    fn rasterize_solid_color(
        &self,
        solid: &SolidColorItem,
        bounds: &Rect,
        opacity: f32,
        buffer: &mut PixelBuffer,
    ) {
        let mut color = solid.color.clone();
        color.a = (color.a as f32 * opacity) as u8;

        let x_start = bounds.x.max(0.0) as u32;
        let y_start = bounds.y.max(0.0) as u32;
        let x_end = (bounds.x + bounds.width).min(buffer.width as f32) as u32;
        let y_end = (bounds.y + bounds.height).min(buffer.height as f32) as u32;

        if let Some(radii) = &solid.radii {
            // Rounded rectangle
            self.fill_rounded_rect(buffer, x_start, y_start, x_end, y_end, radii, &color);
        } else {
            // Simple rectangle
            for y in y_start..y_end {
                for x in x_start..x_end {
                    if self.is_inside_clip(x as f32, y as f32) {
                        buffer.blend_pixel(x, y, color.clone());
                    }
                }
            }
        }
    }

    /// Fill a rounded rectangle.
    fn fill_rounded_rect(
        &self,
        buffer: &mut PixelBuffer,
        x_start: u32,
        y_start: u32,
        x_end: u32,
        y_end: u32,
        radii: &CornerRadii,
        color: &Color,
    ) {
        let width = x_end - x_start;
        let height = y_end - y_start;

        for y in y_start..y_end {
            for x in x_start..x_end {
                let local_x = x - x_start;
                let local_y = y - y_start;

                let inside = self.is_inside_rounded_rect(
                    local_x, local_y, width, height, radii,
                );

                if inside && self.is_inside_clip(x as f32, y as f32) {
                    buffer.blend_pixel(x, y, color.clone());
                }
            }
        }
    }

    /// Check if a point is inside a rounded rectangle.
    fn is_inside_rounded_rect(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        radii: &CornerRadii,
    ) -> bool {
        let x = x as f32;
        let y = y as f32;
        let w = width as f32;
        let h = height as f32;

        // Top-left corner
        if x < radii.top_left && y < radii.top_left {
            let dx = radii.top_left - x;
            let dy = radii.top_left - y;
            if dx * dx + dy * dy > radii.top_left * radii.top_left {
                return false;
            }
        }

        // Top-right corner
        if x > w - radii.top_right && y < radii.top_right {
            let dx = x - (w - radii.top_right);
            let dy = radii.top_right - y;
            if dx * dx + dy * dy > radii.top_right * radii.top_right {
                return false;
            }
        }

        // Bottom-right corner
        if x > w - radii.bottom_right && y > h - radii.bottom_right {
            let dx = x - (w - radii.bottom_right);
            let dy = y - (h - radii.bottom_right);
            if dx * dx + dy * dy > radii.bottom_right * radii.bottom_right {
                return false;
            }
        }

        // Bottom-left corner
        if x < radii.bottom_left && y > h - radii.bottom_left {
            let dx = radii.bottom_left - x;
            let dy = y - (h - radii.bottom_left);
            if dx * dx + dy * dy > radii.bottom_left * radii.bottom_left {
                return false;
            }
        }

        true
    }

    /// Check if a point is inside the current clip region.
    fn is_inside_clip(&self, x: f32, y: f32) -> bool {
        for clip in &self.clip_stack {
            if !clip.rect.contains_point(Point::new(x, y)) {
                return false;
            }
            // TODO: Check rounded clip and path clip
        }
        true
    }

    /// Rasterize text.
    fn rasterize_text(&self, text: &TextItem, bounds: &Rect, opacity: f32, buffer: &mut PixelBuffer) {
        let mut color = text.color.clone();
        color.a = (color.a as f32 * opacity) as u8;

        // Get font from cache
        let font = match self.font_cache.get_font(&text.font_key) {
            Some(f) => f,
            None => return,
        };

        // Rasterize each glyph
        for glyph in &text.glyphs {
            let glyph_bitmap = font.rasterize(glyph.glyph_index, text.font_size);

            let x_start = glyph.point.x as i32;
            let y_start = glyph.point.y as i32 - glyph_bitmap.metrics.ymin;

            for gy in 0..glyph_bitmap.height {
                for gx in 0..glyph_bitmap.width {
                    let alpha = glyph_bitmap.data[(gy * glyph_bitmap.width + gx) as usize];
                    if alpha == 0 {
                        continue;
                    }

                    let px = x_start + gx as i32;
                    let py = y_start + gy as i32;

                    if px < 0 || py < 0 {
                        continue;
                    }

                    let px = px as u32;
                    let py = py as u32;

                    if self.is_inside_clip(px as f32, py as f32) {
                        let glyph_color = Color::new(
                            color.r,
                            color.g,
                            color.b,
                            ((color.a as u32 * alpha as u32) / 255) as u8,
                        );
                        buffer.blend_pixel(px, py, glyph_color);
                    }
                }
            }
        }
    }

    /// Rasterize an image.
    fn rasterize_image(&self, image: &ImageItem, bounds: &Rect, buffer: &mut PixelBuffer) {
        let image_data = match self.image_cache.get(&image.image_key) {
            Some(data) => data,
            None => return,
        };

        // Calculate scaling
        let scale_x = bounds.width / image_data.width as f32;
        let scale_y = bounds.height / image_data.height as f32;

        let x_start = bounds.x.max(0.0) as u32;
        let y_start = bounds.y.max(0.0) as u32;
        let x_end = (bounds.x + bounds.width).min(buffer.width as f32) as u32;
        let y_end = (bounds.y + bounds.height).min(buffer.height as f32) as u32;

        for y in y_start..y_end {
            for x in x_start..x_end {
                if !self.is_inside_clip(x as f32, y as f32) {
                    continue;
                }

                // Map to source coordinates
                let src_x = ((x as f32 - bounds.x) / scale_x) as u32;
                let src_y = ((y as f32 - bounds.y) / scale_y) as u32;

                if src_x < image_data.width && src_y < image_data.height {
                    let color = image_data.get_pixel(src_x, src_y);
                    buffer.blend_pixel(x, y, color);
                }
            }
        }
    }

    /// Rasterize a border.
    fn rasterize_border(&self, border: &BorderItem, bounds: &Rect, buffer: &mut PixelBuffer) {
        // Top border
        if border.widths[0] > 0.0 && !matches!(border.styles[0], BorderStyle::None | BorderStyle::Hidden) {
            let rect = Rect::new(bounds.x, bounds.y, bounds.width, border.widths[0]);
            self.fill_border_side(&rect, &border.colors[0], border.styles[0], buffer);
        }

        // Right border
        if border.widths[1] > 0.0 && !matches!(border.styles[1], BorderStyle::None | BorderStyle::Hidden) {
            let rect = Rect::new(
                bounds.x + bounds.width - border.widths[1],
                bounds.y,
                border.widths[1],
                bounds.height,
            );
            self.fill_border_side(&rect, &border.colors[1], border.styles[1], buffer);
        }

        // Bottom border
        if border.widths[2] > 0.0 && !matches!(border.styles[2], BorderStyle::None | BorderStyle::Hidden) {
            let rect = Rect::new(
                bounds.x,
                bounds.y + bounds.height - border.widths[2],
                bounds.width,
                border.widths[2],
            );
            self.fill_border_side(&rect, &border.colors[2], border.styles[2], buffer);
        }

        // Left border
        if border.widths[3] > 0.0 && !matches!(border.styles[3], BorderStyle::None | BorderStyle::Hidden) {
            let rect = Rect::new(bounds.x, bounds.y, border.widths[3], bounds.height);
            self.fill_border_side(&rect, &border.colors[3], border.styles[3], buffer);
        }
    }

    /// Fill a border side.
    fn fill_border_side(
        &self,
        rect: &Rect,
        color: &Color,
        style: BorderStyle,
        buffer: &mut PixelBuffer,
    ) {
        let x_start = rect.x.max(0.0) as u32;
        let y_start = rect.y.max(0.0) as u32;
        let x_end = (rect.x + rect.width).min(buffer.width as f32) as u32;
        let y_end = (rect.y + rect.height).min(buffer.height as f32) as u32;

        match style {
            BorderStyle::Solid => {
                for y in y_start..y_end {
                    for x in x_start..x_end {
                        if self.is_inside_clip(x as f32, y as f32) {
                            buffer.blend_pixel(x, y, color.clone());
                        }
                    }
                }
            }
            BorderStyle::Dotted => {
                let dot_size = 2;
                for y in y_start..y_end {
                    for x in x_start..x_end {
                        if ((x / dot_size) + (y / dot_size)) % 2 == 0 {
                            if self.is_inside_clip(x as f32, y as f32) {
                                buffer.blend_pixel(x, y, color.clone());
                            }
                        }
                    }
                }
            }
            BorderStyle::Dashed => {
                let dash_size = 4;
                for y in y_start..y_end {
                    for x in x_start..x_end {
                        if (x / dash_size) % 2 == 0 || (y / dash_size) % 2 == 0 {
                            if self.is_inside_clip(x as f32, y as f32) {
                                buffer.blend_pixel(x, y, color.clone());
                            }
                        }
                    }
                }
            }
            _ => {
                // Fallback to solid for other styles
                for y in y_start..y_end {
                    for x in x_start..x_end {
                        if self.is_inside_clip(x as f32, y as f32) {
                            buffer.blend_pixel(x, y, color.clone());
                        }
                    }
                }
            }
        }
    }

    /// Rasterize a box shadow.
    fn rasterize_box_shadow(&self, shadow: &BoxShadowItem, bounds: &Rect, buffer: &mut PixelBuffer) {
        // Simple box shadow implementation
        let blur = shadow.blur_radius.max(1.0);

        let x_start = (bounds.x - blur).max(0.0) as u32;
        let y_start = (bounds.y - blur).max(0.0) as u32;
        let x_end = (bounds.x + bounds.width + blur).min(buffer.width as f32) as u32;
        let y_end = (bounds.y + bounds.height + blur).min(buffer.height as f32) as u32;

        for y in y_start..y_end {
            for x in x_start..x_end {
                // Calculate distance from box edge
                let box_x = bounds.x + shadow.offset_x;
                let box_y = bounds.y + shadow.offset_y;
                let box_w = bounds.width + shadow.spread_radius * 2.0;
                let box_h = bounds.height + shadow.spread_radius * 2.0;

                let dx = if (x as f32) < box_x {
                    box_x - x as f32
                } else if (x as f32) > box_x + box_w {
                    x as f32 - (box_x + box_w)
                } else {
                    0.0
                };

                let dy = if (y as f32) < box_y {
                    box_y - y as f32
                } else if (y as f32) > box_y + box_h {
                    y as f32 - (box_y + box_h)
                } else {
                    0.0
                };

                let distance = (dx * dx + dy * dy).sqrt();

                if distance < blur {
                    let alpha = ((1.0 - distance / blur) * shadow.color.a as f32) as u8;
                    let color = Color::new(
                        shadow.color.r,
                        shadow.color.g,
                        shadow.color.b,
                        alpha,
                    );

                    if self.is_inside_clip(x as f32, y as f32) {
                        buffer.blend_pixel(x, y, color);
                    }
                }
            }
        }
    }

    /// Rasterize a linear gradient.
    fn rasterize_linear_gradient(
        &self,
        gradient: &LinearGradientItem,
        bounds: &Rect,
        buffer: &mut PixelBuffer,
    ) {
        let x_start = bounds.x.max(0.0) as u32;
        let y_start = bounds.y.max(0.0) as u32;
        let x_end = (bounds.x + bounds.width).min(buffer.width as f32) as u32;
        let y_end = (bounds.y + bounds.height).min(buffer.height as f32) as u32;

        let dx = gradient.end.x - gradient.start.x;
        let dy = gradient.end.y - gradient.start.y;
        let len = (dx * dx + dy * dy).sqrt();

        if len < 0.001 {
            return;
        }

        for y in y_start..y_end {
            for x in x_start..x_end {
                // Calculate position along gradient
                let px = x as f32 - gradient.start.x;
                let py = y as f32 - gradient.start.y;
                let t = ((px * dx + py * dy) / (len * len)).clamp(0.0, 1.0);

                let color = interpolate_gradient(&gradient.stops, t);

                if self.is_inside_clip(x as f32, y as f32) {
                    buffer.blend_pixel(x, y, color);
                }
            }
        }
    }

    /// Rasterize a radial gradient.
    fn rasterize_radial_gradient(
        &self,
        gradient: &RadialGradientItem,
        bounds: &Rect,
        buffer: &mut PixelBuffer,
    ) {
        let x_start = bounds.x.max(0.0) as u32;
        let y_start = bounds.y.max(0.0) as u32;
        let x_end = (bounds.x + bounds.width).min(buffer.width as f32) as u32;
        let y_end = (bounds.y + bounds.height).min(buffer.height as f32) as u32;

        for y in y_start..y_end {
            for x in x_start..x_end {
                // Calculate distance from center
                let dx = (x as f32 - gradient.center.x) / gradient.radius_x;
                let dy = (y as f32 - gradient.center.y) / gradient.radius_y;
                let t = (dx * dx + dy * dy).sqrt().clamp(0.0, 1.0);

                let color = interpolate_gradient(&gradient.stops, t);

                if self.is_inside_clip(x as f32, y as f32) {
                    buffer.blend_pixel(x, y, color);
                }
            }
        }
    }

    /// Rasterize a line.
    fn rasterize_line(&self, line: &LineItem, buffer: &mut PixelBuffer) {
        // Bresenham's line algorithm
        let x0 = line.start.x as i32;
        let y0 = line.start.y as i32;
        let x1 = line.end.x as i32;
        let y1 = line.end.y as i32;

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x0;
        let mut y = y0;

        loop {
            if x >= 0 && y >= 0 {
                let px = x as u32;
                let py = y as u32;

                if self.is_inside_clip(px as f32, py as f32) {
                    // Draw line with thickness
                    let half_width = (line.width / 2.0) as i32;
                    for ty in -half_width..=half_width {
                        for tx in -half_width..=half_width {
                            let fx = (px as i32 + tx) as u32;
                            let fy = (py as i32 + ty) as u32;
                            buffer.blend_pixel(fx, fy, line.color.clone());
                        }
                    }
                }
            }

            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
}

/// Interpolate between gradient stops.
fn interpolate_gradient(stops: &[GradientStop], t: f32) -> Color {
    if stops.is_empty() {
        return Color::transparent();
    }

    if stops.len() == 1 {
        return stops[0].color.clone();
    }

    // Find surrounding stops
    let mut prev = &stops[0];
    let mut next = &stops[0];

    for stop in stops {
        if stop.position <= t {
            prev = stop;
        }
        if stop.position >= t {
            next = stop;
            break;
        }
    }

    // Interpolate
    if (next.position - prev.position).abs() < 0.001 {
        return prev.color.clone();
    }

    let local_t = (t - prev.position) / (next.position - prev.position);
    Color::new(
        lerp_u8(prev.color.r, next.color.r, local_t),
        lerp_u8(prev.color.g, next.color.g, local_t),
        lerp_u8(prev.color.b, next.color.b, local_t),
        lerp_u8(prev.color.a, next.color.a, local_t),
    )
}

/// Linear interpolation for u8.
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    ((a as f32) * (1.0 - t) + (b as f32) * t) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_buffer() {
        let mut buffer = PixelBuffer::new(100, 100);
        assert_eq!(buffer.width, 100);
        assert_eq!(buffer.height, 100);
        assert_eq!(buffer.data.len(), 100 * 100 * 4);

        buffer.set_pixel(50, 50, Color::rgb(255, 0, 0));
        let pixel = buffer.get_pixel(50, 50);
        assert_eq!(pixel.r, 255);
        assert_eq!(pixel.g, 0);
        assert_eq!(pixel.b, 0);
    }

    #[test]
    fn test_gradient_interpolation() {
        let stops = vec![
            GradientStop { position: 0.0, color: Color::rgb(0, 0, 0) },
            GradientStop { position: 1.0, color: Color::rgb(255, 255, 255) },
        ];

        let color = interpolate_gradient(&stops, 0.5);
        assert!(color.r > 120 && color.r < 135);
    }
}
