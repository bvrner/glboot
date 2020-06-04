#begin vertex
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNorm;
layout (location = 2) in vec2 aTex;
layout (location = 3) in vec3 aTang;
layout (location = 4) in vec3 aBitang;

out vec3 Pos;
out vec3 Normals;
out vec2 TexCoords;
out mat3 TBN;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 arc;
void main() {
    mat4 nmodel = model * arc;

    vec3 T = normalize(vec3(nmodel * vec4(aTang, 0.0)));
    vec3 B = normalize(vec3(nmodel * vec4(aBitang, 0.0)));
    vec3 N = normalize(vec3(nmodel * vec4(aNorm, 0.0)));
    TBN = mat3(T, B, N);

    TexCoords = aTex;
    Normals = normalize(mat3(transpose(inverse(nmodel))) * aNorm);
    Pos = vec3(nmodel * vec4(aPos, 1.0));

    gl_Position = projection * view * vec4(Pos, 1.0);
}
#end vertex

#begin fragment
#version 330 core

in vec2 TexCoords;
in vec3 Normals;
in vec3 Pos;
in mat3 TBN;

out vec4 Col;

uniform vec3 col;

const vec3 light_col = vec3(1.0, 1.0, 1.0);
const vec3 light_pos = vec3(0.0, 0.5, 0.5);
const vec3 view_pos = vec3(0.0, 0.5, 0.5);

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

struct Light {
    vec3 position;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    float constant;
    float linear;
    float quadratic;
} light;

uniform Material material;

void main() {
    light.position = light_pos;
    light.ambient = vec3(0.2, 0.2, 0.2);
    light.diffuse = vec3(0.5, 0.5, 0.5);
    light.specular = vec3(1.0, 1.0, 1.0);
    light.constant = 1.0;
    light.linear = 0.09;
    light.quadratic = 0.032;

    float dist = length(light.position - Pos);
    float attenuation = 1.0 / (light.constant + light.linear * dist + light.quadratic * (dist * dist));

    vec3 ambient = light.ambient * texture(material.diffuse_tex, TexCoords).rgb;

    vec3 norm = texture(material.normal_tex, TexCoords).rgb;
    norm = norm * 2.0 - 1.0;
    norm = normalize(TBN * norm);

    vec3 lightDir = normalize(light_pos - Pos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = light.diffuse * diff * texture(material.diffuse_tex, TexCoords).rgb;

    // specular
    vec3 viewDir = normalize(view_pos - Pos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    vec3 specular = light.specular * spec * texture(material.specular_tex, TexCoords).rgb;

    ambient *= attenuation;
    diffuse *= attenuation;
    specular *= specular;
    vec3 result = ambient + diffuse + specular;
    Col = vec4(result, 1.0);
}
#end fragment
