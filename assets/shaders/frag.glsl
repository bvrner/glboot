#version 330 core

in vec2 TexCoords;
in vec3 Normals;
out vec4 Col;

uniform vec3 col;
uniform sampler2D diffuse;
uniform sampler2D specular;
uniform sampler2D normal;

void main() {
    // Col = vec4(col, 1.0);
    // Col = vec4(Normals, 1.0);
    Col = texture(diffuse, TexCoords);
    // Col = vec4(TexCoords, 0.0, 1.0);
}
