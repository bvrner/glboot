// Flat fragment shader, use the material diffuse color and nothing else

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
