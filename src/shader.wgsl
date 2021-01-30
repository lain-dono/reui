[[block]] struct Viewport {
    inv_size: vec2<f32>;
};

[[location(0)]] var<in> in_position_vs: vec2<f32>;
[[location(1)]] var<in> in_texcoord_vs: vec2<f32>;
[[location(2)]] var<in> in_paint_matrix_vs: vec4<f32>;
[[location(3)]] var<in> in_inner_color_vs: vec4<f32>;
[[location(4)]] var<in> in_outer_color_vs: vec4<f32>;
[[location(5)]] var<in> in_erf_vs: vec4<f32>;
[[location(6)]] var<in> in_stroke_vs: vec2<f32>;

[[builtin(position)]] var<out> output_vs: vec4<f32>;

[[location(0)]] var<out> out_position_vs: vec2<f32>;
[[location(1)]] var<out> out_texcoord_vs: vec2<f32>;
[[location(2)]] var<out> out_paint_matrix_vs: vec4<f32>;
[[location(3)]] var<out> out_inner_color_vs: vec4<f32>;
[[location(4)]] var<out> out_outer_color_vs: vec4<f32>;
[[location(5)]] var<out> out_erf_vs: vec4<f32>;
[[location(6)]] var<out> out_stroke_vs: vec2<f32>;

[[group(0), binding(0)]] var<uniform> viewport: Viewport;

[[stage(vertex)]]
fn vertex() {
    out_position_vs = in_position_vs;
    out_texcoord_vs = in_texcoord_vs;

    out_paint_matrix_vs = in_paint_matrix_vs;
    out_inner_color_vs = in_inner_color_vs;
    out_outer_color_vs = in_outer_color_vs;
    out_erf_vs = in_erf_vs;
    out_stroke_vs = in_stroke_vs;

    var position: vec2<f32> = 2.0 * in_position_vs * viewport.inv_size;
    output_vs = vec4<f32>(position.x - 1.0, 1.0 - position.y, 0.0, 1.0);
}

fn sdroundrect(pt: vec2<f32>, ext: vec2<f32>, rad: f32) -> f32 {
    var d: vec2<f32> = abs(pt) - ext + vec2<f32>(rad, rad);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0)) - rad;
}

[[location(0)]] var<out> out_color_fs: vec4<f32>;

[[location(0)]] var<in> in_position_fs: vec2<f32>;
[[location(1)]] var<in> in_texcoord_fs: vec2<f32>;
[[location(2)]] var<in> in_paint_matrix_fs: vec4<f32>;
[[location(3)]] var<in> in_inner_color_fs: vec4<f32>;
[[location(4)]] var<in> in_outer_color_fs: vec4<f32>;
[[location(5)]] var<in> in_erf_fs: vec4<f32>;
[[location(6)]] var<in> in_stroke_fs: vec2<f32>;

[[stage(fragment)]]
fn main() {
    var uv: vec2<f32> = in_texcoord_fs;
    var scale: f32 = in_stroke_fs.x;
    var limit: f32 = in_stroke_fs.y;

    // Stroke - from [0..1] to clipped pyramid, where the slope is 1px.
    var stroke_alpha: f32 = min(1.0, (1.0 - abs(uv.x * 2.0 - 1.0)) * scale) * min(1.0, uv.y);
    if (stroke_alpha < limit) {
        discard;
    }

    var pos: vec2<f32> = in_position_fs.xy;
    var re: f32 = in_paint_matrix_fs.x;
    var im: f32 = in_paint_matrix_fs.y;
    var pt: vec2<f32> = in_paint_matrix_fs.zw + vec2<f32>(pos.x * re - pos.y * im, pos.x * im + pos.y * re);

    var extent: vec2<f32> = in_erf_fs.xy;
    var radius: f32 = in_erf_fs.z;
    var feather: f32 = in_erf_fs.w;

    // Calculate gradient color using box gradient
    var d: f32 = sdroundrect(pt, extent, radius) / feather + 0.5;
    var color: vec4<f32> = mix(in_inner_color_fs, in_outer_color_fs, clamp(d, 0.0, 1.0));

    // Combine alpha
    color.a = color.a * stroke_alpha;
    out_color_fs = color;
}

[[stage(fragment)]]
fn stencil() {}

[[location(1)]] var<in> in_tex_coord_fs: vec2<f32>;
[[location(2)]] var<in> in_color_fs: vec4<f32>;

[[location(0)]] var<out> out_color_fs: vec4<f32>;

[[group(1), binding(0)]] var s_color: sampler;
[[group(1), binding(1)]] var t_color: texture_2d<f32>;

[[stage(fragment)]]
fn image() {
    var tex: vec4<f32> = textureSample(t_color, s_color, in_tex_coord_fs); //  * in_color_fs
    out_color_fs = tex;
}