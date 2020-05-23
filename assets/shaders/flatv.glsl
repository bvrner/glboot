// A flat vertex shader, only transform the positions

#version 330 core

layout (location = 0) in vec3 aPos;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 arc;

void main() {
    gl_Position = projection * view * (model * arc) * vec4(aPos, 1.0);
}
