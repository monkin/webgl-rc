precision highp float;

attribute vec3 a_color;
attribute vec2 a_position;
uniform float u_time;

varying vec3 v_color;

void main() {
    v_color = a_color;
    gl_Position = vec4(a_position * 0.9, 0, 1);
}
