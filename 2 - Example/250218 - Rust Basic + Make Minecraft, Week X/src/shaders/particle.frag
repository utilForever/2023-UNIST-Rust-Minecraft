#version 460 core

out vec4 Color;

uniform sampler2DArray array_texture;

in VertexAttributes {
    vec3 texture_coords;
} attrs;

void main() {
    vec4 diffuse_frag = texture(array_texture, attrs.texture_coords);
    Color = diffuse_frag;
}
