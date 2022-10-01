#version 330 core
layout (location = 0) in vec3 Position;
layout (location = 1) in vec3 Color;

out vec3 vertColor;

uniform mat4 Model;
uniform mat4 View;
uniform mat4 Projection;

void main()
{
    vertColor = Color;
    gl_Position = Projection * View * Model * vec4(Position, 1.0);
} 