#begin vertex
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 2) in vec2 aTex;

out vec2 TexCoords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 arc;
uniform mat4 default_model;

void main() {
    TexCoords = aTex;
    gl_Position = projection * view * default_model * vec4(aPos, 1.0);
}
#end vertex

#begin fragment
#version 330 core

in vec2 TexCoords;
out vec4 Col;

struct Material {
    vec4 base_color;
    sampler2D base_tex;

    bool has_base_color;
    bool has_base_tex;

    float metallic;
    float roughness;
    sampler2D metallic_tex;

    sampler2D normal;
    sampler2D occlusion_tex;
    float occlusion_str;
};

uniform Material material;

void main() {
    if (material.has_base_tex && material.has_base_color) {
        Col = texture(material.base_tex, TexCoords) * material.base_color;
    } else if (material.has_base_color) {
        Col = material.base_color;
    } else {
        Col = vec4(1.0, 1.0, 1.0, 1.0);
    }
}
#end fragment
