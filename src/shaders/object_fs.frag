#version 460
layout(location = 0) in vec3 frag_position;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec2 frag_tex_coord;

layout(location = 0) out vec4 out_color;

void main() {
    // Placeholder: Solid color for now (texture support later)
    out_color = vec4(1.0, 0.5, 0.5, 1.0); // Light red
}
