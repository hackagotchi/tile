// shader.vert
#version 450

layout(location=0) in vec3 a_position;
layout(location=0) out vec2 v_tex_coords;
layout(location=1) flat out uint v_which_tex;

layout(set=1, binding=0)
uniform Uniforms {
    vec4 u_camera_up;
    vec4 u_camera_right;
    mat4 u_view_proj;
};

struct Instance {
    vec3 s_position;
    vec3 s_scale;
    uvec2 tex_index;
};

layout(set=1, binding=1)
buffer Instances {
    Instance instances[];
};

void main() {
    Instance i = instances[gl_InstanceIndex];

    gl_Position = u_view_proj * vec4(
        i.s_position
            + u_camera_right.xyz * a_position.x * i.s_scale.x
            + u_camera_up.xyz * a_position.y * i.s_scale.y,
	1.0
    );

    v_tex_coords = vec2(a_position.x + 0.5, 0.5 - a_position.y);
    v_which_tex = i.tex_index.x;
}
