// Flat texturing, uses the diffuse texture, nothing else

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
