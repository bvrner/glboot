#begin vertex
#version 330 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;

out vec3 Normal;
out vec3 Position;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 arc;

void main() {
    mat4 rmodel = model * arc;
    Normal = mat3(transpose(inverse(rmodel))) * aNormal;
    Position = vec3(rmodel * vec4(aPos, 1.0));
    gl_Position = projection * view * vec4(Position, 1.0);
}
#end vertex

#begin fragment
#version 330 core

out vec4 Color;

in vec3 Position;
in vec3 Normal;

uniform bool refraction = true;
uniform vec3 cameraPos = vec3(0.0, 0.3, 0.5);
uniform samplerCube skybox;

void main() {
    vec3 I = normalize(Position - cameraPos);

    if (refraction) {
        float ratio = 1.33 / 1.52;
        vec3 R = refract(I, normalize(Normal), ratio);
        Color = vec4(texture(skybox, R).rgb, 1.0);
    } else {
        vec3 R = reflect(I, normalize(Normal));
        Color = vec4(texture(skybox, R).rgb, 1.0);
    }
}
#end fragment
