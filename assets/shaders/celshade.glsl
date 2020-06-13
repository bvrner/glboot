#begin vertex
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 arc;

out vec3 Pos;
out vec3 Normal;

void main() {
    mat4 nmodel = arc * model;
    Pos = vec3(view * nmodel * vec4(aPos, 1.0));
    Normal = mat3(inverse(transpose(nmodel))) * aNormal;
    gl_Position = projection * vec4(Pos, 1.0);
}

#end vertex

#begin fragment
#version 330 core

out vec4 Col;

in vec3 Pos;
in vec3 Normal;

const vec3 light_pos = vec3(-0.3, 0.3, 0.3);

void main() {
    vec3 lightv = normalize(light_pos - Pos);
    float factor = dot(normalize(Normal), lightv);

    if(factor > 0.5) {
        Col = vec4(1.0, 1.0, 1.0, 1.0);
    } else if(factor > 0.0) {
        Col = vec4(0.33, 0.33, 0.33, 1.0);
    } else {
        Col = vec4(0.0, 0.0, 0.0, 1.0);
    }

}

#end fragment
