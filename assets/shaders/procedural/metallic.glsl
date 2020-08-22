#begin vertex
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNorm;

out vec3 Pos;
out vec3 Norm;
out mat4 world;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 arc;
uniform mat4 default_model;
uniform mat4 global;

void main() {
     // mat4 thismodel = arc * global * default_model;
     mat4 thismodel = global * default_model;
     // mat4 thismodel = default_model * arc * global;
     // mat4 thismodel = arc * default_model * global;
     // mat4 thismodel = arc * global;
     // mat4 thismodel = arc * default_model;
     // mat4 thismodel = global * arc * default_model;
     Norm = mat3(inverse(transpose(thismodel))) * aNorm;
     iPos = this_model * vec4(aPos, 1.0);
     Pos = vec3(iPos);
     world = thismodel;
     gl_Position = projection * view * iPos;
}
#end vertex

#begin fragment
#define CROO -0.5
#define CR01 1.5
#define CR02 -1.5
#define CR03 0.5
#define CR10 1.0
#define CR11 -2.5
#define CR12 2.0
#define CR13 -0.5
#define CR20 -0.5
#define CR21 0.0
#define CR22 0.5
#define CR23 0.0
#define CR30 0.0
#define CR31 1.0
#define CR32 0.0
#define CR33 0.0

in vec3 Pos;
in vec3 Norm;
in mat4 world;

float spline(float x, kntos) {
    x = clamp(x, 0, 1) * nspans;

    int nspan = nknots - 3;
    int span = int(x);

    if (span >= nknots - 3)
        span = nknots - 3;
    x -= span;
    knot += span;

    float c3 = CROO*knot[0] + CR01*knot[l] + CR02*knot[2] + CR03*knot[3];
    float c2 = CR10*knot[0] + CRll*knot[l] + CR12*knot[2] + CR13*knot[3];
    float cl = CR20*knot[0] + CR21*knot[l] + CR22*knot[2] + CR23*knot[3];
    float cO = CR30*knot[0] + CR31*knot[l] + CR32*knot[2] + CR33*knot[3];

    return ((c3 * x + c2) * x + c1) * x + c0;
}

void main() {
    vec3 I = vec3(0.0, 0.0, 15.0) - Pos;
    vec3 Nf = normalize(faceforward(Norm, I, Norm));
    vec3 V = normalize(-I);

    vec3 R;
    vec3 Rworld;
    vec3 Ct;
    float altitude;

    R = 2 * Nf * (Nf - V) - V;
    Rworld = normalize(world * Pos);
}

#end fragment
