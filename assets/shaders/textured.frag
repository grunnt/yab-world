#version 330 core

uniform sampler2DArray blockTextures;

in VS_OUTPUT {
    vec2 TexCoord;
    flat float Layer;
    vec3 Normal;
} IN;

out vec4 fragColor;

void main()
{
    fragColor = texture(blockTextures, vec3(IN.TexCoord, IN.Layer)).xyz;
}