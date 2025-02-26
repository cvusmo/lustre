#version 460
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 frag_normal;
layout(location = 1) out vec3 frag_position;

layout(push_constant) uniform PushConstants {
    mat4 mvp;
    mat4 model;
} pc;

void main() {
    frag_position = vec3(pc.model * vec4(position, 1.0));
    frag_normal = mat3(pc.model) * normal;
    gl_Position = pc.mvp * vec4(position, 1.0);
}

