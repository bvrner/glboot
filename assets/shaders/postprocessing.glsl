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

uniform sampler2D screenTex;

void main() {
    Color = texture(screenTex, TexCoords);
}

#end fragment
