#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 tex_coords;
layout(location = 2) in vec3 v_color;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D tex;

layout(set = 0, binding = 1) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;

vec3 hemisphere_light(vec3 normal, vec3 lightDirection, vec3 sky, vec3 ground) {
  float weight = 0.5 * dot(normalize(normal), lightDirection) + 0.5;
  return mix(ground, sky, weight);
}

void main() {
    vec3 light = hemisphere_light(v_normal, vec3(0.188144, -0.940721, 0.282216), vec3(1.0, 1.0, 1.0), vec3(0.6, 0.6, 0.6));
    f_color = vec4(/*v_normal * */ light * texture(tex, tex_coords).xyz /* * v_color */, 1.0);
}
