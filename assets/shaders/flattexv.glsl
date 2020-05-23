// Flat texturing vertex shader

#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 2) in vec2 aTex;

out vec3 Normals;
out vec2 TexCoords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 arc;

void main() {
    TexCoords = aTex;
    gl_Position = projection * view * (model * arc) * vec4(aPos, 1.0);
}
