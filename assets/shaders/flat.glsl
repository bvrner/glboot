#begin vertex
#version 330 core

layout (location = 0) in vec3 aPos;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 arc;

void main() {
    gl_Position = projection * view * (arc * model) * vec4(aPos, 1.0);
}
#end vertex

#begin fragment
#version 330 core

out vec4 Col;

struct Material {
    vec4 base_color;
    bool has_base_color;
};

uniform Material material;

void main() {
    if (material.has_base_color) {
        Col = material.base_color;
    } else {
        Col = vec4(1.0, 1.0, 1.0, 1.0);
    }
}
#end fragment
