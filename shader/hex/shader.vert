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

struct Instance {
    mat4 s_model;
    uvec4 tex_index;
};

layout(set=1, binding=1)
buffer Instances {
    Instance instances[];
};

void main() {
    Instance i = instances[gl_InstanceIndex];

    gl_Position = u_view_proj * i.s_model * vec4(a_position, 1.0);

    v_tex_coords = a_tex_coords;
    if (a_which_tex == 0)
        v_which_tex = i.tex_index.x;
    else
        v_which_tex = i.tex_index.y;
}
