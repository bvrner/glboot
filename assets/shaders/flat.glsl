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
    vec3 ambient;
    vec3 specular;
    float shininess;

    bool has_diffuse;
    bool has_normal;
    bool has_specular;

    sampler2D diffuse_tex;
    sampler2D normal_tex;
    sampler2D specular_tex;
};

uniform Material material;

void main() {
    Col = vec4(material.diffuse, 1.0);
}
#end fragment
