#version 460 core

out vec4 Color;

uniform sampler2D tex;

in VertexAttributes {
    vec2 texture_coords;
} attrs;

void main() {
    vec4 diffuse_frag = texture(tex, attrs.texture_coords);

    if (diffuse_frag.a == 0) {
        discard;
    }

    Color = diffuse_frag;
}
