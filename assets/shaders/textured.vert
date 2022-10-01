#version 330 core
layout (location = 0) in vec3 Position;
layout (location = 2) in vec3 Normal;
layout (location = 1) in vec3 Texture;

out VS_OUTPUT {
    vec2 TexCoord;
    flat float Layer;
    vec3 Normal;
} OUT;

uniform mat4 Model;
uniform mat4 View;
uniform mat4 Projection;

void main()
{
    OUT.TexCoord = Texture.xy;
    OUT.Layer = Texture.z;
    OUT.Normal =  Normal;

    gl_Position = Projection * View * Model * vec4(Position, 1.0);
} 