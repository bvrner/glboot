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

const mat3 sobel[2] = mat3[](
    mat3( 1.0, 2.0, 1.0, 0.0, 0.0, 0.0, -1.0, -2.0, -1.0 ),
    mat3( 1.0, 0.0, -1.0, 2.0, 0.0, -2.0, 1.0, 0.0, -1.0 )
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

void main() {
    Color = sobel_kernel();
    // Color = vec4(vec3(1.0 - Color), 1.0);
}

#end fragment
