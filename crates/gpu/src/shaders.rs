//! WGSL shaders for GPU rendering.

/// Solid color shader.
pub const SOLID_COLOR_SHADER: &str = r#"
// Uniforms
struct Uniforms {
    transform: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

// Vertex output
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = uniforms.transform * vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
"#;

/// Textured shader.
pub const TEXTURED_SHADER: &str = r#"
// Uniforms
struct Uniforms {
    transform: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var t_texture: texture_2d<f32>;

@group(1) @binding(1)
var s_texture: sampler;

// Vertex input
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec4<f32>,
}

// Vertex output
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = uniforms.transform * vec4<f32>(input.position, 0.0, 1.0);
    output.tex_coord = input.tex_coord;
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_texture, s_texture, input.tex_coord);
    return tex_color * input.color;
}
"#;

/// Text rendering shader (alpha from texture).
pub const TEXT_SHADER: &str = r#"
// Uniforms
struct Uniforms {
    transform: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var t_glyph: texture_2d<f32>;

@group(1) @binding(1)
var s_glyph: sampler;

// Vertex input
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec4<f32>,
}

// Vertex output
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = uniforms.transform * vec4<f32>(input.position, 0.0, 1.0);
    output.tex_coord = input.tex_coord;
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(t_glyph, s_glyph, input.tex_coord).a;
    return vec4<f32>(input.color.rgb, input.color.a * alpha);
}
"#;

/// Gradient shader.
pub const GRADIENT_SHADER: &str = r#"
// Uniforms
struct Uniforms {
    transform: mat4x4<f32>,
}

struct GradientUniforms {
    start: vec2<f32>,
    end: vec2<f32>,
    color0: vec4<f32>,
    color1: vec4<f32>,
    color2: vec4<f32>,
    color3: vec4<f32>,
    stop0: f32,
    stop1: f32,
    stop2: f32,
    stop3: f32,
    num_stops: u32,
    _padding: vec3<u32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

// Vertex output
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = uniforms.transform * vec4<f32>(input.position, 0.0, 1.0);
    output.world_pos = input.position;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Simple linear gradient fallback
    // Full implementation would use GradientUniforms
    let t = input.world_pos.x / 1000.0;
    return vec4<f32>(t, t, t, 1.0);
}
"#;

/// Box shadow shader.
pub const SHADOW_SHADER: &str = r#"
// Uniforms
struct ShadowUniforms {
    transform: mat4x4<f32>,
    shadow_color: vec4<f32>,
    box_rect: vec4<f32>,  // x, y, width, height
    blur_radius: f32,
    spread_radius: f32,
    offset: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: ShadowUniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec2<f32>,
}

// Vertex output
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = uniforms.transform * vec4<f32>(input.position, 0.0, 1.0);
    output.world_pos = input.position;
    return output;
}

// Calculate distance to box edge
fn box_distance(p: vec2<f32>, b: vec2<f32>) -> f32 {
    let d = abs(p) - b;
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let box_center = uniforms.box_rect.xy + uniforms.box_rect.zw * 0.5 + uniforms.offset;
    let box_half_size = uniforms.box_rect.zw * 0.5 + uniforms.spread_radius;

    let dist = box_distance(input.world_pos - box_center, box_half_size);

    // Gaussian-like falloff for blur
    let alpha = 1.0 - smoothstep(-uniforms.blur_radius, uniforms.blur_radius, dist);

    return vec4<f32>(uniforms.shadow_color.rgb, uniforms.shadow_color.a * alpha);
}
"#;

/// Blur shader (for backdrop-filter, etc.).
pub const BLUR_SHADER: &str = r#"
// Uniforms
struct BlurUniforms {
    direction: vec2<f32>,  // (1, 0) for horizontal, (0, 1) for vertical
    radius: f32,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: BlurUniforms;

@group(1) @binding(0)
var t_source: texture_2d<f32>;

@group(1) @binding(1)
var s_source: sampler;

// Vertex input
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
}

// Vertex output
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.tex_coord = input.tex_coord;
    return output;
}

// Gaussian weights (9-tap)
const WEIGHTS: array<f32, 5> = array<f32, 5>(
    0.227027,
    0.1945946,
    0.1216216,
    0.054054,
    0.016216
);

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tex_size = vec2<f32>(textureDimensions(t_source));
    let pixel_size = 1.0 / tex_size;
    let offset = uniforms.direction * pixel_size * uniforms.radius;

    var color = textureSample(t_source, s_source, input.tex_coord) * WEIGHTS[0];

    for (var i = 1; i < 5; i = i + 1) {
        let sample_offset = offset * f32(i);
        color += textureSample(t_source, s_source, input.tex_coord + sample_offset) * WEIGHTS[i];
        color += textureSample(t_source, s_source, input.tex_coord - sample_offset) * WEIGHTS[i];
    }

    return color;
}
"#;

/// Rounded rectangle shader with SDF.
pub const ROUNDED_RECT_SHADER: &str = r#"
// Uniforms
struct RoundedRectUniforms {
    transform: mat4x4<f32>,
    rect: vec4<f32>,  // x, y, width, height
    radii: vec4<f32>,  // top_left, top_right, bottom_right, bottom_left
    color: vec4<f32>,
    border_width: f32,
    border_color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: RoundedRectUniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec2<f32>,
}

// Vertex output
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = uniforms.transform * vec4<f32>(input.position, 0.0, 1.0);
    output.local_pos = input.position - uniforms.rect.xy;
    return output;
}

// SDF for rounded rectangle
fn rounded_rect_sdf(p: vec2<f32>, size: vec2<f32>, radii: vec4<f32>) -> f32 {
    // Select radius based on quadrant
    var r: f32;
    if p.x > size.x * 0.5 {
        if p.y > size.y * 0.5 {
            r = radii.z;  // bottom_right
        } else {
            r = radii.y;  // top_right
        }
    } else {
        if p.y > size.y * 0.5 {
            r = radii.w;  // bottom_left
        } else {
            r = radii.x;  // top_left
        }
    }

    let q = abs(p - size * 0.5) - size * 0.5 + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let size = uniforms.rect.zw;
    let dist = rounded_rect_sdf(input.local_pos, size, uniforms.radii);

    // Anti-aliasing
    let aa = fwidth(dist);
    let alpha = 1.0 - smoothstep(-aa, aa, dist);

    // Border
    if uniforms.border_width > 0.0 {
        let inner_dist = dist + uniforms.border_width;
        let border_alpha = smoothstep(-aa, aa, inner_dist);
        let fill_color = vec4<f32>(uniforms.color.rgb, uniforms.color.a * (1.0 - border_alpha));
        let border_color_result = vec4<f32>(uniforms.border_color.rgb, uniforms.border_color.a * border_alpha);
        return mix(fill_color, border_color_result, border_alpha) * alpha;
    }

    return vec4<f32>(uniforms.color.rgb, uniforms.color.a * alpha);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_strings() {
        // Just verify the shaders are not empty
        assert!(!SOLID_COLOR_SHADER.is_empty());
        assert!(!TEXTURED_SHADER.is_empty());
        assert!(!TEXT_SHADER.is_empty());
        assert!(!GRADIENT_SHADER.is_empty());
        assert!(!SHADOW_SHADER.is_empty());
        assert!(!BLUR_SHADER.is_empty());
        assert!(!ROUNDED_RECT_SHADER.is_empty());
    }
}
