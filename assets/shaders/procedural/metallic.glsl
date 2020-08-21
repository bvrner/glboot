#begin vertex
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNorm;

out vec3 Pos;
out vec3 Norm;

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
     gl_Position = projection * view * iPos;
}
#end vertex

#begin fragment

in vec3 Pos;
in vec3 Norm;

void main() {
    vec3 I = vec3(0.0, 0.0, 15.0) - Pos;
    vec3 Nf = normalize(faceforward(Norm, I, Norm));
    vec3 V = normalize(-I);

    vec3 R;
    vec3 Rworld;
    vec3 Ct;
    float altitude;


}

#end fragment
