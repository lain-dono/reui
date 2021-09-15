[[block]] struct Viewport {
    size: vec2<f32>;
    inv_size: vec2<f32>;
};

[[group(0), binding(0)]] var<uniform> viewport: Viewport;

struct VertexInput {
    [[builtin(vertex_index)]] vertex_index: u32;
    [[location(0)]] position: vec4<f32>; // (left,top) and (right,bottom)
    [[location(1)]] texcoord: vec4<f32>;
    [[location(2)]] color: vec4<f32>;
    [[location(3)]] zindex: f32;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] texcoord: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

// generate positional data based on vertex ID
[[stage(vertex)]]
fn glyph_vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.color = in.color;

    var pos: vec2<f32> = vec2<f32>(0.0, 0.0);

    if (0u == in.vertex_index) { pos = in.position.zw; out.texcoord = in.texcoord.zw; }
    if (1u == in.vertex_index) { pos = in.position.zy; out.texcoord = in.texcoord.zy; }
    if (2u == in.vertex_index) { pos = in.position.xw; out.texcoord = in.texcoord.xw; }
    if (3u == in.vertex_index) { pos = in.position.xy; out.texcoord = in.texcoord.xy; }

    var pos: vec2<f32> = vec2<f32>(2.0, 2.0) * pos * viewport.inv_size;
    out.position = vec4<f32>(pos.x - 1.0, 1.0 - pos.y, in.zindex, 1.0);
    return out;
}


[[group(1), binding(0)]] var font_sampler: sampler;
[[group(1), binding(1)]] var font_color: texture_2d<f32>;

[[stage(fragment)]]
fn glyph_fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var alpha: f32 = textureSample(font_color, font_sampler, in.texcoord).r;
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}