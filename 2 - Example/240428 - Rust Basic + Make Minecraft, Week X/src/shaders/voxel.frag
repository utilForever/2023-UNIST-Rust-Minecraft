#version 460 core

out vec4 Color;

uniform sampler2D atlas;

in VertexAttributes {
    vec3 frag_pos;
    vec2 texture_coords;
} attrs;

void main() {
    vec4 diffuse_frag = texture(atlas, attrs.texture_coords);

    if (diffuse_frag.a == 0) {
        discard;
    }

    Color = diffuse_frag;
}
