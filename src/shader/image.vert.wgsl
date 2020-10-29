[[location 0]] var<in> a_position : vec2<f32>;
[[location 1]] var<in> a_texcoord : vec2<f32>;
# location 2 ---
[[location 3]] var<in> a_color : vec4<f32>;

[[builtin position]] var<out> o_position : vec4<f32>;
[[location 0]] var<out> v_texcoord : vec2<f32>;
[[location 1]] var<out> v_color : vec4<f32>;

#layout(set = 0, binding = 0) uniform Viewport { vec2 u_ViewportSize; };


fn main_vert() -> void {
  v_texcoord = a_texcoord;
  v_color = a_color;

  vec2 position = 2.0 * a_position / u_ViewportSize;
  o_position = vec4<f32>(position.x - 1.0, 1.0 - position.y, 0.0, 1.0);

  return;
}
entry_point vertex as "main" = main_vert;