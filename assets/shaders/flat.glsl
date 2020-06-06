#begin vertex
#version 330 core

layout (location = 0) in vec3 aPos;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 arc;

void main() {
    gl_Position = projection * view * (model * arc) * vec4(aPos, 1.0);
}
#end vertex

#begin fragment
#version 330 core

out vec4 Col;

struct Material {
    vec3 diffuse;
    bool has_diffuse;
};

uniform Material material;

void main() {
    if (material.has_diffuse) {
        Col = vec4(material.diffuse, 1.0);
    } else {
        Col = vec4(1.0, 1.0, 1.0, 1.0);
    }
}
#end fragment
