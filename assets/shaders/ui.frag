#version 330 core

out vec3 fragColor;

in vec3 vertColor;

void main()
{
    fragColor = vertColor;
}