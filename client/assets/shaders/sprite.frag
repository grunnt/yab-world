#version 330 core

uniform sampler2D textureAtlas;

out vec4 fragColor;

in vec4 vertColor;
in vec2 texCoords;

void main()
{
    vec4 color = texture(textureAtlas, texCoords);
    fragColor = vertColor * color;
}