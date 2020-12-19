#begin vertex
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 2) in vec2 aTex;
layout (location = 3) in vec4 aJoints;
layout (location = 4) in vec4 aWeights;

out vec2 TexCoords;

// rem
uniform mat4 joints[512];

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main() {
    TexCoords = aTex;

   mat4 skinning = aWeights.x * joints[int(aJoints.x)] +
    aWeights.y * joints[int(aJoints.y)] +
    aWeights.z * joints[int(aJoints.z)] +
    aWeights.w * joints[int(aJoints.w)];

    mat4 mv = view * model;
    vec4 pos = mv * skinning * vec4(aPos, 1.0);
    gl_Position = projection * pos;
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
    } else if (material.has_base_tex) {
        Col = texture(material.base_tex, TexCoords);
    }
    else if (material.has_base_color) {
        Col = material.base_color;
    } else {
        Col = vec4(1.0, 1.0, 1.0, 1.0);
    }
}
#end fragment
