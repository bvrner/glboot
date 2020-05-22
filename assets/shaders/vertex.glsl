#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNorm;
layout (location = 2) in vec2 aTex;
layout (location = 3) in vec3 aTang;
layout (location = 4) in vec3 aBitang;

out vec3 Pos;
out vec3 Normals;
out vec2 TexCoords;
out mat3 TBN;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 arc;
void main() {
    mat4 nmodel = model * arc;

    vec3 T = normalize(vec3(nmodel * vec4(aTang, 0.0)));
    vec3 B = normalize(vec3(nmodel * vec4(aBitang, 0.0)));
    vec3 N = normalize(vec3(nmodel * vec4(aNorm, 0.0)));
    TBN = mat3(T, B, N);

    TexCoords = aTex;
    Normals = normalize(mat3(transpose(inverse(nmodel))) * aNorm);
    Pos = vec3(nmodel * vec4(aPos, 1.0));

    gl_Position = projection * view * vec4(Pos, 1.0);
}
