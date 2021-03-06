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

out vec4 Color;
in vec2 TexCoords;

uniform sampler2D screenTex;

void main() {
    Color = vec4(texture(screenTex, TexCoords).rgb, 1.0);
    // Color = vec4(1.0, 1.0, 1.0, 1.0);
}

#end fragment
