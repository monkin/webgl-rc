precision mediump float;

#include <color.glsl>

varying vec3 v_color;

void main() {
    gl_FragColor = vec4(to_srgb(v_color), 1);
}
