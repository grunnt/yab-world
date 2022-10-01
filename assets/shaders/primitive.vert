#version 330 core
layout (location = 0) in vec2 Position;
layout (location = 1) in vec4 Color;

out vec4 vertColor;

uniform mat4 projection;

void main()
{
    vertColor = Color;
    gl_Position = projection * vec4(Position, 0.0, 1.0);
} 