#version 450

// precision highp float;

// TODO: can be passed via per-instance vertex buffer
struct FragState {
  vec4 scissor_transform;
  vec2 scissor_extent;
  vec2 scissor_scale;

  vec4 paint_transform;
  vec4 inner_color;
  vec4 outer_color;
  vec2 extent;
  float radius;
  float feather;
  float stroke_mul;
  float stroke_thr;
  float _padding;
  float type;
};

layout(location = 0) out vec4 Target0;

layout(location = 0) in vec2 v_Position;
layout(location = 1) in vec2 v_TexCoord;
layout(location = 2) in flat uint v_Index;

layout(std140, set = 0, binding = 1) buffer State { FragState states[]; };

#define state states[v_Index]

float sdroundrect(vec2 pt, vec2 ext, float rad) {
  vec2 ext2 = ext - vec2(rad, rad);
  vec2 d = abs(pt) - ext2;
  return min(max(d.x, d.y), 0.0) + length(max(d, 0.0)) - rad;
}

vec2 applyTransform(vec4 transform, vec2 pt) {
  float re = transform.x;
  float im = transform.y;
  return transform.zw + vec2(pt.x * re - pt.y * im, pt.x * im + pt.y * re);
}

// Scissoring
float scissorMask(vec2 p) {
  vec2 sc = abs(applyTransform(state.scissor_transform, p));
  sc -= state.scissor_extent;
  sc *= state.scissor_scale;
  return clamp(0.5 - sc.x, 0.0, 1.0) * clamp(0.5 - sc.y, 0.0, 1.0);
}

// Stroke - from [0..1] to clipped pyramid, where the slope is 1px.
float strokeMask() {
  return min(1.0, (1.0 - abs(v_TexCoord.x * 2.0 - 1.0)) * state.stroke_mul) *
         min(1.0, v_TexCoord.y);
}

void main() {
  float strokeAlpha = strokeMask();
  if (strokeAlpha < state.stroke_thr) {
    discard;
  }

  if (state.type == 0) {
    // Stencil fill
    Target0 = vec4(1.0, 1.0, 1.0, 1.0);
  } else if (state.type == 1) {
    float scissor = scissorMask(v_Position);

    // Calculate gradient color using box gradient
    vec2 pt = applyTransform(state.paint_transform, v_Position);
    float d = clamp(
        (sdroundrect(pt, state.extent, state.radius) + state.feather * 0.5) /
            state.feather,
        0.0, 1.0);
    vec4 color = mix(state.inner_color, state.outer_color, d);
    // Combine alpha
    color.a *= strokeAlpha * scissor;
    Target0 = color;
  }
}