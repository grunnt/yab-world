#version 330 core
layout (location = 0) out vec3 gPosition;
layout (location = 1) out vec3 gColor;
layout (location = 2) out vec3 gNormal;
layout (location = 3) out vec3 gLight;

uniform sampler2DArray blockTextures;

in VS_OUTPUT {
    vec3 Position;
    vec2 TexCoord;
    flat float Layer;
    vec3 Normal;
    float Light;
} IN;

void main()
{
    // Store the fragment position in the gbuffer texture
    gPosition = IN.Position;
    // Store the per-fragment normal in the gbuffer texture
    gNormal = normalize(IN.Normal);
    // Store the fragment color in the gbuffer texture
    gColor = texture(blockTextures, vec3(IN.TexCoord, IN.Layer)).xyz;
    // Store light values in the gbuffer light texture
    gLight = vec3(IN.Light, 0.0, 1.0); // Z value is used to detect background
}