#version 460 core

out vec4 Color;

uniform sampler2DArray array_texture;

in VertexAttributes {
    vec3 texture_coords;
    vec3 normal;
} attrs;

void main() {
    vec4 diffuse_frag = texture(array_texture, attrs.texture_coords);

    if (diffuse_frag.a == 0) {
        discard;
    }

    Color = diffuse_frag;

    Color.rgb *= (1.0 - abs(attrs.normal.z) * 0.2);
    Color.rgb *= (1.0 - abs(attrs.normal.x) * 0.4);
}
