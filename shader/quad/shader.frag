// shader.frag
#version 450

layout(location=0) in vec2 v_tex_coords;
layout(location=1) flat in uint v_which_tex;
layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2DArray t_diffuse;
layout(set = 0, binding = 1) uniform sampler s_diffuse;

void main() {
    f_color = texture(sampler2DArray(t_diffuse, s_diffuse), vec3(v_tex_coords, v_which_tex));
}
