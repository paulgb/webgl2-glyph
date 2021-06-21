#version 300 es
precision mediump float;

uniform sampler2D u_texture;

in vec2 v_tex_coord;
in vec4 v_color;

out vec4 f_color;

void main() {
    float alpha = texture(u_texture, v_tex_coord).r;
    if (alpha == 0.) {
        discard;
    }
    f_color = v_color * alpha;
}