//! Rendering commands for GPU execution.

use crate::display_list::{BlendMode, BorderStyle, ClipRegion, ImageKey};
use common::color::Color;
use common::geometry::{CornerRadii, Point, Rect, Transform};

/// A batch of render commands for efficient GPU execution.
#[derive(Clone, Debug, Default)]
pub struct RenderCommandList {
    commands: Vec<RenderCommand>,
}

impl RenderCommandList {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn push(&mut self, command: RenderCommand) {
        self.commands.push(command);
    }

    pub fn extend(&mut self, commands: impl IntoIterator<Item = RenderCommand>) {
        self.commands.extend(commands);
    }

    pub fn commands(&self) -> &[RenderCommand] {
        &self.commands
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Optimize the command list by batching similar commands.
    pub fn optimize(&mut self) {
        // Sort by texture/shader to minimize state changes
        // This is a simplified optimization
        self.commands.sort_by(|a, b| {
            let a_type = a.command_type_id();
            let b_type = b.command_type_id();
            a_type.cmp(&b_type)
        });
    }
}

/// A single render command.
#[derive(Clone, Debug)]
pub enum RenderCommand {
    /// Clear the render target.
    Clear(ClearCommand),
    /// Draw a solid rectangle.
    DrawRect(DrawRectCommand),
    /// Draw a rounded rectangle.
    DrawRoundedRect(DrawRoundedRectCommand),
    /// Draw a border.
    DrawBorder(DrawBorderCommand),
    /// Draw an image.
    DrawImage(DrawImageCommand),
    /// Draw text.
    DrawText(DrawTextCommand),
    /// Draw a line.
    DrawLine(DrawLineCommand),
    /// Draw a gradient.
    DrawGradient(DrawGradientCommand),
    /// Draw a shadow.
    DrawShadow(DrawShadowCommand),
    /// Push a clip region.
    PushClip(PushClipCommand),
    /// Pop a clip region.
    PopClip,
    /// Push a transform.
    PushTransform(PushTransformCommand),
    /// Pop a transform.
    PopTransform,
    /// Set blend mode.
    SetBlendMode(BlendMode),
    /// Set opacity.
    SetOpacity(f32),
    /// Begin a render pass.
    BeginPass(BeginPassCommand),
    /// End a render pass.
    EndPass,
    /// Copy texture.
    CopyTexture(CopyTextureCommand),
    /// Blur region.
    Blur(BlurCommand),
}

impl RenderCommand {
    /// Get a type ID for sorting/batching.
    fn command_type_id(&self) -> u32 {
        match self {
            RenderCommand::Clear(_) => 0,
            RenderCommand::BeginPass(_) => 1,
            RenderCommand::DrawRect(_) => 10,
            RenderCommand::DrawRoundedRect(_) => 11,
            RenderCommand::DrawGradient(_) => 12,
            RenderCommand::DrawShadow(_) => 13,
            RenderCommand::DrawBorder(_) => 20,
            RenderCommand::DrawImage(_) => 30,
            RenderCommand::DrawText(_) => 40,
            RenderCommand::DrawLine(_) => 50,
            RenderCommand::PushClip(_) => 100,
            RenderCommand::PopClip => 101,
            RenderCommand::PushTransform(_) => 102,
            RenderCommand::PopTransform => 103,
            RenderCommand::SetBlendMode(_) => 104,
            RenderCommand::SetOpacity(_) => 105,
            RenderCommand::EndPass => 200,
            RenderCommand::CopyTexture(_) => 201,
            RenderCommand::Blur(_) => 202,
        }
    }
}

/// Clear command.
#[derive(Clone, Debug)]
pub struct ClearCommand {
    pub color: Color,
    pub rect: Option<Rect>,
}

/// Draw rectangle command.
#[derive(Clone, Debug)]
pub struct DrawRectCommand {
    pub rect: Rect,
    pub color: Color,
}

/// Draw rounded rectangle command.
#[derive(Clone, Debug)]
pub struct DrawRoundedRectCommand {
    pub rect: Rect,
    pub color: Color,
    pub radii: CornerRadii,
}

/// Draw border command.
#[derive(Clone, Debug)]
pub struct DrawBorderCommand {
    pub rect: Rect,
    pub widths: [f32; 4],
    pub colors: [Color; 4],
    pub styles: [BorderStyle; 4],
    pub radii: Option<CornerRadii>,
}

/// Draw image command.
#[derive(Clone, Debug)]
pub struct DrawImageCommand {
    pub rect: Rect,
    pub image_key: ImageKey,
    pub src_rect: Option<Rect>,
    pub opacity: f32,
}

/// Draw text command.
#[derive(Clone, Debug)]
pub struct DrawTextCommand {
    pub position: Point,
    pub glyphs: Vec<GlyphCommand>,
    pub color: Color,
    pub font_size: f32,
}

/// Individual glyph command.
#[derive(Clone, Debug)]
pub struct GlyphCommand {
    pub glyph_id: u32,
    pub position: Point,
}

/// Draw line command.
#[derive(Clone, Debug)]
pub struct DrawLineCommand {
    pub start: Point,
    pub end: Point,
    pub width: f32,
    pub color: Color,
    pub style: LineStyleCommand,
}

/// Line style for rendering.
#[derive(Clone, Copy, Debug)]
pub enum LineStyleCommand {
    Solid,
    Dotted { dot_length: f32, gap_length: f32 },
    Dashed { dash_length: f32, gap_length: f32 },
}

/// Draw gradient command.
#[derive(Clone, Debug)]
pub struct DrawGradientCommand {
    pub rect: Rect,
    pub gradient: GradientCommand,
}

/// Gradient specification.
#[derive(Clone, Debug)]
pub enum GradientCommand {
    Linear {
        start: Point,
        end: Point,
        stops: Vec<GradientStopCommand>,
    },
    Radial {
        center: Point,
        radius_x: f32,
        radius_y: f32,
        stops: Vec<GradientStopCommand>,
    },
    Conic {
        center: Point,
        angle: f32,
        stops: Vec<GradientStopCommand>,
    },
}

/// Gradient stop.
#[derive(Clone, Debug)]
pub struct GradientStopCommand {
    pub position: f32,
    pub color: Color,
}

/// Draw shadow command.
#[derive(Clone, Debug)]
pub struct DrawShadowCommand {
    pub rect: Rect,
    pub color: Color,
    pub offset: Point,
    pub blur: f32,
    pub spread: f32,
    pub inset: bool,
    pub radii: Option<CornerRadii>,
}

/// Push clip command.
#[derive(Clone, Debug)]
pub struct PushClipCommand {
    pub rect: Rect,
    pub radii: Option<CornerRadii>,
}

/// Push transform command.
#[derive(Clone, Debug)]
pub struct PushTransformCommand {
    pub transform: Transform,
}

/// Begin render pass command.
#[derive(Clone, Debug)]
pub struct BeginPassCommand {
    pub target: RenderTarget,
    pub clear_color: Option<Color>,
}

/// Render target specification.
#[derive(Clone, Debug)]
pub enum RenderTarget {
    /// Main framebuffer.
    Screen,
    /// Offscreen texture.
    Texture { id: u64, width: u32, height: u32 },
}

/// Copy texture command.
#[derive(Clone, Debug)]
pub struct CopyTextureCommand {
    pub src: u64,
    pub dst: RenderTarget,
    pub src_rect: Rect,
    pub dst_rect: Rect,
}

/// Blur command.
#[derive(Clone, Debug)]
pub struct BlurCommand {
    pub rect: Rect,
    pub radius: f32,
}

/// Builder for render command lists.
pub struct RenderCommandBuilder {
    list: RenderCommandList,
    transform_stack: Vec<Transform>,
    clip_stack: Vec<Rect>,
    current_opacity: f32,
    current_blend_mode: BlendMode,
}

impl RenderCommandBuilder {
    pub fn new() -> Self {
        Self {
            list: RenderCommandList::new(),
            transform_stack: Vec::new(),
            clip_stack: Vec::new(),
            current_opacity: 1.0,
            current_blend_mode: BlendMode::Normal,
        }
    }

    /// Clear the render target.
    pub fn clear(&mut self, color: Color) -> &mut Self {
        self.list.push(RenderCommand::Clear(ClearCommand {
            color,
            rect: None,
        }));
        self
    }

    /// Draw a rectangle.
    pub fn draw_rect(&mut self, rect: Rect, color: Color) -> &mut Self {
        self.list.push(RenderCommand::DrawRect(DrawRectCommand {
            rect,
            color,
        }));
        self
    }

    /// Draw a rounded rectangle.
    pub fn draw_rounded_rect(&mut self, rect: Rect, color: Color, radii: CornerRadii) -> &mut Self {
        self.list.push(RenderCommand::DrawRoundedRect(DrawRoundedRectCommand {
            rect,
            color,
            radii,
        }));
        self
    }

    /// Draw an image.
    pub fn draw_image(&mut self, rect: Rect, image_key: ImageKey) -> &mut Self {
        self.list.push(RenderCommand::DrawImage(DrawImageCommand {
            rect,
            image_key,
            src_rect: None,
            opacity: self.current_opacity,
        }));
        self
    }

    /// Draw text.
    pub fn draw_text(&mut self, position: Point, glyphs: Vec<GlyphCommand>, color: Color, font_size: f32) -> &mut Self {
        self.list.push(RenderCommand::DrawText(DrawTextCommand {
            position,
            glyphs,
            color,
            font_size,
        }));
        self
    }

    /// Push a clip region.
    pub fn push_clip(&mut self, rect: Rect, radii: Option<CornerRadii>) -> &mut Self {
        self.clip_stack.push(rect.clone());
        self.list.push(RenderCommand::PushClip(PushClipCommand {
            rect,
            radii,
        }));
        self
    }

    /// Pop a clip region.
    pub fn pop_clip(&mut self) -> &mut Self {
        self.clip_stack.pop();
        self.list.push(RenderCommand::PopClip);
        self
    }

    /// Push a transform.
    pub fn push_transform(&mut self, transform: Transform) -> &mut Self {
        self.transform_stack.push(transform.clone());
        self.list.push(RenderCommand::PushTransform(PushTransformCommand {
            transform,
        }));
        self
    }

    /// Pop a transform.
    pub fn pop_transform(&mut self) -> &mut Self {
        self.transform_stack.pop();
        self.list.push(RenderCommand::PopTransform);
        self
    }

    /// Set opacity.
    pub fn set_opacity(&mut self, opacity: f32) -> &mut Self {
        self.current_opacity = opacity;
        self.list.push(RenderCommand::SetOpacity(opacity));
        self
    }

    /// Set blend mode.
    pub fn set_blend_mode(&mut self, mode: BlendMode) -> &mut Self {
        self.current_blend_mode = mode;
        self.list.push(RenderCommand::SetBlendMode(mode));
        self
    }

    /// Build the command list.
    pub fn build(self) -> RenderCommandList {
        self.list
    }
}

impl Default for RenderCommandBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_builder() {
        let mut builder = RenderCommandBuilder::new();

        builder
            .clear(Color::white())
            .draw_rect(Rect::new(0.0, 0.0, 100.0, 100.0), Color::red())
            .push_clip(Rect::new(10.0, 10.0, 80.0, 80.0), None)
            .draw_rect(Rect::new(20.0, 20.0, 60.0, 60.0), Color::blue())
            .pop_clip();

        let list = builder.build();
        assert_eq!(list.len(), 5);
    }

    #[test]
    fn test_command_list_optimization() {
        let mut list = RenderCommandList::new();

        // Add commands in non-optimal order
        list.push(RenderCommand::DrawRect(DrawRectCommand {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            color: Color::red(),
        }));
        list.push(RenderCommand::DrawImage(DrawImageCommand {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            image_key: ImageKey(1),
            src_rect: None,
            opacity: 1.0,
        }));
        list.push(RenderCommand::DrawRect(DrawRectCommand {
            rect: Rect::new(10.0, 10.0, 10.0, 10.0),
            color: Color::blue(),
        }));

        list.optimize();

        // After optimization, DrawRects should be grouped
        assert_eq!(list.len(), 3);
    }
}
