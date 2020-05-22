#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNorm;
layout (location = 2) in vec2 aTex;
layout (location = 3) in vec3 aTang;
layout (location = 4) in vec3 aBitang;

out VS_OUT {
    vec3 Pos;
    vec2 TexCoords;
    vec3 TangentLightPos;
    vec3 TangentViewPos;
    vec3 TangentFragPos;
} vs_out;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 arc;
void main() {
    mat4 nmodel = model * arc;
    mat3 normalMatrix = transpose(inverse(mat3(nmodel)));

    vec3 T = normalize(vec3(normalMatrix * aTang));
    vec3 N = normalize(vec3(normalMatrix * aNorm));
    T = normalize(T - dot(T, N) * N);
    vec3 B = cross(N, T);
    mat3 TBN = transpose(mat3(T, B, N));

    // hardocoded for now
    vs_out.TangentLightPos = TBN * vec3(0.0, 0.5, 0.5);
    vs_out.TangentViewPos = TBN * vec3(0.0, 0.5, 0.5);
    vs_out.TangentFragPos = TBN * vec3(nmodel * vec4(aPos, 1.0));
    vs_out.TexCoords = aTex;
    vec3 Pos = vec3(nmodel * vec4(aPos, 1.0));
    vs_out.Pos = Pos;
    //Normals = normalize(mat3(transpose(inverse(nmodel))) * aNorm);

    gl_Position = projection * view * vec4(Pos, 1.0);
}
