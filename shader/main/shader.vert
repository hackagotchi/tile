// shader.vert
#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_tex_coords;
layout(location=2) in uint a_which_tex;
layout(location=0) out vec2 v_tex_coords;
layout(location=1) flat out uint v_which_tex;

layout(set=1, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

layout(set=1, binding=1)
buffer Instances {
    mat4 s_models[];
};

void main() {
    gl_Position = u_view_proj * s_models[gl_InstanceIndex] * vec4(a_position, 1.0);
    v_tex_coords = a_tex_coords;
    v_which_tex = a_which_tex;
}