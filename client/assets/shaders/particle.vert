#version 330 core
layout (location = 0) in vec3 position;
layout (location = 1) in vec3 layer_size_life;

flat out float layer;
flat out float life;

uniform mat4 projection;
uniform mat4 view;
uniform float viewport_height;

void main()
{
    layer = layer_size_life.x;
    life = layer_size_life.z;
    gl_Position = projection * view * vec4(position, 1.0);
    gl_PointSize = viewport_height * projection[1][1] * layer_size_life.y / gl_Position.w;
} 