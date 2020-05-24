#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;

out vec3 Normal;
out vec3 Position;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 arc;

void main() {
    mat4 rmodel = model * arc;
    Normal = mat3(transpose(inverse(rmodel))) * aNormal;
    Position = vec3(rmodel * vec4(aPos, 1.0));
    gl_Position = projection * view * vec4(Position, 1.0);
}
