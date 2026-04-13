#version 460 core

uniform mat4 model;
uniform mat4 projection;

layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 texture_coords;

out VertexAttributes {
    vec2 texture_coords;
} attrs;

void main() {
    gl_Position = projection * model * vec4(pos, 1.0);

    attrs.texture_coords = texture_coords;
}
