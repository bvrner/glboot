#version 330 core

in vec2 TexCoords;
in vec3 Normals;
out vec4 Col;

uniform vec3 col;
uniform sampler2D tex;

void main() {
    // Col = vec4(col, 1.0);
    Col = vec4(Normals, 1.0);
    // Col = texture(tex, TexCoords);
}
