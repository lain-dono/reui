#version 450

precision highp float;

layout(location = 0) out vec4 Target0;

layout(location = 0) in vec2 v_Position;
layout(location = 1) in vec2 v_TexCoord;

layout(location = 2) in vec4 v_PaintMatrix;
layout(location = 3) in vec4 v_InnerColor;
layout(location = 4) in vec4 v_OuterColor;
layout(location = 5) in vec4 v_ERF;
layout(location = 6) in vec2 v_Stroke;

#define EXTENT v_ERF.xy
#define RADIUS v_ERF.z
#define FEATHER v_ERF.w

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

// Stroke - from [0..1] to clipped pyramid, where the slope is 1px.
float strokeMask(vec2 uv, float scale) {
  return min(1.0, (1.0 - abs(uv.x * 2.0 - 1.0)) * scale) * min(1.0, uv.y);
}

void main() {
  float strokeAlpha = strokeMask(v_TexCoord, v_Stroke.x);
  if (strokeAlpha < v_Stroke.y) {
    discard;
  }

  // Calculate gradient color using box gradient
  vec2 pt = applyTransform(v_PaintMatrix, v_Position);
  float d = sdroundrect(pt, EXTENT, RADIUS) / FEATHER + 0.5;
  vec4 color = mix(v_InnerColor, v_OuterColor, clamp(d, 0.0, 1.0));
  // Combine alpha
  color.a *= strokeAlpha;
  Target0 = color;
}