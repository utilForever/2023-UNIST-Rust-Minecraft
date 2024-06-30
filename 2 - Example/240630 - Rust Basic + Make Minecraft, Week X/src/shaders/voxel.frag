#version 460 core

out vec4 Color;

uniform sampler2D atlas;

in VertexAttributes {
    vec3 frag_pos;
    vec2 texture_coords;
    vec3 normal;
    float ao;
} attrs;

void main() {
    vec4 diffuse_frag = texture(atlas, attrs.texture_coords);

    if (diffuse_frag.a == 0) {
        discard;
    }

    Color = diffuse_frag;

    if (attrs.normal.x != 0.0) {
        Color.rgb *= 0.7;
    } else if (attrs.normal.z != 0.0) {
        Color.rgb *= 0.85;
    } else if (attrs.normal.y != 0.0) {
        Color.rgb *= 1.0;
    }

    Color.rgb *= (1.0 - attrs.ao * 0.15);
}
