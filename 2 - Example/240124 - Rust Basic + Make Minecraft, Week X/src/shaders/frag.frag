#version 460 core

out vec4 fragColor;
in float texture_id;
in vec2 texture_coords;

uniform sampler2D textures[32];

void main() {
    int texture_id_int = int(texture_id);
    fragColor = texture(textures[texture_id_int], texture_coords);
}
