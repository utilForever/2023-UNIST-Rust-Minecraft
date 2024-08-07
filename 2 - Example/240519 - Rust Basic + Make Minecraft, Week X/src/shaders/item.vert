#version 460 core

uniform mat4 model;
uniform mat4 projection;

layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 texture_coords;
layout (location = 2) in vec3 normal;

out VertexAttributes {
    vec2 texture_coords;
    vec3 normal;
} attrs;

void main() {
    gl_Position = projection * model * vec4(pos, 1.0);

    attrs.texture_coords = texture_coords;
    attrs.normal = normal;
}
