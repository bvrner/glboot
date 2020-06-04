#begin vertex
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
#end vertex

#begin fragment
#version 330 core

in vec2 TexCoords;
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
    Col = texture(material.diffuse_tex, TexCoords);
}
#end fragment
