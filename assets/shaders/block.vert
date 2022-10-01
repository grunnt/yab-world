#version 330 core
layout (location = 0) in vec3 Position;
layout (location = 1) in vec3 Texture;
layout (location = 2) in uint Normal;
layout (location = 3) in uint Light;

out VS_OUTPUT {
    vec3 Position;
    vec2 TexCoord;
    flat float Layer;
    vec3 Normal;
    float Light;
} OUT;

uniform mat4 Model;
uniform mat4 View;
uniform mat4 Projection;
uniform float zOffset;

void main()
{
    // Send view position, normal and color to fragment shader
    vec4 viewPos  = View * Model * vec4(Position.xy, Position.z * zOffset, 1.0);
    OUT.Position = viewPos.xyz; 

    // Get the texture coordinates    
    OUT.TexCoord = Texture.xy;
    OUT.Layer = Texture.z;

    // Unpack the normal
    vec3 unpackedNormal = normalize(vec3(int(Normal & 3u) - 1, int((Normal >> 2u) & 3u) - 1, int((Normal >> 4u) & 3u) - 1));
    OUT.Normal = transpose(inverse(mat3(View * Model))) * unpackedNormal;
    
    // Get the light values
    OUT.Light = float(Light) / 15.0;

    gl_Position = Projection * viewPos;
}