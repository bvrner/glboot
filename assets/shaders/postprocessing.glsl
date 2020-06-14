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
const float sharp[9] = float[](
    -1.0, -1.0,-1.0,
    -1.0, 9.0, -1.0,
    -1.0,-1.0,-1.0
);

const float edge[9] = float[](
    1.0, 1.0,1.0,
    1.0, -8.0, 1.0,
    1.0,1.0,1.0
);

const mat3 sobel[2] = mat3[](
    mat3( 1.0, 2.0, 1.0, 0.0, 0.0, 0.0, -1.0, -2.0, -1.0 ),
    mat3( 1.0, 0.0, -1.0, 2.0, 0.0, -2.0, 1.0, 0.0, -1.0 )
);

const float blur[9] = float[](
    1.0 / 16, 2.0 / 16, 1.0 / 16,
    2.0 / 16, 4.0 / 16, 2.0 / 16,
    1.0 / 16, 2.0 / 16, 1.0 / 16
);

vec4 sobel_kernel() {
    mat3 I;
    vec3 s;
    for (int i=0; i<3; i++) {
        for (int j=0; j<3; j++) {
            s = texelFetch(screenTex, ivec2(gl_FragCoord) + ivec2(i-1,j-1), 0 ).rgb;
            I[i][j] = length(s);
        }
    }

    float cnv[2];
    for (int i=0; i<2; i++) {
        float dp3 = dot(sobel[i][0], I[0]) + dot(sobel[i][1], I[1]) + dot(sobel[i][2], I[2]);
        cnv[i] = dp3 * dp3;
    }

    return vec4(vec3(sqrt(cnv[0] * cnv[0] + cnv[1] * cnv[1])), 1.0);
}

vec4 apply_kernel(float kernel[9]) {
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
    for (int i = 0; i < 9; i++) {
        sample_tex[i] = vec3(texture(screenTex, TexCoords.st + offsets[i]));
    }

    vec3 col = vec3(0.0);
    for (int i = 0; i < 9; i++)
        col += sample_tex[i] * kernel[i];

    return vec4(col, 1.0);
}

void main() {
    switch (option) {
        case 0:
            Color = texture(screenTex, TexCoords);
            break;
        case 1:
            Color = vec4(vec3(1.0 - texture(screenTex, TexCoords)), 1.0);
            break;
        case 2:
            Color = texture(screenTex, TexCoords);
            float average = (Color.r + Color.g + Color.b) / 3.0;
            Color = vec4(average, average, average, 1.0);
            break;
        case 3:
            Color = apply_kernel(sharp);
            break;
        case 4:
            Color = apply_kernel(blur);
            break;
        case 5:
            Color = apply_kernel(edge);
            break;
        case 6:
            Color = sobel_kernel();
            break;
        case 7:
            Color = sobel_kernel();
            Color = vec4(vec3(1.0 - Color), 1.0);
            break;
    }
}

#end fragment
