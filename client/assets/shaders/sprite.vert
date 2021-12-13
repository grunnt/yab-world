#version 330 core
layout (location = 0) in vec2 Position;
layout (location = 1) in vec2 TexCoords;
layout (location = 2) in vec4 Color;

out vec4 vertColor;
out vec2 texCoords;

uniform mat4 projection;

void main()
{
    vertColor = Color;
    texCoords = TexCoords;
    gl_Position = projection * vec4(Position, 0.0, 1.0);
} 