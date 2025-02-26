#version 460
layout(location = 0) in vec3 frag_normal;
layout(location = 1) in vec3 frag_position;

layout(location = 0) out vec4 out_color;

const vec3 light_dir = normalize(vec3(0.5, -1.0, -0.3));
const vec3 light_color = vec3(0.5);
const vec3 view_pos = vec3(3.0, 3.0, 3.0);

void main() {
    vec3 norm = normalize(frag_normal);
    float diff = max(dot(norm, -light_dir), 0.0);
    vec3 diffuse = diff * light_color;
    
    vec3 view_dir = normalize(view_pos - frag_position);
    vec3 reflect_dir = reflect(light_dir, norm);
    float spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    vec3 specular = spec * light_color;
    
    vec3 object_color = vec3(0.6, 0.7, 1.0);
    vec3 result = (diffuse + specular) * object_color;
    out_color = vec4(result, 1.0);
}
