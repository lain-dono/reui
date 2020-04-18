#version 450

out gl_PerVertex {
    vec4 gl_Position;
};

/*
layout(location = 0) in vec2 a_Position;
layout(location = 1) in vec2 a_TexCoord;
layout(location = 0) out vec2 v_Position;
layout(location = 1) out vec2 v_TexCoord;

layout(set = 0, binding = 0) uniform Viewport {
    vec2 size;
} viewport;
*/

void main() {
    vec2 position = vec2(gl_VertexIndex, (gl_VertexIndex & 1) * 2) - 1;
    gl_Position = vec4(position, 0.0, 1.0);

    /*
    v_TexCoord = a_TexCoord;
    v_Position = a_Position;

    float x = 2.0 * a_Position.x / viewport.size.x - 1.0;
    float y = 1.0 - 2.0 * a_Position.y / viewport.size.y;

    gl_Position = vec4(x, y, 0.0, 1.0);
    */
}