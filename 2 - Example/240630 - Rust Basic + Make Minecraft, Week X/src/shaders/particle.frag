#version 460 core

out vec4 Color;

in VertexAttributes {
    vec2 texture_coords;
} attrs;

void main() {
    vec4 diffuse_frag = vec4(1.0, 1.0, 1.0, 1.0);
    Color = diffuse_frag;
}
