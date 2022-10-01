#version 330 core

out vec4 fragColor;

in vec4 vertColor;

void main()
{
    fragColor = vertColor;
}