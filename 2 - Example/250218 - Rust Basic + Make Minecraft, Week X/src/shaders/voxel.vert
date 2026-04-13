#version 460 core

const float fog_gradient = 10.0;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform float render_distance;

layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 texture_coords;
layout (location = 2) in vec3 normal;
layout (location = 3) in float ao;

out VertexAttributes {
    vec3 texture_coords;
    vec3 normal;
    float ao;
    float visibility;
} attrs;

void main() {
    attrs.texture_coords = texture_coords;
    attrs.normal = normal;
    attrs.ao = ao;
    attrs.visibility = 1.0;

    vec4 frag_pos = view * model * vec4(pos, 1.0f);
    gl_Position = projection * frag_pos;

    // Fog
    float fog_density = 0.150 / render_distance;
    float distance = length(frag_pos.xyz);
    attrs.visibility = exp(-pow(distance * fog_density, fog_gradient));
}
