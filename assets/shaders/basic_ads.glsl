#begin vertex
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 arc;

out vec3 Pos;
out vec3 Normal;

void main() {
    mat4 nmodel = arc * model;
    Pos = vec3(view * nmodel * vec4(aPos, 1.0));
    Normal = mat3(inverse(transpose(nmodel))) * aNormal;
    gl_Position = projection * vec4(Pos, 1.0);
}
#end vertex

#begin fragment
#version 330 core

in vec3 Pos;
in vec3 Normal;

out vec4 Color;

// TODO use material and light structs as arguments
vec3 ads_light(vec3 normal, vec3 position) {
    const vec3 light_pos = vec3(-0.3, 0.3, 0.3);

    const vec3 light_ambient = vec3(0.2, 0.2, 0.2);
    const vec3 light_specular = vec3(1.0, 1.0, 1.0);
    const vec3 light_diffuse = vec3(1.0, 1.0, 1.0);

    const vec3 material_ambient = vec3(1.0, 0.5, 0.0);
    const vec3 material_diffuse = vec3(1.0, 0.6, 0.0);
    const vec3 material_specular = vec3(1.0, 0.6, 0.6);

    const float material_shininess = 80.0;

    vec3 norm = normalize(normal);
    vec3 lightv = normalize(light_pos - position);
    vec3 viewv = normalize(vec3(0.0, 0.3, 0.5) - position); // TODO use the camera struct
    vec3 refl = reflect(-lightv, norm);

    vec3 ambient = material_ambient * light_ambient;
    vec3 diffuse = max(0.0, dot(lightv, norm)) * material_diffuse * light_diffuse;

    vec3 specular = vec3(0.0, 0.0, 0.0);
    if(dot(lightv, viewv) > 0.0) {
        specular = pow(max(0.0, dot(viewv, refl)), material_shininess) * material_specular * light_specular;
    }

    return clamp(ambient + diffuse + specular, 0.0, 1.0);
}

void main() {
    Color = vec4(ads_light(Normal, Pos), 1.0);
}
#end fragment
