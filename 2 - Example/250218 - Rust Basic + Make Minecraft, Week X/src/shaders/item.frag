#version 460 core

out vec4 Color;

uniform sampler2DArray tex;

in VertexAttributes {
    vec3 texture_coords;
    vec3 normal;
} attrs;

void main() {
    vec4 diffuse_frag = texture(tex, attrs.texture_coords);

    if (diffuse_frag.a == 0) {
        discard;
    }

    Color = diffuse_frag;

    if (attrs.normal.z == 1.0) {
        Color.rgb *= 0.5;
    } else if (attrs.normal.x == -1.0) {
        Color.rgb *= 0.7;
    }
}
