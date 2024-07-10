#version 460 core

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 texture_coords;

out VertexAttributes {
    vec2 texture_coords;
} attrs;

void main() {
    // Billboarding
    mat4 model_view_matrix = view * model;
    model_view_matrix[0][0] = 0.2;
    model_view_matrix[1][1] = 0.2;
    model_view_matrix[2][2] = 0.2;
    model_view_matrix[0][1] = 0.0;
    model_view_matrix[0][2] = 0.0;
    model_view_matrix[1][0] = 0.0;
    model_view_matrix[1][2] = 0.0;
    model_view_matrix[2][0] = 0.0;
    model_view_matrix[2][1] = 0.0;

    gl_Position = projection * model_view_matrix * vec4(pos, 1.0);

    attrs.texture_coords = texture_coords;
}
