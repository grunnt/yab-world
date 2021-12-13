#version 330 core
layout (location = 0) in vec2 Position;
layout (location = 1) in vec3 Color;

out vec3 vertColor;

uniform mat4 Model;
uniform mat4 ProjectionView;

void main()
{
    vertColor = Color;
    vec4 pos = ProjectionView * Model * vec4(Position, 0.0, 1.0);
    gl_Position = vec4(pos.xy, 0.0, 1.0);
} 