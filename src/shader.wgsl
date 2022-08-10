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
fn vertex(in: Input) -> Variable {
    var out: Variable;

    out.position = in.position;
    out.texcoord = in.texcoord;

    out.transform = in.transform;
    out.inner_color = in.inner_color;
    out.outer_color = in.outer_color;
    out.erf = in.erf;
    out.stroke = in.stroke;

    var position: vec2<f32> = 2.0 * in.position * viewport.inv_size;
    out.vertex_position = vec4<f32>(position.x - 1.0, 1.0 - position.y, 0.0, 1.0);
    return out;
}

fn sdroundrect(pt: vec2<f32>, ext: vec2<f32>, rad: f32) -> f32 {
    var d: vec2<f32> = abs(pt) - ext + vec2<f32>(rad, rad);
    return min(max(d.x, d.y), 0.0) + length(max(d, vec2<f32>(0.0, 0.0))) - rad;
}

@fragment
fn main(in: Input) -> @location(0) vec4<f32> {
    var uv: vec2<f32> = in.texcoord;
    var scale: f32 = in.stroke.x;
    var limit: f32 = in.stroke.y;

    // Stroke - from [0..1] to clipped pyramid, where the slope is 1px.
    var stroke_alpha: f32 = min(1.0, (1.0 - abs(uv.x * 2.0 - 1.0)) * scale) * min(1.0, uv.y);
    if (stroke_alpha < limit) {
        discard;
    }

    var pos: vec2<f32> = in.position.xy;
    var re: f32 = in.transform.x;
    var im: f32 = in.transform.y;
    var pt: vec2<f32> = in.transform.zw + vec2<f32>(pos.x * re - pos.y * im, pos.x * im + pos.y * re);

    var extent: vec2<f32> = in.erf.xy;
    var radius: f32 = in.erf.z;
    var feather: f32 = in.erf.w;

    // Calculate gradient color using box gradient
    var d: f32 = sdroundrect(pt, extent, radius) / feather + 0.5;
    var d: f32 = clamp(d, 0.0, 1.0);
    var color: vec4<f32> = mix(in.inner_color, in.outer_color, vec4<f32>(d, d, d, d));

    // Combine alpha
    color.a = color.a * stroke_alpha;
    return color;
}

@fragment
fn stencil(in: Input) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0);
}

@fragment
fn image(in: Input) -> @location(0) vec4<f32> {
    var tex: vec4<f32> = textureSample(t_color, s_color, in.texcoord);
    return tex * in.inner_color;
}
