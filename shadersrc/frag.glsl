#version 330 core

out vec4 Col;
uniform vec3 col;
void main() {
    Col = vec4(col, 1.0);
}
