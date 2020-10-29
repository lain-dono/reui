#version 450

precision highp float;

out gl_PerVertex { vec4 gl_Position; };

layout(location = 0) in vec2 a_Position;
layout(location = 1) in vec2 a_TexCoord;

layout(location = 3) in vec4 a_Color;

layout(location = 0) out vec2 v_TexCoord;
layout(location = 1) out vec4 v_Color;

layout(set = 0, binding = 0) uniform Viewport { vec2 u_ViewportInvSize; };

void main() {
  v_TexCoord = a_TexCoord;
  v_Color = a_Color;

  vec2 position = 2.0 * a_Position * u_ViewportInvSize;
  gl_Position = vec4(position.x - 1.0, 1.0 - position.y, 0.0, 1.0);
}