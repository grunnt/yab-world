#version 330 core
layout (location = 0) in vec2 Position;
layout (location = 1) in vec2 TexCoords;

out vec2 vertTexCoords;

uniform mat4 projection;

void main()
{
    gl_Position = projection * vec4(Position, 0.0, 1.0);
    vertTexCoords = TexCoords;
} 