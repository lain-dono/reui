#version 450

precision highp float;

out gl_PerVertex { vec4 gl_Position; };

layout(location = 0) in vec2 a_Position;
layout(location = 1) in vec2 a_TexCoord;

layout(location = 2) in vec4 a_PaintMatrix;
layout(location = 3) in vec4 a_InnerColor;
layout(location = 4) in vec4 a_OuterColor;
layout(location = 5) in vec4 a_ERF;
layout(location = 6) in vec2 a_Stroke;

layout(location = 0) out vec2 v_Position;
layout(location = 1) out vec2 v_TexCoord;
layout(location = 2) out vec4 v_PaintMatrix;
layout(location = 3) out vec4 v_InnerColor;
layout(location = 4) out vec4 v_OuterColor;
layout(location = 5) out vec4 v_ERF;
layout(location = 6) out vec2 v_Stroke;

layout(set = 0, binding = 0) uniform Viewport { vec2 u_ViewportInvSize; };

void main() {
  v_TexCoord = a_TexCoord;
  v_Position = a_Position;

  v_PaintMatrix = a_PaintMatrix;
  v_InnerColor = a_InnerColor;
  v_OuterColor = a_OuterColor;
  v_ERF = a_ERF;
  v_Stroke = a_Stroke;

  vec2 position = 2.0 * a_Position * u_ViewportInvSize;
  gl_Position = vec4(position.x - 1.0, 1.0 - position.y, 0.0, 1.0);
}