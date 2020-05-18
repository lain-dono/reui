#version 450

precision highp float;

out gl_PerVertex { vec4 gl_Position; };

layout(location = 0) in vec2 a_Position;

layout(set = 0, binding = 0) uniform Viewport { vec2 u_ViewportSize; };

void main() {
  vec2 position = 2.0 * a_Position / u_ViewportSize;
  gl_Position = vec4(position.x - 1.0, 1.0 - position.y, 0.0, 1.0);
}