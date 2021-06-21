#version 300 es

uniform mat4 u_transform;

in vec3 a_position;
in vec2 a_tex_coord;
in vec4 a_color;

out vec2 v_tex_coord;
out vec4 v_color;

void main() {
    v_color = a_color;
    v_tex_coord = a_tex_coord;
    gl_Position = u_transform * vec4(a_position, 1.0);
}
