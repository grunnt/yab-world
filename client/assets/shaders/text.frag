#version 330 core

uniform sampler2D fontTexture;

uniform vec3 color;

out vec4 fragColor;

in vec2 vertTexCoords;

void main()
{
    fragColor = vec4(color, texture(fontTexture, vertTexCoords).r);
}