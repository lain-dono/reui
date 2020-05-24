#version 450

precision highp float;

layout(location = 0) out vec4 Target0;

layout(location = 0) in vec2 v_TexCoord;
layout(location = 1) in vec4 v_Color;

layout(set = 0, binding = 1) uniform sampler s_Color;
layout(set = 1, binding = 0) uniform texture2D t_Color;

void main() {
  vec4 color = texture(sampler2D(t_Color, s_Color), v_TexCoord);
  Target0 = color;
}