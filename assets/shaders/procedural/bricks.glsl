#begin vertex
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNorm;
layout (location = 2) in vec2 aTexCoord;

out vec3 Pos;
out vec3 Norm;
out vec2 TexCoord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 arc;
uniform mat4 default_model;
uniform mat4 global;

void main() {
     mat4 thismodel = arc * global * default_model;
     Norm = mat3(inverse(transpose(thismodel))) * aNorm;
     vec4 iPos = thismodel * vec4(aPos, 1.0);
     Pos = vec3(iPos);
     TexCoord = aTexCoord;
     gl_Position = projection * view * iPos;
}
#end vertex

#begin fragment

#version 330 core

#define BRICKWIDTH 0.25
#define BRICKHEIGHT 0.08
#define MORTARTHICKNESS 0.01

#define BMWIDTH (BRICKWIDTH + MORTARTHICKNESS)
#define BMHEIGHT (BRICKHEIGHT + MORTARTHICKNESS)

#define MWF (MORTARTHICKNESS * 0.5 / BMWIDTH)
#define MHF (MORTARTHICKNESS * 0.5 / BMHEIGHT)

in vec3 Pos;
in vec3 Norm;
in vec2 TexCoord;

out vec4 Tex;

uniform float Ka = 1.0;
uniform float kd = 1.0;
uniform vec3 Cbrick = vec3(0.5, 0.15, 0.14);
uniform vec3 Cmortar = vec3(0.5, 0.5, 0.5);

void main() {
    vec3 I = Pos - vec3(0.0, 0.0, 15.0);
    float scoord = TexCoord.x;
    float tcoord = TexCoord.y;

    vec3 Nf = normalize(faceforward(Norm, I, Norm));

    float ss = scoord / BMWIDTH;
    float tt = tcoord / BMHEIGHT;

    if ((mod(tt * 0.5, 1) > 0.5))
        ss -= 0.5;

    float sbrick = floor(ss);
    float tbrick = floor(tt);
    ss -= sbrick;
    tt -= tbrick;

    float w = step(MWF, ss) - step(1 - MWF, ss);
    float h = step(MHF, tt) - step(1 - MHF, tt);

    vec3 Ct = mix(Cmortar, Cbrick, w * h);
    Tex = vec4(Ct, 1.0);
}

#end fragment
