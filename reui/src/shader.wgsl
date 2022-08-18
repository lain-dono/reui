struct Viewport {
    inv_size: vec2<f32>,
}

struct Input {
    @location(0) position: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
    @location(2) transform: vec4<f32>,
    @location(3) inner_color: vec4<f32>,
    @location(4) outer_color: vec4<f32>,
    @location(5) erf: vec4<f32>,
    @location(6) stroke: vec2<f32>,
}

struct Variable {
    @builtin(position) vertex_position: vec4<f32>,

    @location(0) position: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
    @location(2) transform: vec4<f32>,
    @location(3) inner_color: vec4<f32>,
    @location(4) outer_color: vec4<f32>,
    @location(5) erf: vec4<f32>,
    @location(6) stroke: vec2<f32>,
}

@group(0) @binding(0) var<uniform> viewport: Viewport;
@group(1) @binding(0) var s_color: sampler;
@group(1) @binding(1) var t_color: texture_2d<f32>;

@vertex
fn vertex_main(in: Input) -> Variable {
    let position = in.position * viewport.inv_size * 2.0;

    var out: Variable;
    out.vertex_position = vec4<f32>(position.x - 1.0, 1.0 - position.y, 0.0, 1.0);

    out.position = in.position;
    out.texcoord = in.texcoord;

    out.transform = in.transform;
    out.inner_color = in.inner_color;
    out.outer_color = in.outer_color;
    out.erf = in.erf;
    out.stroke = in.stroke;

    return out;
}

fn sdroundrect(pt: vec2<f32>, ext: vec2<f32>, rad: f32) -> f32 {
    let d = abs(pt) - ext + vec2<f32>(rad, rad);
    return min(max(d.x, d.y), 0.0) + length(max(d, vec2<f32>(0.0))) - rad;
}

@fragment
fn fragment_main(in: Input) -> @location(0) vec4<f32> {
    let uv = in.texcoord;
    let scale = in.stroke.x;
    let limit = in.stroke.y;

    // Stroke - from [0..1] to clipped pyramid, where the slope is 1px.
    let stroke_alpha = min(1.0, (1.0 - abs(uv.x * 2.0 - 1.0)) * scale) * uv.y;
    if (stroke_alpha < limit) {
        discard;
    }

    let pos = in.position.xy;
    let re = in.transform.x;
    let im = in.transform.y;
    let pt = in.transform.zw + vec2<f32>(pos.x * re - pos.y * im, pos.x * im + pos.y * re);

    let extent = in.erf.xy;
    let radius = in.erf.z;
    let feather = in.erf.w;

    // Calculate gradient color using box gradient
    let d = sdroundrect(pt, extent, radius) * feather + 0.5;
    let d = clamp(d, 0.0, 1.0);
    let color = mix(in.inner_color, in.outer_color, vec4<f32>(d));

    // Combine alpha
    return vec4<f32>(color.rgb, color.a * stroke_alpha);
}

@fragment
fn fragment_convex_simple(in: Input) -> @location(0) vec4<f32> {
    let uv = in.texcoord;
    let scale = in.stroke.x;
    let alpha = min(1.0, (1.0 - abs(uv.x * 2.0 - 1.0)) * scale) * uv.y;
    let color = in.inner_color;
    return vec4<f32>(color.rgb, color.a * alpha);
}

@vertex
fn vertex_stencil(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    let position = position * viewport.inv_size * 2.0;
    return vec4<f32>(position.x - 1.0, 1.0 - position.y, 0.0, 1.0);
}

@fragment
fn fragment_stencil() {}

struct BlitOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
}

@vertex
fn vertex_blit(
    @location(0) position: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
) -> BlitOutput {
    let position = position * viewport.inv_size * 2.0;
    let position = vec4<f32>(position.x - 1.0, 1.0 - position.y, 0.0, 1.0);
    return BlitOutput(position, texcoord);
}

@fragment
fn fragment_premultiplied(@location(0) texcoord: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(t_color, s_color, texcoord);
}

@fragment
fn fragment_unmultiplied(@location(0) texcoord: vec2<f32>) -> @location(0) vec4<f32> {
    let color = textureSample(t_color, s_color, texcoord);
    return vec4<f32>(color.rgb * color.a, color.a);
}

@fragment
fn fragment_font(@location(0) texcoord: vec2<f32>) -> @location(0) vec4<f32> {
    let alpha = textureSample(t_color, s_color, texcoord).r;
    return vec4<f32>(0.0, 0.0, 0.0, alpha);
}