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
