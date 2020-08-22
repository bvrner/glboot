#begin vertex
#version 330 core

layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 aTexCoords;

out vec2 TexCoords;

void main() {
    TexCoords = aTexCoords;
    gl_Position = vec4(aPos.x, aPos.y, 0.0, 1.0);
}
#end vertex

#begin fragment

#version 330 core

in vec2 TexCoords;

out vec4 Color;

uniform int option = 0;
uniform sampler2D screenTex;

// kernel effects
const mat3 kernels[] = mat3[](
    // sharp
    mat3(
        -1.0, -1.0,-1.0,
        -1.0, 9.0, -1.0,
        -1.0,-1.0,-1.0
         ),
    // blur
    mat3(
        1.0 / 16, 2.0 / 16, 1.0 / 16,
        2.0 / 16, 4.0 / 16, 2.0 / 16,
        1.0 / 16, 2.0 / 16, 1.0 / 16
         )
    // edge
    mat3(
        1.0, 1.0,1.0,
        1.0, -8.0, 1.0,
        1.0,1.0,1.0
         ),

);


vec4 apply_kernel(mat3 kernel) {
    const float offset = 1.0 / 300.0;
    vec2 offsets[9] = vec2[](
        vec2(-offset,  offset), // top-left
        vec2( 0.0f,    offset), // top-center
        vec2( offset,  offset), // top-right
        vec2(-offset,  0.0f),   // center-left
        vec2( 0.0f,    0.0f),   // center-center
        vec2( offset,  0.0f),   // center-right
        vec2(-offset, -offset), // bottom-left
        vec2( 0.0f,   -offset), // bottom-center
        vec2( offset, -offset)  // bottom-right
    );

    vec3 sample_tex[9];
    for (int i = 0; i < 3; i++)
        for(int j = 0; j < 3; j++)
            sample_tex[i + 3 * j] = vec3(texture(screenTex, TexCoords.st + offsets[i + 3 * j]));

    vec3 col = vec3(0.0);
    for (int i = 0; i < 3; i++)
        for (int j = 0; j < 3; j++)
            col += sample_tex[i + 3 * j] * kernel[i][j];

    return vec4(col, 1.0);
}

void main() {
    Color = apply_kernel(kernels[option]);
}

#end fragment
