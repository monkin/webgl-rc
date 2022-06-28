#ifndef __COLOR__
#define __COLOR__

float to_srgb(float c) {
    return pow(c, 1.0 / 2.2);
}

vec3 to_srgb(vec3 c) {
    return pow(c, vec3(1.0 / 2.2));
}

#endif