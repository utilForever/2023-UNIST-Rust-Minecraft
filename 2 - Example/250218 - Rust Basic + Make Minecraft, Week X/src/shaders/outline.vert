#version 460 core

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

layout (location = 0) in vec3 pos;

float z_offset = -0.001;

void main() {
    vec4 position = projection * view * model * vec4(pos, 1.0);
    position.z += z_offset;
    gl_Position = position;
}
